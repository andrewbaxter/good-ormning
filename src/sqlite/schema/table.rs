use std::{
    fmt::{
        Debug,
        Display,
    },
    collections::BTreeMap,
};
use serde::{
    Serialize,
    Deserialize,
};
use super::{
    field::{
        Field,
        SchemaFieldId,
    },
    index::{
        Index,
        SchemaIndexId,
    },
    constraint::{
        Constraint,
        SchemaConstraintId,
    },
};

#[derive(Clone, Eq, PartialEq, Hash, Debug, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SchemaTableId(pub String);

impl Display for SchemaTableId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TableRef(pub SchemaTableId);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    pub id: String,
    pub fields: BTreeMap<SchemaFieldId, Field>,
    pub indices: BTreeMap<SchemaIndexId, Index>,
    pub constraints: BTreeMap<SchemaConstraintId, Constraint>,
}
