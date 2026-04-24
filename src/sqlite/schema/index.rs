use std::{
    fmt::Display,
};
use serde::{
    Serialize,
    Deserialize,
};
use super::{
    field::SchemaFieldId,
};

#[derive(Clone, Eq, PartialEq, Hash, Debug, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SchemaIndexId(pub String);

impl Display for SchemaIndexId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Index {
    pub id: String,
    pub fields: Vec<SchemaFieldId>,
    pub unique: bool,
}
