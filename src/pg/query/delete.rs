use std::collections::HashMap;
use crate::{
    utils::Tokens,
    pg::{
        QueryResCount,
        schema::table::Table,
    },
};
use super::{
    expr::{
        Expr,
        ExprType,
        check_bool,
        ExprValName,
    },
    utils::{
        QueryBody,
        build_returning,
    },
    select::Returning,
};

pub struct Delete {
    pub(crate) table: Table,
    pub(crate) where_: Option<Expr>,
    pub(crate) returning: Vec<Returning>,
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
            scope.insert(ExprValName::field(k), v.clone());
        }

        // Build query
        let mut out = Tokens::new();
        out.s("delete from").id(&self.table.id);
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
