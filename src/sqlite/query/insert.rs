use std::{
    collections::{
        HashMap,
        HashSet,
    },
};
use crate::{
    sqlite::{
        schema::{
            field::Field,
            table::Table,
        },
        QueryResCount,
    },
    utils::Tokens,
};
use super::{
    expr::{
        check_assignable,
        Expr,
        ExprType,
        Binding,
    },
    select_body::Returning,
    utils::{
        build_returning,
        build_set,
        build_with,
        QueryBody,
        With,
    },
};

pub enum InsertConflict {
    DoNothing,
    DoUpdate(Vec<(Field, Expr)>),
}

pub struct Insert {
    pub with: Option<With>,
    pub table: Table,
    pub values: Vec<(Field, Expr)>,
    pub on_conflict: Option<InsertConflict>,
    pub returning: Vec<Returning>,
}

impl QueryBody for Insert {
    fn build(
        &self,
        ctx: &mut super::utils::SqliteQueryCtx,
        path: &rpds::Vector<String>,
        res_count: QueryResCount,
    ) -> (ExprType, Tokens) {
        let mut out = Tokens::new();

        // Prep
        if let Some(w) = &self.with {
            out.s(&build_with(ctx, path, w).to_string());
        }
        let mut check_inserting_fields = HashSet::new();
        for p in &self.values {
            if p.0.type_.type_.opt {
                continue;
            }
            if !check_inserting_fields.insert(p.0.clone()) {
                ctx.errs.err(path, format!("Duplicate field {} in insert", p.0));
            }
        }
        let mut scope = HashMap::new();
        for field in match ctx.tables.get(&self.table) {
            Some(t) => t,
            None => {
                ctx.errs.err(path, format!("Unknown table {} for insert", self.table));
                return (ExprType(vec![]), Tokens::new());
            },
        } {
            scope.insert(Binding::field(field), field.type_.type_.clone());
            if !field.type_.type_.opt && field.schema_id.0 != "rowid" && !check_inserting_fields.remove(field) {
                ctx.errs.err(path, format!("{} is a non-optional field but is missing in insert", field));
            }
        }
        drop(check_inserting_fields);

        // Build query
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
            let field = match ctx.tables.get(&field.table).and_then(|t| t.get(&field)) {
                Some(t) => t,
                None => {
                    ctx.errs.err(path, format!("Insert destination value field {} is not known", field));
                    continue;
                },
            }.clone();
            let path = path.push_back(format!("Insert value {} ({})", i, field));
            let res = val.build(ctx, &path, &scope);
            check_assignable(&mut ctx.errs, &path, &field.type_.type_, &res.0);
            out.s(&res.1.to_string());
        }
        out.s(")");
        if let Some(c) = &self.on_conflict {
            out.s("on conflict do");
            match c {
                InsertConflict::DoNothing => {
                    out.s("nothing");
                },
                InsertConflict::DoUpdate(values) => {
                    out.s("update");
                    build_set(ctx, path, &scope, &mut out, values);
                },
            }
        }
        match (&res_count, &self.on_conflict) {
            (QueryResCount::MaybeOne, Some(InsertConflict::DoUpdate(_))) => {
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
            (QueryResCount::One, Some(InsertConflict::DoUpdate(_))) |
            (QueryResCount::MaybeOne, Some(InsertConflict::DoNothing)) => {
                // ok
            },
        }
        let out_type = build_returning(ctx, path, &scope, &mut out, &self.returning, res_count);
        (out_type, out)
    }
}
