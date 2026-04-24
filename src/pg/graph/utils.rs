use crate::utils::Errs;
use super::Node;

pub struct PgMigrateCtx {
    pub(crate) errs: Errs,
    pub statements: Vec<String>,
}

impl PgMigrateCtx {
    pub fn new(errs: Errs) -> Self {
        Self {
            errs: errs,
            statements: vec![],
        }
    }
}

pub type MigrateNode = crate::graphmigrate::Node<Node>;
