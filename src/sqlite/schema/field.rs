use serde::{
    Serialize,
    Deserialize,
};
use crate::{
    sqlite::{
        types::{
            Type,
        },
        query::expr::{
            SerialExpr,
        },
    },
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FieldType {
    pub type_: Type,
    pub migration_default: Option<SerialExpr>,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FieldRef {
    pub table_id: String,
    pub field_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    pub id: String,
    pub renamed_from: Option<String>,
    pub type_: FieldType,
}

pub struct FieldTypeBuilder(pub FieldType);

impl FieldTypeBuilder {
    pub fn new(t: Type) -> Self {
        Self(FieldType {
            type_: t,
            migration_default: None,
        })
    }

    pub fn migrate_fill(mut self, e: SerialExpr) -> Self {
        self.0.migration_default = Some(e);
        self
    }

    pub fn build(self) -> FieldType {
        self.0
    }
}

pub fn field_str() -> FieldTypeBuilder {
    FieldTypeBuilder::new(crate::sqlite::types::type_str().build())
}

pub fn field_i16() -> FieldTypeBuilder {
    FieldTypeBuilder::new(crate::sqlite::types::type_i16().build())
}

pub fn field_i32() -> FieldTypeBuilder {
    FieldTypeBuilder::new(crate::sqlite::types::type_i32().build())
}

pub fn field_i64() -> FieldTypeBuilder {
    FieldTypeBuilder::new(crate::sqlite::types::type_i64().build())
}

pub fn field_u32() -> FieldTypeBuilder {
    FieldTypeBuilder::new(crate::sqlite::types::type_u32().build())
}

pub fn field_f32() -> FieldTypeBuilder {
    FieldTypeBuilder::new(crate::sqlite::types::type_f32().build())
}

pub fn field_f64() -> FieldTypeBuilder {
    FieldTypeBuilder::new(crate::sqlite::types::type_f64().build())
}

pub fn field_bool() -> FieldTypeBuilder {
    FieldTypeBuilder::new(crate::sqlite::types::type_bool().build())
}

pub fn field_bytes() -> FieldTypeBuilder {
    FieldTypeBuilder::new(crate::sqlite::types::type_bytes().build())
}

pub fn field_auto() -> FieldTypeBuilder {
    FieldTypeBuilder::new(crate::sqlite::types::type_auto().build())
}

pub fn field_utctime_s_chrono() -> FieldTypeBuilder {
    FieldTypeBuilder::new(crate::sqlite::types::type_utctime_s_chrono().build())
}

pub fn field_utctime_ms_chrono() -> FieldTypeBuilder {
    FieldTypeBuilder::new(crate::sqlite::types::type_utctime_ms_chrono().build())
}

pub fn field_utctime_s_jiff() -> FieldTypeBuilder {
    FieldTypeBuilder::new(crate::sqlite::types::type_utctime_s_jiff().build())
}

pub fn field_utctime_ms_jiff() -> FieldTypeBuilder {
    FieldTypeBuilder::new(crate::sqlite::types::type_utctime_ms_jiff().build())
}

impl FieldTypeBuilder {
    pub fn opt(mut self) -> Self {
        self.0.type_.opt = true;
        self
    }

    pub(crate) fn custom(mut self, s: impl ToString) -> Self {
        self.0.type_.type_.custom = Some(s.to_string());
        self
    }
}
