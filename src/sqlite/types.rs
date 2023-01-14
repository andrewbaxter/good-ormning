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
    UtcTimeS,
    /// Time with millisecond granularity, stored as string
    UtcTimeMs,
}

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
        SimpleSimpleType::UtcTimeS => "integer",
        SimpleSimpleType::UtcTimeMs => "text",
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

    /// Use a custom Rust type for this type. This must be the full path to the type, like
    /// `crate::abcdef::MyType`.
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

pub fn type_utctime_s() -> TypeBuilder {
    TypeBuilder::new(SimpleSimpleType::UtcTimeS)
}

pub fn type_utctime_ms() -> TypeBuilder {
    TypeBuilder::new(SimpleSimpleType::UtcTimeMs)
}
