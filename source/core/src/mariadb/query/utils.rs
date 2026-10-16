use {
    crate::{
        mariadb::{
            QueryResCount,
            schema::{
                field::FieldRef,
                table::TableRef,
            },
            types::{
                SimpleSimpleType,
                Type,
            },
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
        Expr,
        ExprParent,
        ExprType,
        ExprValName,
        check_assignable,
    },
};

clone_trait_object!(QueryBody);

pub trait QueryBody: dyn_clone::DynClone + std::fmt::Debug {
    fn build(
        &self,
        ctx: &mut MariadbQueryCtx,
        path: &rpds::Vector<String>,
        res_count: QueryResCount,
    ) -> (ExprType, Tokens);
}

pub fn build_returning(
    ctx: &mut MariadbQueryCtx,
    path: &rpds::Vector<String>,
    scope: &HashMap<ExprValName, Type>,
    out: &mut Tokens,
    outputs: &[Returning],
    res_count: QueryResCount,
) -> ExprType {
    if !outputs.is_empty() {
        out.s("returning");
    }
    return build_returning_values(ctx, path, scope, out, outputs, res_count);
}

pub fn build_returning_values(
    ctx: &mut MariadbQueryCtx,
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
    ctx: &mut MariadbQueryCtx,
    path: &rpds::Vector<String>,
    scope: &HashMap<ExprValName, Type>,
    out: &mut Tokens,
    values: &[(FieldRef, Expr)],
    qualify: bool,
) {
    for (i, (field, val)) in values.iter().enumerate() {
        let path = path.push_back(format!("Set field {}", i));
        if i > 0 {
            out.s(",");
        }
        let alias_table = ctx.table_aliases.get(&field.table_id).cloned();
        let table_ref = alias_table.clone().unwrap_or_else(|| TableRef(field.table_id.clone()));
        let Some(table_info) = ctx.tables.get(&table_ref).cloned() else {
            ctx.errs.err(&path, format!("Set field {:?} is in an unknown table", field));
            continue;
        };
        let Some(field_info) = table_info.fields.get(&FieldRef {
            table_id: table_ref.0.clone(),
            field_id: field.field_id.clone(),
        }).cloned() else {
            ctx.errs.err(&path, format!("Set field {:?} is not known", field));
            continue;
        };
        if qualify {
            if alias_table.is_some() {
                out.id(&field.table_id).s(".");
            } else {
                out.id(&table_info.sql_name).s(".");
            }
        }
        out.id(&field_info.sql_name).s("=");
        let res = val.build(ctx, &path, scope);
        check_assignable(&mut ctx.errs, &path, &field_info.type_, &res.0, Some(val));
        out.s(&res.1.to_string());
    }
}

pub fn check_limit_arg(ctx: &mut MariadbQueryCtx, path: &rpds::Vector<String>, e: &Expr) {
    if !matches!(
        e,
        Expr::Param { .. } | Expr::LitAnyInt(_) | Expr::LitI16(_) | Expr::LitI32(_) | Expr::LitI64(_) |
            Expr::LitU32(_) |
            Expr::LitU64(_)
    ) {
        ctx
            .errs
            .err(path, "MariaDB only accepts integer literals or parameters in `limit` and `offset`".to_string());
    }
}

pub fn check_limit_type(ctx: &mut MariadbQueryCtx, path: &rpds::Vector<String>, t: &ExprType) {
    let Some(t) = t.assert_scalar(&mut ctx.errs, path) else {
        return;
    };
    if !matches!(
        t.type_.type_,
        SimpleSimpleType::AnyInt | SimpleSimpleType::I8 | SimpleSimpleType::I16 | SimpleSimpleType::I32 |
            SimpleSimpleType::I64 |
            SimpleSimpleType::U8 |
            SimpleSimpleType::U16 |
            SimpleSimpleType::U32 |
            SimpleSimpleType::U64
    ) ||
        t.opt {
        ctx.errs.err(path, format!("Limits must be non-optional integers, but the expression has type {:?}", t));
    }
}

pub fn build_order_limit(
    ctx: &mut MariadbQueryCtx,
    path: &rpds::Vector<String>,
    scope: &HashMap<ExprValName, Type>,
    out: &mut Tokens,
    order: &[(Expr, super::select::Order)],
    limit: &Option<Expr>,
) {
    if !order.is_empty() {
        out.s("order by");
        for (i, o) in order.iter().enumerate() {
            let path = path.push_back(format!("Order by clause {}", i));
            if i > 0 {
                out.s(",");
            }
            let (_, o_tokens) = o.0.build(ctx, &path, scope);
            out.s(&o_tokens.to_string());
            match o.1 {
                super::select::Order::Asc => {
                    out.s("asc");
                },
                super::select::Order::Desc => {
                    out.s("desc");
                },
            }
        }
    }
    if let Some(l) = limit {
        check_limit_arg(ctx, path, l);
        out.s("limit");
        let path = path.push_back("Limit".into());
        let (limit_t, limit_tokens) = l.build(ctx, &path, scope);
        check_limit_type(ctx, &path, &limit_t);
        out.s(&limit_tokens.to_string());
    }
}

pub fn build_with(ctx: &mut MariadbQueryCtx, path: &rpds::Vector<String>, with: &With) -> Tokens {
    let mut out = Tokens::new_mariadb();
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
        if with.recursive {
            let real_errs = ctx.errs.clone();
            ctx.errs = Errs::new();
            let (first_type, _) = cte.body.build(ctx, &path, QueryResCount::Many);
            ctx.errs = real_errs;
            let mut fields = HashMap::new();
            for (i, (field_id, sql_name, want)) in cte.columns.iter().enumerate() {
                let type_ = match (want, first_type.0.get(i).map(|t| &t.1)) {
                    (Some(want), _) => want.clone(),
                    (None, Some(got)) => got.clone(),
                    (None, None) => {
                        continue;
                    },
                };
                fields.insert(FieldRef {
                    table_id: cte.table_id.clone(),
                    field_id: field_id.clone(),
                }, MariadbFieldInfo {
                    sql_name: sql_name.clone(),
                    type_: type_,
                    has_default: false,
                });
            }
            ctx.tables.insert(TableRef(cte.table_id.clone()), MariadbTableInfo {
                sql_name: cte.table_id.clone(),
                fields: fields,
            });
        }
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
                        None,
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
            }, MariadbFieldInfo {
                sql_name: sql_name.clone(),
                type_: type_,
                has_default: false,
            });
        }
        ctx.tables.insert(TableRef(cte.table_id.clone()), MariadbTableInfo {
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
pub struct MariadbFieldInfo {
    pub has_default: bool,
    pub sql_name: String,
    pub type_: Type,
}

pub const PARAM_MARKER: char = '\u{1}';
pub const PARAM_LIST: char = 'L';
pub const PARAM_LIST_DECIMAL: char = 'D';

pub struct MariadbQueryCtx {
    pub default_time_library: Option<crate::utils::TimeLibrary>,
    pub errs: Errs,
    pub expr_parent: ExprParent,
    pub in_duplicate_key_update: bool,
    pub list_params: std::collections::HashSet<String>,
    pub outer_scopes: Vec<HashMap<ExprValName, Type>>,
    pub query_args: Vec<TokenStream>,
    pub rust_arg_lookup: HashMap<String, (Type, TokenStream)>,
    pub rust_args: Vec<(String, TokenStream)>,
    pub table_aliases: HashMap<String, TableRef>,
    pub tables: HashMap<TableRef, MariadbTableInfo>,
}

impl MariadbQueryCtx {
    pub fn new(errs: Errs, tables: HashMap<TableRef, MariadbTableInfo>) -> Self {
        return Self {
            default_time_library: None,
            tables: tables,
            table_aliases: Default::default(),
            errs: errs,
            rust_arg_lookup: Default::default(),
            rust_args: Default::default(),
            query_args: Default::default(),
            expr_parent: ExprParent::Other,
            in_duplicate_key_update: false,
            list_params: Default::default(),
            outer_scopes: Default::default(),
        };
    }
}

#[derive(Clone, Debug)]
pub struct MariadbTableInfo {
    pub fields: HashMap<FieldRef, MariadbFieldInfo>,
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
