#![cfg_attr(all(feature = "pg", feature = "sqlite"), doc = include_str!("../readme.md"))]

#[cfg(feature = "pg")]
pub mod pg;
#[cfg(feature = "sqlite")]
pub mod sqlite;
pub mod runtime;
#[cfg(feature = "import")]
pub mod import;

pub use good_ormning_core::{
    utils,
    graphmigrate,
    QueryResCount,
};

/// Create a module containing the generated code for a database.
///
/// # Parameters
///
/// * `$vis` - (Optional) Visibility of the module (e.g., `pub`).
///
/// * `$mod_name` - The name of the module to create.
///
/// * `$db_name` - (Optional) The database name string literal. Must match the name
///   passed to `generate`.
///
/// # Example
///
/// ```rust,ignore
/// good_module!(dbm);
/// good_module!(pub my_db, "custom_db");
/// ```
#[macro_export]
macro_rules! good_module{
    ($vis: vis $mod_name: ident) => {
        #[allow(unused)] $vis mod $mod_name {
            include!(concat!(env!("OUT_DIR"), "/good_ormning_.rs"));
        }
    };
    ($vis: vis $mod_name: ident, $db_name: literal) => {
        #[allow(unused)] $vis mod $mod_name {
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
    GenerateArgs as PgGenerateArgs,
};
#[cfg(all(feature = "pg", feature = "chrono"))]
pub use crate::pg::pg_type_utctime_s_chrono;
#[cfg(all(feature = "pg", feature = "jiff"))]
pub use crate::pg::pg_type_utctime_s_jiff;
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
    GenerateArgs as SqliteGenerateArgs,
};
#[cfg(all(feature = "sqlite", feature = "chrono"))]
pub use crate::sqlite::sqlite_type_utctime_s_chrono;
#[cfg(all(feature = "sqlite", feature = "jiff"))]
pub use crate::sqlite::sqlite_type_utctime_s_jiff;
