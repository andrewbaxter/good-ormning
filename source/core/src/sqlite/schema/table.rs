use {
    serde::{
        Deserialize,
        Serialize,
    },
    std::collections::BTreeMap,
    super::{
        constraint::Constraint,
        field::Field,
        index::Index,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    pub constraints: BTreeMap<String, Constraint>,
    pub fields: BTreeMap<String, Field>,
    pub id: String,
    pub indices: BTreeMap<String, Index>,
    pub renamed_from: Option<String>,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TableRef(pub String);
