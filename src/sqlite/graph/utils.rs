use std::collections::HashMap;
use crate::{
    utils::Errs,
    sqlite::{
        Version,
    },
};
use super::Node;

pub struct SqliteMigrateCtx {
    pub(crate) errs: Errs,
    pub statements: Vec<String>,
    pub(crate) table_sql_names: HashMap<String, String>,
    pub(crate) version: Version,
}

impl SqliteMigrateCtx {
    pub fn new(errs: Errs, table_sql_names: HashMap<String, String>, version: Version) -> Self {
        Self {
            errs: errs,
            statements: vec![],
            table_sql_names: table_sql_names,
            version: version,
        }
    }
}

pub type MigrateNode = crate::graphmigrate::Node<Node>;
