use {
    crate::pg::types::Type,
    serde::{
        Deserialize,
        Serialize,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomType {
    pub id: String,
    pub renamed_from: Option<String>,
    pub rust_type: String,
    pub base_type: Type,
}
