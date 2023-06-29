use std::collections::HashSet;
use crate::{
    graphmigrate::Comparison,
    sqlite::schema::{
        constraint::{
            Constraint,
        },
    },
    utils::Tokens,
};
use super::{
    utils::{
        SqliteNodeDataDispatch,
        SqliteMigrateCtx,
        SqliteNodeData,
    },
    GraphId,
    Node,
};

#[derive(Clone)]
pub(crate) struct NodeConstraint_ {
    pub def: Constraint,
}

impl NodeConstraint_ {
    pub fn compare(&self, old: &Self, created: &HashSet<GraphId>) -> Comparison {
        if created.contains(&GraphId::Table(self.def.table.schema_id.clone())) || self.def.type_ != old.def.type_ ||
            self.def.id != old.def.id {
            Comparison::Recreate
        } else {
            Comparison::DoNothing
        }
    }

    fn display_path(&self) -> rpds::Vector<String> {
        rpds::vector![self.def.to_string()]
    }
}

impl SqliteNodeDataDispatch for NodeConstraint_ {
    fn create_coalesce(&mut self, other: Node) -> Option<Node> {
        Some(other)
    }

    fn create(&self, ctx: &mut SqliteMigrateCtx) {
        ctx.errs.err(&self.display_path(), format!("New constraints cannot be added after a table was created"));
    }

    fn delete_coalesce(&mut self, other: Node) -> Option<Node> {
        Some(other)
    }

    fn delete(&self, ctx: &mut SqliteMigrateCtx) {
        ctx
            .statements
            .push(
                Tokens::new()
                    .s("alter table")
                    .id(&self.def.table.id)
                    .s("drop constraint")
                    .id(&self.def.id)
                    .to_string(),
            );
    }
}

impl SqliteNodeData for NodeConstraint_ {
    fn update(&self, _ctx: &mut SqliteMigrateCtx, _old: &Self) {
        unreachable!()
    }
}
