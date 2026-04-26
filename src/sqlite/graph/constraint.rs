use std::collections::HashSet;
use crate::{
    sqlite::{
        schema::{
            constraint::{
                Constraint,
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
pub(crate) struct NodeConstraint_ {
    pub table_id: String,
    pub table_renamed_from: Option<String>,
    pub def: Constraint,
}

impl NodeConstraint_ {
    pub fn compare(&self, old: &Self, created: &HashSet<GraphId>) -> Comparison {
        if created.contains(&GraphId::Table(self.table_id.clone())) || self.table_id != old.table_id ||
            self.def.id != old.def.id ||
            self.def.type_ != old.def.type_ {
            Comparison::Recreate
        } else {
            Comparison::DoNothing
        }
    }
}

impl NodeData for NodeConstraint_ {
    fn update(&self, _ctx: &mut SqliteMigrateCtx, _old: &Self) { }
}

impl NodeDataDispatch for NodeConstraint_ {
    fn create(&self, _ctx: &mut SqliteMigrateCtx) { }

    fn delete(&self, _ctx: &mut SqliteMigrateCtx) { }

    fn create_coalesce(&mut self, other: Node) -> Option<Node> {
        Some(other)
    }

    fn delete_coalesce(&mut self, other: Node) -> Option<Node> {
        Some(other)
    }
}
