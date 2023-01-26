use std::collections::{
    HashSet,
    HashMap,
};
use crate::{
    pg::{
        schema::field::Field,
        types::{
            to_sql_type,
            Type,
            SimpleSimpleType,
        },
        query::{
            utils::PgQueryCtx,
            expr::{
                ExprType,
                ExprValName,
                check_same,
            },
        },
    },
    graphmigrate::Comparison,
    utils::Tokens,
};
use super::{
    GraphId,
    utils::{
        NodeData,
        PgMigrateCtx,
        NodeDataDispatch,
    },
    Node,
};

#[derive(Clone)]
pub(crate) struct NodeField_ {
    pub def: Field,
}

impl NodeField_ {
    pub fn compare(&self, old: &Self, created: &HashSet<GraphId>) -> Comparison {
        if created.contains(&GraphId::Table(self.def.table.0.schema_id.clone())) {
            return Comparison::Recreate;
        }
        let t = &self.def.type_.type_;
        let old_t = &old.def.type_.type_;
        if self.def.id != old.def.id || t.opt != old_t.opt || t.type_.type_ != old_t.type_.type_ {
            Comparison::Update
        } else {
            Comparison::DoNothing
        }
    }

    fn display_path(&self) -> rpds::Vector<String> {
        rpds::vector![self.def.to_string()]
    }
}

impl NodeData for NodeField_ {
    fn update(&self, ctx: &mut PgMigrateCtx, old: &Self) {
        if self.def.id != old.def.id {
            let mut stmt = Tokens::new();
            stmt
                .s("alter table")
                .id(&self.def.table.0.id)
                .s("rename column")
                .id(&old.def.id)
                .s("to")
                .id(&self.def.id);
            ctx.statements.push(stmt.to_string());
        }
        let t = &self.def.type_.type_;
        let old_t = &old.def.type_.type_;
        if t.type_.type_ != old_t.type_.type_ {
            ctx
                .statements
                .push(
                    Tokens::new()
                        .s("alter table")
                        .id(&self.def.table.id)
                        .s("alter column")
                        .id(&self.def.id)
                        .s("set type")
                        .s(to_sql_type(&t.type_.type_))
                        .to_string(),
                );
        }
    }
}

impl NodeDataDispatch for NodeField_ {
    fn create(&self, ctx: &mut PgMigrateCtx) {
        let path = self.display_path();
        if matches!(self.def.type_.type_.type_.type_, SimpleSimpleType::Auto) {
            ctx.errs.err(&path, format!("Auto (serial) fields can't be added after table creation"));
        }
        let mut stmt = Tokens::new();
        stmt
            .s("alter table")
            .id(&self.def.table.0.id)
            .s("add column")
            .id(&self.def.id)
            .s(to_sql_type(&self.def.type_.type_.type_.type_));
        if !self.def.type_.type_.opt {
            if let Some(d) = &self.def.type_.migration_default {
                stmt.s("not null default");
                let qctx_fields = HashMap::new();
                let mut qctx = PgQueryCtx::new(ctx.errs.clone(), &qctx_fields);
                let e_res = d.build(&mut qctx, &path, &HashMap::new());
                check_same(&mut qctx.errs, &path, &ExprType(vec![(ExprValName::empty(), Type {
                    type_: self.def.type_.type_.type_.clone(),
                    opt: false,
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
        if !self.def.type_.type_.opt && self.def.type_.migration_default.is_some() {
            ctx
                .statements
                .push(
                    Tokens::new()
                        .s("alter table")
                        .id(&self.def.table.id)
                        .s("alter column")
                        .id(&self.def.id)
                        .s("drop default")
                        .to_string(),
                );
        }
    }

    fn delete(&self, ctx: &mut PgMigrateCtx) {
        ctx
            .statements
            .push(
                Tokens::new().s("alter table").id(&self.def.table.id).s("drop column").id(&self.def.id).to_string(),
            );
    }

    fn create_coalesce(&mut self, other: Node) -> Option<Node> {
        Some(other)
    }

    fn delete_coalesce(&mut self, other: Node) -> Option<Node> {
        Some(other)
    }
}
