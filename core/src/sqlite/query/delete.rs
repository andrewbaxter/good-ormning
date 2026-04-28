use crate::sqlite::query::utils::Returning;
use std::collections::HashMap;
use crate::{
    utils::Tokens,
    sqlite::{
        QueryResCount,
        schema::table::TableRef,
    },
};
use super::{
    expr::{
        Expr,
        ExprType,
        check_bool,
        Binding,
    },
    utils::{
        SqliteQueryCtx,
        QueryBody,
        build_returning,
    },
    select::IndexHint,
};

#[derive(Clone, Debug)]
pub struct Delete {
    pub table: TableRef,
    pub where_: Option<Expr>,
    pub returning: Vec<Returning>,
    pub index_hint: Option<IndexHint>,
}

impl QueryBody for Delete {
    fn build(
        &self,
        ctx: &mut SqliteQueryCtx,
        path: &rpds::Vector<String>,
        res_count: QueryResCount,
    ) -> (super::expr::ExprType, crate::utils::Tokens) {
        // Prep
        let table_info = match ctx.tables.get(&self.table) {
            Some(t) => t.clone(),
            None => {
                ctx.errs.err(path, format!("Unknown table {:?} for delete", self.table));
                return (ExprType(vec![]), Tokens::new());
            },
        };
        let mut scope = HashMap::new();
        for (k, info) in &table_info.fields {
            scope.insert(Binding::field(k), info.type_.clone());
        }

        // Build query
        let mut out = Tokens::new();
        out.s("delete from").id(&table_info.sql_name);
        if let Some(hint) = &self.index_hint {
            match hint {
                IndexHint::IndexedBy(name) => {
                    out.s("indexed by").id(name);
                },
                IndexHint::NotIndexed => {
                    out.s("not indexed");
                },
            }
        }
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
