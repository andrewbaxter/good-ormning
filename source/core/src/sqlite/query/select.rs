use {
    crate::{
        sqlite::{
            QueryResCount,
            query::{
                expr::{
                    Binding,
                    Expr,
                    ExprType,
                    check_bool,
                },
                select_body::{
                    SelectJunction,
                    build_select_junction,
                },
                utils::{
                    QueryBody,
                    Returning,
                    SqliteQueryCtx,
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
pub enum IndexHint {
    IndexedBy(String),
    NotIndexed,
}

#[derive(Clone, Debug)]
pub struct Join {
    pub on: Option<Expr>,
    pub source: NamedSelectSource,
    pub type_: JoinType,
}

#[derive(Clone, Debug)]
pub enum JoinSource {
    Empty,
    Func(String, Vec<Expr>),
    NestedJoin(Box<NamedSelectSource>, Vec<Join>),
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
    pub index_hint: Option<IndexHint>,
    pub source: JoinSource,
}

impl NamedSelectSource {
    pub fn build(&self, ctx: &mut SqliteQueryCtx, path: &rpds::Vector<String>) -> (ExprType, Tokens) {
        let mut out = Tokens::new();
        let mut new_fields: Vec<(Binding, Type)> = match &self.source {
            JoinSource::Subsel(s) => {
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
                        return (ExprType(vec![]), Tokens::new());
                    },
                };
                out.id(&table_info.sql_name);
                if let Some(hint) = &self.index_hint {
                    match hint {
                        IndexHint::IndexedBy(name) => {
                            out.s("indexed by").id(name);
                        },
                        IndexHint::NotIndexed => {
                            out.s("not indexed");
                        },
                    }
                }
                table_info.fields.iter().map(|(id, info)| (Binding::field(id), info.type_.clone())).collect()
            },
            JoinSource::Func(name, args) => {
                if name == "__good_ormning_rarray" {
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 {
                            out.s(",");
                        }
                        let (_, tokens) = arg.build(ctx, &path.push_back(format!("Arg {}", i)), &HashMap::new());
                        out.s(&tokens.to_string());
                    }
                } else {
                    out.s(name).s("(");
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 {
                            out.s(",");
                        }
                        let (_, tokens) = arg.build(ctx, &path.push_back(format!("Arg {}", i)), &HashMap::new());
                        out.s(&tokens.to_string());
                    }
                    out.s(")");
                }
                if name == "rarray" || name == "__good_ormning_rarray" {
                    // The `value` column has the element type of the array argument
                    let mut value_t = None;
                    if let Some(arg) = args.first() {
                        if let Expr::Param { type_, .. } = arg {
                            let mut t = type_.clone();
                            t.arr = false;
                            value_t = Some(t);
                        }
                    }
                    let Some(value_t) = value_t else {
                        ctx
                            .errs
                            .err(
                                &path.push_back("From".to_string()),
                                "rarray requires a single array parameter argument".to_string(),
                            );
                        return (ExprType(vec![]), Tokens::new());
                    };
                    vec![(Binding::local("value".into()), value_t)]
                } else {
                    ctx
                        .errs
                        .err(
                            &path.push_back("From".to_string()),
                            format!(
                                "Unsupported table function {}, its output columns aren't known (supported: rarray)",
                                name
                            ),
                        );
                    vec![]
                }
            },
            JoinSource::NestedJoin(base, joins) => {
                let (base_fields, base_tokens) = base.build(ctx, path);
                let mut fields: Vec<(Binding, Type)> = base_fields.0;
                let mut inner = Tokens::new();
                inner.s(&base_tokens.to_string());
                for (i, j) in joins.iter().enumerate() {
                    let path = path.push_back(format!("Nested join {}", i));
                    match j.type_ {
                        JoinType::Inner => {
                            inner.s("inner join");
                        },
                        JoinType::Left => {
                            inner.s("left join");
                        },
                        JoinType::Right => {
                            inner.s("right join");
                        },
                        JoinType::Full => {
                            inner.s("full outer join");
                        },
                        JoinType::Cross => {
                            inner.s("cross join");
                        },
                    }
                    let (join_fields, join_tokens) = j.source.build(ctx, &path);
                    inner.s(&join_tokens.to_string());
                    match j.type_ {
                        JoinType::Inner => {
                            for (k, v) in join_fields.0 {
                                fields.push((k, v));
                            }
                        },
                        JoinType::Left => {
                            for (k, mut v) in join_fields.0 {
                                if !v.opt {
                                    v = Type {
                                        opt: true,
                                        arr: false,
                                        type_: v.type_,
                                    };
                                }
                                fields.push((k, v));
                            }
                        },
                        JoinType::Right => {
                            for (_, v) in fields.iter_mut() {
                                v.opt = true;
                            }
                            for (k, v) in join_fields.0 {
                                fields.push((k, v));
                            }
                        },
                        JoinType::Full => {
                            for (_, v) in fields.iter_mut() {
                                v.opt = true;
                            }
                            for (k, mut v) in join_fields.0 {
                                if !v.opt {
                                    v = Type {
                                        opt: true,
                                        arr: false,
                                        type_: v.type_,
                                    };
                                }
                                fields.push((k, v));
                            }
                        },
                        JoinType::Cross => {
                            for (k, v) in join_fields.0 {
                                fields.push((k, v));
                            }
                        },
                    }
                    if let Some(on) = &j.on {
                        inner.s("on");
                        let scope: HashMap<Binding, Type> = fields.iter().cloned().collect();
                        let (_, on_tokens) = on.build(ctx, &path.push_back("On".to_string()), &scope);
                        inner.s(&on_tokens.to_string());
                    }
                }
                out.s("(").s(&inner.to_string()).s(")");
                fields
            },
            JoinSource::Empty => {
                vec![]
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
pub struct Select {
    pub distinct: bool,
    pub group: Vec<Expr>,
    pub having: Option<Expr>,
    pub join: Vec<Join>,
    pub junction: Vec<SelectJunction>,
    pub limit: Option<Expr>,
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
        ctx: &mut SqliteQueryCtx,
        path: &rpds::Vector<String>,
        res_count: QueryResCount,
    ) -> (ExprType, Tokens) {
        let mut scope = HashMap::new();
        let mut out = Tokens::new();
        if let Some(with) = &self.with {
            out.s(&build_with(ctx, path, with).to_string());
        }
        out.s("select");
        if self.distinct {
            out.s("distinct");
        }
        if self.returning.is_empty() {
            ctx.errs.err(path, "Select must have at least one output, but outputs are empty".to_string());
        }
        let (fields, table_tokens): (ExprType, Tokens) = self.table.build(ctx, path);
        for (k, v) in fields.0 {
            scope.insert(k, v);
        }
        let mut join_tokens = vec![];
        for (i, j) in self.join.iter().enumerate() {
            let path = path.push_back(format!("Join {}", i));
            let mut out = Tokens::new();
            match j.type_ {
                JoinType::Inner => {
                    out.s("inner join");
                },
                JoinType::Left => {
                    out.s("left join");
                },
                JoinType::Right => {
                    out.s("right join");
                },
                JoinType::Full => {
                    out.s("full outer join");
                },
                JoinType::Cross => {
                    out.s("cross join");
                },
            }
            let (fields, source_tokens): (ExprType, Tokens) = j.source.build(ctx, &path);
            out.s(&source_tokens.to_string());
            match j.type_ {
                JoinType::Inner => {
                    for (k, v) in fields.0 {
                        scope.insert(k, v);
                    }
                },
                JoinType::Left => {
                    for (k, mut v) in fields.0 {
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
                    for (k, v) in fields.0 {
                        scope.insert(k, v);
                    }
                },
                JoinType::Full => {
                    for v in scope.values_mut() {
                        v.opt = true;
                    }
                    for (k, mut v) in fields.0 {
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
                    for (k, v) in fields.0 {
                        scope.insert(k, v);
                    }
                },
            }
            if let Some(on) = &j.on {
                out.s("on");
                let (on_t, on_tokens): (ExprType, Tokens) =
                    on.build(ctx, &path.push_back("On".to_string()), &scope);
                check_bool(ctx, &path, &on_t);
                out.s(&on_tokens.to_string());
            }
            join_tokens.push(out);
        }
        let out_type = build_returning_values(ctx, path, &scope, &mut out, &self.returning, res_count);
        if !matches!(self.table.source, JoinSource::Empty) {
            out.s("from").s(&table_tokens.to_string());
            for j in join_tokens {
                out.s(&j.to_string());
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
                if i > 0 {
                    out.s(",");
                }
                let path = path.push_back(format!("Group by {}", i));
                let (_, tokens): (ExprType, Tokens) = g.build(ctx, &path, &scope);
                out.s(&tokens.to_string());
            }
        }
        if let Some(having) = &self.having {
            out.s("having");
            let path = path.push_back("Having".into());
            let (having_t, having_tokens): (ExprType, Tokens) = having.build(ctx, &path, &scope);
            check_bool(ctx, &path, &having_t);
            out.s(&having_tokens.to_string());
        }
        if !self.junction.is_empty() {
            let junction_tokens = build_select_junction(ctx, path, &out_type, &self.junction);
            out.s(&junction_tokens.to_string());
        }

        // Output names can also be used in order by
        let mut order_scope = scope.clone();
        for (k, v) in &out_type.0 {
            if !k.id.is_empty() {
                order_scope.insert(Binding::local(k.id.clone()), v.clone());
            }
        }
        if !self.order.is_empty() {
            out.s("order by");
            for (i, (e, o)) in self.order.iter().enumerate() {
                if i > 0 {
                    out.s(",");
                }
                let path = path.push_back(format!("Order by {}", i));
                let (_, tokens): (ExprType, Tokens) = e.build(ctx, &path, &order_scope);
                out.s(&tokens.to_string());
                out.s(o.sql());
            }
        }
        for (clause, e) in [("limit", &self.limit), ("offset", &self.offset)] {
            let Some(e) = e else {
                continue;
            };
            out.s(clause);
            let path = path.push_back(clause.to_string());
            let (t, tokens): (ExprType, Tokens) = e.build(ctx, &path, &scope);
            if let Some(t) = t.assert_scalar(&mut ctx.errs, &path) {
                if !matches!(
                    t.type_.type_,
                    crate::sqlite::types::SimpleSimpleType::AnyInt | crate::sqlite::types::SimpleSimpleType::I16 |
                        crate::sqlite::types::SimpleSimpleType::I32 |
                        crate::sqlite::types::SimpleSimpleType::I64 |
                        crate::sqlite::types::SimpleSimpleType::U32
                ) ||
                    t.opt {
                    ctx
                        .errs
                        .err(
                            &path,
                            format!("{} must be a non-optional integer, but the expression has type {:?}", clause, t),
                        );
                }
            }
            out.s(&tokens.to_string());
        }
        return (out_type, out);
    }
}
