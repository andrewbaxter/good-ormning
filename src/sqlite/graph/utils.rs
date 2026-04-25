use std::collections::HashMap;
use enum_dispatch::enum_dispatch;
use crate::{
    utils::Errs,
    sqlite::{
        Version,
        schema::table::SchemaTableId,
    },
};
use super::Node;

pub struct SqliteMigrateCtx {
    pub(crate) errs: Errs,
    pub statements: Vec<String>,
    pub(crate) table_sql_names: HashMap<SchemaTableId, String>,
    pub(crate) version: Version,
}

impl SqliteMigrateCtx {
    pub fn new(errs: Errs, table_sql_names: HashMap<SchemaTableId, String>, version: Version) -> Self {
        Self {
            errs: errs,
            statements: vec![],
            table_sql_names: table_sql_names,
            version: version,
        }
    }
}

pub type MigrateNode = crate::graphmigrate::Node<Node>;
