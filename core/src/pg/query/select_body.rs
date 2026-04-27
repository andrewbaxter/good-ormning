use std::collections::HashMap;
use crate::{
    pg::{
        types::Type,
        QueryResCount,
        query::{
            utils::{
                PgQueryCtx,
                QueryBody,
                build_returning_values,
                Returning,
            },
            expr::{
                Expr,
                ExprType,
                check_bool,
                ExprValName,
            },
            select::{
                Join,
                NamedSelectSource,
                Order,
            },
        },
    },
    utils::Tokens,
};

#[derive(Clone, Debug)]
pub enum SelectJunctionOperator {
    Union,
    UnionAll,
    Intersect,
    Except,
}

#[derive(Clone, Debug)]
pub struct SelectJunction {
    pub op: SelectJunctionOperator,
    pub body: Box<dyn QueryBody>,
}

#[derive(Clone, Debug)]
pub struct SelectBody {
    pub table: NamedSelectSource,
    pub returning: Vec<Returning>,
    pub join: Vec<Join>,
    pub where_: Option<Expr>,
    pub group: Vec<Expr>,
    pub order: Vec<(Expr, Order)>,
    pub limit: Option<Expr>,
    pub distinct: bool,
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
                crate::pg::query::select::JoinType::Left => out.s("left"),
                crate::pg::query::select::JoinType::Inner => out.s("inner"),
            };
            out.s("join");
            let source: (ExprType, Tokens) = je.source.build(ctx, &path);
            out.s(&source.1.to_string());
            match je.type_ {
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
                crate::pg::query::select::JoinType::Inner => {
                    for (k, v) in source.0.0 {
                        scope.insert(k, v);
                    }
                },
            }
            out.s("on");
            let (_on_t, on_tokens): (ExprType, Tokens) = je.on.build(ctx, &path, &scope);
            out.s(&on_tokens.to_string());
            joins.push(out.to_string());
        }

        // Build query
        let mut out = Tokens::new();
        out.s("select");
        if self.distinct {
            out.s("distinct");
        }
        if self.returning.is_empty() {
            ctx.errs.err(path, format!("Select must have at least one output, but outputs are empty"));
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
        if self.group.len() > 0 {
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
        (out_type, out)
    }
}
