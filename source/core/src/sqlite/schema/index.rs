use serde::{
    Serialize,
    Deserialize,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Index {
    pub fields: Vec<String>,
    pub id: String,
    pub renamed_from: Option<String>,
    pub unique: bool,
}
