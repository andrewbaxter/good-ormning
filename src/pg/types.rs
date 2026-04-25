use serde::{
    Serialize,
    Deserialize,
};
use quote::{
    quote,
};
use crate::utils::RustTypes;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum SimpleSimpleType {
    Auto,
    I16,
    I32,
    I64,
    U32,
    F32,
    F64,
    Bool,
    String,
    Bytes,
    #[cfg(feature = "chrono")]
    UtcTimeSChrono,
    #[cfg(feature = "chrono")]
    UtcTimeMsChrono,
    #[cfg(feature = "chrono")]
    FixedOffsetTimeChrono,
    #[cfg(feature = "jiff")]
    UtcTimeSJiff,
    #[cfg(feature = "jiff")]
    UtcTimeMsJiff,
}

pub fn to_sql_type(t: &SimpleSimpleType) -> &'static str {
    match t {
        SimpleSimpleType::Auto => "bigserial",
        SimpleSimpleType::I16 => "smallint",
        SimpleSimpleType::I32 => "int",
        SimpleSimpleType::I64 => "bigint",
        SimpleSimpleType::U32 => "bigint",
        SimpleSimpleType::F32 => "real",
        SimpleSimpleType::F64 => "double precision",
        SimpleSimpleType::Bool => "bool",
        SimpleSimpleType::String => "text",
        SimpleSimpleType::Bytes => "bytea",
        #[cfg(feature = "chrono")]
        SimpleSimpleType::UtcTimeSChrono => "timestamp with time zone",
        #[cfg(feature = "chrono")]
        SimpleSimpleType::UtcTimeMsChrono => "timestamp with time zone",
        #[cfg(feature = "chrono")]
        SimpleSimpleType::FixedOffsetTimeChrono => "timestamp with time zone",
        #[cfg(feature = "jiff")]
        SimpleSimpleType::UtcTimeSJiff => "timestamp with time zone",
        #[cfg(feature = "jiff")]
        SimpleSimpleType::UtcTimeMsJiff => "timestamp with time zone",
    }
}

pub fn to_rust_types(t: &SimpleSimpleType) -> RustTypes {
    match t {
        SimpleSimpleType::Auto => RustTypes {
            custom_trait: quote!(good_ormning_runtime::pg::GoodOrmningCustomAuto),
            ret_type: quote!(i64),
            arg_type: quote!(i64),
        },
        SimpleSimpleType::I16 => RustTypes {
            custom_trait: quote!(good_ormning_runtime::pg::GoodOrmningCustomI16),
            ret_type: quote!(i16),
            arg_type: quote!(i16),
        },
        SimpleSimpleType::I32 => RustTypes {
            custom_trait: quote!(good_ormning_runtime::pg::GoodOrmningCustomI32),
            ret_type: quote!(i32),
            arg_type: quote!(i32),
        },
        SimpleSimpleType::I64 => RustTypes {
            custom_trait: quote!(good_ormning_runtime::pg::GoodOrmningCustomI64),
            ret_type: quote!(i64),
            arg_type: quote!(i64),
        },
        SimpleSimpleType::U32 => RustTypes {
            custom_trait: quote!(good_ormning_runtime::pg::GoodOrmningCustomU32),
            ret_type: quote!(i64),
            arg_type: quote!(i64),
        },
        SimpleSimpleType::F32 => RustTypes {
            custom_trait: quote!(good_ormning_runtime::pg::GoodOrmningCustomF32),
            ret_type: quote!(f32),
            arg_type: quote!(f32),
        },
        SimpleSimpleType::F64 => RustTypes {
            custom_trait: quote!(good_ormning_runtime::pg::GoodOrmningCustomF64),
            ret_type: quote!(f64),
            arg_type: quote!(f64),
        },
        SimpleSimpleType::Bool => RustTypes {
            custom_trait: quote!(good_ormning_runtime::pg::GoodOrmningCustomBool),
            ret_type: quote!(bool),
            arg_type: quote!(bool),
        },
        SimpleSimpleType::String => RustTypes {
            custom_trait: quote!(good_ormning_runtime::pg::GoodOrmningCustomString),
            ret_type: quote!(String),
            arg_type: quote!(&str),
        },
        SimpleSimpleType::Bytes => RustTypes {
            custom_trait: quote!(good_ormning_runtime::pg::GoodOrmningCustomBytes),
            ret_type: quote!(Vec < u8 >),
            arg_type: quote!(&[u8]),
        },
        #[cfg(feature = "chrono")]
        SimpleSimpleType::UtcTimeSChrono => RustTypes {
            custom_trait: quote!(good_ormning_runtime::pg::GoodOrmningCustomUtcTimeChrono),
            ret_type: quote!(chrono:: DateTime < chrono:: Utc >),
            arg_type: quote!(chrono:: DateTime < chrono:: Utc >),
        },
        #[cfg(feature = "chrono")]
        SimpleSimpleType::UtcTimeMsChrono => RustTypes {
            custom_trait: quote!(good_ormning_runtime::pg::GoodOrmningCustomUtcTimeChrono),
            ret_type: quote!(chrono:: DateTime < chrono:: Utc >),
            arg_type: quote!(chrono:: DateTime < chrono:: Utc >),
        },
        #[cfg(feature = "chrono")]
        SimpleSimpleType::FixedOffsetTimeChrono => RustTypes {
            custom_trait: quote!(good_ormning_runtime::pg::GoodOrmningCustomFixedOffsetTimeChrono),
            ret_type: quote!(chrono:: DateTime < chrono:: FixedOffset >),
            arg_type: quote!(chrono:: DateTime < chrono:: FixedOffset >),
        },
        #[cfg(feature = "jiff")]
        SimpleSimpleType::UtcTimeSJiff => RustTypes {
            custom_trait: quote!(good_ormning_runtime::pg::GoodOrmningCustomUtcTimeJiff),
            ret_type: quote!(jiff::Timestamp),
            arg_type: quote!(jiff::Timestamp),
        },
        #[cfg(feature = "jiff")]
        SimpleSimpleType::UtcTimeMsJiff => RustTypes {
            custom_trait: quote!(good_ormning_runtime::pg::GoodOrmningCustomUtcTimeJiff),
            ret_type: quote!(jiff::Timestamp),
            arg_type: quote!(jiff::Timestamp),
        },
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct SimpleType {
    pub type_: SimpleSimpleType,
    pub custom: Option<String>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Type {
    pub type_: SimpleType,
    pub opt: bool,
    pub arr: bool,
}

impl Type {
    pub fn opt(mut self) -> Self {
        self.opt = true;
        self
    }

    pub fn arr(mut self) -> Self {
        self.arr = true;
        self
    }
}

pub struct TypeBuilder {
    t: SimpleSimpleType,
    opt: bool,
    arr: bool,
    custom: Option<String>,
}

impl TypeBuilder {
    fn new(t: SimpleSimpleType) -> TypeBuilder {
        TypeBuilder {
            t: t,
            opt: false,
            arr: false,
            custom: None,
        }
    }

    /// Make this value optional.
    pub fn opt(mut self) -> TypeBuilder {
        self.opt = true;
        self
    }

    pub fn arr(mut self) -> TypeBuilder {
        self.arr = true;
        self
    }

    /// Use a custom Rust type for this type. This must be the full path to the type,
    /// like `crate::abcdef::MyType`.
    pub(crate) fn custom(mut self, type_: impl ToString) -> TypeBuilder {
        self.custom = Some(type_.to_string());
        self
    }

    pub fn build(self) -> Type {
        Type {
            type_: SimpleType {
                custom: self.custom,
                type_: self.t,
            },
            opt: self.opt,
            arr: self.arr,
        }
    }
}

pub fn type_auto() -> TypeBuilder {
    TypeBuilder::new(SimpleSimpleType::Auto)
}

pub fn type_bool() -> TypeBuilder {
    TypeBuilder::new(SimpleSimpleType::Bool)
}

pub fn type_i16() -> TypeBuilder {
    TypeBuilder::new(SimpleSimpleType::I16)
}

pub fn type_i32() -> TypeBuilder {
    TypeBuilder::new(SimpleSimpleType::I32)
}

pub fn type_i64() -> TypeBuilder {
    TypeBuilder::new(SimpleSimpleType::I64)
}

pub fn type_u32() -> TypeBuilder {
    TypeBuilder::new(SimpleSimpleType::U32)
}

pub fn type_f32() -> TypeBuilder {
    TypeBuilder::new(SimpleSimpleType::F32)
}

pub fn type_f64() -> TypeBuilder {
    TypeBuilder::new(SimpleSimpleType::F64)
}

pub fn type_str() -> TypeBuilder {
    TypeBuilder::new(SimpleSimpleType::String)
}

pub fn type_bytes() -> TypeBuilder {
    TypeBuilder::new(SimpleSimpleType::Bytes)
}

#[cfg(feature = "chrono")]
pub fn type_utctime_s_chrono() -> TypeBuilder {
    TypeBuilder::new(SimpleSimpleType::UtcTimeSChrono)
}

#[cfg(feature = "chrono")]
pub fn type_utctime_ms_chrono() -> TypeBuilder {
    TypeBuilder::new(SimpleSimpleType::UtcTimeMsChrono)
}

#[cfg(feature = "jiff")]
pub fn type_utctime_s_jiff() -> TypeBuilder {
    TypeBuilder::new(SimpleSimpleType::UtcTimeSJiff)
}

#[cfg(feature = "jiff")]
pub fn type_utctime_ms_jiff() -> TypeBuilder {
    TypeBuilder::new(SimpleSimpleType::UtcTimeMsJiff)
}
