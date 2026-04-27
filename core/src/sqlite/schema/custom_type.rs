use serde::{
    Serialize,
    Deserialize,
};
use crate::sqlite::types::Type;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomType {
    pub id: String,
    pub renamed_from: Option<String>,
    pub rust_type: String,
    pub base_type: Type,
}
