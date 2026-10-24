use {
    crate::{
        graphmigrate::Comparison,
        mariadb::schema::index::Index,
        utils::Tokens,
    },
    std::collections::HashSet,
    super::{
        GraphId,
        utils::MariadbMigrateCtx,
    },
};

#[derive(Clone)]
pub struct NodeIndex_ {
    pub def: Index,
    pub table_id: String,
    pub table_renamed_from: Option<String>,
    pub fields_old_ids: Vec<String>,
}

impl NodeIndex_ {
    pub fn compare(&self, old: &Self, created: &HashSet<GraphId>) -> Comparison {
        return if created.contains(&GraphId::Table(self.table_id.clone())) || self.def.unique != old.def.unique ||
            self.fields_old_ids != old.def.fields {
            Comparison::Recreate
        } else if self.def.id != old.def.id {
            Comparison::Update
        } else {
            Comparison::DoNothing
        };
    }

    pub fn create_clauses(&self, ctx: &MariadbMigrateCtx, merged: bool) -> Vec<Vec<String>> {
        let mut out = vec![];
        let mut clause = Tokens::new_mariadb();
        if merged {
            let mut drop = Tokens::new_mariadb();
            drop.s("drop index if exists").id(&self.def.id);
            out.push(drop.to_string());
        }
        clause.s("add");
        if self.def.unique {
            clause.s("unique");
        }
        clause.s("index");
        if !merged {
            clause.s("if not exists");
        }
        clause.id(&self.def.id).s("(");
        for (i, f_id) in self.def.fields.iter().enumerate() {
            if i > 0 {
                clause.s(",");
            }
            clause.id(&ctx.version.tables.get(&self.table_id).unwrap().fields.get(f_id).unwrap().id);
        }
        clause.s(")");
        out.push(clause.to_string());
        return vec![out];
    }

    pub fn delete_clauses(&self) -> Vec<Vec<String>> {
        let mut clause = Tokens::new_mariadb();
        clause.s("drop index if exists").id(&self.def.id);
        return vec![vec![clause.to_string()]];
    }

    pub fn update_clauses(&self, old: &Self) -> Vec<Vec<String>> {
        if self.def.id == old.def.id {
            return vec![];
        }
        let mut clause = Tokens::new_mariadb();
        clause.s("rename index if exists").id(&old.def.id).s("to").id(&self.def.id);
        return vec![vec![clause.to_string()]];
    }
}
