use std::collections::HashMap;
use crate::{
    utils::Tokens,
    pg::schema::table::TableId,
};
use super::{
    expr::{
        Expr,
        ExprType,
    },
    utils::{
        QueryBody,
        build_returning,
    },
    select::SelectOutput,
};

pub struct Delete {
    pub(crate) table: TableId,
    pub(crate) where_: Option<Expr>,
    pub(crate) returning: Vec<SelectOutput>,
}

impl QueryBody for Delete {
    fn build(&self, ctx: &mut super::utils::PgQueryCtx) -> (super::expr::ExprType, crate::utils::Tokens) {
        // Prep
        let mut scope = HashMap::new();
        for (k, v) in match ctx.tables.get(&self.table) {
            Some(t) => t,
            None => {
                ctx.errs.err(format!("Unknown table {} for delete", self.table));
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
            where_.build(ctx, &scope);
        }
        let out_type = build_returning(ctx, &scope, &mut out, &self.returning);
        (out_type, out)
    }
}
