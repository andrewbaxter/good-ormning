use {
    crate::{
        graphmigrate::Comparison,
        pg::schema::index::Index,
        utils::Tokens,
    },
    std::collections::HashSet,
    super::{
        GraphId,
        Node,
        NodeData,
        NodeDataDispatch,
        utils::PgMigrateCtx,
    },
};

#[derive(Clone)]
pub struct NodeIndex_ {
    pub def: Index,
    pub table_id: String,
    pub table_renamed_from: Option<String>,
    /// `def.fields` with field ids replaced by their pre-rename ids, for comparing
    /// with the previous version.
    pub fields_old_ids: Vec<String>,
}

impl NodeIndex_ {
    pub fn compare(&self, old: &Self, created: &HashSet<GraphId>) -> Comparison {
        // The old node is looked up through renames, so table renames don't matter
        if created.contains(&GraphId::Table(self.table_id.clone())) || self.def.unique != old.def.unique ||
            self.fields_old_ids != old.def.fields {
            Comparison::Recreate
        } else if self.def.id != old.def.id {
            Comparison::Update
        } else {
            Comparison::DoNothing
        }
    }
}

impl NodeData for NodeIndex_ {
    fn update(&self, ctx: &mut PgMigrateCtx, old: &Self) {
        if self.def.id != old.def.id {
            ctx
                .statements
                .push(Tokens::new().s("alter index").id(&old.def.id).s("rename to").id(&self.def.id).to_string());
        }
    }
}

impl NodeDataDispatch for NodeIndex_ {
    fn create(&self, ctx: &mut PgMigrateCtx) {
        let mut stmt = Tokens::new();
        stmt.s("create");
        if self.def.unique {
            stmt.s("unique");
        }
        stmt.s("index").id(&self.def.id).s("on").id(&self.table_id).s("(");
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
        Some(other)
    }

    fn delete(&self, ctx: &mut PgMigrateCtx) {
        ctx.statements.push(Tokens::new().s("drop index").id(&self.def.id).to_string());
    }

    fn delete_coalesce(&mut self, other: Node) -> Option<Node> {
        Some(other)
    }
}
