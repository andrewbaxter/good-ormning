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

#[derive(Clone, Eq, PartialEq, Hash, Debug, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TableRef(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    pub id: String,
    pub renamed_from: Option<String>,
    pub fields: BTreeMap<String, Field>,
    pub indices: BTreeMap<String, Index>,
    pub constraints: BTreeMap<String, Constraint>,
}
