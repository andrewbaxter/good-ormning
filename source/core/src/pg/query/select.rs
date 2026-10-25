use {
    crate::{
        pg::{
            QueryResCount,
            query::{
                expr::{
                    Expr,
                    ExprType,
                    ExprValName,
                    check_bool,
                    check_general_same,
                },
                utils::{
                    PgQueryCtx,
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
    std::collections::HashMap,
};

#[derive(Clone, Debug)]
pub struct Join {
    pub on: Option<Expr>,
    pub source: Box<NamedSelectSource>,
    pub type_: JoinType,
}

#[derive(Clone, Debug)]
pub enum JoinSource {
    Empty,
    Func {
        name: String,
        args: Vec<Expr>,
        with_ordinality: bool,
        columns: Vec<String>,
    },
    Lateral(Box<Select>),
    Subsel(Box<Select>),
    Table(TableRef),
}

#[derive(Clone, Debug)]
pub enum JoinType {
    Cross,
    Full,
    Inner,
    Left,
    Right,
}

#[derive(Clone, Debug)]
pub struct NamedSelectSource {
    pub alias: Option<String>,
    pub source: JoinSource,
}

impl NamedSelectSource {
    pub fn build(&self, ctx: &mut PgQueryCtx, path: &rpds::Vector<String>) -> (ExprType, Tokens) {
        let mut out = Tokens::new();
        let mut new_fields: Vec<(ExprValName, Type)> = match &self.source {
            JoinSource::Subsel(s) | JoinSource::Lateral(s) => {
                let res: (ExprType, Tokens) =
                    s.build(ctx, &path.push_back("From subselect".to_string()), QueryResCount::Many);
                if matches!(self.source, JoinSource::Lateral(_)) {
                    out.s("lateral");
                }
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
                        return (ExprType(vec![]), Tokens::new());
                    },
                };
                out.id(&table_info.sql_name);
                table_info.fields.iter().map(|(id, info)| (ExprValName::field(id), info.type_.clone())).collect()
            },
            JoinSource::Func { name, args, with_ordinality, columns } => {
                out.id(name).s("(");
                let mut arg_types = vec![];
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        out.s(",");
                    }
                    let path = path.push_back(format!("Arg {}", i));
                    let (t, tokens) = arg.build(ctx, &path, &HashMap::new());
                    arg_types.push((path, t));
                    out.s(&tokens.to_string());
                }
                out.s(")");
                if *with_ordinality {
                    out.s("with ordinality");
                }
                'fields: {
                    let mut types = match name.as_str() {
                        "unnest" => {
                            let mut types = vec![];
                            for (path, t) in &arg_types {
                                let Some(t) = t.assert_scalar(&mut ctx.errs, path) else {
                                    break 'fields vec![];
                                };
                                if !t.arr {
                                    ctx
                                        .errs
                                        .err(path, format!("`unnest` arguments must be arrays, but got {:?}", t));
                                    break 'fields vec![];
                                }
                                types.push(Type {
                                    arr: false,
                                    ..t
                                });
                            }
                            types
                        },
                        "generate_series" => {
                            let Some((path, t)) = arg_types.first() else {
                                ctx.errs.err(path, "`generate_series` needs a start argument".to_string());
                                break 'fields vec![];
                            };
                            let Some(t) = t.assert_scalar(&mut ctx.errs, path) else {
                                break 'fields vec![];
                            };
                            vec![Type {
                                arr: false,
                                opt: false,
                                ..t
                            }]
                        },
                        _ => {
                            ctx
                                .errs
                                .err(
                                    path,
                                    format!(
                                        "Unsupported table function {}, its output columns aren't known (supported: unnest, generate_series)",
                                        name
                                    ),
                                );
                            break 'fields vec![];
                        },
                    };
                    let mut names = if types.len() == 1 {
                        vec![self.alias.as_deref().unwrap_or(name).to_string()]
                    } else {
                        vec![
                            name.to_string();
                            types.len()
                        ]
                    };
                    if *with_ordinality {
                        types.push(Type {
                            arr: false,
                            opt: false,
                            type_: crate::pg::types::SimpleType {
                                custom: None,
                                type_: crate::pg::types::SimpleSimpleType::I64,
                            },
                        });
                        names.push("ordinality".to_string());
                    }
                    if !columns.is_empty() {
                        if self.alias.is_none() {
                            ctx.errs.err(path, "Column aliases on a table function need a table alias".to_string());
                        }
                        if columns.len() > names.len() {
                            ctx
                                .errs
                                .err(
                                    path,
                                    format!(
                                        "Table function {} has {} columns but {} aliases were given",
                                        name,
                                        names.len(),
                                        columns.len()
                                    ),
                                );
                        }
                        for (n, c) in names.iter_mut().zip(columns) {
                            *n = c.clone();
                        }
                    }
                    let mut seen = std::collections::HashSet::new();
                    for n in &names {
                        if !seen.insert(n) {
                            ctx
                                .errs
                                .err(
                                    path,
                                    format!(
                                        "Table function {} has multiple columns named {:?}, give them column aliases",
                                        name,
                                        n
                                    ),
                                );
                        }
                    }
                    break 'fields names.into_iter().zip(types).map(|(n, t)| (ExprValName {
                        table_id: name.to_string(),
                        id: n,
                    }, t)).collect();
                }
            },
            JoinSource::Empty => {
                vec![]
            },
        };
        if let Some(s) = &self.alias {
            out.s("as").id(s);
            if let JoinSource::Func { columns, .. } = &self.source {
                if !columns.is_empty() {
                    out.s("(");
                    for (i, c) in columns.iter().enumerate() {
                        if i > 0 {
                            out.s(",");
                        }
                        out.id(c);
                    }
                    out.s(")");
                }
            }
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
    AscNullsFirst,
    AscNullsLast,
    DescNullsFirst,
    DescNullsLast,
}

impl Order {
    pub fn sql(&self) -> &'static str {
        return match self {
            Order::Asc => "asc",
            Order::Desc => "desc",
            Order::AscNullsFirst => "asc nulls first",
            Order::AscNullsLast => "asc nulls last",
            Order::DescNullsFirst => "desc nulls first",
            Order::DescNullsLast => "desc nulls last",
        };
    }
}

#[derive(Clone, Debug)]
pub struct SelectFetch {
    pub quantity: Option<Expr>,
    pub with_ties: bool,
}

#[derive(Clone, Debug)]
pub enum SelectLockStrength {
    Update,
    Share,
}

#[derive(Clone, Debug)]
pub enum SelectLockWait {
    Wait,
    Nowait,
    SkipLocked,
}

#[derive(Clone, Debug)]
pub struct SelectLock {
    pub strength: SelectLockStrength,
    pub of: Vec<String>,
    pub wait: SelectLockWait,
}

#[derive(Clone, Debug)]
pub struct Select {
    pub distinct: bool,
    /// `distinct on (exprs)`
    pub distinct_on: Vec<Expr>,
    pub fetch: Option<SelectFetch>,
    pub group: Vec<Expr>,
    pub having: Option<Expr>,
    pub join: Vec<Join>,
    pub junctions: Vec<super::select_body::SelectJunction>,
    pub limit: Option<Expr>,
    pub lock: Vec<SelectLock>,
    pub offset: Option<Expr>,
    pub order: Vec<(Expr, Order)>,
    pub returning: Vec<Returning>,
    pub table: NamedSelectSource,
    pub where_: Option<Expr>,
    pub with: Option<With>,
}

impl QueryBody for Select {
    fn build(
        &self,
        ctx: &mut PgQueryCtx,
        path: &rpds::Vector<String>,
        res_count: QueryResCount,
    ) -> (ExprType, Tokens) {
        let mut out = Tokens::new();
        if let Some(with) = &self.with {
            out.s(&build_with(ctx, path, with).to_string());
        }

        // Prep
        let source: (ExprType, Tokens) = self.table.build(ctx, path);
        let mut fields = HashMap::new();
        for (k, v) in source.0.0 {
            fields.insert(k, v);
        }
        let mut scope = fields.clone();
        let mut joins = vec![];
        for (i, je) in self.join.iter().enumerate() {
            let path = path.push_back(format!("Join {}", i));
            let mut out = Tokens::new();
            match je.type_ {
                JoinType::Inner => out.s("inner"),
                JoinType::Left => out.s("left"),
                JoinType::Right => out.s("right"),
                JoinType::Full => out.s("full outer"),
                JoinType::Cross => out.s("cross"),
            };
            out.s("join");
            let lateral = matches!(je.source.source, JoinSource::Lateral(_) | JoinSource::Func { .. });
            if lateral {
                ctx.outer_scopes.push(scope.clone());
            }
            let source: (ExprType, Tokens) = je.source.build(ctx, &path);
            if lateral {
                ctx.outer_scopes.pop();
            }
            out.s(&source.1.to_string());
            match je.type_ {
                JoinType::Inner => {
                    for (k, v) in source.0.0 {
                        scope.insert(k, v);
                    }
                },
                JoinType::Left => {
                    for (k, mut v) in source.0.0 {
                        if !v.opt {
                            v = Type {
                                opt: true,
                                arr: false,
                                type_: v.type_,
                            };
                        }
                        scope.insert(k, v);
                    }
                },
                JoinType::Right => {
                    for v in scope.values_mut() {
                        v.opt = true;
                    }
                    for v in fields.values_mut() {
                        v.opt = true;
                    }
                    for (k, v) in source.0.0 {
                        scope.insert(k, v);
                    }
                },
                JoinType::Full => {
                    for v in scope.values_mut() {
                        v.opt = true;
                    }
                    for v in fields.values_mut() {
                        v.opt = true;
                    }
                    for (k, mut v) in source.0.0 {
                        if !v.opt {
                            v = Type {
                                opt: true,
                                arr: false,
                                type_: v.type_,
                            };
                        }
                        scope.insert(k, v);
                    }
                },
                JoinType::Cross => {
                    for (k, v) in source.0.0 {
                        scope.insert(k, v);
                    }
                },
            }
            if let Some(on) = &je.on {
                out.s("on");
                let (on_t, on_tokens): (ExprType, Tokens) =
                    on.build(ctx, &path.push_back("On".to_string()), &scope);
                check_bool(ctx, &path, &on_t);
                out.s(&on_tokens.to_string());
            }
            joins.push(out.to_string());
        }

        // Build query
        out.s("select");
        if self.distinct {
            out.s("distinct");
        }
        if !self.distinct_on.is_empty() {
            out.s("distinct on (");
            for (i, e) in self.distinct_on.iter().enumerate() {
                if i > 0 {
                    out.s(",");
                }
                let (_, tokens): (ExprType, Tokens) =
                    e.build(ctx, &path.push_back(format!("Distinct on {}", i)), &scope);
                out.s(&tokens.to_string());
            }
            out.s(")");
        }
        if self.returning.is_empty() {
            ctx.errs.err(path, "Select must have at least one output, but outputs are empty".to_string());
        }
        let out_type = build_returning_values(ctx, path, &scope, &mut out, &self.returning, res_count);
        if !matches!(self.table.source, JoinSource::Empty) {
            out.s("from");
            out.s(&source.1.to_string());
            for join in joins {
                out.s(&join);
            }
        }
        if let Some(where_) = &self.where_ {
            out.s("where");
            let path = path.push_back("Where".into());
            let (where_t, where_tokens): (ExprType, Tokens) = where_.build(ctx, &path, &scope);
            check_bool(ctx, &path, &where_t);
            out.s(&where_tokens.to_string());
        }
        if !self.group.is_empty() {
            out.s("group by");
            for (i, g) in self.group.iter().enumerate() {
                let path = path.push_back(format!("Group by clause {}", i));
                if i > 0 {
                    out.s(",");
                }
                let (_, g_tokens): (ExprType, Tokens) = g.build(ctx, &path, &scope);
                out.s(&g_tokens.to_string());
            }
        }
        if let Some(having) = &self.having {
            out.s("having");
            let path = path.push_back("Having".into());
            let (having_t, having_tokens): (ExprType, Tokens) = having.build(ctx, &path, &scope);
            check_bool(ctx, &path, &having_t);
            out.s(&having_tokens.to_string());
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
            check_general_same(ctx, &path, &out_type, &j_body_type);
            out.s(&j_body_tokens.to_string());
        }

        // Output names can also be used in order by
        let mut order_scope = scope.clone();
        for (k, v) in &out_type.0 {
            if !k.id.is_empty() {
                order_scope.insert(ExprValName::local(k.id.clone()), v.clone());
            }
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
                out.s(o.1.sql());
            }
        }
        for (clause, e) in [("limit", &self.limit), ("offset", &self.offset)] {
            let Some(e) = e else {
                continue;
            };
            out.s(clause);
            let path = path.push_back(clause.to_string());
            let (t, tokens): (ExprType, Tokens) = e.build(ctx, &path, &scope);
            check_count_type(ctx, &path, clause, &t);
            out.s(&tokens.to_string());
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
                let path = path.push_back("Fetch".into());
                let (t, tokens) = quantity.build(ctx, &path, &scope);
                check_count_type(ctx, &path, "fetch", &t);
                out.s(&tokens.to_string());
            }
            out.s(if fetch.with_ties {
                "rows with ties"
            } else {
                "rows only"
            });
        }
        if !self.lock.is_empty() &&
            (self.distinct || !self.distinct_on.is_empty() || !self.group.is_empty() || self.having.is_some() ||
                !self.junctions.is_empty()) {
            ctx
                .errs
                .err(
                    path,
                    "Row locks can't be used with `distinct`, `group by`, `having`, or set operations".to_string(),
                );
        }
        for (i, lock) in self.lock.iter().enumerate() {
            let path = path.push_back(format!("Lock {}", i));
            out.s(match lock.strength {
                SelectLockStrength::Update => "for update",
                SelectLockStrength::Share => "for share",
            });
            if !lock.of.is_empty() {
                out.s("of");
                for (j, name) in lock.of.iter().enumerate() {
                    if j > 0 {
                        out.s(",");
                    }
                    let sources = Iterator::chain([&self.table].into_iter(), self.join.iter().map(|j| &*j.source));
                    let mut found = None;
                    for source in sources {
                        match (&source.alias, &source.source) {
                            (Some(alias), _) if alias == name => {
                                found = Some(alias.clone());
                            },
                            (None, JoinSource::Table(tr)) if &tr.0 == name => {
                                found = ctx.tables.get(tr).map(|t| t.sql_name.clone());
                            },
                            _ => continue,
                        }
                        break;
                    }
                    match found {
                        Some(sql_name) => {
                            out.id(&sql_name);
                        },
                        None => {
                            ctx.errs.err(&path, format!("Locked table {:?} isn't a source in this select", name));
                        },
                    }
                }
            }
            match lock.wait {
                SelectLockWait::Wait => { },
                SelectLockWait::Nowait => {
                    out.s("nowait");
                },
                SelectLockWait::SkipLocked => {
                    out.s("skip locked");
                },
            }
        }
        return (out_type, out);
    }
}

fn check_count_type(ctx: &mut PgQueryCtx, path: &rpds::Vector<String>, clause: &str, t: &ExprType) {
    if let Some(t) = t.assert_scalar(&mut ctx.errs, path) {
        if !matches!(
            t.type_.type_,
            crate::pg::types::SimpleSimpleType::AnyInt | crate::pg::types::SimpleSimpleType::I16 |
                crate::pg::types::SimpleSimpleType::I32 |
                crate::pg::types::SimpleSimpleType::I64 |
                crate::pg::types::SimpleSimpleType::U32
        ) ||
            t.opt {
            ctx
                .errs
                .err(path, format!("{} must be a non-optional integer, but the expression has type {:?}", clause, t));
        }
    }
}
