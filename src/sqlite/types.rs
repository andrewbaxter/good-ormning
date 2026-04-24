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
    I32,
    I64,
    F32,
    F64,
    Bool,
    String,
    Bytes,
    #[cfg(feature = "chrono")]
    UtcTimeChrono,
    #[cfg(feature = "chrono")]
    FixedOffsetTimeChrono,
    #[cfg(feature = "jiff")]
    UtcTimeJiff,
}

pub fn to_rust_types(t: &SimpleSimpleType) -> RustTypes {
    match t {
        SimpleSimpleType::Auto => RustTypes {
            ret_type: quote!(i64),
            arg_type: quote!(i64),
            custom_trait: quote!(good_ormning_runtime::sqlite::GoodOrmningCustomAuto),
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
        SimpleSimpleType::UtcTimeChrono => RustTypes {
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
        SimpleSimpleType::UtcTimeJiff => RustTypes {
            ret_type: quote!(jiff::Timestamp),
            arg_type: quote!(jiff::Timestamp),
            custom_trait: quote!(good_ormning_runtime::sqlite::GoodOrmningCustomUtcTimeJiff),
        },
    }
}

pub fn to_sql_type(t: &SimpleSimpleType) -> &'static str {
    match t {
        SimpleSimpleType::Auto => "integer",
        SimpleSimpleType::I32 => "integer",
        SimpleSimpleType::I64 => "integer",
        SimpleSimpleType::F32 => "real",
        SimpleSimpleType::F64 => "real",
        SimpleSimpleType::Bool => "integer",
        SimpleSimpleType::String => "text",
        SimpleSimpleType::Bytes => "blob",
        #[cfg(feature = "chrono")]
        SimpleSimpleType::UtcTimeChrono => "text",
        #[cfg(feature = "chrono")]
        SimpleSimpleType::FixedOffsetTimeChrono => "text",
        #[cfg(feature = "jiff")]
        SimpleSimpleType::UtcTimeJiff => "text",
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

    pub fn build(self) -> Type {
        self.0
    }
}

pub fn type_auto() -> TypeBuilder {
    TypeBuilder::new(SimpleSimpleType::Auto)
}

pub fn type_i32() -> TypeBuilder {
    TypeBuilder::new(SimpleSimpleType::I32)
}

pub fn type_i64() -> TypeBuilder {
    TypeBuilder::new(SimpleSimpleType::I64)
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
pub fn type_utctime_chrono() -> TypeBuilder {
    TypeBuilder::new(SimpleSimpleType::UtcTimeChrono)
}

#[cfg(feature = "jiff")]
pub fn type_utctime_jiff() -> TypeBuilder {
    TypeBuilder::new(SimpleSimpleType::UtcTimeJiff)
}
