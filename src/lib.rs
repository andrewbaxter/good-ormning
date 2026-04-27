#![cfg_attr(all(feature = "pg", feature = "sqlite"), doc = include_str!("../readme.md"))]

#[cfg(feature = "pg")]
pub mod pg;
#[cfg(feature = "sqlite")]
pub mod sqlite;
pub mod runtime;

pub use good_ormning_core::{
    utils,
    graphmigrate,
    QueryResCount,
};

#[macro_export]
macro_rules! good_module{
    ($vis: vis $mod_name: ident) => {
        $vis mod $mod_name {
            include!(concat!(env!("OUT_DIR"), "/good_ormning_default.rs"));
        }
    };
    ($vis: vis $mod_name: ident, $db_name: literal) => {
        $vis mod $mod_name {
            include!(concat!(env!("OUT_DIR"), "/good_ormning_", $db_name, ".rs"));
        }
    };
}

#[cfg(feature = "pg")]
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
#[cfg(feature = "sqlite")]
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
