use std::collections::HashSet;
use crate::graphmigrate::Comparison;
pub use self::utils::PgMigrateCtx;
use super::schema::{
    table::SchemaTableId,
    field::SchemaFieldId,
    constraint::SchemaConstraintId,
    index::SchemaIndexId,
};

pub mod table;
pub mod field;
pub mod constraint;
pub mod index;
pub mod utils;

pub trait NodeDataDispatch {
    fn create(&self, ctx: &mut PgMigrateCtx);
    fn create_coalesce(&mut self, other: Node) -> Option<Node>;
    fn delete_coalesce(&mut self, other: Node) -> Option<Node>;
    fn delete(&self, ctx: &mut PgMigrateCtx);
}

pub trait NodeData: NodeDataDispatch {
    fn update(&self, ctx: &mut PgMigrateCtx, old: &Self);
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, PartialOrd, Ord)]
pub enum GraphId {
    Table(SchemaTableId),
    Field(SchemaTableId, SchemaFieldId),
    Constraint(SchemaTableId, SchemaConstraintId),
    Index(SchemaTableId, SchemaIndexId),
}

#[derive(Clone)]
pub enum Node {
    Table(table::NodeTable_),
    Field(field::NodeField_),
    Constraint(constraint::NodeConstraint_),
    Index(index::NodeIndex_),
}

impl Node {
    pub(crate) fn table(t: table::NodeTable_) -> Self {
        Node::Table(t)
    }

    pub(crate) fn field(t: field::NodeField_) -> Self {
        Node::Field(t)
    }

    pub(crate) fn table_constraint(t: constraint::NodeConstraint_) -> Self {
        Node::Constraint(t)
    }

    pub(crate) fn table_index(t: index::NodeIndex_) -> Self {
        Node::Index(t)
    }
}

impl NodeDataDispatch for Node {
    fn create(&self, ctx: &mut PgMigrateCtx) {
        match self {
            Node::Table(x) => x.create(ctx),
            Node::Field(x) => x.create(ctx),
            Node::Constraint(x) => x.create(ctx),
            Node::Index(x) => x.create(ctx),
        }
    }

    fn create_coalesce(&mut self, other: Node) -> Option<Node> {
        match self {
            Node::Table(x) => x.create_coalesce(other),
            Node::Field(x) => x.create_coalesce(other),
            Node::Constraint(x) => x.create_coalesce(other),
            Node::Index(x) => x.create_coalesce(other),
        }
    }

    fn delete_coalesce(&mut self, other: Node) -> Option<Node> {
        match self {
            Node::Table(x) => x.delete_coalesce(other),
            Node::Field(x) => x.delete_coalesce(other),
            Node::Constraint(x) => x.delete_coalesce(other),
            Node::Index(x) => x.delete_coalesce(other),
        }
    }

    fn delete(&self, ctx: &mut PgMigrateCtx) {
        match self {
            Node::Table(x) => x.delete(ctx),
            Node::Field(x) => x.delete(ctx),
            Node::Constraint(x) => x.delete(ctx),
            Node::Index(x) => x.delete(ctx),
        }
    }
}

impl crate::graphmigrate::NodeData for Node {
    type O = PgMigrateCtx;
    type I = GraphId;

    fn compare(&self, other: &Self, created: &HashSet<Self::I>) -> Comparison {
        match (self, other) {
            (Node::Table(a), Node::Table(b)) => a.compare(b, created),
            (Node::Field(a), Node::Field(b)) => a.compare(b, created),
            (Node::Constraint(a), Node::Constraint(b)) => a.compare(b, created),
            (Node::Index(a), Node::Index(b)) => a.compare(b, created),
            _ => unreachable!(),
        }
    }

    fn create(&self, ctx: &mut PgMigrateCtx) {
        <Self as NodeDataDispatch>::create(self, ctx)
    }

    fn delete(&self, ctx: &mut PgMigrateCtx) {
        <Self as NodeDataDispatch>::delete(self, ctx)
    }

    fn update(&self, ctx: &mut PgMigrateCtx, old: &Self) {
        match (self, old) {
            (Node::Table(a), Node::Table(b)) => a.update(ctx, b),
            (Node::Field(a), Node::Field(b)) => a.update(ctx, b),
            (Node::Constraint(a), Node::Constraint(b)) => a.update(ctx, b),
            (Node::Index(a), Node::Index(b)) => a.update(ctx, b),
            _ => unreachable!(),
        }
    }

    fn create_coalesce(&mut self, other: Self) -> Option<Self> {
        <Self as NodeDataDispatch>::create_coalesce(self, other)
    }

    fn delete_coalesce(&mut self, other: Self) -> Option<Self> {
        <Self as NodeDataDispatch>::delete_coalesce(self, other)
    }
}
