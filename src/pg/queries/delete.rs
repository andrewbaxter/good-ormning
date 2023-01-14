use std::collections::HashMap;
use crate::{
    utils::Tokens,
    pg::{
        schema::table::TableId,
        QueryResCount,
    },
};
use super::{
    expr::{
        Expr,
        ExprType,
        check_bool,
    },
    utils::{
        QueryBody,
        build_returning,
    },
    select::Returning,
};

pub struct Delete {
    pub table: TableId,
    pub where_: Option<Expr>,
    pub returning: Vec<Returning>,
}

impl QueryBody for Delete {
    fn build(
        &self,
        ctx: &mut super::utils::PgQueryCtx,
        path: &rpds::Vector<String>,
        res_count: QueryResCount,
    ) -> (super::expr::ExprType, crate::utils::Tokens) {
        // Prep
        let mut scope = HashMap::new();
        for (k, v) in match ctx.tables.get(&self.table) {
            Some(t) => t,
            None => {
                ctx.errs.err(path, format!("Unknown table {} for delete", self.table));
                return (ExprType(vec![]), Tokens::new());
            },
        } {
            scope.insert(k.clone(), v.clone());
        }

        // Build query
        let mut out = Tokens::new();
        out.s("delete from").id(&self.table.0);
        if let Some(where_) = &self.where_ {
            out.s("where");
            let path = path.push_back("Where".into());
            let (where_t, where_tokens) = where_.build(ctx, &path, &scope);
            check_bool(ctx, &path, &where_t);
            out.s(&where_tokens.to_string());
        }
        let out_type = build_returning(ctx, path, &scope, &mut out, &self.returning, res_count);
        (out_type, out)
    }
}
