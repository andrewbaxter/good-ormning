use std::{
    rc::Rc,
    ops::Deref,
    fmt::Display,
};
use super::{
    table::{
        Table,
    },
    field::Field,
};

#[derive(Clone, Eq, PartialEq, Hash, Debug, PartialOrd, Ord)]
pub struct SchemaConstraintId(pub String);

impl Display for SchemaConstraintId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

#[derive(Clone, PartialEq)]
pub struct PrimaryKeyDef {
    pub fields: Vec<Field>,
}

#[derive(Clone, PartialEq)]
pub struct ForeignKeyDef {
    pub fields: Vec<(Field, Field)>,
}

#[derive(Clone, PartialEq)]
pub enum ConstraintType {
    PrimaryKey(PrimaryKeyDef),
    ForeignKey(ForeignKeyDef),
}

pub struct Constraint_ {
    pub table: Table,
    pub schema_id: SchemaConstraintId,
    pub id: String,
    pub type_: ConstraintType,
}

#[derive(Clone)]
pub struct Constraint(pub Rc<Constraint_>);

impl Display for Constraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(
            &format!("{}.{} ({}.{})", self.0.table.id, self.0.id, self.0.table.schema_id, self.0.schema_id),
            f,
        )
    }
}

impl PartialEq for Constraint {
    fn eq(&self, other: &Self) -> bool {
        self.table == other.table && self.schema_id == other.schema_id
    }
}

impl Eq for Constraint { }

impl Deref for Constraint {
    type Target = Constraint_;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}
