use {
    crate::{
        graphmigrate::Comparison,
        mariadb::{
            schema::{
                constraint::ConstraintType,
                table::Table,
            },
        },
        utils::Tokens,
    },
    std::collections::HashSet,
    super::{
        GraphId,
        Node,
        constraint::foreign_key_actions,
        field::column_definition,
        utils::MariadbMigrateCtx,
    },
};

#[derive(Clone)]
pub struct NodeTable_ {
    pub def: Table,
    pub table_id: String,
}

impl NodeTable_ {
    pub fn compare(&self, old: &Self, _created: &HashSet<GraphId>) -> Comparison {
        return if old.def.id != self.def.id {
            Comparison::Update
        } else {
            Comparison::DoNothing
        };
    }

    pub fn update(&self, ctx: &mut MariadbMigrateCtx, old: &Self) {
        if old.def.id != self.def.id {
            let mut stmt = Tokens::new_mariadb();
            stmt.s("rename table if exists").id(&old.def.id).s("to").id(&self.def.id);
            ctx.statements.push(stmt.to_string());
        }
    }

    pub fn create(&self, ctx: &mut MariadbMigrateCtx) {
        let mut stmt = Tokens::new_mariadb();
        stmt.s("create table if not exists").id(&self.def.id).s("(");
        let mut first = true;
        for f in self.def.fields.values() {
            if !first {
                stmt.s(",");
            }
            first = false;
            stmt.id(&f.id).s(&column_definition(ctx, &self.table_id, f));
        }
        for c in self.def.constraints.values() {
            if let ConstraintType::ForeignKey(x) = &c.type_ {
                if x.remote_table != self.table_id {
                    continue;
                }
            }
            if !first {
                stmt.s(",");
            }
            first = false;
            match &c.type_ {
                ConstraintType::PrimaryKey(x) => {
                    stmt.s("primary key (");
                    for (i, f_id) in x.fields.iter().enumerate() {
                        if i > 0 {
                            stmt.s(",");
                        }
                        stmt.id(&self.def.fields.get(f_id).unwrap().id);
                    }
                    stmt.s(")");
                },
                ConstraintType::ForeignKey(x) => {
                    stmt.s("constraint").id(&c.id).s("foreign key (");
                    for (i, (l_id, _)) in x.fields.iter().enumerate() {
                        if i > 0 {
                            stmt.s(",");
                        }
                        stmt.id(&self.def.fields.get(l_id).unwrap().id);
                    }
                    stmt.s(") references").id(ctx.table_sql_names.get(&x.remote_table).unwrap()).s("(");
                    for (i, (_, r_id)) in x.fields.iter().enumerate() {
                        if i > 0 {
                            stmt.s(",");
                        }
                        stmt.id(&ctx.version.tables.get(&x.remote_table).unwrap().fields.get(r_id).unwrap().id);
                    }
                    stmt.s(")");
                    foreign_key_actions(&mut stmt, x);
                },
            }
        }
        for i in self.def.indices.values() {
            if !first {
                stmt.s(",");
            }
            first = false;
            if i.unique {
                stmt.s("unique");
            }
            stmt.s("index").id(&i.id).s("(");
            for (j, f_id) in i.fields.iter().enumerate() {
                if j > 0 {
                    stmt.s(",");
                }
                stmt.id(&self.def.fields.get(f_id).unwrap().id);
            }
            stmt.s(")");
        }
        stmt.s(")");
        ctx.statements.push(stmt.to_string());
    }

    pub fn create_coalesce(&mut self, other: Node) -> Option<Node> {
        return match other {
            Node::Field(f) if f.table_id == self.table_id => {
                None
            },
            Node::Constraint(c) if c.table_id == self.table_id && match &c.def.type_ {
                ConstraintType::PrimaryKey(_) => true,
                ConstraintType::ForeignKey(x) => x.remote_table == self.table_id,
            } => {
                self.def.constraints.insert(c.def.id.clone(), c.def.clone());
                None
            },
            Node::Index(i) if i.table_id == self.table_id => {
                self.def.indices.insert(i.def.id.clone(), i.def.clone());
                None
            },
            other => Some(other),
        };
    }

    pub fn delete(&self, ctx: &mut MariadbMigrateCtx) {
        ctx.statements.push(Tokens::new_mariadb().s("drop table if exists").id(&self.def.id).to_string());
    }

    pub fn delete_coalesce(&mut self, other: Node) -> Option<Node> {
        return match other {
            Node::Field(f) if f.table_id == self.table_id => None,
            Node::Constraint(e) if e.table_id == self.table_id && match &e.def.type_ {
                ConstraintType::PrimaryKey(_) => true,
                ConstraintType::ForeignKey(x) => x.remote_table == self.table_id,
            } => None,
            Node::Index(e) if e.table_id == self.table_id => None,
            other => Some(other),
        };
    }
}
