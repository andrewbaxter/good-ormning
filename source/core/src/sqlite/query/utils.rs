use {
    crate::{
        sqlite::{
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
        Binding,
        Expr,
        ExprType,
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
    pub body_junctions: Vec<crate::sqlite::query::select_body::SelectJunction>,
}

pub struct CteBuilder {
    table_id: String,
    columns: Vec<(String, String, Type)>,
    body: Box<dyn QueryBody>,
    body_junctions: Vec<crate::sqlite::query::select_body::SelectJunction>,
}

impl CteBuilder {
    pub fn new(id: impl AsRef<str>, body: Box<dyn QueryBody>) -> Self {
        let table_id = id.as_ref().to_string();
        return Self {
            table_id: table_id,
            columns: vec![],
            body: body,
            body_junctions: vec![],
        };
    }

    pub fn body_junction(&mut self, j: crate::sqlite::query::select_body::SelectJunction) {
        self.body_junctions.push(j);
    }

    pub fn field(&mut self, id: impl AsRef<str>, type_: Type) -> (String, String, Type) {
        let field_id = id.as_ref().to_string();
        let f = (field_id.clone(), field_id, type_);
        self.columns.push(f.clone());
        return f;
    }

    pub fn column(mut self, id: impl AsRef<str>, type_: Type) -> Self {
        let field_id = id.as_ref().to_string();
        self.columns.push((field_id.clone(), field_id, type_));
        return self;
    }

    pub fn build(self) -> Cte {
        return Cte {
            table_id: self.table_id,
            columns: self.columns,
            body: self.body,
            body_junctions: self.body_junctions,
        };
    }
}

impl From<CteBuilder> for Cte {
    fn from(builder: CteBuilder) -> Self {
        return builder.build();
    }
}

pub fn build_with(ctx: &mut SqliteQueryCtx, path: &rpds::Vector<String>, with: &With) -> Tokens {
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
                check_assignable(&mut ctx.errs, &path, want, &ExprType(vec![(Binding::empty(), got.clone())]));
            }
        }
        out.s(&body_tokens.to_string());
        for (i, j) in cte.body_junctions.iter().enumerate() {
            let path = path.push_back(format!("Junction clause {} - {:?}", i, j.op));
            match j.op {
                crate::sqlite::query::select_body::SelectJunctionOperator::Union => {
                    out.s("union");
                },
                crate::sqlite::query::select_body::SelectJunctionOperator::UnionAll => {
                    out.s("union all");
                },
                crate::sqlite::query::select_body::SelectJunctionOperator::Intersect => {
                    out.s("intersect");
                },
                crate::sqlite::query::select_body::SelectJunctionOperator::Except => {
                    out.s("except");
                },
            }
            let (j_body_type, j_body_tokens) = j.body.build(ctx, &path, QueryResCount::Many);
            if j_body_type.0.len() != cte.columns.len() {
                ctx
                    .errs
                    .err(
                        &path,
                        format!(
                            "Select returns {} columns but the CTE needs exactly {} columns",
                            j_body_type.0.len(),
                            cte.columns.len()
                        ),
                    );
            } else {
                for (
                    i,
                    ((_, got), (_, _, want)),
                ) in Iterator::zip(j_body_type.0.iter(), cte.columns.iter()).enumerate() {
                    let path = path.push_back(format!("Select return {}", i));
                    check_assignable(&mut ctx.errs, &path, want, &ExprType(vec![(Binding::empty(), got.clone())]));
                }
            }
            out.s(&j_body_tokens.to_string());
        }
        out.s(")");
        let mut fields = HashMap::new();
        for (field_id, sql_name, type_) in &cte.columns {
            fields.insert(FieldRef {
                table_id: cte.table_id.clone(),
                field_id: field_id.clone(),
            }, SqliteFieldInfo {
                sql_name: sql_name.clone(),
                type_: type_.clone(),
            });
        }
        ctx.tables.insert(TableRef(cte.table_id.clone()), SqliteTableInfo {
            sql_name: cte.table_id.clone(),
            fields,
        });
    }
    return out;
}

#[derive(Clone, Debug)]
pub struct SqliteFieldInfo {
    pub sql_name: String,
    pub type_: Type,
}

#[derive(Clone, Debug)]
pub struct SqliteTableInfo {
    pub sql_name: String,
    pub fields: HashMap<FieldRef, SqliteFieldInfo>,
}

pub struct SqliteQueryCtx {
    pub tables: HashMap<TableRef, SqliteTableInfo>,
    pub errs: Errs,
    pub rust_arg_lookup: HashMap<String, (usize, Type)>,
    pub rust_args: Vec<TokenStream>,
    pub query_args: Vec<TokenStream>,
    pub op_stack: Vec<BinOp>,
    pub outer_scopes: Vec<HashMap<Binding, Type>>,
}

#[derive(Clone, Debug)]
pub struct Returning {
    pub e: Expr,
    pub rename: Option<String>,
}

impl SqliteQueryCtx {
    pub fn new(errs: Errs, tables: HashMap<TableRef, SqliteTableInfo>) -> Self {
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

pub trait QueryBody: dyn_clone::DynClone + std::fmt::Debug {
    fn build(
        &self,
        ctx: &mut SqliteQueryCtx,
        path: &rpds::Vector<String>,
        res_count: QueryResCount,
    ) -> (ExprType, Tokens);
}

clone_trait_object!(QueryBody);

pub fn build_set(
    ctx: &mut SqliteQueryCtx,
    path: &rpds::Vector<String>,
    scope: &HashMap<Binding, Type>,
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

pub fn build_returning(
    ctx: &mut SqliteQueryCtx,
    path: &rpds::Vector<String>,
    scope: &HashMap<Binding, Type>,
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
    ctx: &mut SqliteQueryCtx,
    path: &rpds::Vector<String>,
    scope: &HashMap<Binding, Type>,
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
        let mut name = Binding::empty();
        out.s(&tokens.to_string());
        if let Some(s) = &r.rename {
            out.s("as").id(s);
            name.id = s.clone();
        } else {
            if let Expr::Field(f) = &r.e {
                name = Binding::field(f);
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
