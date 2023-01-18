use enum_dispatch::enum_dispatch;
use crate::buildtime::utils::Errs;
use super::{
    node::{
        Node,
    },
};

pub(crate) struct SqliteMigrateCtx {
    pub(crate) errs: Errs,
    pub statements: Vec<String>,
    pub version: i64,
}

impl SqliteMigrateCtx {
    pub fn new(errs: Errs, version: i64) -> Self {
        Self {
            errs: errs,
            statements: Default::default(),
            version: version,
        }
    }
}

pub(crate) type MigrateNode = crate::buildtime::graphmigrate::Node<Node>;

#[enum_dispatch]
pub(crate) trait SqliteNodeDataDispatch {
    fn create_coalesce(&mut self, other: Node) -> Option<Node>;
    fn create(&self, ctx: &mut SqliteMigrateCtx);
    fn delete_coalesce(&mut self, other: Node) -> Option<Node>;
    fn delete(&self, ctx: &mut SqliteMigrateCtx);
}

pub(crate) trait SqliteNodeData: SqliteNodeDataDispatch {
    fn update(&self, ctx: &mut SqliteMigrateCtx, old: &Self);
}
