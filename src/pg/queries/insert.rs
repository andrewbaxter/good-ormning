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
    },
    utils::{
        Query,
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
    pub table: TableId,
    pub values: Vec<(FieldId, Expr)>,
    pub on_conflict: Option<InsertConflict>,
    pub returning: Vec<SelectOutput>,
}

impl Query for Insert {
    fn build(&self, ctx: &mut super::utils::PgQueryCtx) -> (ExprType, Tokens) {
        // Prep
        let mut all_fields = HashMap::new();
        for (k, v) in match ctx.tables.get(&self.table) {
            Some(t) => t,
            None => {
                ctx.errs.err(format!("Unknown table {} for insert", self.table));
                return (ExprType(vec![]), Tokens::new());
            },
        } {
            all_fields.insert(k.clone(), v.clone());
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
        for (i, (_, v)) in self.values.iter().enumerate() {
            if i > 0 {
                out.s(",");
            }
            let res = v.build(ctx, &all_fields);
            if res.0.assert_scalar(ctx).is_none() {
                continue;
            }
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
                    build_set(ctx, &all_fields, &mut out, values);
                },
            }
        }
        let out_type = build_returning(ctx, &all_fields, &mut out, &self.returning);
        (out_type, out)
    }
}
