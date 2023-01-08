use std::collections::HashMap;
use crate::{
    pg::{
        schema::{
            table::TableId,
            field::FieldId,
        },
    },
    utils::Tokens,
};
use super::{
    expr::{
        Expr,
        ExprType,
        ExprValName,
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
    pub where_: Option<Expr>,
    pub values: Vec<(FieldId, Expr)>,
    pub returning: Vec<SelectOutput>,
}

impl QueryBody for Update {
    fn build(&self, ctx: &mut super::utils::PgQueryCtx) -> (super::expr::ExprType, crate::utils::Tokens) {
        // Prep
        let mut all_fields = HashMap::new();
        for (k, v) in match ctx.tables.get(&self.table) {
            Some(t) => t,
            None => {
                ctx.errs.err(format!("Unknown table {} for update", self.table));
                return (ExprType(vec![]), Tokens::new());
            },
        } {
            all_fields.insert(ExprValName::from(k.clone()), v.clone());
        }

        // Build query
        let mut out = Tokens::new();
        out.s("update").id(&self.table.0);
        build_set(ctx, &all_fields, &mut out, &self.values);
        if let Some(where_) = &self.where_ {
            out.s("where");
            where_.build(ctx, &all_fields);
        }
        let out_type = build_returning(ctx, &all_fields, &mut out, &self.returning);
        (out_type, out)
    }
}
