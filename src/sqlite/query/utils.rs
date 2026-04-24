use std::{
    collections::HashMap,
};
use proc_macro2::TokenStream;
use crate::{
    sqlite::{
        types::Type,
        QueryResCount,
        schema::{
            field::{
                Field,
                FieldRef,
            },
            table::{
                Table,
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
        Binding,
        Expr,
        check_assignable,
    },
};

pub struct SqliteFieldInfo {
    pub sql_name: String,
    pub type_: Type,
}

pub struct SqliteTableInfo {
    pub sql_name: String,
    pub fields: HashMap<FieldRef, SqliteFieldInfo>,
}

pub struct SqliteQueryCtx<'a> {
    pub(crate) tables: &'a HashMap<TableRef, SqliteTableInfo>,
    pub errs: Errs,
    pub(crate) rust_arg_lookup: HashMap<String, (usize, Type)>,
    pub(crate) rust_args: Vec<TokenStream>,
    pub(crate) query_args: Vec<TokenStream>,
}

#[derive(Clone, Debug)]
pub struct Returning {
    pub e: Expr,
    pub rename: Option<String>,
}

impl<'a> SqliteQueryCtx<'a> {
    pub(crate) fn new(errs: Errs, tables: &'a HashMap<TableRef, SqliteTableInfo>) -> Self {
        Self {
            tables: tables,
            errs: errs,
            rust_arg_lookup: Default::default(),
            rust_args: Default::default(),
            query_args: Default::default(),
        }
    }
}

pub trait QueryBody {
    fn build(
        &self,
        ctx: &mut SqliteQueryCtx,
        path: &rpds::Vector<String>,
        res_count: QueryResCount,
    ) -> (ExprType, Tokens);
}

pub fn build_set(
    ctx: &mut SqliteQueryCtx,
    path: &rpds::Vector<String>,
    scope: &HashMap<Binding, Type>,
    out: &mut Tokens,
    values: &Vec<(FieldRef, Expr)>,
) {
    out.s("set");
    for (i, (field, val)) in values.iter().enumerate() {
        let path = path.push_back(format!("Set field {}", i));
        if i > 0 {
            out.s(",");
        }
        let field_info = match ctx.tables.get(&TableRef(field.table_id.clone())).and_then(|t| t.fields.get(&field)) {
            Some(t) => t,
            None => {
                ctx.errs.err(&path, format!("Update destination value field {:?} is not known", field));
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
    ctx: &mut SqliteQueryCtx,
    path: &rpds::Vector<String>,
    scope: &HashMap<Binding, Type>,
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
    ctx: &mut SqliteQueryCtx,
    path: &rpds::Vector<String>,
    scope: &HashMap<Binding, Type>,
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
        let mut name = Binding::empty();
        out.s(&tokens.to_string());
        if let Some(s) = &r.rename {
            out.s("as").id(s);
            name.id = s.clone();
        } else {
            match &r.e {
                Expr::Field(f) => {
                    name = Binding::field(f);
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
