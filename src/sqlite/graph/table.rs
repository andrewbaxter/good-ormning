use std::collections::HashSet;
use crate::{
    sqlite::{
        schema::{
            table::Table,
            field::Field,
            constraint::{
                Constraint,
                ConstraintType,
            },
        },
        types::to_sql_type,
    },
    graphmigrate::Comparison,
    utils::Tokens,
};
use super::{
    utils::{
        SqliteNodeData,
        SqliteMigrateCtx,
        SqliteNodeDataDispatch,
    },
    Node,
    GraphId,
};

#[derive(Clone)]
pub struct NodeTable_ {
    pub def: Table,
    pub fields: Vec<Field>,
    pub constraints: Vec<Constraint>,
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

impl SqliteNodeData for NodeTable_ {
    fn update(&self, ctx: &mut SqliteMigrateCtx, old: &Self) {
        if old.def.id != self.def.id {
            let mut stmt = Tokens::new();
            stmt.s("alter table").id(&old.def.id).s("rename to").id(&self.def.id);
            ctx.statements.push(stmt.to_string());
        }
    }
}

impl SqliteNodeDataDispatch for NodeTable_ {
    fn create_coalesce(&mut self, other: Node) -> Option<Node> {
        match other {
            Node::Field(f) if f.def.table == self.def => {
                self.fields.push(f.def.clone());
                None
            },
            Node::Constraint(c) if c.def.table == self.def => {
                self.constraints.push(c.def.clone());
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

    fn create(&self, ctx: &mut SqliteMigrateCtx) {
        let mut stmt = Tokens::new();
        stmt.s("create table").id(&self.def.id).s("(");
        let mut i = 0usize;
        for f in &self.fields {
            if f.id == "rowid" {
                continue;
            }
            if i > 0 {
                stmt.s(",");
            }
            i += 1;
            stmt.id(&f.id).s(to_sql_type(&f.0.type_.type_.type_.type_));
            if !f.type_.type_.opt {
                stmt.s("not null");
            }
        }
        for c in &self.constraints {
            if i > 0 {
                stmt.s(",");
            }
            i += 1;
            stmt.s("constraint").id(&c.id);
            match &c.type_ {
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
        }
        stmt.s(")");
        ctx.statements.push(stmt.to_string());
    }

    fn delete(&self, ctx: &mut SqliteMigrateCtx) {
        ctx.statements.push(Tokens::new().s("drop table").id(&self.def.id).to_string());
    }
}
