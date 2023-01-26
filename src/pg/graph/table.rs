use std::collections::HashSet;
use crate::{
    pg::{
        schema::{
            table::Table,
            field::Field,
        },
        types::to_sql_type,
    },
    graphmigrate::Comparison,
    utils::Tokens,
};
use super::{
    utils::{
        NodeData,
        PgMigrateCtx,
        NodeDataDispatch,
    },
    Node,
    GraphId,
};

#[derive(Clone)]
pub struct NodeTable_ {
    pub def: Table,
    pub fields: Vec<Field>,
}

impl NodeTable_ {
    pub fn compare(&self, old: &Self, _created: &HashSet<GraphId>) -> Comparison {
        if old.def.id != self.def.id {
            Comparison::Update
        } else {
            Comparison::DoNothing
        }
    }
}

impl NodeData for NodeTable_ {
    fn update(&self, ctx: &mut PgMigrateCtx, old: &Self) {
        if old.def.id != self.def.id {
            let mut stmt = Tokens::new();
            stmt.s("alter table").id(&old.def.id).s("rename to").id(&self.def.id);
            ctx.statements.push(stmt.to_string());
        }
    }
}

impl NodeDataDispatch for NodeTable_ {
    fn create_coalesce(&mut self, other: Node) -> Option<Node> {
        match other {
            Node::Field(f) if f.def.table == self.def => {
                self.fields.push(f.def.clone());
                None
            },
            other => Some(other),
        }
    }

    fn delete_coalesce(&mut self, other: Node) -> Option<Node> {
        match other {
            Node::Field(f) if f.def.table == self.def => None,
            Node::Constraint(e) if e.def.table == self.def => None,
            Node::Index(e) if e.def.table == self.def => None,
            other => Some(other),
        }
    }

    fn create(&self, ctx: &mut PgMigrateCtx) {
        let mut stmt = Tokens::new();
        stmt.s("create table").id(&self.def.id).s("(");
        for (i, f) in self.fields.iter().enumerate() {
            if i > 0 {
                stmt.s(",");
            }
            stmt.id(&f.id).s(to_sql_type(&f.0.type_.type_.type_.type_));
            if !f.type_.type_.opt {
                stmt.s("not null");
            }
        }
        stmt.s(")");
        ctx.statements.push(stmt.to_string());
    }

    fn delete(&self, ctx: &mut PgMigrateCtx) {
        ctx.statements.push(Tokens::new().s("drop table").id(&self.def.id).to_string());
    }
}
