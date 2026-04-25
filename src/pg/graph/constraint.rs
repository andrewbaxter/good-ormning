use std::collections::{
    HashSet,
    HashMap,
};
use crate::{
    graphmigrate::Comparison,
    pg::schema::{
        constraint::{
            Constraint,
            ConstraintType,
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
    utils::PgMigrateCtx,
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

    fn create(&self, ctx: &mut PgMigrateCtx) {
        let mut stmt = Tokens::new();
        stmt.s("alter table").id(&self.table_sql_name).s("add constraint").id(&self.def.id);
        match &self.def.type_ {
            crate::pg::schema::constraint::ConstraintType::PrimaryKey(x) => {
                stmt.s("primary key (");
                for (i, f_id) in x.fields.iter().enumerate() {
                    if i > 0 {
                        stmt.s(",");
                    }
                    stmt.id(self.local_field_sql_names.get(f_id).unwrap());
                }
                stmt.s(")");
            },
            crate::pg::schema::constraint::ConstraintType::ForeignKey(x) => {
                stmt.s("foreign key (");
                for (i, (l_id, _)) in x.fields.iter().enumerate() {
                    if i > 0 {
                        stmt.s(",");
                    }
                    stmt.id(self.local_field_sql_names.get(l_id).unwrap());
                }
                stmt.s(") references").id(self.remote_table_sql_name.as_ref().unwrap()).s("(");
                for (i, (_, r_id)) in x.fields.iter().enumerate() {
                    if i > 0 {
                        stmt.s(",");
                    }
                    stmt.id(self.remote_field_sql_names.get(r_id).unwrap());
                }
                stmt.s(")");
            },
        }
        ctx.statements.push(stmt.to_string());
    }

    fn delete_coalesce(&mut self, other: Node) -> Option<Node> {
        Some(other)
    }

    fn delete(&self, ctx: &mut PgMigrateCtx) {
        ctx
            .statements
            .push(
                Tokens::new()
                    .s("alter table")
                    .id(&self.table_sql_name)
                    .s("drop constraint")
                    .id(&self.def.id)
                    .to_string(),
            );
    }
}

impl NodeData for NodeConstraint_ {
    fn update(&self, ctx: &mut PgMigrateCtx, old: &Self) {
        if self.def.id != old.def.id {
            let mut stmt = Tokens::new();
            stmt
                .s("alter table")
                .id(&self.table_sql_name)
                .s("rename constraint")
                .id(&old.def.id)
                .s("to")
                .id(&self.def.id);
            ctx.statements.push(stmt.to_string());
        }
    }
}
