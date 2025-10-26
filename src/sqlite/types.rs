use quote::{
    quote,
};
use crate::utils::RustTypes;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SimpleSimpleType {
    U32,
    I32,
    I64,
    F32,
    F64,
    Bool,
    String,
    Bytes,
    /// Time with second granularity, stored as int
    #[cfg(feature = "chrono")]
    UtcTimeSChrono,
    /// Time with millisecond granularity, stored as string
    #[cfg(feature = "chrono")]
    UtcTimeMsChrono,
    /// Time with millisecond granularity, stored as string
    #[cfg(feature = "chrono")]
    FixedOffsetTimeMsChrono,
    /// Time with second granularity, stored as int
    #[cfg(feature = "jiff")]
    UtcTimeSJiff,
    /// Time with millisecond granularity, stored as string
    #[cfg(feature = "jiff")]
    UtcTimeMsJiff,
}

#[doc(hidden)]
pub fn to_sql_type(t: &SimpleSimpleType) -> &'static str {
    match t {
        SimpleSimpleType::U32 => "integer",
        SimpleSimpleType::I32 => "integer",
        SimpleSimpleType::I64 => "integer",
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
        SimpleSimpleType::FixedOffsetTimeMsChrono => "text",
        #[cfg(feature = "jiff")]
        SimpleSimpleType::UtcTimeSJiff => "integer",
        #[cfg(feature = "jiff")]
        SimpleSimpleType::UtcTimeMsJiff => "text",
    }
}

pub fn to_rust_types(t: &SimpleSimpleType) -> RustTypes {
    match t {
        SimpleSimpleType::U32 => RustTypes {
            custom_trait: quote!(good_ormning_runtime::sqlite::GoodOrmningCustomU32),
            ret_type: quote!(u32),
            arg_type: quote!(u32),
        },
        SimpleSimpleType::I32 => RustTypes {
            custom_trait: quote!(good_ormning_runtime::sqlite::GoodOrmningCustomI32),
            ret_type: quote!(i32),
            arg_type: quote!(i32),
        },
        SimpleSimpleType::I64 => RustTypes {
            custom_trait: quote!(good_ormning_runtime::sqlite::GoodOrmningCustomI64),
            ret_type: quote!(i64),
            arg_type: quote!(i64),
        },
        SimpleSimpleType::F32 => RustTypes {
            custom_trait: quote!(good_ormning_runtime::sqlite::GoodOrmningCustomF32),
            ret_type: quote!(f32),
            arg_type: quote!(f32),
        },
        SimpleSimpleType::F64 => RustTypes {
            custom_trait: quote!(good_ormning_runtime::sqlite::GoodOrmningCustomF64),
            ret_type: quote!(f64),
            arg_type: quote!(f64),
        },
        SimpleSimpleType::Bool => RustTypes {
            custom_trait: quote!(good_ormning_runtime::sqlite::GoodOrmningCustomBool),
            ret_type: quote!(bool),
            arg_type: quote!(bool),
        },
        SimpleSimpleType::String => RustTypes {
            custom_trait: quote!(good_ormning_runtime::sqlite::GoodOrmningCustomString),
            ret_type: quote!(String),
            arg_type: quote!(&str),
        },
        SimpleSimpleType::Bytes => RustTypes {
            custom_trait: quote!(good_ormning_runtime::sqlite::GoodOrmningCustomBytes),
            ret_type: quote!(Vec < u8 >),
            arg_type: quote!(&[u8]),
        },
        #[cfg(feature = "chrono")]
        SimpleSimpleType::UtcTimeSChrono => RustTypes {
            custom_trait: quote!(good_ormning_runtime::sqlite::GoodOrmningCustomUtcTimeChrono),
            ret_type: quote!(chrono:: DateTime < chrono:: Utc >),
            arg_type: quote!(chrono:: DateTime < chrono:: Utc >),
        },
        #[cfg(feature = "chrono")]
        SimpleSimpleType::UtcTimeMsChrono => RustTypes {
            custom_trait: quote!(good_ormning_runtime::sqlite::GoodOrmningCustomUtcTimeChrono),
            ret_type: quote!(chrono:: DateTime < chrono:: Utc >),
            arg_type: quote!(chrono:: DateTime < chrono:: Utc >),
        },
        #[cfg(feature = "chrono")]
        SimpleSimpleType::FixedOffsetTimeMsChrono => RustTypes {
            custom_trait: quote!(good_ormning_runtime::sqlite::GoodOrmningCustomFixedOffsetTimeChrono),
            ret_type: quote!(chrono:: DateTime < chrono:: FixedOffset >),
            arg_type: quote!(chrono:: DateTime < chrono:: FixedOffset >),
        },
        #[cfg(feature = "jiff")]
        SimpleSimpleType::UtcTimeSJiff => RustTypes {
            custom_trait: quote!(good_ormning_runtime::sqlite::GoodOrmningCustomUtcTimeJiff),
            ret_type: quote!(jiff::Timestamp),
            arg_type: quote!(jiff::Timestamp),
        },
        #[cfg(feature = "jiff")]
        SimpleSimpleType::UtcTimeMsJiff => RustTypes {
            custom_trait: quote!(good_ormning_runtime::sqlite::GoodOrmningCustomUtcTimeJiff),
            ret_type: quote!(jiff::Timestamp),
            arg_type: quote!(jiff::Timestamp),
        },
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SimpleType {
    pub type_: SimpleSimpleType,
    pub custom: Option<String>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Type {
    pub type_: SimpleType,
    pub opt: bool,
    pub array: bool,
}

pub struct TypeBuilder {
    t: SimpleSimpleType,
    opt: bool,
    array: bool,
    custom: Option<String>,
}

impl TypeBuilder {
    fn new(t: SimpleSimpleType) -> TypeBuilder {
        TypeBuilder {
            t: t,
            opt: false,
            array: false,
            custom: None,
        }
    }

    /// Make this value an array.
    pub fn array(mut self) -> TypeBuilder {
        self.array = true;
        self
    }

    /// Make this value optional.
    pub fn opt(mut self) -> TypeBuilder {
        self.opt = true;
        self
    }

    /// Use a custom Rust type for this type. This must be the full path to the type,
    /// like `crate::abcdef::MyType`.
    pub fn custom(mut self, type_: impl ToString) -> TypeBuilder {
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
            array: self.array,
        }
    }
}

pub fn type_bool() -> TypeBuilder {
    TypeBuilder::new(SimpleSimpleType::Bool)
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
pub fn type_utctime_s() -> TypeBuilder {
    TypeBuilder::new(SimpleSimpleType::UtcTimeSChrono)
}

#[cfg(feature = "chrono")]
pub fn type_utctime_ms() -> TypeBuilder {
    TypeBuilder::new(SimpleSimpleType::UtcTimeMsChrono)
}

#[cfg(feature = "chrono")]
pub fn type_fixedoffsettime_ms() -> TypeBuilder {
    TypeBuilder::new(SimpleSimpleType::FixedOffsetTimeMsChrono)
}

#[cfg(feature = "jiff")]
pub fn type_utctime_s_jiff() -> TypeBuilder {
    TypeBuilder::new(SimpleSimpleType::UtcTimeSJiff)
}

#[cfg(feature = "jiff")]
pub fn type_utctime_ms_jiff() -> TypeBuilder {
    TypeBuilder::new(SimpleSimpleType::UtcTimeMsJiff)
}
