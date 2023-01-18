use std::{
    collections::HashSet,
};
use enum_dispatch::enum_dispatch;
use samevariant::samevariant;
use crate::buildtime::graphmigrate::Comparison;
use super::{
    table::{
        NodeTable_,
        TableId,
    },
    field::{
        NodeField_,
        FieldId,
    },
    constraint::{
        NodeConstraint_,
        ConstraintId,
    },
    index::{
        NodeIndex_,
        IndexId,
    },
    utils::{
        PgMigrateCtx,
        NodeDataDispatch,
        NodeData,
    },
};

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum Id {
    Table(TableId),
    Field(FieldId),
    Constraint(ConstraintId),
    Index(IndexId),
}

#[derive(Clone)]
#[enum_dispatch(NodeDataDispatch)]
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

impl<'a> crate::buildtime::graphmigrate::NodeData for Node {
    type O = PgMigrateCtx;
    type I = Id;

    fn compare(&self, other: &Self, created: &HashSet<Self::I>) -> Comparison {
        match PairwiseNode::pairs(self, &other) {
            PairwiseNode::Table(current, old) => current.compare(old, created),
            PairwiseNode::Field(current, old) => current.compare(old, created),
            PairwiseNode::Constraint(current, old) => current.compare(old, created),
            PairwiseNode::Index(current, old) => current.compare(old, created),
            PairwiseNode::Nonmatching(_, _) => unreachable!(),
        }
    }

    fn create(&self, ctx: &mut PgMigrateCtx) {
        NodeDataDispatch::create(self, ctx)
    }

    fn delete(&self, ctx: &mut PgMigrateCtx) {
        NodeDataDispatch::delete(self, ctx)
    }

    fn update(&self, ctx: &mut PgMigrateCtx, old: &Self) {
        match PairwiseNode::pairs(self, &old) {
            PairwiseNode::Table(current, old) => current.update(ctx, &old),
            PairwiseNode::Field(current, old) => current.update(ctx, &old),
            PairwiseNode::Constraint(current, old) => current.update(ctx, &old),
            PairwiseNode::Index(current, old) => current.update(ctx, &old),
            PairwiseNode::Nonmatching(_, _) => unreachable!(),
        }
    }

    fn create_coalesce(&mut self, other: Self) -> Option<Self> {
        NodeDataDispatch::create_coalesce(self, other)
    }

    fn delete_coalesce(&mut self, other: Self) -> Option<Self> {
        NodeDataDispatch::delete_coalesce(self, other)
    }
}
