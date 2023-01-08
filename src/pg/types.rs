#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SimpleSimpleType {
    Auto,
    U32,
    U64,
    I32,
    I64,
    F32,
    F64,
    Bool,
    String,
    Bytes,
    UtcTime,
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

pub fn to_sql_type(t: &SimpleType) -> &'static str {
    match t.type_ {
        SimpleSimpleType::Auto => "serial",
        SimpleSimpleType::U32 => "int",
        SimpleSimpleType::U64 => "bigint",
        SimpleSimpleType::I32 => "int",
        SimpleSimpleType::I64 => "bigint",
        SimpleSimpleType::F32 => "real",
        SimpleSimpleType::F64 => "double",
        SimpleSimpleType::Bool => "bool",
        SimpleSimpleType::String => "text",
        SimpleSimpleType::Bytes => "bytea",
        SimpleSimpleType::UtcTime => "timestamp with time zone",
    }
}
