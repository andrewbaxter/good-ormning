use std::{
    collections::{
        HashMap,
        HashSet,
    },
};
use crate::{
    pg::{
        QueryResCount,
        schema::{
            field::FieldRef,
            table::TableRef,
        },
        types::SimpleSimpleType,
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
        PgQueryCtx,
        QueryBody,
        build_returning,
        build_set,
        Returning,
    },
};

#[derive(Clone, Debug)]
pub enum InsertConflict {
    DoNothing,
    DoUpdate {
        conflict: Vec<FieldRef>,
        set: Vec<(FieldRef, Expr)>,
    },
}

#[derive(Clone, Debug)]
pub struct Insert {
    pub table: TableRef,
    pub values: Vec<(FieldRef, Expr)>,
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
        let mut check_inserting_fields = HashSet::new();
        for p in &self.values {
            let field_info = match ctx.tables.get(&self.table).and_then(|t| t.fields.get(&p.0)) {
                Some(f) => f,
                None => {
                    ctx.errs.err(path, format!("Unknown field {:?} for insert into {:?}", p.0, self.table));
                    continue;
                },
            };
            if field_info.type_.opt {
                continue;
            }
            if !check_inserting_fields.insert(p.0.clone()) {
                ctx.errs.err(path, format!("Duplicate field {:?} in insert", p.0));
            }
        }
        let mut scope = HashMap::new();
        let table_info = match ctx.tables.get(&self.table) {
            Some(t) => t.clone(),
            None => {
                ctx.errs.err(path, format!("Unknown table {:?} for insert", self.table));
                return (ExprType(vec![]), Tokens::new());
            },
        };
        for (field_ref, info) in &table_info.fields {
            scope.insert(ExprValName::field(field_ref), info.type_.clone());
            if !info.type_.opt && info.type_.type_.type_ != SimpleSimpleType::Auto &&
                !check_inserting_fields.remove(field_ref) {
                ctx
                    .errs
                    .err(path, format!("Field {:?} is a non-optional field but is missing in insert", field_ref));
            }
        }
        drop(check_inserting_fields);

        // Build query
        let mut out = Tokens::new();
        out.s("insert into").id(&table_info.sql_name).s("(");
        for (i, (field_ref, _)) in self.values.iter().enumerate() {
            if i > 0 {
                out.s(",");
            }
            let field_info = table_info.fields.get(field_ref).unwrap().clone();
            out.id(&field_info.sql_name);
        }
        out.s(") values (");
        for (i, (field_ref, val)) in self.values.iter().enumerate() {
            if i > 0 {
                out.s(",");
            }
            let field_info = table_info.fields.get(field_ref).unwrap().clone();
            let path = path.push_back(format!("Insert value {} ({:?})", i, field_ref));
            let res = val.build(ctx, &path, &scope);
            check_assignable(&mut ctx.errs, &path, &field_info.type_, &res.0);
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
                        let field_info = table_info.fields.get(f).unwrap();
                        out.id(&field_info.sql_name);
                    }
                    out.s(")");
                    out.s("do update");
                    build_set(ctx, path, &scope, &mut out, set);
                },
            }
        }
        let out_type = build_returning(ctx, path, &scope, &mut out, &self.returning, res_count);
        (out_type, out)
    }
}
