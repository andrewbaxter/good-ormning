use serde::{
    Serialize,
    Deserialize,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    pub id: String,
    pub renamed_from: Option<String>,
    pub type_: ConstraintType,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum ConstraintType {
    ForeignKey(ForeignKeyDef),
    PrimaryKey(PrimaryKeyDef),
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ForeignKeyDef {
    pub fields: Vec<(String, String)>,
    pub on_delete: Option<ForeignKeyAction>,
    pub on_update: Option<ForeignKeyAction>,
    pub remote_table: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum ForeignKeyAction {
    Cascade,
    NoAction,
    Restrict,
    SetNull,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct PrimaryKeyDef {
    pub fields: Vec<String>,
}
