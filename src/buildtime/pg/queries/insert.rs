use std::collections::HashMap;
use crate::buildtime::{
    pg::{
        schema::{
            field::FieldId,
            table::TableId,
        },
        QueryResCount,
    },
    utils::Tokens,
};
use super::{
    expr::{
        Expr,
        ExprType,
        check_assignable,
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
        conflict: Vec<FieldId>,
        set: Vec<(FieldId, Expr)>,
    },
}

pub struct Insert {
    pub table: TableId,
    pub values: Vec<(FieldId, Expr)>,
    pub on_conflict: Option<InsertConflict>,
    pub returning: Vec<Returning>,
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
        for (k, v) in match ctx.tables.get(&self.table) {
            Some(t) => t,
            None => {
                ctx.errs.err(path, format!("Unknown table {} for insert", self.table));
                return (ExprType(vec![]), Tokens::new());
            },
        } {
            scope.insert(k.clone(), v.clone());
        }

        // Build query
        let mut out = Tokens::new();
        out.s("insert into").id(&self.table.at(ctx.version)).s("(");
        for (i, (k, _)) in self.values.iter().enumerate() {
            if i > 0 {
                out.s(",");
            }
            out.id(&k.1);
        }
        out.s(") values (");
        for (i, (k, v)) in self.values.iter().enumerate() {
            if i > 0 {
                out.s(",");
            }
            let field_type = match ctx.tables.get(&k.0).and_then(|t| t.get(&k)) {
                Some(t) => t,
                None => {
                    ctx.errs.err(path, format!("Insert destination value field {} is not known", k));
                    continue;
                },
            };
            let path = path.push_back(format!("Insert value {} ({} {})", i, k, field_type.0));
            let res = v.build(ctx, &path, &scope);
            check_assignable(&mut ctx.errs, &path, &field_type.1, &res.0);
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
                        out.id(&f.1);
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
