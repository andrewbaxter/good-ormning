use {
    crate::{
        graphmigrate::Comparison,
        pg::schema::constraint::{
            Constraint,
            ConstraintType,
        },
        utils::Tokens,
    },
    std::collections::HashSet,
    super::{
        GraphId,
        Node,
        NodeData,
        NodeDataDispatch,
        utils::PgMigrateCtx,
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
    /// For foreign keys, the remote unique constraints and indices backing it. The
    /// foreign key must be dropped before and added after these.
    pub remote_keys: Vec<GraphId>,
}

impl NodeConstraint_ {
    pub fn compare(&self, old: &Self, created: &HashSet<GraphId>) -> Comparison {
        return if created.contains(&GraphId::Table(self.table_id.clone())) || self.type_old_ids != old.def.type_ ||
            self.remote_keys.iter().any(|k| created.contains(k)) {
            Comparison::Recreate
        } else if self.def.id != old.def.id {
            Comparison::Update
        } else {
            Comparison::DoNothing
        };
    }
}

impl NodeData for NodeConstraint_ {
    fn update(&self, ctx: &mut PgMigrateCtx, old: &Self) {
        if self.def.id != old.def.id {
            let mut stmt = Tokens::new();
            stmt
                .s("alter table")
                .id(&self.table_id)
                .s("rename constraint")
                .id(&old.def.id)
                .s("to")
                .id(&self.def.id);
            ctx.statements.push(stmt.to_string());
        }
    }
}

impl NodeDataDispatch for NodeConstraint_ {
    fn create(&self, ctx: &mut PgMigrateCtx) {
        let mut stmt = Tokens::new();
        stmt.s("alter table").id(&self.table_id).s("add constraint").id(&self.def.id);
        match &self.def.type_ {
            ConstraintType::PrimaryKey(x) => {
                stmt.s("primary key (");
                for (i, f_id) in x.fields.iter().enumerate() {
                    if i > 0 {
                        stmt.s(",");
                    }
                    stmt.id(&ctx.version.tables.get(&self.table_id).unwrap().fields.get(f_id).unwrap().id);
                }
                stmt.s(")");
            },
            ConstraintType::ForeignKey(x) => {
                stmt.s("foreign key (");
                for (i, (l_id, _)) in x.fields.iter().enumerate() {
                    if i > 0 {
                        stmt.s(",");
                    }
                    stmt.id(&ctx.version.tables.get(&self.table_id).unwrap().fields.get(l_id).unwrap().id);
                }
                stmt.s(") references").id(ctx.table_sql_names.get(&x.remote_table).unwrap()).s("(");
                for (i, (_, r_id)) in x.fields.iter().enumerate() {
                    if i > 0 {
                        stmt.s(",");
                    }
                    stmt.id(&ctx.version.tables.get(&x.remote_table).unwrap().fields.get(r_id).unwrap().id);
                }
                stmt.s(")");
            },
        }
        ctx.statements.push(stmt.to_string());
    }

    fn create_coalesce(&mut self, other: Node) -> Option<Node> {
        return Some(other);
    }

    fn delete(&self, ctx: &mut PgMigrateCtx) {
        ctx
            .statements
            .push(
                Tokens::new().s("alter table").id(&self.table_id).s("drop constraint").id(&self.def.id).to_string(),
            );
    }

    fn delete_coalesce(&mut self, other: Node) -> Option<Node> {
        return Some(other);
    }
}
