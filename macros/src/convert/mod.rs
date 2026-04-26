use std::collections::BTreeMap;
use good_ormning::pg::schema::custom_type::CustomType as PgCustomType;
use good_ormning::sqlite::schema::custom_type::CustomType as SqliteCustomType;
use good_ormning::pg::types::{
    Type as PgType,
    SimpleType as PgSimpleType,
    SimpleSimpleType as PgSimpleSimpleType,
};
use good_ormning::sqlite::types::{
    Type as SqliteType,
    SimpleType as SqliteSimpleType,
    SimpleSimpleType as SqliteSimpleSimpleType,
};
use crate::ParamType;

pub fn param_type_to_pg_type(pt: &ParamType, custom_types: &BTreeMap<String, PgCustomType>) -> PgType {
    let (simple_type, custom) = match pt.base.as_str() {
        "i16" => (PgSimpleSimpleType::I16, None),
        "i32" => (PgSimpleSimpleType::I32, None),
        "i64" => (PgSimpleSimpleType::I64, None),
        "u32" => (PgSimpleSimpleType::U32, None),
        "f32" => (PgSimpleSimpleType::F32, None),
        "f64" => (PgSimpleSimpleType::F64, None),
        "bool" => (PgSimpleSimpleType::Bool, None),
        "string" => (PgSimpleSimpleType::String, None),
        "bytes" => (PgSimpleSimpleType::Bytes, None),
        "utctime_s_chrono" => (PgSimpleSimpleType::UtcTimeSChrono, None),
        "utctime_ms_chrono" => (PgSimpleSimpleType::UtcTimeMsChrono, None),
        "utctime_s_jiff" => (PgSimpleSimpleType::UtcTimeSJiff, None),
        "utctime_ms_jiff" => (PgSimpleSimpleType::UtcTimeMsJiff, None),
        "auto" => (PgSimpleSimpleType::Auto, None),
        _ => {
            if let Some(ct) = custom_types.get(&pt.base) {
                (ct.base_type.type_.type_.clone(), Some(ct.rust_type.clone()))
            } else {
                (PgSimpleSimpleType::I32, Some(pt.base.clone()))
            }
        },
    };
    PgType {
        type_: PgSimpleType {
            type_: simple_type,
            custom,
        },
        opt: pt.opt,
        arr: pt.arr,
    }
}

pub fn param_type_to_sqlite_type(
    pt: &ParamType,
    custom_types: &BTreeMap<String, SqliteCustomType>,
) -> SqliteType {
    let (simple_type, custom) = match pt.base.as_str() {
        "i16" => (SqliteSimpleSimpleType::I16, None),
        "i32" => (SqliteSimpleSimpleType::I32, None),
        "i64" => (SqliteSimpleSimpleType::I64, None),
        "u32" => (SqliteSimpleSimpleType::U32, None),
        "f32" => (SqliteSimpleSimpleType::F32, None),
        "f64" => (SqliteSimpleSimpleType::F64, None),
        "bool" => (SqliteSimpleSimpleType::Bool, None),
        "string" => (SqliteSimpleSimpleType::String, None),
        "bytes" => (SqliteSimpleSimpleType::Bytes, None),
        "utctime_s_chrono" => (SqliteSimpleSimpleType::UtcTimeSChrono, None),
        "utctime_ms_chrono" => (SqliteSimpleSimpleType::UtcTimeMsChrono, None),
        "utctime_s_jiff" => (SqliteSimpleSimpleType::UtcTimeSJiff, None),
        "utctime_ms_jiff" => (SqliteSimpleSimpleType::UtcTimeMsJiff, None),
        "auto" => (SqliteSimpleSimpleType::Auto, None),
        _ => {
            if let Some(ct) = custom_types.get(&pt.base) {
                (ct.base_type.type_.type_.clone(), Some(ct.rust_type.clone()))
            } else {
                (SqliteSimpleSimpleType::I32, Some(pt.base.clone()))
            }
        },
    };
    SqliteType {
        type_: SqliteSimpleType {
            type_: simple_type,
            custom,
        },
        opt: pt.opt,
        arr: pt.arr,
    }
}

pub mod pg;
pub mod sqlite;
pub mod template;

pub fn sql_type_to_pg_type(
    t: &sqlparser::ast::DataType,
    custom_types: &BTreeMap<String, PgCustomType>,
) -> PgType {
    let (simple_type, custom) = match t {
        sqlparser::ast::DataType::SmallInt(_) => (PgSimpleSimpleType::I16, None),
        sqlparser::ast::DataType::Int(_) | sqlparser::ast::DataType::Integer(_) => (PgSimpleSimpleType::I32, None),
        sqlparser::ast::DataType::BigInt(_) => (PgSimpleSimpleType::I64, None),
        sqlparser::ast::DataType::Float(_) | sqlparser::ast::DataType::Real => (PgSimpleSimpleType::F32, None),
        sqlparser::ast::DataType::DoublePrecision => (PgSimpleSimpleType::F64, None),
        sqlparser::ast::DataType::Boolean => (PgSimpleSimpleType::Bool, None),
        sqlparser::ast::DataType::Text | sqlparser::ast::DataType::Varchar(_) => (PgSimpleSimpleType::String, None),
        sqlparser::ast::DataType::Bytea | sqlparser::ast::DataType::Binary(_) | sqlparser::ast::DataType::Varbinary(_) => (PgSimpleSimpleType::Bytes, None),
        sqlparser::ast::DataType::Timestamp(precision, tz) => {
            // Very simplified
            (PgSimpleSimpleType::UtcTimeSChrono, None)
        },
        sqlparser::ast::DataType::Custom(name, ..) => {
            let name_str = name.to_string();
            if let Some(ct) = custom_types.get(&name_str) {
                (ct.base_type.type_.type_.clone(), Some(ct.rust_type.clone()))
            } else {
                (PgSimpleSimpleType::I32, Some(name_str))
            }
        },
        _ => (PgSimpleSimpleType::I32, None),
    };
    PgType {
        type_: PgSimpleType {
            type_: simple_type,
            custom,
        },
        opt: false,
        arr: false,
    }
}

pub fn sql_type_to_sqlite_type(
    t: &sqlparser::ast::DataType,
    custom_types: &BTreeMap<String, SqliteCustomType>,
) -> SqliteType {
    let (simple_type, custom) = match t {
        sqlparser::ast::DataType::SmallInt(_) => (SqliteSimpleSimpleType::I16, None),
        sqlparser::ast::DataType::Int(_) | sqlparser::ast::DataType::Integer(_) => (SqliteSimpleSimpleType::I32, None),
        sqlparser::ast::DataType::BigInt(_) => (SqliteSimpleSimpleType::I64, None),
        sqlparser::ast::DataType::Float(_) | sqlparser::ast::DataType::Real => (SqliteSimpleSimpleType::F32, None),
        sqlparser::ast::DataType::DoublePrecision => (SqliteSimpleSimpleType::F64, None),
        sqlparser::ast::DataType::Boolean => (SqliteSimpleSimpleType::Bool, None),
        sqlparser::ast::DataType::Text | sqlparser::ast::DataType::Varchar(_) => (SqliteSimpleSimpleType::String, None),
        sqlparser::ast::DataType::Binary(_) | sqlparser::ast::DataType::Varbinary(_) | sqlparser::ast::DataType::Blob(_) => (SqliteSimpleSimpleType::Bytes, None),
        sqlparser::ast::DataType::Timestamp(..) => {
            (SqliteSimpleSimpleType::UtcTimeSChrono, None)
        },
        sqlparser::ast::DataType::Custom(name, ..) => {
            let name_str = name.to_string();
            if let Some(ct) = custom_types.get(&name_str) {
                (ct.base_type.type_.type_.clone(), Some(ct.rust_type.clone()))
            } else {
                (SqliteSimpleSimpleType::I32, Some(name_str))
            }
        },
        _ => (SqliteSimpleSimpleType::I32, None),
    };
    SqliteType {
        type_: SqliteSimpleType {
            type_: simple_type,
            custom,
        },
        opt: false,
        arr: false,
    }
}
