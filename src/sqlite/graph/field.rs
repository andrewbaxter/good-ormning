use std::collections::{
    HashSet,
    HashMap,
};
use crate::{
    sqlite::{
        schema::field::Field,
        types::{
            to_sql_type,
            Type,
        },
        query::{
            utils::SqliteQueryCtx,
            expr::{
                ExprType,
                Binding,
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
        SqliteNodeData,
        SqliteMigrateCtx,
        SqliteNodeDataDispatch,
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

impl SqliteNodeData for NodeField_ {
    fn update(&self, ctx: &mut SqliteMigrateCtx, old: &Self) {
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
        if t.opt != old_t.opt {
            ctx.errs.err(&self.display_path(), format!("Column optionality cannot be changed in sqlite"));
        }
        if t.type_.type_ != old_t.type_.type_ {
            ctx.errs.err(&self.display_path(), format!("Column types cannot be changed in sqlite"));
        }
    }
}

impl SqliteNodeDataDispatch for NodeField_ {
    fn create(&self, ctx: &mut SqliteMigrateCtx) {
        let path = self.display_path();
        if &self.def.schema_id.0 == "rowid" {
            return;
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
                let mut qctx = SqliteQueryCtx::new(ctx.errs.clone(), HashMap::new());
                let e_res = d.build(&mut qctx, &path, &HashMap::new());
                check_same(&mut qctx.errs, &path, &ExprType(vec![(Binding::empty(), Type {
                    type_: self.def.type_.type_.type_.clone(),
                    opt: false,
                    array: false,
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

    fn delete(&self, ctx: &mut SqliteMigrateCtx) {
        if &self.def.schema_id.0 == "rowid" {
            return;
        }
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
