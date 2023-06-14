use std::{
    fmt::{
        Debug,
        Display,
    },
    rc::Rc,
    ops::Deref,
};
use crate::{
    pg::{
        types::{
            SimpleSimpleType,
            SimpleType,
            Type,
        },
        query::{
            expr::{
                Expr,
            },
        },
    },
};
use super::table::{
    Table,
};

#[derive(Clone, Debug)]
pub struct FieldType {
    pub type_: Type,
    pub migration_default: Option<Expr>,
}

impl FieldType {
    /// Create a field type from the specified value type.
    pub fn with(t: &Type) -> Self {
        Self {
            type_: t.clone(),
            migration_default: None,
        }
    }

    /// Create a field type from the specified value type, and provide a migration fill
    /// value.
    pub fn with_migration(t: &Type, def: Option<Expr>) -> Self {
        if t.opt {
            panic!("Optional fields can't have defaults.");
        }
        Self {
            type_: t.clone(),
            migration_default: def,
        }
    }
}

pub struct FieldBuilder {
    t: SimpleSimpleType,
    default_: Option<Expr>,
    opt: bool,
    custom: Option<String>,
}

impl FieldBuilder {
    fn new(t: SimpleSimpleType) -> FieldBuilder {
        FieldBuilder {
            t: t,
            opt: false,
            default_: None,
            custom: None,
        }
    }

    /// Make the field optional.
    pub fn opt(mut self) -> FieldBuilder {
        if self.default_.is_some() {
            panic!("Optional fields can't have migration fill expressions.");
        }
        self.opt = true;
        self
    }

    /// Specify an expression to use to populate the new column in existing rows. This
    /// is must be specified (only) for non-opt fields in a new version of an existing
    /// table.
    pub fn migrate_fill(mut self, expr: Expr) -> FieldBuilder {
        if self.opt {
            panic!("Optional fields can't have migration fill expressions.");
        }
        self.default_ = Some(expr);
        self
    }

    /// Use a custom Rust type for this field. This must be the full path to the type,
    /// like `crate::abcdef::MyType`.
    pub fn custom(mut self, type_: impl ToString) -> FieldBuilder {
        self.custom = Some(type_.to_string());
        self
    }

    pub fn build(self) -> FieldType {
        FieldType {
            type_: Type {
                type_: SimpleType {
                    custom: self.custom,
                    type_: self.t,
                },
                opt: self.opt,
            },
            migration_default: self.default_,
        }
    }
}

pub fn field_auto() -> FieldBuilder {
    FieldBuilder::new(SimpleSimpleType::Auto)
}

pub fn field_bool() -> FieldBuilder {
    FieldBuilder::new(SimpleSimpleType::Bool)
}

pub fn field_i32() -> FieldBuilder {
    FieldBuilder::new(SimpleSimpleType::I32)
}

pub fn field_i64() -> FieldBuilder {
    FieldBuilder::new(SimpleSimpleType::I64)
}

pub fn field_f32() -> FieldBuilder {
    FieldBuilder::new(SimpleSimpleType::F32)
}

pub fn field_f64() -> FieldBuilder {
    FieldBuilder::new(SimpleSimpleType::F64)
}

pub fn field_str() -> FieldBuilder {
    FieldBuilder::new(SimpleSimpleType::String)
}

pub fn field_bytes() -> FieldBuilder {
    FieldBuilder::new(SimpleSimpleType::Bytes)
}

#[cfg(feature = "chrono")]
pub fn field_utctime() -> FieldBuilder {
    FieldBuilder::new(SimpleSimpleType::UtcTime)
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, PartialOrd, Ord)]
pub struct SchemaFieldId(pub String);

impl Display for SchemaFieldId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

#[derive(Debug)]
pub struct Field_ {
    pub table: Table,
    pub schema_id: SchemaFieldId,
    pub id: String,
    pub type_: FieldType,
}

#[derive(Clone, Debug)]
pub struct Field(pub Rc<Field_>);

impl std::hash::Hash for Field {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.schema_id.hash(state)
    }
}

impl PartialEq for Field {
    fn eq(&self, other: &Self) -> bool {
        self.table == other.table && self.schema_id == other.schema_id
    }
}

impl Eq for Field { }

impl Deref for Field {
    type Target = Field_;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&format!("{}.{} ({}.{})", self.table.id, self.id, self.table.schema_id.0, self.schema_id.0), f)
    }
}
