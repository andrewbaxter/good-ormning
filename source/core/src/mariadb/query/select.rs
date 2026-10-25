use {
    crate::{
        mariadb::{
            QueryResCount,
            query::{
                expr::{
                    Expr,
                    ExprType,
                    ExprValName,
                    WindowSpec,
                    check_bool,
                    check_general_same,
                },
                utils::{
                    MariadbQueryCtx,
                    QueryBody,
                    Returning,
                    With,
                    build_returning_values,
                    build_with,
                },
            },
            schema::table::TableRef,
            types::Type,
        },
        utils::Tokens,
    },
    std::collections::{
        HashMap,
        HashSet,
    },
};

#[derive(Clone, Debug)]
pub struct Join {
    pub condition: Option<JoinCondition>,
    pub source: Box<NamedSelectSource>,
    pub type_: JoinType,
}

#[derive(Clone, Debug)]
pub enum JoinCondition {
    Natural,
    On(Expr),
    Using(Vec<String>),
}

#[derive(Clone, Debug)]
pub enum JoinSource {
    Empty,
    Subsel(Box<Select>),
    Table(TableRef),
    Unnest {
        param: String,
        types: Vec<Type>,
        with_ordinality: bool,
        columns: Vec<String>,
    },
}

#[derive(Clone, Debug)]
pub enum JoinType {
    Comma,
    Cross,
    Inner,
    Left,
    Right,
}

pub fn build_joins(
    ctx: &mut MariadbQueryCtx,
    path: &rpds::Vector<String>,
    joins: &[Join],
    scope: &mut HashMap<ExprValName, Type>,
) -> Tokens {
    let mut out = Tokens::new_mariadb();
    let mut hidden = HashSet::new();
    for (i, je) in joins.iter().enumerate() {
        let path = path.push_back(format!("Join {}", i));
        if matches!(je.type_, JoinType::Comma) {
            hidden.extend(scope.keys().cloned());
        }
        if matches!(je.condition, Some(JoinCondition::Natural)) {
            if matches!(je.type_, JoinType::Comma | JoinType::Cross) {
                ctx.errs.err(&path, "Natural joins can't be comma or cross joins".to_string());
            }
            out.s("natural");
        }
        match je.type_ {
            JoinType::Comma => out.s(","),
            JoinType::Inner => out.s(if matches!(je.condition, Some(JoinCondition::Natural)) {
                "join"
            } else {
                "inner join"
            }),
            JoinType::Left => out.s("left join"),
            JoinType::Right => out.s("right join"),
            JoinType::Cross => out.s("cross join"),
        };
        if matches!(je.type_, JoinType::Left | JoinType::Right) && je.condition.is_none() {
            ctx.errs.err(&path, "MariaDB requires `on` or `using` for left and right joins".to_string());
        }
        let source: (ExprType, Tokens) = je.source.build(ctx, &path);
        out.s(&source.1.to_string());
        let mut merged = vec![];
        match &je.condition {
            Some(JoinCondition::Using(cols)) => {
                for col in cols {
                    let left =
                        scope
                            .keys()
                            .find(|k| k.id == *col && !k.table_id.is_empty() && !hidden.contains(*k))
                            .cloned();
                    let right = source.0.0.iter().find(|(k, _)| k.id == *col).map(|(k, _)| k.clone());
                    match (left, right) {
                        (Some(left), Some(right)) => merged.push((col.clone(), left, right)),
                        (None, _) => {
                            ctx
                                .errs
                                .err(&path, format!("Join `using` column {} isn't in any table on the left", col));
                        },
                        (_, None) => {
                            ctx.errs.err(&path, format!("Join `using` column {} isn't in the joined table", col));
                        },
                    }
                }
            },
            Some(JoinCondition::Natural) => {
                for (right, _) in &source.0.0 {
                    if let Some(left) =
                        scope
                            .keys()
                            .find(|k| k.id == right.id && !k.table_id.is_empty() && !hidden.contains(*k))
                            .cloned() {
                        merged.push((right.id.clone(), left, right.clone()));
                    }
                }
            },
            _ => { },
        }
        match je.type_ {
            JoinType::Comma | JoinType::Inner | JoinType::Cross => {
                for (k, v) in source.0.0 {
                    scope.insert(k, v);
                }
            },
            JoinType::Left => {
                for (k, mut v) in source.0.0 {
                    v.opt = true;
                    scope.insert(k, v);
                }
            },
            JoinType::Right => {
                for v in scope.values_mut() {
                    v.opt = true;
                }
                for (k, v) in source.0.0 {
                    scope.insert(k, v);
                }
            },
        }
        for (col, left, right) in merged {
            let t = if matches!(je.type_, JoinType::Right) {
                scope.get(&right).cloned()
            } else {
                scope.get(&left).cloned()
            };
            if let Some(t) = t {
                scope.insert(ExprValName::local(col), t);
            }
        }
        match &je.condition {
            Some(JoinCondition::On(on)) => {
                out.s("on");
                let on_scope =
                    scope
                        .iter()
                        .filter(|(k, _)| !hidden.contains(*k))
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect::<HashMap<_, _>>();
                let (on_t, on_tokens): (ExprType, Tokens) =
                    on.build(ctx, &path.push_back("On".to_string()), &on_scope);
                check_bool(ctx, &path, &on_t);
                out.s(&on_tokens.to_string());
            },
            Some(JoinCondition::Using(cols)) => {
                out.s("using (");
                for (i, col) in cols.iter().enumerate() {
                    if i > 0 {
                        out.s(",");
                    }
                    out.id(col);
                }
                out.s(")");
            },
            Some(JoinCondition::Natural) | None => { },
        }
    }
    return out;
}

#[derive(Clone, Debug)]
pub struct NamedSelectSource {
    pub alias: Option<String>,
    pub source: JoinSource,
}

impl NamedSelectSource {
    pub fn build(&self, ctx: &mut MariadbQueryCtx, path: &rpds::Vector<String>) -> (ExprType, Tokens) {
        let mut out = Tokens::new_mariadb();
        let mut new_fields: Vec<(ExprValName, Type)> = match &self.source {
            JoinSource::Subsel(s) => {
                if self.alias.is_none() {
                    ctx.errs.err(path, "MariaDB requires derived tables to have an alias".to_string());
                }
                let res: (ExprType, Tokens) =
                    s.build(ctx, &path.push_back("From subselect".to_string()), QueryResCount::Many);
                out.s("(").s(&res.1.to_string()).s(")");
                res.0.0.clone()
            },
            JoinSource::Table(s) => {
                let table_info = match ctx.tables.get(s) {
                    Some(f) => f,
                    None => {
                        ctx
                            .errs
                            .err(
                                &path.push_back("From".to_string()),
                                format!("No table with id {:?} in version", s),
                            );
                        return (ExprType(vec![]), Tokens::new_mariadb());
                    },
                };
                out.id(&table_info.sql_name);
                table_info.fields.iter().map(|(id, info)| (ExprValName::field(id), info.type_.clone())).collect()
            },
            JoinSource::Empty => {
                vec![]
            },
            JoinSource::Unnest { param, types, with_ordinality, columns } => {
                let path = path.push_back(format!("Unnest ({})", param));
                let Some(alias) = &self.alias else {
                    ctx.errs.err(&path, "MariaDB requires derived tables to have an alias".to_string());
                    return (ExprType(vec![]), Tokens::new_mariadb());
                };
                let Some(values) = super::expr::list_param_values(ctx, &path, param, types) else {
                    return (ExprType(vec![]), Tokens::new_mariadb());
                };
                let mut types = types.clone();
                let mut names = if types.len() == 1 {
                    vec![alias.clone()]
                } else {
                    vec![
                        "unnest".to_string();
                        types.len()
                    ]
                };
                if *with_ordinality {
                    types.push(Type {
                        opt: false,
                        type_: crate::mariadb::types::SimpleType {
                            custom: None,
                            type_: crate::mariadb::types::SimpleSimpleType::I64,
                        },
                    });
                    names.push("ordinality".to_string());
                }
                if columns.len() > names.len() {
                    ctx
                        .errs
                        .err(
                            &path,
                            format!("unnest has {} columns but {} aliases were given", names.len(), columns.len()),
                        );
                }
                for (n, c) in names.iter_mut().zip(columns) {
                    *n = c.clone();
                }
                let mut seen = HashSet::new();
                for n in &names {
                    if !seen.insert(n) {
                        ctx
                            .errs
                            .err(
                                &path,
                                format!("unnest has multiple columns named {:?}, give them column aliases", n),
                            );
                    }
                }
                let templates = super::expr::list_param_templates(&types[..types.len() - *with_ordinality as usize]);
                let with_ordinality = *with_ordinality;
                ctx.query_args.push(quote::quote!(#values, &[#(#templates), *], &[#(#names), *], #with_ordinality));
                out.s(
                    &format!(
                        "{}{}{}{}",
                        super::utils::PARAM_MARKER,
                        super::utils::PARAM_TABLE,
                        ctx.query_args.len() - 1,
                        super::utils::PARAM_MARKER
                    ),
                );
                names.into_iter().zip(types).map(|(n, t)| (ExprValName {
                    table_id: "".to_string(),
                    id: n,
                }, t)).collect()
            },
        };
        if let Some(s) = &self.alias {
            out.s("as").id(s);
            if let JoinSource::Table(tr) = &self.source {
                ctx.table_aliases.insert(s.clone(), tr.clone());
            }
            let mut new_fields2 = vec![];
            for (k, v) in new_fields {
                new_fields2.push((k.with_alias(s), v));
            }
            new_fields = new_fields2;
        }
        return (ExprType(new_fields), out);
    }
}

#[derive(Clone, Debug)]
pub enum Order {
    Asc,
    Desc,
}

#[derive(Clone, Debug)]
pub struct SelectFetch {
    pub quantity: Option<Expr>,
    pub with_ties: bool,
}

#[derive(Clone, Debug)]
pub enum SelectLock {
    ForUpdate,
    ForUpdateNowait,
    ForUpdateSkipLocked,
}

#[derive(Clone, Debug)]
pub struct Select {
    pub distinct: bool,
    pub fetch: Option<SelectFetch>,
    pub first: Option<Box<Select>>,
    pub group: Vec<Expr>,
    pub having: Option<Expr>,
    pub join: Vec<Join>,
    pub junctions: Vec<super::select_body::SelectJunction>,
    pub limit: Option<Expr>,
    pub lock: Option<SelectLock>,
    pub offset: Option<Expr>,
    pub order: Vec<(Expr, Order)>,
    pub returning: Vec<Returning>,
    pub table: NamedSelectSource,
    pub where_: Option<Expr>,
    pub windows: Vec<(String, WindowSpec)>,
    pub with: Option<With>,
}

impl QueryBody for Select {
    fn build(
        &self,
        ctx: &mut MariadbQueryCtx,
        path: &rpds::Vector<String>,
        res_count: QueryResCount,
    ) -> (ExprType, Tokens) {
        let with_tokens = self.with.as_ref().map(|with| build_with(ctx, path, with));
        let mut out = Tokens::new_mariadb();
        let (mut out_type, scope) = match &self.first {
            Some(first) => {
                if !matches!(self.table.source, JoinSource::Empty) || !self.returning.is_empty() {
                    ctx
                        .errs
                        .err(path, "A select with a first query can't have its own source or outputs".to_string());
                }
                let (first_type, first_tokens) =
                    first.build(ctx, &path.push_back("First".into()), QueryResCount::Many);
                out.s("(").s(&first_tokens.to_string()).s(")");
                (first_type, HashMap::new())
            },
            None => {
                let source: (ExprType, Tokens) = self.table.build(ctx, path);
                let mut scope = HashMap::new();
                for (k, v) in source.0.0 {
                    scope.insert(k, v);
                }
                let joins = build_joins(ctx, path, &self.join, &mut scope);
                out.s("select");
                if self.distinct {
                    out.s("distinct");
                }
                if self.returning.is_empty() {
                    ctx.errs.err(path, "Select must have at least one output, but outputs are empty".to_string());
                }
                let out_type = build_returning_values(ctx, path, &scope, &mut out, &self.returning, res_count);
                if !matches!(self.table.source, JoinSource::Empty) {
                    out.s("from");
                    out.s(&source.1.to_string()).s(&joins.to_string());
                }
                if let Some(where_) = &self.where_ {
                    out.s("where");
                    let path = path.push_back("Where".into());
                    let (where_t, where_tokens): (ExprType, Tokens) = where_.build(ctx, &path, &scope);
                    check_bool(ctx, &path, &where_t);
                    out.s(&where_tokens.to_string());
                }
                let mut group_scope = scope.clone();
                for (k, v) in &out_type.0 {
                    if !k.id.is_empty() && !scope.keys().any(|s| s.id == k.id) {
                        group_scope.insert(ExprValName::local(k.id.clone()), v.clone());
                    }
                }
                if !self.group.is_empty() {
                    out.s("group by");
                    for (i, g) in self.group.iter().enumerate() {
                        let path = path.push_back(format!("Group by clause {}", i));
                        if i > 0 {
                            out.s(",");
                        }
                        let (_, g_tokens): (ExprType, Tokens) = g.build(ctx, &path, &group_scope);
                        out.s(&g_tokens.to_string());
                    }
                }
                if let Some(having) = &self.having {
                    out.s("having");
                    let path = path.push_back("Having".into());
                    let (having_t, having_tokens): (ExprType, Tokens) = having.build(ctx, &path, &group_scope);
                    check_bool(ctx, &path, &having_t);
                    out.s(&having_tokens.to_string());
                }
                if !self.windows.is_empty() {
                    out.s("window");
                    for (i, (name, spec)) in self.windows.iter().enumerate() {
                        if i > 0 {
                            out.s(",");
                        }
                        let spec_tokens = spec.build(ctx, &path.push_back(format!("Window {}", name)), &scope);
                        out.id(name).s("as (").s(&spec_tokens.to_string()).s(")");
                    }
                }
                (out_type, scope)
            },
        };
        let mut order_scope = scope.clone();
        for (k, v) in &out_type.0 {
            if !k.id.is_empty() {
                order_scope.insert(ExprValName::local(k.id.clone()), v.clone());
            }
        }
        for (i, j) in self.junctions.iter().enumerate() {
            let path = path.push_back(format!("Junction {}", i));
            match j.op {
                super::select_body::SelectJunctionOperator::Union => out.s("union"),
                super::select_body::SelectJunctionOperator::UnionAll => out.s("union all"),
                super::select_body::SelectJunctionOperator::Intersect => out.s("intersect"),
                super::select_body::SelectJunctionOperator::IntersectAll => out.s("intersect all"),
                super::select_body::SelectJunctionOperator::Except => out.s("except"),
                super::select_body::SelectJunctionOperator::ExceptAll => out.s("except all"),
            };
            let (j_body_type, j_body_tokens) = j.body.build(ctx, &path, QueryResCount::Many);
            out_type = check_general_same(ctx, &path, &out_type, &j_body_type);
            out.s("(").s(&j_body_tokens.to_string()).s(")");
        }
        if !self.order.is_empty() {
            out.s("order by");
            for (i, o) in self.order.iter().enumerate() {
                let path = path.push_back(format!("Order by clause {}", i));
                if i > 0 {
                    out.s(",");
                }
                let (_, o_tokens): (ExprType, Tokens) = o.0.build(ctx, &path, &order_scope);
                out.s(&o_tokens.to_string());
                match o.1 {
                    Order::Asc => {
                        out.s("asc");
                    },
                    Order::Desc => {
                        out.s("desc");
                    },
                }
            }
        }
        for (clause, e) in [("limit", &self.limit), ("offset", &self.offset)] {
            let Some(e) = e else {
                continue;
            };
            super::utils::check_limit_arg(ctx, path, e);
            out.s(clause);
            let path = path.push_back(clause.to_string());
            let (t, tokens): (ExprType, Tokens) = e.build(ctx, &path, &scope);
            super::utils::check_limit_type(ctx, &path, &t);
            out.s(&tokens.to_string());
            if clause == "offset" && self.limit.is_none() {
                out.s("rows");
            }
        }
        if let Some(fetch) = &self.fetch {
            if self.limit.is_some() {
                ctx.errs.err(path, "A select can't have both `limit` and `fetch`".to_string());
            }
            if fetch.with_ties && self.order.is_empty() {
                ctx.errs.err(path, "`fetch ... with ties` requires `order by`".to_string());
            }
            out.s("fetch first");
            if let Some(quantity) = &fetch.quantity {
                super::utils::check_limit_arg(ctx, path, quantity);
                let path = path.push_back("Fetch".into());
                let (t, tokens) = quantity.build(ctx, &path, &scope);
                super::utils::check_limit_type(ctx, &path, &t);
                out.s(&tokens.to_string());
            }
            out.s(if fetch.with_ties {
                "rows with ties"
            } else {
                "rows only"
            });
        }
        match self.lock {
            Some(SelectLock::ForUpdate) => {
                out.s("for update");
            },
            Some(SelectLock::ForUpdateNowait) => {
                out.s("for update nowait");
            },
            Some(SelectLock::ForUpdateSkipLocked) => {
                out.s("for update skip locked");
            },
            None => { },
        }
        if let Some(with_tokens) = with_tokens {
            let mut wrapped = Tokens::new_mariadb();
            wrapped.s(&with_tokens.to_string()).s(&out.to_string());
            out = wrapped;
        }
        return (out_type, out);
    }
}
