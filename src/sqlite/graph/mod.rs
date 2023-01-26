use std::collections::HashSet;
use enum_dispatch::enum_dispatch;
use samevariant::samevariant;
use crate::graphmigrate::Comparison;
use self::{
    table::NodeTable_,
    field::NodeField_,
    constraint::NodeConstraint_,
    index::NodeIndex_,
    utils::{
        SqliteMigrateCtx,
        SqliteNodeDataDispatch,
        SqliteNodeData,
    },
};
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

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum GraphId {
    Table(SchemaTableId),
    Field(SchemaTableId, SchemaFieldId),
    Constraint(SchemaTableId, SchemaConstraintId),
    Index(SchemaTableId, SchemaIndexId),
}

#[derive(Clone)]
#[enum_dispatch(SqliteNodeDataDispatch)]
#[samevariant(PairwiseNode)]
pub(crate) enum Node {
    Table(NodeTable_),
    Field(NodeField_),
    Constraint(NodeConstraint_),
    Index(NodeIndex_),
}

impl Node {
    pub(crate) fn table(t: NodeTable_) -> Self {
        Node::Table(t)
    }

    pub(crate) fn field(t: NodeField_) -> Self {
        Node::Field(t)
    }

    pub(crate) fn table_constraint(t: NodeConstraint_) -> Self {
        Node::Constraint(t)
    }

    pub(crate) fn table_index(t: NodeIndex_) -> Self {
        Node::Index(t)
    }
}

impl<'a> crate::graphmigrate::NodeData for Node {
    type O = SqliteMigrateCtx;
    type I = GraphId;

    fn compare(&self, other: &Self, created: &HashSet<Self::I>) -> Comparison {
        match PairwiseNode::pairs(self, &other) {
            PairwiseNode::Table(current, old) => current.compare(old, created),
            PairwiseNode::Field(current, old) => current.compare(old, created),
            PairwiseNode::Constraint(current, old) => current.compare(old, created),
            PairwiseNode::Index(current, old) => current.compare(old, created),
            PairwiseNode::Nonmatching(_, _) => unreachable!(),
        }
    }

    fn create(&self, ctx: &mut SqliteMigrateCtx) {
        SqliteNodeDataDispatch::create(self, ctx)
    }

    fn delete(&self, ctx: &mut SqliteMigrateCtx) {
        SqliteNodeDataDispatch::delete(self, ctx)
    }

    fn update(&self, ctx: &mut SqliteMigrateCtx, old: &Self) {
        match PairwiseNode::pairs(self, &old) {
            PairwiseNode::Table(current, old) => current.update(ctx, &old),
            PairwiseNode::Field(current, old) => current.update(ctx, &old),
            PairwiseNode::Constraint(current, old) => current.update(ctx, &old),
            PairwiseNode::Index(current, old) => current.update(ctx, &old),
            PairwiseNode::Nonmatching(_, _) => unreachable!(),
        }
    }

    fn create_coalesce(&mut self, other: Self) -> Option<Self> {
        SqliteNodeDataDispatch::create_coalesce(self, other)
    }

    fn delete_coalesce(&mut self, other: Self) -> Option<Self> {
        SqliteNodeDataDispatch::delete_coalesce(self, other)
    }
}
