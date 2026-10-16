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
                },
                select::{
                    Join,
                    Order,
                    build_joins,
                },
                utils::{
                    MariadbQueryCtx,
                    QueryBody,
                    build_order_limit,
                    build_set,
                },
            },
            schema::{
                field::FieldRef,
                table::TableRef,
            },
        },
        utils::Tokens,
    },
    std::collections::HashMap,
};

#[derive(Clone, Debug)]
pub struct Update {
    pub alias: Option<String>,
    pub join: Vec<Join>,
    pub limit: Option<Expr>,
    pub order: Vec<(Expr, Order)>,
    pub table: TableRef,
    pub values: Vec<(FieldRef, Expr)>,
    pub where_: Option<Expr>,
}

impl QueryBody for Update {
    fn build(
        &self,
        ctx: &mut MariadbQueryCtx,
        path: &rpds::Vector<String>,
        res_count: QueryResCount,
    ) -> (ExprType, Tokens) {
        let table_info = match ctx.tables.get(&self.table) {
            Some(t) => t.clone(),
            None => {
                ctx.errs.err(path, format!("Unknown table {:?} for update", self.table));
                return (ExprType(vec![]), Tokens::new_mariadb());
            },
        };
        if !matches!(res_count, QueryResCount::None) {
            ctx
                .errs
                .err(
                    path,
                    format!(
                        "MariaDB doesn't support returning values from updates, but result count is {:?}",
                        res_count
                    ),
                );
        }
        let mut scope = HashMap::new();
        for (k, info) in &table_info.fields {
            let mut name = ExprValName::field(k);
            if let Some(alias) = &self.alias {
                name = name.with_alias(alias);
            }
            scope.insert(name, info.type_.clone());
        }
        if let Some(alias) = &self.alias {
            ctx.table_aliases.insert(alias.clone(), self.table.clone());
        }
        let joins = build_joins(ctx, path, &self.join, &mut scope);
        let mut out = Tokens::new_mariadb();
        out.s("update").id(&table_info.sql_name);
        if let Some(alias) = &self.alias {
            out.s("as").id(alias);
        }
        out.s(&joins.to_string());
        out.s("set");
        let values = self.values.iter().map(|(f, e)| {
            let mut f = f.clone();
            if let Some(alias) = &self.alias {
                if f.table_id == self.table.0 {
                    f.table_id = alias.clone();
                }
            }
            return (f, e.clone());
        }).collect::<Vec<_>>();
        build_set(ctx, path, &scope, &mut out, &values, true);
        if let Some(where_) = &self.where_ {
            out.s("where");
            let path = path.push_back("Where".into());
            let (where_t, where_tokens) = where_.build(ctx, &path, &scope);
            check_bool(ctx, &path, &where_t);
            out.s(&where_tokens.to_string());
        }
        if !self.join.is_empty() && (!self.order.is_empty() || self.limit.is_some()) {
            ctx.errs.err(path, "MariaDB doesn't support `order by` or `limit` in updates with joins".to_string());
        }
        build_order_limit(ctx, path, &scope, &mut out, &self.order, &self.limit);
        return (ExprType(vec![]), out);
    }
}
