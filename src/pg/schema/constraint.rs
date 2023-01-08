use std::fmt::Display;
use crate::{
    utils::Tokens,
    graphmigrate::Comparison,
};
use super::{
    table::TableId,
    field::FieldId,
    utils::{
        NodeDataDispatch,
        PgMigrateCtx,
        NodeData,
    },
    node::Node_,
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
pub enum TableConstraintTypeDef {
    PrimaryKey(PrimaryKeyDef),
    ForeignKey(ForeignKeyDef),
}

#[derive(Clone)]
pub struct ConstraintDef {
    pub type_: TableConstraintTypeDef,
}

#[derive(Clone)]
pub(crate) struct NodeConstraint_ {
    pub id: ConstraintId,
    pub def: ConstraintDef,
}

impl NodeConstraint_ {
    pub fn compare(&self, other: &Self) -> Comparison {
        if self.def.type_ == other.def.type_ {
            Comparison::DoNothing
        } else {
            Comparison::DeleteCreate
        }
    }
}

impl NodeDataDispatch for NodeConstraint_ {
    fn create_coalesce(&mut self, _other: &Node_) -> bool {
        false
    }

    fn create(&self, ctx: &mut PgMigrateCtx) {
        let mut stmt = Tokens::new();
        stmt.s("alter table").id(&self.id.0.0).s("add constraint").id(&self.id.1);
        match &self.def.type_ {
            TableConstraintTypeDef::PrimaryKey(x) => {
                stmt.s("primary key (").f(|t| {
                    for (i, id) in x.fields.iter().enumerate() {
                        if i > 0 {
                            t.s(",");
                        }
                        t.id(&id.1);
                    }
                }).s(")");
            },
            TableConstraintTypeDef::ForeignKey(x) => {
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

    fn delete_coalesce(&mut self, _other: &Node_) -> bool {
        false
    }

    fn delete(&self, ctx: &mut PgMigrateCtx) {
        ctx
            .statements
            .push(
                Tokens::new().s("alter table").id(&self.id.0.0).s("drop constraint").id(&self.id.1).to_string(),
            );
    }
}

impl NodeData for NodeConstraint_ {
    fn update(&self, _ctx: &mut PgMigrateCtx, _old: &Self) {
        unreachable!()
    }
}
