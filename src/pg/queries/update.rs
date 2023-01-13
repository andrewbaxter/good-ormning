use std::collections::HashMap;
use crate::{
    pg::{
        schema::{
            table::TableId,
            field::FieldId,
        },
        QueryResCount,
    },
    utils::Tokens,
};
use super::{
    expr::{
        Expr,
        ExprType,
    },
    utils::{
        QueryBody,
        build_returning,
        build_set,
    },
    select::SelectOutput,
};

pub struct Update {
    pub table: TableId,
    pub values: Vec<(FieldId, Expr)>,
    pub where_: Option<Expr>,
    pub returning: Vec<SelectOutput>,
}

impl QueryBody for Update {
    fn build(
        &self,
        ctx: &mut super::utils::PgQueryCtx,
        res_count: QueryResCount,
    ) -> (super::expr::ExprType, crate::utils::Tokens) {
        // Prep
        let mut scope = HashMap::new();
        for (k, v) in match ctx.tables.get(&self.table) {
            Some(t) => t,
            None => {
                ctx.errs.err(format!("Unknown table {} for update", self.table));
                return (ExprType(vec![]), Tokens::new());
            },
        } {
            scope.insert(k.clone(), v.clone());
        }

        // Build query
        let mut out = Tokens::new();
        out.s("update").id(&self.table.0);
        build_set(ctx, &scope, &mut out, &self.values);
        if let Some(where_) = &self.where_ {
            out.s("where");
            where_.build(ctx, &scope);
        }
        let out_type = build_returning(ctx, &scope, &mut out, &self.returning, res_count);
        (out_type, out)
    }
}
