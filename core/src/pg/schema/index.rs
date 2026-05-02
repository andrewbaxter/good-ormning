use serde::{
    Serialize,
    Deserialize,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Index {
    pub id: String,
    pub renamed_from: Option<String>,
    pub fields: Vec<String>,
    pub unique: bool,
}
