use std::fmt::Display;
use crate::{
    utils::Tokens,
    graphmigrate::Comparison,
};
use super::{
    table::TableId,
    field::FieldId,
    utils::{
        NodeDataDispatch,
        PgMigrateCtx,
        NodeData,
    },
    node::Node_,
};

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct IndexId(pub TableId, pub String);

impl Display for IndexId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&format!("{}.index {}", self.0, self.1), f)
    }
}

#[derive(Clone)]
pub struct IndexDef {
    pub field_ids: Vec<FieldId>,
    pub unique: bool,
}

#[derive(Clone)]
pub(crate) struct NodeIndex_ {
    pub id: IndexId,
    pub def: IndexDef,
}

impl NodeIndex_ {
    pub fn compare(&self, other: &Self) -> Comparison {
        if self.def.field_ids == other.def.field_ids {
            Comparison::DoNothing
        } else {
            Comparison::DeleteCreate
        }
    }
}

impl NodeDataDispatch for NodeIndex_ {
    fn create_coalesce(&mut self, _other: &Node_) -> bool {
        false
    }

    fn create(&self, ctx: &mut PgMigrateCtx) {
        ctx.statements.push(Tokens::new().s("create").f(|t| {
            if self.def.unique {
                t.s("unique");
            }
        }).s("index").id(&self.id.1).s("on").id(&self.id.0.0).s("(").f(|t| {
            for (i, id) in self.def.field_ids.iter().enumerate() {
                if i > 0 {
                    t.s(",");
                }
                t.id(&id.1);
            }
        }).s(")").to_string());
    }

    fn delete_coalesce(&mut self, _other: &Node_) -> bool {
        false
    }

    fn delete(&self, ctx: &mut PgMigrateCtx) {
        ctx.statements.push(Tokens::new().s("drop index").id(&self.id.1).to_string());
    }
}

impl NodeData for NodeIndex_ {
    fn update(&self, _ctx: &mut PgMigrateCtx, _old: &Self) {
        unreachable!()
    }
}
