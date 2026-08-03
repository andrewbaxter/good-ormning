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
            types::{
                Type,
                type_i64,
            },
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
    Func(String, Vec<Expr>),
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
                table_info.fields.iter().map(|(id, info)| (ExprValName::field(id), info.type_.clone())).collect()
            },
            JoinSource::Func(name, args) => {
                out.id(name).s("(");
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        out.s(",");
                    }
                    let (_, tokens) = arg.build(ctx, &path.push_back(format!("Arg {}", i)), &HashMap::new());
                    out.s(&tokens.to_string());
                }
                out.s(")");
                vec![]
            },
            JoinSource::Empty => {
                vec![]
            },
        };
        if let Some(s) = &self.alias {
            out.s("as").id(s);
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
pub struct Select {
    pub distinct: bool,
    pub group: Vec<Expr>,
    pub having: Option<Expr>,
    pub join: Vec<Join>,
    pub junctions: Vec<super::select_body::SelectJunction>,
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
            let source: (ExprType, Tokens) = je.source.build(ctx, &path);
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
                let (_, on_tokens): (ExprType, Tokens) = on.build(ctx, &path, &scope);
                out.s(&on_tokens.to_string());
            }
            joins.push(out.to_string());
        }

        // Build query
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
        if !self.order.is_empty() {
            out.s("order by");
            for (i, o) in self.order.iter().enumerate() {
                let path = path.push_back(format!("Order by clause {}", i));
                if i > 0 {
                    out.s(",");
                }
                let (_, o_tokens): (ExprType, Tokens) = o.0.build(ctx, &path, &scope);
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
        if let Some(l) = &self.limit {
            out.s("limit");
            let path = path.push_back("Limit".into());
            let (limit_t, limit_tokens): (ExprType, Tokens) = l.build(ctx, &path, &scope);
            check_general_same(ctx, &path, &limit_t, &ExprType(vec![(ExprValName::empty(), type_i64().build())]));
            out.s(&limit_tokens.to_string());
        }
        if let Some(o) = &self.offset {
            out.s("offset");
            let path = path.push_back("Offset".into());
            let (offset_t, offset_tokens): (ExprType, Tokens) = o.build(ctx, &path, &scope);
            check_general_same(ctx, &path, &offset_t, &ExprType(vec![(ExprValName::empty(), type_i64().build())]));
            out.s(&offset_tokens.to_string());
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
        return (out_type, out);
    }
}
