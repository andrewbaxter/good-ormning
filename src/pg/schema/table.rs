use std::{
    fmt::{
        Debug,
        Display,
    },
    rc::Rc,
    ops::Deref,
    hash::Hash,
};

#[derive(Clone, Eq, PartialEq, Hash, Debug, PartialOrd, Ord)]
pub struct SchemaTableId(pub String);

impl Display for SchemaTableId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

#[derive(Debug)]
pub struct Table_ {
    pub schema_id: SchemaTableId,
    pub id: String,
}

#[derive(Clone, Debug)]
pub struct Table(pub Rc<Table_>);

impl Hash for Table {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.schema_id.hash(state)
    }
}

impl PartialEq for Table {
    fn eq(&self, other: &Self) -> bool {
        self.schema_id == other.schema_id
    }
}

impl Eq for Table { }

impl Deref for Table {
    type Target = Table_;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&format!("{} ({})", self.id, self.schema_id.0), f)
    }
}
