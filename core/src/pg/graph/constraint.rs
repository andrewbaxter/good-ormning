use std::collections::HashSet;
use crate::{
    pg::{
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
    utils::PgMigrateCtx,
};

#[derive(Clone)]
pub struct NodeConstraint_ {
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
    fn update(&self, _ctx: &mut PgMigrateCtx, _old: &Self) { }
}

impl NodeDataDispatch for NodeConstraint_ {
    fn create(&self, ctx: &mut PgMigrateCtx) {
        let mut stmt = Tokens::new();
        stmt.s("alter table").id(&self.table_id).s("add constraint").id(&self.def.id);
        match &self.def.type_ {
            crate::pg::schema::constraint::ConstraintType::PrimaryKey(x) => {
                stmt.s("primary key (");
                for (i, f_id) in x.fields.iter().enumerate() {
                    if i > 0 {
                        stmt.s(",");
                    }
                    stmt.id(&ctx.version.tables.get(&self.table_id).unwrap().fields.get(f_id).unwrap().id);
                }
                stmt.s(")");
            },
            crate::pg::schema::constraint::ConstraintType::ForeignKey(x) => {
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

    fn delete(&self, ctx: &mut PgMigrateCtx) {
        ctx
            .statements
            .push(
                Tokens::new().s("alter table").id(&self.table_id).s("drop constraint").id(&self.def.id).to_string(),
            );
    }

    fn create_coalesce(&mut self, other: Node) -> Option<Node> {
        Some(other)
    }

    fn delete_coalesce(&mut self, other: Node) -> Option<Node> {
        Some(other)
    }
}
