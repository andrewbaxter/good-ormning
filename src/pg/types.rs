use quote::{
    quote,
};
use crate::utils::RustTypes;

#[derive(Clone, PartialEq, Eq, Debug)]
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
    UtcTime,
}

pub fn to_sql_type(t: &SimpleSimpleType) -> &'static str {
    match t {
        SimpleSimpleType::Auto => "bigserial",
        SimpleSimpleType::I32 => "int",
        SimpleSimpleType::I64 => "bigint",
        SimpleSimpleType::F32 => "real",
        SimpleSimpleType::F64 => "double",
        SimpleSimpleType::Bool => "bool",
        SimpleSimpleType::String => "text",
        SimpleSimpleType::Bytes => "bytea",
        #[cfg(feature = "chrono")]
        SimpleSimpleType::UtcTime => "timestamp with time zone",
    }
}

pub fn to_rust_types(t: &SimpleSimpleType) -> RustTypes {
    match t {
        SimpleSimpleType::Auto => RustTypes {
            custom_trait: quote!(good_ormning_runtime::pg::GoodOrmningCustomAuto),
            ret_type: quote!(i64),
            arg_type: quote!(i64),
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
        SimpleSimpleType::UtcTime => RustTypes {
            custom_trait: quote!(good_ormning_runtime::pg::GoodOrmningCustomUtcTime),
            ret_type: quote!(chrono:: DateTime < chrono:: Utc >),
            arg_type: quote!(chrono:: DateTime < chrono:: Utc >),
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
}

pub struct TypeBuilder {
    t: SimpleSimpleType,
    opt: bool,
    custom: Option<String>,
}

impl TypeBuilder {
    fn new(t: SimpleSimpleType) -> TypeBuilder {
        TypeBuilder {
            t: t,
            opt: false,
            custom: None,
        }
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
        }
    }
}

pub fn type_auto() -> TypeBuilder {
    TypeBuilder::new(SimpleSimpleType::Auto)
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
pub fn type_utctime() -> TypeBuilder {
    TypeBuilder::new(SimpleSimpleType::UtcTime)
}
