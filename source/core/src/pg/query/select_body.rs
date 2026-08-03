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
                select::{
                    Join,
                    NamedSelectSource,
                    Order,
                },
                utils::{
                    PgQueryCtx,
                    QueryBody,
                    Returning,
                    build_returning_values,
                },
            },
            types::Type,
        },
        utils::Tokens,
    },
    std::collections::HashMap,
};

#[derive(Clone, Debug)]
pub struct SelectBody {
    pub distinct: bool,
    pub group: Vec<Expr>,
    pub having: Option<Expr>,
    pub join: Vec<Join>,
    pub junctions: Vec<SelectJunction>,
    pub limit: Option<Expr>,
    pub offset: Option<Expr>,
    pub order: Vec<(Expr, Order)>,
    pub returning: Vec<Returning>,
    pub table: NamedSelectSource,
    pub where_: Option<Expr>,
}

impl SelectBody {
    pub fn build_internal(
        &self,
        ctx: &mut PgQueryCtx,
        inject_scope: &HashMap<ExprValName, Type>,
        path: &rpds::Vector<String>,
        res_count: QueryResCount,
    ) -> (ExprType, Tokens) {
        // Prep
        let source: (ExprType, Tokens) = self.table.build(ctx, path);
        let mut scope = inject_scope.clone();
        for (k, v) in source.0.0 {
            scope.insert(k, v);
        }
        let mut joins = vec![];
        for (i, je) in self.join.iter().enumerate() {
            let path = path.push_back(format!("Join {}", i));
            let mut out = Tokens::new();
            match je.type_ {
                crate::pg::query::select::JoinType::Inner => out.s("inner"),
                crate::pg::query::select::JoinType::Left => out.s("left"),
                crate::pg::query::select::JoinType::Right => out.s("right"),
                crate::pg::query::select::JoinType::Full => out.s("full outer"),
                crate::pg::query::select::JoinType::Cross => out.s("cross"),
            };
            out.s("join");
            let source: (ExprType, Tokens) = je.source.build(ctx, &path);
            out.s(&source.1.to_string());
            match je.type_ {
                crate::pg::query::select::JoinType::Inner => {
                    for (k, v) in source.0.0 {
                        scope.insert(k, v);
                    }
                },
                crate::pg::query::select::JoinType::Left => {
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
                crate::pg::query::select::JoinType::Right => {
                    for v in scope.values_mut() {
                        v.opt = true;
                    }
                    for (k, v) in source.0.0 {
                        scope.insert(k, v);
                    }
                },
                crate::pg::query::select::JoinType::Full => {
                    for v in scope.values_mut() {
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
                crate::pg::query::select::JoinType::Cross => {
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
        let mut out = Tokens::new();
        out.s("select");
        if self.distinct {
            out.s("distinct");
        }
        if self.returning.is_empty() {
            ctx.errs.err(path, "Select must have at least one output, but outputs are empty".to_string());
        }
        let out_type = build_returning_values(ctx, path, &scope, &mut out, &self.returning, res_count);
        out.s("from");
        out.s(&source.1.to_string());
        for join in joins {
            out.s(&join);
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
            crate::pg::query::expr::check_general_same(
                ctx,
                &path,
                &limit_t,
                &ExprType(vec![(ExprValName::empty(), crate::pg::types::type_i64().build())]),
            );
            out.s(&limit_tokens.to_string());
        }
        if let Some(o) = &self.offset {
            out.s("offset");
            let path = path.push_back("Offset".into());
            let (offset_t, offset_tokens): (ExprType, Tokens) = o.build(ctx, &path, &scope);
            crate::pg::query::expr::check_general_same(
                ctx,
                &path,
                &offset_t,
                &ExprType(vec![(ExprValName::empty(), crate::pg::types::type_i64().build())]),
            );
            out.s(&offset_tokens.to_string());
        }
        for (i, j) in self.junctions.iter().enumerate() {
            let path = path.push_back(format!("Junction {}", i));
            match j.op {
                SelectJunctionOperator::Union => out.s("union"),
                SelectJunctionOperator::UnionAll => out.s("union all"),
                SelectJunctionOperator::Intersect => out.s("intersect"),
                SelectJunctionOperator::IntersectAll => out.s("intersect all"),
                SelectJunctionOperator::Except => out.s("except"),
                SelectJunctionOperator::ExceptAll => out.s("except all"),
            };
            let (j_body_type, j_body_tokens) = j.body.build(ctx, &path, QueryResCount::Many);
            check_general_same(ctx, &path, &out_type, &j_body_type);
            out.s(&j_body_tokens.to_string());
        }
        (out_type, out)
    }
}

impl QueryBody for SelectBody {
    fn build(
        &self,
        ctx: &mut PgQueryCtx,
        path: &rpds::Vector<String>,
        res_count: QueryResCount,
    ) -> (ExprType, Tokens) {
        return self.build_internal(ctx, &HashMap::new(), path, res_count);
    }
}

#[derive(Clone, Debug)]
pub struct SelectJunction {
    pub body: Box<dyn QueryBody>,
    pub op: SelectJunctionOperator,
}

#[derive(Clone, Debug)]
pub enum SelectJunctionOperator {
    Except,
    ExceptAll,
    Intersect,
    IntersectAll,
    Union,
    UnionAll,
}
