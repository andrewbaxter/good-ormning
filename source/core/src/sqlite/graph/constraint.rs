use {
    crate::{
        graphmigrate::Comparison,
        sqlite::schema::constraint::{
            Constraint,
            ConstraintType,
        },
    },
    std::collections::HashSet,
    super::{
        GraphId,
        Node,
        NodeData,
        NodeDataDispatch,
        utils::SqliteMigrateCtx,
    },
};

#[derive(Clone)]
pub struct NodeConstraint_ {
    pub def: Constraint,
    pub table_id: String,
    pub table_renamed_from: Option<String>,
    /// `def.type_` with table and field ids replaced by their pre-rename ids, for
    /// comparing with the previous version.
    pub type_old_ids: ConstraintType,
}

impl NodeConstraint_ {
    pub fn compare(&self, old: &Self, created: &HashSet<GraphId>) -> Comparison {
        // Sqlite constraints are unnamed and the old node is looked up through renames,
        // so only the constraint's (rename-normalized) content matters.
        if created.contains(&GraphId::Table(self.table_id.clone())) || self.type_old_ids != old.def.type_ {
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
    fn create(&self, ctx: &mut SqliteMigrateCtx) {
        // Only reached when the table already exists, otherwise the constraint is
        // coalesced into the table creation.
        ctx
            .errs
            .err(
                &rpds::vector![format!("{}.{}", self.table_id, self.def.id)],
                "Sqlite can't add or change constraints on an existing table. Instead create a new table with the desired constraints, copy the data over in the `migrate` callback, then drop the old table in a later version.".to_string(),
            );
    }

    fn create_coalesce(&mut self, other: Node) -> Option<Node> {
        Some(other)
    }

    fn delete(&self, ctx: &mut SqliteMigrateCtx) {
        // Only reached when the table is kept, otherwise the constraint is coalesced into
        // the table deletion.
        ctx
            .errs
            .err(
                &rpds::vector![format!("{}.{}", self.table_id, self.def.id)],
                "Sqlite can't remove or change constraints on an existing table. Instead create a new table with the desired constraints, copy the data over in the `migrate` callback, then drop the old table in a later version.".to_string(),
            );
    }

    fn delete_coalesce(&mut self, other: Node) -> Option<Node> {
        Some(other)
    }
}
