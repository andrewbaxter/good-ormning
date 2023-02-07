use std::{
    rc::Rc,
    ops::Deref,
    fmt::Display,
};
use super::{
    field::Field,
    table::Table,
};

#[derive(Clone, Eq, PartialEq, Hash, Debug, PartialOrd, Ord)]
pub struct SchemaIndexId(pub String);

impl Display for SchemaIndexId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

pub struct Index_ {
    pub table: Table,
    pub schema_id: SchemaIndexId,
    pub id: String,
    pub fields: Vec<Field>,
    pub unique: bool,
}

#[derive(Clone)]
pub struct Index(pub Rc<Index_>);

impl Display for Index {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(
            &format!("{}.{} ({}.{})", self.0.table.id, self.0.id, self.0.table.schema_id, self.0.schema_id),
            f,
        )
    }
}

impl PartialEq for Index {
    fn eq(&self, other: &Self) -> bool {
        self.table == other.table && self.schema_id == other.schema_id
    }
}

impl Eq for Index { }

impl Deref for Index {
    type Target = Index_;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}
