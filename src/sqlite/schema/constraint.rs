use std::{
    fmt::Display,
    collections::HashSet,
};
use crate::{
    utils::Tokens,
    graphmigrate::Comparison,
};
use super::{
    table::TableId,
    field::FieldId,
    utils::{
        SqliteNodeDataDispatch,
        SqliteMigrateCtx,
        SqliteNodeData,
    },
    node::{
        Node,
        Id,
    },
};

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct ConstraintId(pub TableId, pub String);

impl Display for ConstraintId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&format!("{}.constraint {}", self.0, self.1), f)
    }
}

#[derive(Clone, PartialEq)]
pub struct PrimaryKeyDef {
    pub fields: Vec<FieldId>,
}

#[derive(Clone, PartialEq)]
pub struct ForeignKeyDef {
    pub fields: Vec<(FieldId, FieldId)>,
}

#[derive(Clone, PartialEq)]
pub enum ConstraintType {
    PrimaryKey(PrimaryKeyDef),
    ForeignKey(ForeignKeyDef),
}

#[derive(Clone)]
pub struct ConstraintDef {
    pub type_: ConstraintType,
}

#[derive(Clone)]
pub(crate) struct NodeConstraint_ {
    pub id: ConstraintId,
    pub def: ConstraintDef,
}

impl NodeConstraint_ {
    pub fn compare(&self, other: &Self, created: &HashSet<Id>) -> Comparison {
        if created.contains(&Id::Table(self.id.0.clone())) || self.def.type_ != other.def.type_ {
            Comparison::Recreate
        } else {
            Comparison::DoNothing
        }
    }
}

impl SqliteNodeDataDispatch for NodeConstraint_ {
    fn create_coalesce(&mut self, other: Node) -> Option<Node> {
        Some(other)
    }

    fn create(&self, ctx: &mut SqliteMigrateCtx) {
        let mut stmt = Tokens::new();
        stmt.s("alter table").id(&self.id.0.0).s("add constraint").id(&self.id.1);
        match &self.def.type_ {
            ConstraintType::PrimaryKey(x) => {
                stmt.s("primary key (").f(|t| {
                    for (i, id) in x.fields.iter().enumerate() {
                        if i > 0 {
                            t.s(",");
                        }
                        t.id(&id.1);
                    }
                }).s(")");
            },
            ConstraintType::ForeignKey(x) => {
                stmt.s("foreign key (").f(|t| {
                    for (i, id) in x.fields.iter().enumerate() {
                        if i > 0 {
                            t.s(",");
                        }
                        t.id(&id.1.1);
                    }
                }).s(") references ").f(|t| {
                    for (i, id) in x.fields.iter().enumerate() {
                        if i == 0 {
                            t.id(&id.1.0.0).s("(");
                        } else {
                            t.s(",");
                        }
                        t.id(&id.1.1);
                    }
                }).s(")");
            },
        }
        ctx.statements.push(stmt.to_string());
    }

    fn delete_coalesce(&mut self, other: Node) -> Option<Node> {
        Some(other)
    }

    fn delete(&self, ctx: &mut SqliteMigrateCtx) {
        ctx
            .statements
            .push(
                Tokens::new().s("alter table").id(&self.id.0.0).s("drop constraint").id(&self.id.1).to_string(),
            );
    }
}

impl SqliteNodeData for NodeConstraint_ {
    fn update(&self, _ctx: &mut SqliteMigrateCtx, _old: &Self) {
        unreachable!()
    }
}
