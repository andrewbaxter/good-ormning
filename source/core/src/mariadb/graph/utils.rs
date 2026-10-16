use {
    crate::{
        mariadb::Version,
        utils::Errs,
    },
    std::collections::HashMap,
    super::Node,
};

pub type MigrateNode = crate::graphmigrate::Node<Node>;

pub struct MariadbMigrateCtx {
    pub errs: Errs,
    pub statements: Vec<String>,
    pub(crate) table_sql_names: HashMap<String, String>,
    pub(crate) version: Version,
}

impl MariadbMigrateCtx {
    pub fn new(errs: Errs, table_sql_names: HashMap<String, String>, version: Version) -> Self {
        return Self {
            errs: errs,
            statements: vec![],
            table_sql_names: table_sql_names,
            version: version,
        };
    }
}
