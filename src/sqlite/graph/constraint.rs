use std::collections::{
    HashSet,
    HashMap,
};
use crate::{
    graphmigrate::Comparison,
    sqlite::schema::{
        constraint::{
            Constraint,
            SchemaConstraintId,
        },
        table::SchemaTableId,
        field::SchemaFieldId,
    },
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
    pub table_schema_id: SchemaTableId,
    pub table_sql_name: String,
    pub schema_id: SchemaConstraintId,
    pub def: Constraint,
    pub local_field_sql_names: HashMap<SchemaFieldId, String>,
    pub remote_table_sql_name: Option<String>,
    pub remote_field_sql_names: HashMap<SchemaFieldId, String>,
}

impl NodeConstraint_ {
    pub fn compare(&self, old: &Self, created: &HashSet<GraphId>) -> Comparison {
        if created.contains(&GraphId::Table(self.table_schema_id.clone())) || self.def.type_ != old.def.type_ {
            Comparison::Recreate
        } else if self.def.id != old.def.id {
            Comparison::Update
        } else {
            Comparison::DoNothing
        }
    }
}

impl NodeDataDispatch for NodeConstraint_ {
    fn create_coalesce(&mut self, other: Node) -> Option<Node> {
        Some(other)
    }

    fn create(&self, ctx: &mut SqliteMigrateCtx) {
        ctx.errs.err(&rpds::vector![format!("Constraint {:?}", self.schema_id)], format!("SQLite doesn't support adding constraints after table creation"));
    }

    fn delete_coalesce(&mut self, other: Node) -> Option<Node> {
        Some(other)
    }

    fn delete(&self, ctx: &mut SqliteMigrateCtx) {
        ctx.errs.err(&rpds::vector![format!("Constraint {:?}", self.schema_id)], format!("SQLite doesn't support dropping constraints"));
    }
}

impl NodeData for NodeConstraint_ {
    fn update(&self, _ctx: &mut SqliteMigrateCtx, _old: &Self) {
        // No-op for SQLite as it doesn't support renaming constraints
    }
}
