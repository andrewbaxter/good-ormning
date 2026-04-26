use std::collections::{
    HashSet,
    HashMap,
};
use crate::{
    sqlite::{
        schema::{
            field::{
                Field,
                FieldRef,
            },
            table::TableRef,
        },
        types::{
            to_sql_type,
            Type,
            SimpleSimpleType,
        },
        SqliteQueryCtx,
        SqliteTableInfo,
        SqliteFieldInfo,
        query::{
            expr::{
                ExprType,
                Binding,
                check_same,
                Expr,
            },
        },
    },
    graphmigrate::Comparison,
    utils::Tokens,
};
use super::{
    GraphId,
    NodeDataDispatch,
    NodeData,
    Node,
    utils::SqliteMigrateCtx,
};

#[derive(Clone)]
pub(crate) struct NodeField_ {
    pub table_id: String,
    pub table_renamed_from: Option<String>,
    pub def: Field,
}

impl NodeField_ {
    pub fn compare(&self, old: &Self, created: &HashSet<GraphId>) -> Comparison {
        if created.contains(&GraphId::Table(self.table_id.clone())) {
            return Comparison::Recreate;
        }
        let t = &self.def.type_.type_;
        let old_t = &old.def.type_.type_;
        if t.opt != old_t.opt || t.type_.type_ != old_t.type_.type_ {
            Comparison::Recreate
        } else if self.def.id != old.def.id || self.table_id != old.table_id {
            Comparison::Update
        } else {
            Comparison::DoNothing
        }
    }

    fn display_path(&self) -> rpds::Vector<String> {
        rpds::vector![format!("{}.{}", self.table_id, self.def.id)]
    }
}

impl NodeData for NodeField_ {
    fn update(&self, ctx: &mut SqliteMigrateCtx, old: &Self) {
        if self.def.id != old.def.id {
            let mut stmt = Tokens::new();
            stmt.s("alter table").id(&self.table_id).s("rename column").id(&old.def.id).s("to").id(&self.def.id);
            ctx.statements.push(stmt.to_string());
        }
    }
}

impl NodeDataDispatch for NodeField_ {
    fn create(&self, ctx: &mut SqliteMigrateCtx) {
        let path = self.display_path();
        if matches!(self.def.type_.type_.type_.type_, SimpleSimpleType::Auto) {
            ctx.errs.err(&path, format!("Auto (serial) fields can't be added after table creation"));
        }
        let mut stmt = Tokens::new();
        stmt
            .s("alter table")
            .id(&self.table_id)
            .s("add column")
            .id(&self.def.id)
            .s(to_sql_type(&self.def.type_.type_.type_.type_));
        if !self.def.type_.type_.opt {
            if let Some(d) = &self.def.type_.migration_default {
                stmt.s("not null default");
                let mut qctx_tables = HashMap::new();

                // Create a dummy table info for validation
                let mut fields = HashMap::new();
                let field_ref = FieldRef {
                    table_id: self.table_id.clone(),
                    field_id: self.def.id.clone(),
                };
                fields.insert(field_ref, SqliteFieldInfo {
                    sql_name: self.def.id.clone(),
                    type_: self.def.type_.type_.clone(),
                });
                qctx_tables.insert(TableRef(self.table_id.clone()), SqliteTableInfo {
                    sql_name: self.table_id.clone(),
                    fields: fields,
                });
                let mut qctx = SqliteQueryCtx::new(ctx.errs.clone(), qctx_tables);
                let expr: Expr = Expr::from(d.clone());
                let e_res = expr.build(&mut qctx, &path, &HashMap::new());
                check_same(&mut qctx.errs, &path, &ExprType(vec![(Binding::empty(), Type {
                    type_: self.def.type_.type_.type_.clone(),
                    opt: false,
                    arr: false,
                })]), &e_res.0);
                if !qctx.rust_args.is_empty() {
                    qctx
                        .errs
                        .err(
                            &path,
                            format!(
                                "Default expressions must not have any parameters, but this has {} parameters",
                                qctx.rust_args.len()
                            ),
                        );
                }
                stmt.s(&e_res.1.to_string());
            } else {
                ctx.errs.err(&path, format!("New column missing default"));
            }
        }
        ctx.statements.push(stmt.to_string());
    }

    fn delete(&self, _ctx: &mut SqliteMigrateCtx) { }

    fn create_coalesce(&mut self, other: Node) -> Option<Node> {
        Some(other)
    }

    fn delete_coalesce(&mut self, other: Node) -> Option<Node> {
        Some(other)
    }
}
