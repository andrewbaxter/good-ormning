use std::{
    collections::HashMap,
};
use dyn_clone::clone_trait_object;
use proc_macro2::TokenStream;
use crate::{
    pg::{
        types::Type,
        QueryResCount,
        schema::{
            field::{
                FieldRef,
            },
            table::{
                TableRef,
            },
        },
    },
    utils::{
        Tokens,
        Errs,
    },
};
use super::{
    expr::{
        ExprType,
        ExprValName,
        Expr,
        check_assignable,
    },
};

#[derive(Clone, Debug)]
pub struct With {
    pub recursive: bool,
    pub ctes: Vec<Cte>,
}

#[derive(Clone, Debug)]
pub struct Cte {
    pub table_id: String,
    pub columns: Vec<(String, String, Type)>,
    pub body: Box<dyn QueryBody>,
}

pub struct CteBuilder {
    table_id: String,
    columns: Vec<(String, String, Type)>,
    body: Box<dyn QueryBody>,
}

impl CteBuilder {
    pub fn new(id: impl AsRef<str>, body: Box<dyn QueryBody>) -> Self {
        let table_id = id.as_ref().to_string();
        return Self {
            table_id: table_id,
            columns: vec![],
            body: body,
        };
    }

    pub fn field(&mut self, id: impl AsRef<str>, type_: Type) -> (String, String, Type) {
        let field_id = id.as_ref().to_string();
        let f = (field_id.clone(), field_id, type_);
        self.columns.push(f.clone());
        return f;
    }

    pub fn build(self) -> Cte {
        return Cte {
            table_id: self.table_id,
            columns: self.columns,
            body: self.body,
        };
    }
}

pub fn build_with(ctx: &mut PgQueryCtx, path: &rpds::Vector<String>, with: &With) -> Tokens {
    let mut out = Tokens::new();
    out.s("with");
    if with.recursive {
        out.s("recursive");
    }
    for (i, cte) in with.ctes.iter().enumerate() {
        if i > 0 {
            out.s(",");
        }
        let path = path.push_back(format!("CTE {}", i));
        out.id(&cte.table_id);
        out.s("(");
        for (i, (_, sql_name, _)) in cte.columns.iter().enumerate() {
            if i > 0 {
                out.s(",");
            }
            out.id(sql_name);
        }
        out.s(")");
        out.s("as");
        out.s("(");
        let (body_type, body_tokens) = cte.body.build(ctx, &path, QueryResCount::Many);
        if body_type.0.len() != cte.columns.len() {
            ctx
                .errs
                .err(
                    &path,
                    format!(
                        "Select returns {} columns but the CTE needs exactly {} columns",
                        body_type.0.len(),
                        cte.columns.len()
                    ),
                );
        } else {
            for (
                i,
                ((_, got), (_, _, want)),
            ) in Iterator::zip(body_type.0.iter(), cte.columns.iter()).enumerate() {
                let path = path.push_back(format!("Select return {}", i));
                check_assignable(&mut ctx.errs, &path, want, &ExprType(vec![(ExprValName::empty(), got.clone())]));
            }
        }
        out.s(&body_tokens.to_string());
        out.s(")");
        let mut fields = HashMap::new();
        for (field_id, sql_name, type_) in &cte.columns {
            fields.insert(FieldRef {
                table_id: cte.table_id.clone(),
                field_id: field_id.clone(),
            }, PgFieldInfo {
                sql_name: sql_name.clone(),
                type_: type_.clone(),
            });
        }
        ctx.tables.insert(TableRef(cte.table_id.clone()), PgTableInfo {
            sql_name: cte.table_id.clone(),
            fields,
        });
    }
    return out;
}

#[derive(Clone, Debug)]
pub struct PgFieldInfo {
    pub sql_name: String,
    pub type_: Type,
}

#[derive(Clone, Debug)]
pub struct PgTableInfo {
    pub sql_name: String,
    pub fields: HashMap<FieldRef, PgFieldInfo>,
}

#[derive(Clone, Debug)]
pub struct Returning {
    pub e: Expr,
    pub rename: Option<String>,
}

pub struct PgQueryCtx {
    pub(crate) tables: HashMap<TableRef, PgTableInfo>,
    pub errs: Errs,
    pub(crate) rust_arg_lookup: HashMap<String, (usize, Type)>,
    pub(crate) rust_args: Vec<TokenStream>,
    pub(crate) query_args: Vec<TokenStream>,
}

impl PgQueryCtx {
    pub(crate) fn new(errs: Errs, tables: HashMap<TableRef, PgTableInfo>) -> Self {
        Self {
            tables: tables,
            errs: errs,
            rust_arg_lookup: Default::default(),
            rust_args: Default::default(),
            query_args: Default::default(),
        }
    }
}

pub trait QueryBody: dyn_clone::DynClone + std::fmt::Debug {
    fn build(
        &self,
        ctx: &mut PgQueryCtx,
        path: &rpds::Vector<String>,
        res_count: QueryResCount,
    ) -> (ExprType, Tokens);
}

clone_trait_object!(QueryBody);

pub fn build_set(
    ctx: &mut PgQueryCtx,
    path: &rpds::Vector<String>,
    scope: &HashMap<ExprValName, Type>,
    out: &mut Tokens,
    values: &Vec<(FieldRef, Expr)>,
) {
    out.s("set");
    for (i, (field, val)) in values.iter().enumerate() {
        let path = path.push_back(format!("Set field {}", i));
        if i > 0 {
            out.s(",");
        }
        let field_info =
            match ctx.tables.get(&TableRef(field.table_id.clone())).and_then(|t| t.fields.get(&field)) {
                Some(t) => t.clone(),
                None => {
                    ctx.errs.err(&path, format!("Set field {:?} is not known", field));
                    continue;
                },
            };
        out.id(&field_info.sql_name).s("=");
        let res = val.build(ctx, &path, &scope);
        check_assignable(&mut ctx.errs, &path, &field_info.type_, &res.0);
        out.s(&res.1.to_string());
    }
}

pub fn build_returning(
    ctx: &mut PgQueryCtx,
    path: &rpds::Vector<String>,
    scope: &HashMap<ExprValName, Type>,
    out: &mut Tokens,
    outputs: &Vec<Returning>,
    res_count: QueryResCount,
) -> ExprType {
    if !outputs.is_empty() {
        out.s("returning");
    }
    build_returning_values(ctx, path, scope, out, outputs, res_count)
}

pub fn build_returning_values(
    ctx: &mut PgQueryCtx,
    path: &rpds::Vector<String>,
    scope: &HashMap<ExprValName, Type>,
    out: &mut Tokens,
    outputs: &Vec<Returning>,
    res_count: QueryResCount,
) -> ExprType {
    let mut fields = vec![];
    for (i, r) in outputs.iter().enumerate() {
        if i > 0 {
            out.s(",");
        }
        let (t, tokens) = r.e.build(ctx, &path.push_back(format!("Returning {}", i)), scope);
        let t = match t.assert_scalar(&mut ctx.errs, &path.push_back(format!("Returning {}", i))) {
            Some(t) => t,
            None => {
                continue;
            },
        };
        let mut name = ExprValName::empty();
        out.s(&tokens.to_string());
        if let Some(s) = &r.rename {
            out.s("as").id(s);
            name.id = s.clone();
        } else {
            match &r.e {
                Expr::Field(f) => {
                    name = ExprValName::field(f);
                },
                _ => { },
            }
        }
        fields.push((name, t));
    }
    match res_count {
        QueryResCount::None => {
            if !fields.is_empty() {
                ctx.errs.err(path, format!("Query has returning values but result count is None"));
            }
        },
        QueryResCount::MaybeOne | QueryResCount::One | QueryResCount::Many => {
            if fields.is_empty() {
                ctx.errs.err(path, format!("Query has no returning values but result count is {:?}", res_count));
            }
        },
    }
    return ExprType(fields);
}
