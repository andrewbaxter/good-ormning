use {
    crate::sqlite::types::Type,
    serde::{
        Deserialize,
        Serialize,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomType {
    pub base_type: Type,
    pub id: String,
    pub renamed_from: Option<String>,
    pub rust_type: String,
}
