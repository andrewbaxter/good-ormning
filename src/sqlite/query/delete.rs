use std::collections::HashMap;
use crate::{
    utils::Tokens,
    sqlite::{
        QueryResCount,
        schema::table::Table,
    },
};
use super::{
    expr::{
        check_bool,
        Expr,
        ExprType,
        Binding,
    },
    select_body::Returning,
    utils::{
        build_returning,
        build_with,
        QueryBody,
        With,
    },
};

pub struct Delete {
    pub with: Option<With>,
    pub table: Table,
    pub where_: Option<Expr>,
    pub returning: Vec<Returning>,
}

impl QueryBody for Delete {
    fn build(
        &self,
        ctx: &mut super::utils::SqliteQueryCtx,
        path: &rpds::Vector<String>,
        res_count: QueryResCount,
    ) -> (super::expr::ExprType, crate::utils::Tokens) {
        let mut out = Tokens::new();

        // Prep
        if let Some(w) = &self.with {
            out.s(&build_with(ctx, path, w).to_string());
        }
        let mut scope = HashMap::new();
        for field in match ctx.tables.get(&self.table) {
            Some(t) => t,
            None => {
                ctx.errs.err(path, format!("Unknown table {} for delete", self.table));
                return (ExprType(vec![]), Tokens::new());
            },
        } {
            scope.insert(Binding::field(field), field.type_.type_.clone());
        }

        // Build query
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
