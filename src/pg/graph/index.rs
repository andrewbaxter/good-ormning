use std::collections::{
    HashSet,
    HashMap,
};
use crate::{
    graphmigrate::Comparison,
    utils::Tokens,
    pg::schema::{
        index::{
            Index,
            SchemaIndexId,
        },
        table::SchemaTableId,
        field::SchemaFieldId,
    },
};
use super::{
    GraphId,
    NodeDataDispatch,
    NodeData,
    Node,
    utils::PgMigrateCtx,
};

#[derive(Clone)]
pub(crate) struct NodeIndex_ {
    pub table_schema_id: SchemaTableId,
    pub table_id: String, // SQL name
    pub schema_id: SchemaIndexId,
    pub def: Index,
    pub field_sql_names: HashMap<SchemaFieldId, String>,
}

impl NodeIndex_ {
    pub fn compare(&self, old: &Self, created: &HashSet<GraphId>) -> Comparison {
        if created.contains(&GraphId::Table(self.table_schema_id.clone())) || self.def.fields != old.def.fields {
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

    fn create(&self, _ctx: &mut PgMigrateCtx) {
        // Coalesced into table creation
    }

    fn delete_coalesce(&mut self, other: Node) -> Option<Node> {
        Some(other)
    }

    fn delete(&self, _ctx: &mut PgMigrateCtx) {
        // Coalesced into table deletion
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
