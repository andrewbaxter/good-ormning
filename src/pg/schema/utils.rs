use enum_dispatch::enum_dispatch;
use crate::utils::Errs;
use super::{
    node::{
        Node,
        Id,
        Node_,
    },
};

pub(crate) struct PgMigrateCtx<'a> {
    pub(crate) errs: &'a mut Errs,
    pub statements: Vec<String>,
}

impl<'a> PgMigrateCtx<'a> {
    pub fn new(errs: &'a mut Errs) -> Self {
        Self {
            errs: errs,
            statements: Default::default(),
        }
    }
}

pub(crate) type MigrateNode<'a> = crate::graphmigrate::Node<Node<'a>, Id>;

#[enum_dispatch]
pub(crate) trait NodeDataDispatch {
    fn create_coalesce(&mut self, other: &Node_) -> bool;
    fn create(&self, ctx: &mut PgMigrateCtx);
    fn delete_coalesce(&mut self, other: &Node_) -> bool;
    fn delete(&self, ctx: &mut PgMigrateCtx);
}

pub(crate) trait NodeData: NodeDataDispatch {
    fn update(&self, ctx: &mut PgMigrateCtx, old: &Self);
}
