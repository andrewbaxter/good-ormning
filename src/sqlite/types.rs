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

pub fn to_rust_types(t: &SimpleSimpleType) -> RustTypes {
    match t {
        SimpleSimpleType::Auto => RustTypes {
            ret_type: quote!(i64),
            arg_type: quote!(i64),
            custom_trait: quote!(good_ormning_runtime::sqlite::GoodOrmningCustomAuto),
        },
        SimpleSimpleType::I16 => RustTypes {
            ret_type: quote!(i16),
            arg_type: quote!(i16),
            custom_trait: quote!(good_ormning_runtime::sqlite::GoodOrmningCustomI16),
        },
        SimpleSimpleType::I32 => RustTypes {
            ret_type: quote!(i32),
            arg_type: quote!(i32),
            custom_trait: quote!(good_ormning_runtime::sqlite::GoodOrmningCustomI32),
        },
        SimpleSimpleType::I64 => RustTypes {
            ret_type: quote!(i64),
            arg_type: quote!(i64),
            custom_trait: quote!(good_ormning_runtime::sqlite::GoodOrmningCustomI64),
        },
        SimpleSimpleType::U32 => RustTypes {
            ret_type: quote!(u32),
            arg_type: quote!(u32),
            custom_trait: quote!(good_ormning_runtime::sqlite::GoodOrmningCustomU32),
        },
        SimpleSimpleType::F32 => RustTypes {
            ret_type: quote!(f32),
            arg_type: quote!(f32),
            custom_trait: quote!(good_ormning_runtime::sqlite::GoodOrmningCustomF32),
        },
        SimpleSimpleType::F64 => RustTypes {
            ret_type: quote!(f64),
            arg_type: quote!(f64),
            custom_trait: quote!(good_ormning_runtime::sqlite::GoodOrmningCustomF64),
        },
        SimpleSimpleType::Bool => RustTypes {
            ret_type: quote!(bool),
            arg_type: quote!(bool),
            custom_trait: quote!(good_ormning_runtime::sqlite::GoodOrmningCustomBool),
        },
        SimpleSimpleType::String => RustTypes {
            ret_type: quote!(String),
            arg_type: quote!(&str),
            custom_trait: quote!(good_ormning_runtime::sqlite::GoodOrmningCustomString),
        },
        SimpleSimpleType::Bytes => RustTypes {
            ret_type: quote!(Vec < u8 >),
            arg_type: quote!(&[u8]),
            custom_trait: quote!(good_ormning_runtime::sqlite::GoodOrmningCustomBytes),
        },
        #[cfg(feature = "chrono")]
        SimpleSimpleType::UtcTimeSChrono => RustTypes {
            ret_type: quote!(chrono::DateTime < chrono::Utc >),
            arg_type: quote!(chrono::DateTime < chrono::Utc >),
            custom_trait: quote!(good_ormning_runtime::sqlite::GoodOrmningCustomUtcTimeChrono),
        },
        #[cfg(feature = "chrono")]
        SimpleSimpleType::UtcTimeMsChrono => RustTypes {
            ret_type: quote!(chrono::DateTime < chrono::Utc >),
            arg_type: quote!(chrono::DateTime < chrono::Utc >),
            custom_trait: quote!(good_ormning_runtime::sqlite::GoodOrmningCustomUtcTimeChrono),
        },
        #[cfg(feature = "chrono")]
        SimpleSimpleType::FixedOffsetTimeChrono => RustTypes {
            ret_type: quote!(chrono::DateTime < chrono::FixedOffset >),
            arg_type: quote!(chrono::DateTime < chrono::FixedOffset >),
            custom_trait: quote!(good_ormning_runtime::sqlite::GoodOrmningCustomFixedOffsetTimeChrono),
        },
        #[cfg(feature = "jiff")]
        SimpleSimpleType::UtcTimeSJiff => RustTypes {
            ret_type: quote!(jiff::Timestamp),
            arg_type: quote!(jiff::Timestamp),
            custom_trait: quote!(good_ormning_runtime::sqlite::GoodOrmningCustomUtcTimeJiff),
        },
        #[cfg(feature = "jiff")]
        SimpleSimpleType::UtcTimeMsJiff => RustTypes {
            ret_type: quote!(jiff::Timestamp),
            arg_type: quote!(jiff::Timestamp),
            custom_trait: quote!(good_ormning_runtime::sqlite::GoodOrmningCustomUtcTimeJiff),
        },
    }
}

pub fn to_sql_type(t: &SimpleSimpleType) -> &'static str {
    match t {
        SimpleSimpleType::Auto => "integer",
        SimpleSimpleType::I16 => "integer",
        SimpleSimpleType::I32 => "integer",
        SimpleSimpleType::I64 => "integer",
        SimpleSimpleType::U32 => "integer",
        SimpleSimpleType::F32 => "real",
        SimpleSimpleType::F64 => "real",
        SimpleSimpleType::Bool => "integer",
        SimpleSimpleType::String => "text",
        SimpleSimpleType::Bytes => "blob",
        #[cfg(feature = "chrono")]
        SimpleSimpleType::UtcTimeSChrono => "integer",
        #[cfg(feature = "chrono")]
        SimpleSimpleType::UtcTimeMsChrono => "text",
        #[cfg(feature = "chrono")]
        SimpleSimpleType::FixedOffsetTimeChrono => "text",
        #[cfg(feature = "jiff")]
        SimpleSimpleType::UtcTimeSJiff => "integer",
        #[cfg(feature = "jiff")]
        SimpleSimpleType::UtcTimeMsJiff => "text",
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

pub struct TypeBuilder(pub Type);

impl TypeBuilder {
    pub fn new(t: SimpleSimpleType) -> Self {
        Self(Type {
            type_: SimpleType {
                type_: t,
                custom: None,
            },
            opt: false,
            arr: false,
        })
    }

    pub fn custom(mut self, custom: impl ToString) -> Self {
        self.0.type_.custom = Some(custom.to_string());
        self
    }

    pub fn opt(mut self) -> Self {
        self.0.opt = true;
        self
    }

    pub fn arr(mut self) -> Self {
        self.0.arr = true;
        self
    }

    pub fn build(self) -> Type {
        self.0
    }
}

pub fn type_auto() -> TypeBuilder {
    TypeBuilder::new(SimpleSimpleType::Auto)
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

pub fn type_bool() -> TypeBuilder {
    TypeBuilder::new(SimpleSimpleType::Bool)
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
