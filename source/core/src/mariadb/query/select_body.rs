use {
    crate::{
        mariadb::{
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
                    MariadbQueryCtx,
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
        ctx: &mut MariadbQueryCtx,
        inject_scope: &HashMap<ExprValName, Type>,
        path: &rpds::Vector<String>,
        res_count: QueryResCount,
    ) -> (ExprType, Tokens) {
        let source: (ExprType, Tokens) = self.table.build(ctx, path);
        let mut scope = inject_scope.clone();
        for (k, v) in source.0.0 {
            scope.insert(k, v);
        }
        let joins = crate::mariadb::query::select::build_joins(ctx, path, &self.join, &mut scope);
        let mut out = Tokens::new_mariadb();
        out.s("select");
        if self.distinct {
            out.s("distinct");
        }
        if self.returning.is_empty() {
            ctx.errs.err(path, "Select must have at least one output, but outputs are empty".to_string());
        }
        let mut out_type = build_returning_values(ctx, path, &scope, &mut out, &self.returning, res_count);
        out.s("from");
        out.s(&source.1.to_string()).s(&joins.to_string());
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
            super::utils::check_limit_arg(ctx, path, l);
            out.s("limit");
            let path = path.push_back("Limit".into());
            let (limit_t, limit_tokens): (ExprType, Tokens) = l.build(ctx, &path, &scope);
            crate::mariadb::query::expr::check_general_same(
                ctx,
                &path,
                &limit_t,
                &ExprType(vec![(ExprValName::empty(), crate::mariadb::types::type_i64().build())]),
            );
            out.s(&limit_tokens.to_string());
        }
        if let Some(o) = &self.offset {
            super::utils::check_limit_arg(ctx, path, o);
            out.s("offset");
            let path = path.push_back("Offset".into());
            let (offset_t, offset_tokens): (ExprType, Tokens) = o.build(ctx, &path, &scope);
            crate::mariadb::query::expr::check_general_same(
                ctx,
                &path,
                &offset_t,
                &ExprType(vec![(ExprValName::empty(), crate::mariadb::types::type_i64().build())]),
            );
            out.s(&offset_tokens.to_string());
            if self.limit.is_none() {
                out.s("rows");
            }
        }
        if !self.junctions.is_empty() && (!self.order.is_empty() || self.limit.is_some() || self.offset.is_some()) {
            let mut wrapped = Tokens::new_mariadb();
            wrapped.s("(").s(&out.to_string()).s(")");
            out = wrapped;
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
            out_type = check_general_same(ctx, &path, &out_type, &j_body_type);
            out.s("(").s(&j_body_tokens.to_string()).s(")");
        }
        return (out_type, out);
    }
}

impl QueryBody for SelectBody {
    fn build(
        &self,
        ctx: &mut MariadbQueryCtx,
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
