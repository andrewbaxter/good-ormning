use std::collections::HashSet;
use crate::{
    graphmigrate::Comparison,
    utils::Tokens,
    pg::schema::index::Index,
};
use super::{
    utils::{
        NodeDataDispatch,
        PgMigrateCtx,
        NodeData,
    },
    GraphId,
    Node,
};

#[derive(Clone)]
pub(crate) struct NodeIndex_ {
    pub def: Index,
}

impl NodeIndex_ {
    pub fn compare(&self, old: &Self, created: &HashSet<GraphId>) -> Comparison {
        if created.contains(&GraphId::Table(self.def.table.schema_id.clone())) || self.def.fields != old.def.fields {
            Comparison::Recreate
        } else if self.def.id != old.def.id {
            Comparison::Update
        } else {
            Comparison::DoNothing
        }
    }
}

impl NodeDataDispatch for NodeIndex_ {
    fn create_coalesce(&mut self, other: Node) -> Option<Node> {
        Some(other)
    }

    fn create(&self, ctx: &mut PgMigrateCtx) {
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

    fn delete(&self, ctx: &mut PgMigrateCtx) {
        ctx.statements.push(Tokens::new().s("drop index").id(&self.def.id).to_string());
    }
}

impl NodeData for NodeIndex_ {
    fn update(&self, ctx: &mut PgMigrateCtx, old: &Self) {
        if self.def.id != old.def.id {
            let mut stmt = Tokens::new();
            stmt.s("alter index").id(&old.def.id).s("rename to").id(&self.def.id);
            ctx.statements.push(stmt.to_string());
        }
    }
}
