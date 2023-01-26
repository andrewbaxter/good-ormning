use std::collections::HashSet;
use crate::{
    graphmigrate::Comparison,
    sqlite::schema::index::Index,
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
pub(crate) struct NodeIndex_ {
    pub def: Index,
}

impl NodeIndex_ {
    pub fn compare(&self, other: &Self, created: &HashSet<GraphId>) -> Comparison {
        if created.contains(&GraphId::Table(self.def.table.schema_id.clone())) ||
            self.def.fields != other.def.fields {
            Comparison::Recreate
        } else {
            Comparison::DoNothing
        }
    }
}

impl SqliteNodeDataDispatch for NodeIndex_ {
    fn create_coalesce(&mut self, other: Node) -> Option<Node> {
        Some(other)
    }

    fn create(&self, ctx: &mut SqliteMigrateCtx) {
        ctx.statements.push(Tokens::new().s("create").f(|t| {
            if self.def.unique {
                t.s("unique");
            }
        }).s("index").id(&self.def.id).s("on").id(&self.def.table.id).s("(").f(|t| {
            for (i, field) in self.def.fields.iter().enumerate() {
                if i > 0 {
                    t.s(",");
                }
                t.id(&field.id);
            }
        }).s(")").to_string());
    }

    fn delete_coalesce(&mut self, other: Node) -> Option<Node> {
        Some(other)
    }

    fn delete(&self, ctx: &mut SqliteMigrateCtx) {
        ctx.statements.push(Tokens::new().s("drop index").id(&self.def.id).to_string());
    }
}

impl SqliteNodeData for NodeIndex_ {
    fn update(&self, _ctx: &mut SqliteMigrateCtx, _old: &Self) {
        unreachable!()
    }
}
