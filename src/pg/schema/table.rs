use std::fmt::{
    Display,
    Debug,
};
use crate::{
    utils::Tokens,
    pg::types::to_sql_type,
    graphmigrate::Comparison,
};
use super::{
    field::{
        FieldId,
        FieldDef,
    },
    utils::{
        NodeData,
        PgMigrateCtx,
        NodeDataDispatch,
    },
    node::Node_,
};

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct TableId(pub String);

impl Display for TableId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

#[derive(Clone)]
pub struct NodeTable_ {
    pub id: TableId,
    pub fields: Vec<(FieldId, FieldDef)>,
}

impl NodeTable_ {
    pub fn compare(&self, _old: &Self) -> Comparison {
        Comparison::DoNothing
    }
}

impl NodeData for NodeTable_ {
    fn update(&self, _ctx: &mut PgMigrateCtx, _old: &Self) {
        unreachable!();
    }
}

impl NodeDataDispatch for NodeTable_ {
    fn create_coalesce(&mut self, other: &Node_) -> bool {
        match other {
            Node_::Field(f) if f.id.0 == self.id => {
                self.fields.push((f.id.clone(), f.def.clone()));
                true
            },
            _ => false,
        }
    }

    fn delete_coalesce(&mut self, other: &Node_) -> bool {
        match other {
            Node_::Field(f) if f.id.0 == self.id => true,
            Node_::Constraint(e) if e.id.0 == self.id => true,
            Node_::Index(e) if e.id.0 == self.id => true,
            _ => false,
        }
    }

    fn create(&self, ctx: &mut PgMigrateCtx) {
        let mut stmt = Tokens::new();
        stmt.s("create table").id(&self.id.0).s("(");
        for (i, f) in self.fields.iter().enumerate() {
            if i > 0 {
                stmt.s(",");
            }
            stmt.id(&f.0.1).s(to_sql_type(&f.1.type_.type_.type_.type_));
            if !f.1.type_.type_.opt {
                stmt.s("not null");
            }
        }
        stmt.s(")");
        ctx.statements.push(stmt.to_string());
    }

    fn delete(&self, ctx: &mut PgMigrateCtx) {
        ctx.statements.push(Tokens::new().s("drop table").id(&self.id.0).to_string());
    }
}
