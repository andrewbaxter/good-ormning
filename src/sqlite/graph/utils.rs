use enum_dispatch::enum_dispatch;
use crate::utils::Errs;
use super::Node;

pub struct SqliteMigrateCtx {
    pub(crate) errs: Errs,
    pub statements: Vec<String>,
}

impl SqliteMigrateCtx {
    pub fn new(errs: Errs) -> Self {
        Self {
            errs: errs,
            statements: vec![],
        }
    }
}

pub type MigrateNode = crate::graphmigrate::Node<Node>;
