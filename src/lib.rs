#![cfg_attr(all(feature = "pg", feature = "sqlite"), doc = include_str!("../readme.md"))]

#[cfg(feature = "pg")]
pub mod pg;
#[cfg(feature = "sqlite")]
pub mod sqlite;
pub mod runtime;
mod graphmigrate;
pub mod utils;

pub use crate::pg::{
    PgType,
    PgFieldTypeBuilder,
    pg_type_i32,
    pg_type_i64,
    pg_type_u32,
    pg_type_f32,
    pg_type_f64,
    pg_type_bool,
    pg_type_bytes,
    pg_type_str,
    pg_type_utctime_s_chrono,
    pg_type_utctime_s_jiff,
};
pub use crate::sqlite::{
    SqliteType,
    SqliteFieldTypeBuilder,
    sqlite_type_i32,
    sqlite_type_i64,
    sqlite_type_u32,
    sqlite_type_f32,
    sqlite_type_f64,
    sqlite_type_bool,
    sqlite_type_bytes,
    sqlite_type_str,
    sqlite_type_utctime_s_chrono,
    sqlite_type_utctime_s_jiff,
};

/// The number of results this query returns. This determines if the return type is
/// void, `Option`, the value directly, or a `Vec`. It must be a valid value per
/// the query body (e.g. select can't have `None` res count).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum QueryResCount {
    None,
    MaybeOne,
    One,
    Many,
}
