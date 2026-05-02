pub mod graphmigrate;
pub mod pg;
pub mod sqlite;
pub mod utils;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub enum QueryResCount {
    None,
    MaybeOne,
    One,
    Many,
}
