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
                    Returning,
                    build_order_limit,
                    build_returning,
                },
            },
            schema::table::TableRef,
        },
        utils::Tokens,
    },
    std::collections::HashMap,
};

#[derive(Clone, Debug)]
pub struct Delete {
    pub alias: Option<String>,
    pub join: Vec<Join>,
    pub limit: Option<Expr>,
    pub order: Vec<(Expr, Order)>,
    pub returning: Vec<Returning>,
    pub table: TableRef,
    pub targets: Vec<String>,
    pub where_: Option<Expr>,
}

impl QueryBody for Delete {
    fn build(
        &self,
        ctx: &mut MariadbQueryCtx,
        path: &rpds::Vector<String>,
        res_count: QueryResCount,
    ) -> (ExprType, Tokens) {
        let table_info = match ctx.tables.get(&self.table) {
            Some(t) => t.clone(),
            None => {
                ctx.errs.err(path, format!("Unknown table {:?} for delete", self.table));
                return (ExprType(vec![]), Tokens::new_mariadb());
            },
        };
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
        if self.alias.is_some() || !self.join.is_empty() || !self.targets.is_empty() {
            if !self.order.is_empty() || self.limit.is_some() || !self.returning.is_empty() {
                ctx
                    .errs
                    .err(
                        path,
                        "MariaDB doesn't support `order by`, `limit` or `returning` in deletes with joins or a table alias".to_string(),
                    );
            }
            out.s("delete");
            if self.targets.is_empty() {
                out.id(self.alias.as_ref().unwrap_or(&table_info.sql_name));
            } else {
                for (i, target) in self.targets.iter().enumerate() {
                    if i > 0 {
                        out.s(",");
                    }
                    if !ctx.table_aliases.contains_key(target) &&
                        !ctx.tables.contains_key(&TableRef(target.clone())) {
                        ctx.errs.err(path, format!("Delete target {} isn't a table or alias in the query", target));
                    }
                    out.id(target);
                }
            }
            out.s("from").id(&table_info.sql_name);
            if let Some(alias) = &self.alias {
                out.s("as").id(alias);
            }
            out.s(&joins.to_string());
        } else {
            out.s("delete from").id(&table_info.sql_name);
        }
        if let Some(where_) = &self.where_ {
            out.s("where");
            let path = path.push_back("Where".into());
            let (where_t, where_tokens) = where_.build(ctx, &path, &scope);
            check_bool(ctx, &path, &where_t);
            out.s(&where_tokens.to_string());
        }
        build_order_limit(ctx, path, &scope, &mut out, &self.order, &self.limit);
        let out_type = build_returning(ctx, path, &scope, &mut out, &self.returning, res_count);
        return (out_type, out);
    }
}
