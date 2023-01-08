use std::marker::PhantomData;
use enum_dispatch::enum_dispatch;
use samevariant::samevariant;
use crate::graphmigrate::Comparison;
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
pub(crate) enum Node_ {
    Table(NodeTable_),
    Field(NodeField_),
    Constraint(NodeConstraint_),
    Index(NodeIndex_),
}

#[derive(Clone)]
pub(crate) struct Node<'a> {
    pub(crate) n: Node_,
    // Rust is awesome
    _pd: PhantomData<&'a i32>,
}

impl<'a> Node<'a> {
    pub(crate) fn table(t: NodeTable_) -> Self {
        Node {
            n: Node_::Table(t),
            _pd: Default::default(),
        }
    }

    pub(crate) fn field(t: NodeField_) -> Self {
        Node {
            n: Node_::Field(t),
            _pd: Default::default(),
        }
    }

    pub(crate) fn table_constraint(t: NodeConstraint_) -> Self {
        Node {
            n: Node_::Constraint(t),
            _pd: Default::default(),
        }
    }

    pub(crate) fn table_index(t: NodeIndex_) -> Self {
        Node {
            n: Node_::Index(t),
            _pd: Default::default(),
        }
    }
}

impl<'a> crate::graphmigrate::NodeData for Node<'a> {
    type O = PgMigrateCtx<'a>;

    fn compare(&self, other: &Self) -> Comparison {
        match PairwiseNode::pairs(&self.n, &other.n) {
            PairwiseNode::Table(current, old) => current.compare(old),
            PairwiseNode::Field(current, old) => current.compare(old),
            PairwiseNode::Constraint(current, old) => current.compare(old),
            PairwiseNode::Index(current, old) => current.compare(old),
            PairwiseNode::Nonmatching(_, _) => unreachable!(),
        }
    }

    fn create(&self, ctx: &mut PgMigrateCtx) {
        NodeDataDispatch::create(&self.n, ctx)
    }

    fn delete(&self, ctx: &mut PgMigrateCtx) {
        NodeDataDispatch::delete(&self.n, ctx)
    }

    fn update(&self, ctx: &mut PgMigrateCtx, old: &Self) {
        match PairwiseNode::pairs(&self.n, &old.n) {
            PairwiseNode::Table(current, old) => current.update(ctx, &old),
            PairwiseNode::Field(current, old) => current.update(ctx, &old),
            PairwiseNode::Constraint(current, old) => current.update(ctx, &old),
            PairwiseNode::Index(current, old) => current.update(ctx, &old),
            PairwiseNode::Nonmatching(_, _) => unreachable!(),
        }
    }

    fn create_coalesce(&mut self, other: &Self) -> bool {
        NodeDataDispatch::create_coalesce(&mut self.n, &other.n)
    }

    fn delete_coalesce(&mut self, other: &Self) -> bool {
        NodeDataDispatch::delete_coalesce(&mut self.n, &other.n)
    }
}
