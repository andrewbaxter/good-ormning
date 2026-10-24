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

    pub fn create_clauses(&self, ctx: &MariadbMigrateCtx, merged: bool) -> Vec<Vec<String>> {
        let table = ctx.version.tables.get(&self.table_id).unwrap();
        let mut out = vec![];
        let mut clause = Tokens::new_mariadb();
        match &self.def.type_ {
            ConstraintType::PrimaryKey(x) => {
                let mut drop = Tokens::new_mariadb();
                drop.s("drop index if exists").id("PRIMARY");
                out.push(drop.to_string());
                clause.s("add primary key (");
                for (i, f_id) in x.fields.iter().enumerate() {
                    if i > 0 {
                        clause.s(",");
                    }
                    clause.id(&table.fields.get(f_id).unwrap().id);
                }
                clause.s(")");
            },
            ConstraintType::ForeignKey(x) => {
                if merged {
                    let mut drop = Tokens::new_mariadb();
                    drop.s("drop foreign key if exists").id(&self.def.id);
                    out.push(drop.to_string());
                }
                clause.s("add constraint").id(&self.def.id).s("foreign key");
                if !merged {
                    clause.s("if not exists");
                }
                clause.s("(");
                for (i, (l_id, _)) in x.fields.iter().enumerate() {
                    if i > 0 {
                        clause.s(",");
                    }
                    clause.id(&table.fields.get(l_id).unwrap().id);
                }
                clause.s(") references").id(ctx.table_sql_names.get(&x.remote_table).unwrap()).s("(");
                for (i, (_, r_id)) in x.fields.iter().enumerate() {
                    if i > 0 {
                        clause.s(",");
                    }
                    clause.id(&ctx.version.tables.get(&x.remote_table).unwrap().fields.get(r_id).unwrap().id);
                }
                clause.s(")");
                foreign_key_actions(&mut clause, x);
            },
        }
        out.push(clause.to_string());
        return vec![out];
    }

    pub fn delete_clauses(&self) -> Vec<Vec<String>> {
        let mut out = vec![];
        match &self.def.type_ {
            ConstraintType::PrimaryKey(_) => {
                let mut clause = Tokens::new_mariadb();
                clause.s("drop index if exists").id("PRIMARY");
                out.push(clause.to_string());
            },
            ConstraintType::ForeignKey(_) => {
                let mut clause = Tokens::new_mariadb();
                clause.s("drop foreign key if exists").id(&self.def.id);
                out.push(clause.to_string());
                let mut clause = Tokens::new_mariadb();
                clause.s("drop index if exists").id(&self.def.id);
                out.push(clause.to_string());
            },
        }
        return vec![out];
    }

    pub fn update_clauses(&self, ctx: &MariadbMigrateCtx, old: &Self, merged: bool) -> Vec<Vec<String>> {
        if self.def.id == old.def.id {
            return vec![];
        }
        return match &self.def.type_ {
            ConstraintType::PrimaryKey(_) => vec![],
            ConstraintType::ForeignKey(_) => old
                .delete_clauses()
                .into_iter()
                .chain(self.create_clauses(ctx, merged).into_iter())
                .collect(),
        };
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
