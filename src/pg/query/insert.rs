use std::{
    collections::HashMap,
};
use crate::{
    pg::{
        QueryResCount,
        schema::{
            field::Field,
            table::Table,
        },
    },
    utils::Tokens,
};
use super::{
    expr::{
        Expr,
        ExprType,
        check_assignable,
        ExprValName,
    },
    utils::{
        QueryBody,
        build_returning,
        build_set,
    },
    select::Returning,
};

pub enum InsertConflict {
    DoNothing,
    DoUpdate {
        conflict: Vec<Field>,
        set: Vec<(Field, Expr)>,
    },
}

pub struct Insert {
    pub(crate) table: Table,
    pub(crate) values: Vec<(Field, Expr)>,
    pub(crate) on_conflict: Option<InsertConflict>,
    pub(crate) returning: Vec<Returning>,
}

impl QueryBody for Insert {
    fn build(
        &self,
        ctx: &mut super::utils::PgQueryCtx,
        path: &rpds::Vector<String>,
        res_count: QueryResCount,
    ) -> (ExprType, Tokens) {
        // Prep
        let mut scope = HashMap::new();
        for (field, v) in match ctx.tables.get(&self.table) {
            Some(t) => t,
            None => {
                ctx.errs.err(path, format!("Unknown table {} for insert", self.table));
                return (ExprType(vec![]), Tokens::new());
            },
        } {
            scope.insert(ExprValName::field(field), v.clone());
        }

        // Build query
        let mut out = Tokens::new();
        out.s("insert into").id(&self.table.id).s("(");
        for (i, (field, _)) in self.values.iter().enumerate() {
            if i > 0 {
                out.s(",");
            }
            out.id(&field.id);
        }
        out.s(") values (");
        for (i, (field, val)) in self.values.iter().enumerate() {
            if i > 0 {
                out.s(",");
            }
            let field_type = match ctx.tables.get(&field.table).and_then(|t| t.get(&field)) {
                Some(t) => t,
                None => {
                    ctx.errs.err(path, format!("Insert destination value field {} is not known", field));
                    continue;
                },
            };
            let path = path.push_back(format!("Insert value {} ({})", i, field));
            let res = val.build(ctx, &path, &scope);
            check_assignable(&mut ctx.errs, &path, &field_type, &res.0);
            out.s(&res.1.to_string());
        }
        out.s(")");
        if let Some(conflict) = &self.on_conflict {
            out.s("on conflict");
            match conflict {
                InsertConflict::DoNothing => {
                    out.s("do nothing");
                },
                InsertConflict::DoUpdate { conflict, set } => {
                    out.s("(");
                    for (i, f) in conflict.iter().enumerate() {
                        if i > 0 {
                            out.s(",");
                        }
                        out.id(&f.id);
                    }
                    out.s(")");
                    out.s("do update");
                    build_set(ctx, path, &scope, &mut out, set);
                },
            }
        }
        match (&res_count, &self.on_conflict) {
            (QueryResCount::MaybeOne, Some(InsertConflict::DoUpdate { .. })) => {
                ctx.errs.err(path, format!("Insert with [on conflict update] will always return a row"));
            },
            (QueryResCount::One, Some(InsertConflict::DoNothing)) => {
                ctx.errs.err(path, format!("Insert with [on conflict do nothing] may not return a row"));
            },
            (QueryResCount::Many, _) => {
                ctx.errs.err(path, format!("Insert can at most return one row, but res count is many"));
            },
            (QueryResCount::None, _) | (QueryResCount::One, None) | (QueryResCount::MaybeOne, None) => {
                // handled elsewhere, nop
            },
            (QueryResCount::One, Some(InsertConflict::DoUpdate { .. })) |
            (QueryResCount::MaybeOne, Some(InsertConflict::DoNothing)) => {
                // ok
            },
        }
        let out_type = build_returning(ctx, path, &scope, &mut out, &self.returning, res_count);
        (out_type, out)
    }
}
