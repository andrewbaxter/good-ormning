use std::collections::HashSet;
use crate::{
    sqlite::{
        schema::{
            table::{
                Table,
                SchemaTableId,
            },
        },
        types::to_sql_type,
    },
    graphmigrate::Comparison,
    utils::Tokens,
};
use super::{
    GraphId,
    NodeDataDispatch,
    NodeData,
    Node,
    utils::SqliteMigrateCtx,
};

#[derive(Clone)]
pub struct NodeTable_ {
    pub schema_id: SchemaTableId,
    pub def: Table,
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
    fn update(&self, ctx: &mut SqliteMigrateCtx, old: &Self) {
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
            Node::Field(f) if f.table_schema_id == self.schema_id => {
                None
            },
            Node::Constraint(c) if c.table_schema_id == self.schema_id => {
                self.def.constraints.insert(c.schema_id.clone(), c.def.clone());
                None
            },
            Node::Index(i) if i.table_schema_id == self.schema_id => {
                self.def.indices.insert(i.schema_id.clone(), i.def.clone());
                None
            },
            other => Some(other),
        }
    }

    fn delete_coalesce(&mut self, other: Node) -> Option<Node> {
        match other {
            Node::Field(f) if f.table_schema_id == self.schema_id => None,
            Node::Constraint(e) if e.table_schema_id == self.schema_id => None,
            Node::Index(e) if e.table_schema_id == self.schema_id => None,
            other => Some(other),
        }
    }

    fn create(&self, ctx: &mut SqliteMigrateCtx) {
        let mut stmt = Tokens::new();
        stmt.s("create table").id(&self.def.id).s("(");
        let mut first = true;
        for f in self.def.fields.values() {
            if f.id == "rowid" {
                continue;
            }
            if !first {
                stmt.s(",");
            }
            first = false;
            stmt.id(&f.id).s(to_sql_type(&f.type_.type_.type_.type_));
            if !f.type_.type_.opt {
                stmt.s("not null");
            }
        }
        for c in self.def.constraints.values() {
            if !first {
                stmt.s(",");
            }
            first = false;
            stmt.s("constraint").id(&c.id);
            match &c.type_ {
                crate::sqlite::schema::constraint::ConstraintType::PrimaryKey(x) => {
                    stmt.s("primary key (");
                    for (i, f_id) in x.fields.iter().enumerate() {
                        if i > 0 {
                            stmt.s(",");
                        }
                        stmt.id(&self.def.fields.get(f_id).unwrap().id);
                    }
                    stmt.s(")");
                },
                crate::sqlite::schema::constraint::ConstraintType::ForeignKey(x) => {
                    stmt.s("foreign key (");
                    for (i, (l_id, _)) in x.fields.iter().enumerate() {
                        if i > 0 {
                            stmt.s(",");
                        }
                        stmt.id(&self.def.fields.get(l_id).unwrap().id);
                    }
                    stmt.s(") references").id(&ctx.table_sql_names.get(&x.remote_table).unwrap()).s("(");
                    for (i, (_, r_id)) in x.fields.iter().enumerate() {
                        if i > 0 {
                            stmt.s(",");
                        }
                        // This assumes the remote table is already in the context or we can look it up.
                        // In the new architecture, we might need a way to get remote field names.
                        // For now, let's assume the field names are the same as schema IDs or we have a lookup.
                        // Actually, Version has the info.
                        stmt.id(&ctx.version.tables.get(&x.remote_table).unwrap().fields.get(r_id).unwrap().id);
                    }
                    stmt.s(")");
                },
            }
        }
        stmt.s(")");
        ctx.statements.push(stmt.to_string());
        for i in self.def.indices.values() {
            let mut stmt = Tokens::new();
            stmt.s("create");
            if i.unique {
                stmt.s("unique");
            }
            stmt.s("index").id(&i.id).s("on").id(&self.def.id).s("(");
            for (j, f_id) in i.fields.iter().enumerate() {
                if j > 0 {
                    stmt.s(",");
                }
                stmt.id(&self.def.fields.get(f_id).unwrap().id);
            }
            stmt.s(")");
            ctx.statements.push(stmt.to_string());
        }
    }

    fn delete(&self, ctx: &mut SqliteMigrateCtx) {
        ctx.statements.push(Tokens::new().s("drop table").id(&self.def.id).to_string());
    }
}
