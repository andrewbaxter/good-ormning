use {
    crate::{
        graphmigrate::Comparison,
        mariadb::schema::constraint::{
            Constraint,
            ConstraintType,
            ForeignKeyAction,
            ForeignKeyDef,
        },
        utils::Tokens,
    },
    std::collections::HashSet,
    super::{
        GraphId,
        Node,
        NodeData,
        NodeDataDispatch,
        utils::MariadbMigrateCtx,
    },
};

#[derive(Clone)]
pub struct NodeConstraint_ {
    pub def: Constraint,
    pub table_id: String,
    pub table_renamed_from: Option<String>,
    pub type_old_ids: ConstraintType,
    pub remote_keys: Vec<GraphId>,
    pub column_types: Vec<String>,
    pub table_columns: Vec<(String, bool)>,
}

impl NodeConstraint_ {
    pub fn compare(&self, old: &Self, created: &HashSet<GraphId>) -> Comparison {
        return if created.contains(&GraphId::Table(self.table_id.clone())) || self.type_old_ids != old.def.type_ ||
            self.remote_keys.iter().any(|k| created.contains(k)) ||
            (matches!(self.def.type_, ConstraintType::ForeignKey(_)) && self.column_types != old.column_types) {
            Comparison::Recreate
        } else if self.def.id != old.def.id {
            Comparison::Update
        } else {
            Comparison::DoNothing
        };
    }

    fn delete_statement(&self, table_id: &str) -> String {
        let mut stmt = Tokens::new_mariadb();
        stmt.s("alter table").id(table_id);
        match &self.def.type_ {
            ConstraintType::PrimaryKey(_) => {
                stmt.s("drop index if exists").id("PRIMARY");
            },
            ConstraintType::ForeignKey(_) => {
                stmt
                    .s("drop foreign key if exists")
                    .id(&self.def.id)
                    .s(", drop index if exists")
                    .id(&self.def.id);
            },
        }
        return stmt.to_string();
    }
}

impl NodeData for NodeConstraint_ {
    fn update(&self, ctx: &mut MariadbMigrateCtx, old: &Self) {
        if self.def.id != old.def.id {
            match &self.def.type_ {
                ConstraintType::PrimaryKey(_) => { },
                ConstraintType::ForeignKey(_) => {
                    ctx.statements.push(old.delete_statement(&self.table_id));
                    self.create(ctx);
                },
            }
        }
    }
}

impl NodeDataDispatch for NodeConstraint_ {
    fn create(&self, ctx: &mut MariadbMigrateCtx) {
        let mut stmt = Tokens::new_mariadb();
        stmt.s("alter table").id(&self.table_id);
        if let ConstraintType::PrimaryKey(_) = &self.def.type_ {
            stmt.s("drop index if exists").id("PRIMARY").s(",");
        }
        stmt.s("add");
        match &self.def.type_ {
            ConstraintType::PrimaryKey(x) => {
                stmt.s("primary key (");
                for (i, f_id) in x.fields.iter().enumerate() {
                    if i > 0 {
                        stmt.s(",");
                    }
                    stmt.id(&ctx.version.tables.get(&self.table_id).unwrap().fields.get(f_id).unwrap().id);
                }
                stmt.s(")");
            },
            ConstraintType::ForeignKey(x) => {
                stmt.s("constraint").id(&self.def.id).s("foreign key if not exists (");
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
                foreign_key_actions(&mut stmt, x);
            },
        }
        ctx.statements.push(stmt.to_string());
    }

    fn create_coalesce(&mut self, other: Node) -> Option<Node> {
        return Some(other);
    }

    fn delete(&self, ctx: &mut MariadbMigrateCtx) {
        if let ConstraintType::PrimaryKey(_) = &self.def.type_ {
            let new_table =
                ctx
                    .version
                    .tables
                    .iter()
                    .find(|(table_id, table)| table.renamed_from.as_ref().unwrap_or(table_id) == &self.table_id)
                    .map(|(_, table)| table);
            if let Some(new_table) = new_table {
                if new_table.constraints.values().any(|c| matches!(c.type_, ConstraintType::PrimaryKey(_))) {
                    return;
                }
                let old_name = |f: &String| {
                    let field = new_table.fields.get(f).unwrap();
                    return field.renamed_from.clone().unwrap_or_else(|| field.id.clone());
                };
                for (field_id, field) in &new_table.fields {
                    if !field.type_.auto_increment || !self.table_columns.contains(&(old_name(field_id), true)) {
                        continue;
                    }
                    let Some(index) =
                        new_table
                            .indices
                            .values()
                            .find(
                                |i| i.renamed_from.is_none() && i.fields.first() == Some(field_id) &&
                                    i
                                        .fields
                                        .iter()
                                        .all(|f| self.table_columns.iter().any(|(c, _)| *c == old_name(f))),
                            ) else {
                            continue;
                        };
                    let mut stmt = Tokens::new_mariadb();
                    stmt.s("alter table").id(&self.table_id).s(if index.unique {
                        "add unique index if not exists"
                    } else {
                        "add index if not exists"
                    }).id(&index.id).s("(");
                    for (i, f) in index.fields.iter().enumerate() {
                        if i > 0 {
                            stmt.s(",");
                        }
                        stmt.id(&old_name(f));
                    }
                    stmt.s("), drop index if exists").id("PRIMARY");
                    ctx.statements.push(stmt.to_string());
                    return;
                }
            }
        }
        ctx.statements.push(self.delete_statement(&self.table_id));
    }

    fn delete_coalesce(&mut self, other: Node) -> Option<Node> {
        return Some(other);
    }
}

pub fn foreign_key_actions(stmt: &mut Tokens, x: &ForeignKeyDef) {
    for (clause, action) in [("on delete", &x.on_delete), ("on update", &x.on_update)] {
        let Some(action) = action else {
            continue;
        };
        stmt.s(clause).s(match action {
            ForeignKeyAction::Cascade => "cascade",
            ForeignKeyAction::NoAction => "no action",
            ForeignKeyAction::Restrict => "restrict",
            ForeignKeyAction::SetNull => "set null",
        });
    }
}
