use enum_dispatch::enum_dispatch;
use crate::buildtime::utils::Errs;
use super::{
    node::{
        Node,
    },
};

pub(crate) struct PgMigrateCtx {
    pub(crate) errs: Errs,
    pub(crate) statements: Vec<String>,
    pub(crate) version: i64,
}

impl PgMigrateCtx {
    pub fn new(errs: Errs, version: i64) -> Self {
        Self {
            errs: errs,
            version: version,
            statements: Default::default(),
        }
    }
}

pub(crate) type MigrateNode = crate::buildtime::graphmigrate::Node<Node>;

#[enum_dispatch]
pub(crate) trait NodeDataDispatch {
    fn create_coalesce(&mut self, other: Node) -> Option<Node>;
    fn create(&self, ctx: &mut PgMigrateCtx);
    fn delete_coalesce(&mut self, other: Node) -> Option<Node>;
    fn delete(&self, ctx: &mut PgMigrateCtx);
}

pub(crate) trait NodeData: NodeDataDispatch {
    fn update(&self, ctx: &mut PgMigrateCtx, old: &Self);
}
