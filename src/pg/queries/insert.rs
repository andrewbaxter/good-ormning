use std::collections::HashMap;
use crate::{
    pg::{
        schema::{
            field::FieldId,
            table::TableId,
        },
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
    select::SelectOutput,
};

pub enum InsertConflict {
    DoNothing,
    Update(Vec<(FieldId, Expr)>),
}

pub struct Insert {
    pub(crate) table: TableId,
    pub(crate) values: Vec<(FieldId, Expr)>,
    pub(crate) on_conflict: Option<InsertConflict>,
    pub(crate) returning: Vec<SelectOutput>,
}

impl QueryBody for Insert {
    fn build(&self, ctx: &mut super::utils::PgQueryCtx) -> (ExprType, Tokens) {
        // Prep
        let mut scope = HashMap::new();
        for (k, v) in match ctx.tables.get(&self.table) {
            Some(t) => t,
            None => {
                ctx.errs.err(format!("Unknown table {} for insert", self.table));
                return (ExprType(vec![]), Tokens::new());
            },
        } {
            scope.insert(k.clone(), v.clone());
        }

        // Build query
        let mut out = Tokens::new();
        out.s("insert into").id(&self.table.0).s("(");
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
            let res = v.build(ctx, &scope);
            let field_type = match ctx.tables.get(&k.0).and_then(|t| t.get(&k)) {
                Some(t) => t,
                None => {
                    ctx.errs.err(format!("Insert destination value field {} is not known", k));
                    continue;
                },
            };
            check_assignable(&mut ctx.errs, &field_type.1, &res.0);
            out.s(&res.1.to_string());
        }
        out.s(")");
        if let Some(c) = &self.on_conflict {
            out.s("on conflict");
            match c {
                InsertConflict::DoNothing => {
                    out.s("do nothing");
                },
                InsertConflict::Update(values) => {
                    build_set(ctx, &scope, &mut out, values);
                },
            }
        }
        let out_type = build_returning(ctx, &scope, &mut out, &self.returning);
        (out_type, out)
    }
}
