use {
    crate::{
        pg::{
            QueryResCount,
            schema::{
                field::FieldRef,
                table::TableRef,
            },
            types::Type,
        },
        utils::{
            Errs,
            Tokens,
        },
    },
    dyn_clone::clone_trait_object,
    proc_macro2::TokenStream,
    std::collections::HashMap,
    super::expr::{
        BinOp,
        Expr,
        ExprType,
        ExprValName,
        check_assignable,
    },
};

clone_trait_object!(QueryBody);

pub trait QueryBody: dyn_clone::DynClone + std::fmt::Debug {
    fn build(
        &self,
        ctx: &mut PgQueryCtx,
        path: &rpds::Vector<String>,
        res_count: QueryResCount,
    ) -> (ExprType, Tokens);
}

pub fn build_returning(
    ctx: &mut PgQueryCtx,
    path: &rpds::Vector<String>,
    scope: &HashMap<ExprValName, Type>,
    out: &mut Tokens,
    outputs: &[Returning],
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
    outputs: &[Returning],
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
            if let Expr::Field(f) = &r.e {
                name = ExprValName::field(f);
            }
        }
        fields.push((name, t));
    }
    match res_count {
        QueryResCount::None => {
            if !fields.is_empty() {
                ctx.errs.err(path, "Query has returning values but result count is None".to_string());
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

pub fn build_set(
    ctx: &mut PgQueryCtx,
    path: &rpds::Vector<String>,
    scope: &HashMap<ExprValName, Type>,
    out: &mut Tokens,
    values: &[(FieldRef, Expr)],
) {
    out.s("set");
    for (i, (field, val)) in values.iter().enumerate() {
        let path = path.push_back(format!("Set field {}", i));
        if i > 0 {
            out.s(",");
        }
        let field_info =
            match ctx.tables.get(&TableRef(field.table_id.clone())).and_then(|t| t.fields.get(field)) {
                Some(t) => t.clone(),
                None => {
                    ctx.errs.err(&path, format!("Set field {:?} is not known", field));
                    continue;
                },
            };
        out.id(&field_info.sql_name).s("=");
        let res = val.build(ctx, &path, scope);
        check_assignable(&mut ctx.errs, &path, &field_info.type_, &res.0);
        out.s(&res.1.to_string());
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
        }
        let mut fields = HashMap::new();
        let mut column_types = vec![];
        for (i, (field_id, sql_name, want)) in cte.columns.iter().enumerate() {
            let got = body_type.0.get(i).map(|t| &t.1);
            let type_ = match (want, got) {
                (Some(want), Some(got)) => {
                    let path = path.push_back(format!("Select return {}", i));
                    check_assignable(
                        &mut ctx.errs,
                        &path,
                        want,
                        &ExprType(vec![(ExprValName::empty(), got.clone())]),
                    );
                    want.clone()
                },
                (Some(want), None) => want.clone(),
                (None, Some(got)) => got.clone(),
                (None, None) => {
                    continue;
                },
            };
            column_types.push(type_.clone());
            fields.insert(FieldRef {
                table_id: cte.table_id.clone(),
                field_id: field_id.clone(),
            }, PgFieldInfo {
                sql_name: sql_name.clone(),
                type_: type_,
            });
        }
        ctx.tables.insert(TableRef(cte.table_id.clone()), PgTableInfo {
            sql_name: cte.table_id.clone(),
            fields: fields,
        });
        out.s(&body_tokens.to_string());
        out.s(")");
    }
    return out;
}

#[derive(Clone, Debug)]
pub struct Cte {
    pub body: Box<dyn QueryBody>,
    pub columns: Vec<(String, String, Option<Type>)>,
    pub table_id: String,
}

impl From<CteBuilder> for Cte {
    fn from(builder: CteBuilder) -> Self {
        return builder.build();
    }
}

pub struct CteBuilder {
    body: Box<dyn QueryBody>,
    columns: Vec<(String, String, Option<Type>)>,
    table_id: String,
}

impl CteBuilder {
    pub fn build(self) -> Cte {
        return Cte {
            table_id: self.table_id,
            columns: self.columns,
            body: self.body,
        };
    }

    pub fn column(mut self, id: impl AsRef<str>, type_: Type) -> Self {
        let field_id = id.as_ref().to_string();
        self.columns.push((field_id.clone(), field_id, Some(type_)));
        return self;
    }

    pub fn column_inferred(mut self, id: impl AsRef<str>) -> Self {
        let field_id = id.as_ref().to_string();
        self.columns.push((field_id.clone(), field_id, None));
        return self;
    }

    pub fn field(&mut self, id: impl AsRef<str>, type_: Type) -> (String, String, Type) {
        let field_id = id.as_ref().to_string();
        self.columns.push((field_id.clone(), field_id.clone(), Some(type_.clone())));
        return (field_id.clone(), field_id, type_);
    }

    pub fn new(id: impl AsRef<str>, body: Box<dyn QueryBody>) -> Self {
        let table_id = id.as_ref().to_string();
        return Self {
            table_id: table_id,
            columns: vec![],
            body: body,
        };
    }
}

#[derive(Clone, Debug)]
pub struct PgFieldInfo {
    pub sql_name: String,
    pub type_: Type,
}

pub struct PgQueryCtx {
    pub errs: Errs,
    pub op_stack: Vec<BinOp>,
    pub outer_scopes: Vec<HashMap<ExprValName, Type>>,
    pub query_args: Vec<TokenStream>,
    pub rust_arg_lookup: HashMap<String, (usize, Type)>,
    pub rust_args: Vec<TokenStream>,
    pub tables: HashMap<TableRef, PgTableInfo>,
}

impl PgQueryCtx {
    pub fn new(errs: Errs, tables: HashMap<TableRef, PgTableInfo>) -> Self {
        Self {
            tables: tables,
            errs: errs,
            rust_arg_lookup: Default::default(),
            rust_args: Default::default(),
            query_args: Default::default(),
            op_stack: Default::default(),
            outer_scopes: Default::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PgTableInfo {
    pub fields: HashMap<FieldRef, PgFieldInfo>,
    pub sql_name: String,
}

#[derive(Clone, Debug)]
pub struct Returning {
    pub e: Expr,
    pub rename: Option<String>,
}

#[derive(Clone, Debug)]
pub struct With {
    pub ctes: Vec<Cte>,
    pub recursive: bool,
}
