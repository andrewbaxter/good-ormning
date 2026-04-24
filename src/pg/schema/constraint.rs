use std::{
    fmt::Display,
};
use serde::{
    Serialize,
    Deserialize,
};
use super::{
    table::{
        SchemaTableId,
    },
    field::SchemaFieldId,
};

#[derive(Clone, Eq, PartialEq, Hash, Debug, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SchemaConstraintId(pub String);

impl Display for SchemaConstraintId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct PrimaryKeyDef {
    pub fields: Vec<SchemaFieldId>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ForeignKeyDef {
    pub remote_table: SchemaTableId,
    pub fields: Vec<(SchemaFieldId, SchemaFieldId)>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum ConstraintType {
    PrimaryKey(PrimaryKeyDef),
    ForeignKey(ForeignKeyDef),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    pub id: String,
    pub type_: ConstraintType,
}
