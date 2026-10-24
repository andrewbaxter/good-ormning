pub mod constraint;
pub mod field;
pub mod index;
pub mod table;
pub mod utils;
#[cfg(test)]
mod test;

pub use self::utils::SqliteMigrateCtx;
use {
    crate::graphmigrate::{
        Comparison,
        Op,
    },
    std::collections::HashSet,
};

pub trait NodeData: NodeDataDispatch {
    fn update(&self, ctx: &mut SqliteMigrateCtx, old: &Self);
}

pub trait NodeDataDispatch {
    fn create(&self, ctx: &mut SqliteMigrateCtx);
    fn create_coalesce(&mut self, other: Node) -> Option<Node>;
    fn delete(&self, ctx: &mut SqliteMigrateCtx);
    fn delete_coalesce(&mut self, other: Node) -> Option<Node>;
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, PartialOrd, Ord)]
pub enum GraphId {
    Constraint(String, String),
    Field(String, String),
    Index(String, String),
    Table(String),
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum SqlName {
    Column(String, String),
    Index(String),
    Table(String),
}

#[derive(Clone)]
pub enum Node {
    Constraint(constraint::NodeConstraint_),
    Field(field::NodeField_),
    Index(index::NodeIndex_),
    Table(table::NodeTable_),
}

impl crate::graphmigrate::NodeData for Node {
    type I = GraphId;
    type N = SqlName;
    type O = SqliteMigrateCtx;

    fn apply(ctx: &mut SqliteMigrateCtx, ops: &[Op<'_, Self>]) {
        match ops {
            [Op::Create(n)] => n.create(ctx),
            [Op::Delete(n)] => n.delete(ctx),
            [Op::Update(u)] => match (u.new, u.old) {
                (Node::Table(a), Node::Table(b)) => a.update(ctx, b),
                (Node::Field(a), Node::Field(b)) => a.update(ctx, b),
                (Node::Constraint(a), Node::Constraint(b)) => a.update(ctx, b),
                (Node::Index(a), Node::Index(b)) => a.update(ctx, b),
                _ => unreachable!(),
            },
            ops => {
                let describe = |n: &Node| match n {
                    Node::Table(x) => format!("table {}", x.def.id),
                    Node::Field(x) => format!("field {}.{}", x.table_id, x.def.id),
                    Node::Constraint(x) => format!("constraint {}.{}", x.table_id, x.def.id),
                    Node::Index(x) => format!("index {}.{}", x.table_id, x.def.id),
                };
                ctx
                    .errs
                    .err(
                        &rpds::vector![],
                        format!(
                            "These schema changes depend on each other and can't be ordered, split them across versions: {}",
                            ops.iter().map(|op| match op {
                                Op::Create(n) => format!("create {}", describe(n)),
                                Op::Delete(n) => format!("delete {}", describe(n)),
                                Op::Update(u) => format!("update {}", describe(u.new)),
                            }).collect::<Vec<_>>().join(", ")
                        ),
                    );
            },
        }
    }

    fn compare(&self, other: &Self, created: &HashSet<Self::I>) -> Comparison {
        return match (self, other) {
            (Node::Table(a), Node::Table(b)) => a.compare(b, created),
            (Node::Field(a), Node::Field(b)) => a.compare(b, created),
            (Node::Constraint(a), Node::Constraint(b)) => a.compare(b, created),
            (Node::Index(a), Node::Index(b)) => a.compare(b, created),
            _ => unreachable!(),
        };
    }

    fn create_coalesce(&mut self, other: Self) -> Option<Self> {
        return <Self as NodeDataDispatch>::create_coalesce(self, other);
    }

    fn delete_coalesce(&mut self, other: Self) -> Option<Self> {
        return <Self as NodeDataDispatch>::delete_coalesce(self, other);
    }

    fn names(&self, old: bool) -> Vec<SqlName> {
        let scope = |table_id: &String, table_renamed_from: &Option<String>| {
            return if old {
                table_id.clone()
            } else {
                table_renamed_from.clone().unwrap_or_else(|| table_id.clone())
            };
        };
        return match self {
            Node::Table(x) => vec![SqlName::Table(x.def.id.clone())],
            Node::Field(x) => vec![SqlName::Column(scope(&x.table_id, &x.table_renamed_from), x.def.id.clone())],
            Node::Constraint(_) => vec![],
            Node::Index(x) => vec![SqlName::Index(x.def.id.clone())],
        };
    }

    fn renamed_from(&self) -> Option<Self::I> {
        return match self {
            Node::Table(x) => x.def.renamed_from.as_ref().map(|s| GraphId::Table(s.clone())),
            Node::Field(x) => {
                let old_table_id = x.table_renamed_from.as_ref().unwrap_or(&x.table_id);
                if x.table_renamed_from.is_some() || x.def.renamed_from.is_some() {
                    Some(
                        GraphId::Field(
                            old_table_id.clone(),
                            x.def.renamed_from.clone().unwrap_or_else(|| x.def.id.clone()),
                        ),
                    )
                } else {
                    None
                }
            },
            Node::Constraint(x) => {
                let old_table_id = x.table_renamed_from.as_ref().unwrap_or(&x.table_id);
                if x.table_renamed_from.is_some() || x.def.renamed_from.is_some() {
                    Some(
                        GraphId::Constraint(
                            old_table_id.clone(),
                            x.def.renamed_from.clone().unwrap_or_else(|| x.def.id.clone()),
                        ),
                    )
                } else {
                    None
                }
            },
            Node::Index(x) => {
                let old_table_id = x.table_renamed_from.as_ref().unwrap_or(&x.table_id);
                if x.table_renamed_from.is_some() || x.def.renamed_from.is_some() {
                    Some(
                        GraphId::Index(
                            old_table_id.clone(),
                            x.def.renamed_from.clone().unwrap_or_else(|| x.def.id.clone()),
                        ),
                    )
                } else {
                    None
                }
            },
        };
    }
}

impl NodeDataDispatch for Node {
    fn create(&self, ctx: &mut SqliteMigrateCtx) {
        match self {
            Node::Table(x) => x.create(ctx),
            Node::Field(x) => x.create(ctx),
            Node::Constraint(x) => x.create(ctx),
            Node::Index(x) => x.create(ctx),
        }
    }

    fn create_coalesce(&mut self, other: Node) -> Option<Node> {
        return match self {
            Node::Table(x) => x.create_coalesce(other),
            Node::Field(x) => x.create_coalesce(other),
            Node::Constraint(x) => x.create_coalesce(other),
            Node::Index(x) => x.create_coalesce(other),
        };
    }

    fn delete(&self, ctx: &mut SqliteMigrateCtx) {
        match self {
            Node::Table(x) => x.delete(ctx),
            Node::Field(x) => x.delete(ctx),
            Node::Constraint(x) => x.delete(ctx),
            Node::Index(x) => x.delete(ctx),
        }
    }

    fn delete_coalesce(&mut self, other: Node) -> Option<Node> {
        return match self {
            Node::Table(x) => x.delete_coalesce(other),
            Node::Field(x) => x.delete_coalesce(other),
            Node::Constraint(x) => x.delete_coalesce(other),
            Node::Index(x) => x.delete_coalesce(other),
        };
    }
}
