use serde::{
    Serialize,
    Deserialize,
};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct PrimaryKeyDef {
    pub fields: Vec<String>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ForeignKeyDef {
    pub remote_table: String,
    pub fields: Vec<(String, String)>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum ConstraintType {
    PrimaryKey(PrimaryKeyDef),
    ForeignKey(ForeignKeyDef),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    pub id: String,
    pub renamed_from: Option<String>,
    pub type_: ConstraintType,
}
