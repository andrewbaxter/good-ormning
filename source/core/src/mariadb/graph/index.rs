use {
    crate::{
        graphmigrate::Comparison,
        mariadb::schema::index::Index,
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
}

impl NodeData for NodeIndex_ {
    fn update(&self, ctx: &mut MariadbMigrateCtx, old: &Self) {
        if self.def.id != old.def.id {
            ctx
                .statements
                .push(
                    Tokens::new_mariadb()
                        .s("alter table")
                        .id(&self.table_id)
                        .s("rename index if exists")
                        .id(&old.def.id)
                        .s("to")
                        .id(&self.def.id)
                        .to_string(),
                );
        }
    }
}

impl NodeDataDispatch for NodeIndex_ {
    fn create(&self, ctx: &mut MariadbMigrateCtx) {
        let mut stmt = Tokens::new_mariadb();
        stmt.s("create");
        if self.def.unique {
            stmt.s("unique");
        }
        stmt.s("index if not exists").id(&self.def.id).s("on").id(&self.table_id).s("(");
        for (i, f_id) in self.def.fields.iter().enumerate() {
            if i > 0 {
                stmt.s(",");
            }
            stmt.id(&ctx.version.tables.get(&self.table_id).unwrap().fields.get(f_id).unwrap().id);
        }
        stmt.s(")");
        ctx.statements.push(stmt.to_string());
    }

    fn create_coalesce(&mut self, other: Node) -> Option<Node> {
        return Some(other);
    }

    fn delete(&self, ctx: &mut MariadbMigrateCtx) {
        ctx
            .statements
            .push(
                Tokens::new_mariadb()
                    .s("drop index if exists")
                    .id(&self.def.id)
                    .s("on")
                    .id(&self.table_id)
                    .to_string(),
            );
    }

    fn delete_coalesce(&mut self, other: Node) -> Option<Node> {
        return Some(other);
    }
}
