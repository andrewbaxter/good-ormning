use std::collections::HashSet;
use crate::{
    graphmigrate::Comparison,
    pg::schema::{
        constraint::{
            Constraint,
            ConstraintType,
        },
    },
    utils::Tokens,
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
pub(crate) struct NodeConstraint_ {
    pub def: Constraint,
}

impl NodeConstraint_ {
    pub fn compare(&self, old: &Self, created: &HashSet<GraphId>) -> Comparison {
        if created.contains(&GraphId::Table(self.def.table.schema_id.clone())) || self.def.type_ != old.def.type_ {
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
        stmt.s("alter table").id(&self.def.table.id).s("add constraint").id(&self.def.id);
        match &self.def.type_ {
            ConstraintType::PrimaryKey(x) => {
                stmt.s("primary key (").f(|t| {
                    for (i, field) in x.fields.iter().enumerate() {
                        if i > 0 {
                            t.s(",");
                        }
                        t.id(&field.id);
                    }
                }).s(")");
            },
            ConstraintType::ForeignKey(x) => {
                stmt.s("foreign key (").f(|t| {
                    for (i, pair) in x.fields.iter().enumerate() {
                        if i > 0 {
                            t.s(",");
                        }
                        t.id(&pair.0.id);
                    }
                }).s(") references ").f(|t| {
                    for (i, pair) in x.fields.iter().enumerate() {
                        if i == 0 {
                            t.id(&pair.1.table.id).s("(");
                        } else {
                            t.s(",");
                        }
                        t.id(&pair.1.id);
                    }
                }).s(")");
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
                    .id(&self.def.table.id)
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
                .id(&self.def.table.0.id)
                .s("rename constraint")
                .id(&old.def.id)
                .s("to")
                .id(&self.def.id);
            ctx.statements.push(stmt.to_string());
        }
    }
}
