pub mod constraint;
pub mod field;
pub mod index;
pub mod table;
pub mod utils;
#[cfg(test)]
mod test;

pub use self::utils::MariadbMigrateCtx;
use {
    crate::{
        graphmigrate::{
            Comparison,
            Op,
        },
        mariadb::schema::constraint::ConstraintType,
        utils::Tokens,
    },
    std::collections::{
        BTreeMap,
        HashSet,
    },
};

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
    ForeignKey(String),
    Index(String, String),
    Table(String),
}

#[derive(Clone)]
pub enum Node {
    Constraint(constraint::NodeConstraint_),
    Field(field::NodeField_),
    Index(index::NodeIndex_),
    Table(table::NodeTable_),
}

impl Node {
    fn leading_column_new(&self) -> Option<(String, String)> {
        return match self {
            Node::Index(x) => x
                .fields_old_ids
                .first()
                .map(|f| (x.table_renamed_from.clone().unwrap_or_else(|| x.table_id.clone()), f.clone())),
            Node::Constraint(x) => match &x.type_old_ids {
                ConstraintType::PrimaryKey(pk) => pk
                    .fields
                    .first()
                    .map(|f| (x.table_renamed_from.clone().unwrap_or_else(|| x.table_id.clone()), f.clone())),
                ConstraintType::ForeignKey(_) => None,
            },
            _ => None,
        };
    }
}

impl crate::graphmigrate::NodeData for Node {
    type I = GraphId;
    type N = SqlName;
    type O = MariadbMigrateCtx;

    fn apply(ctx: &mut MariadbMigrateCtx, ops: &[Op<'_, Self>]) {
        match ops {
            [Op::Create(Node::Table(x))] => return x.create(ctx),
            [Op::Delete(Node::Table(x))] => return x.delete(ctx),
            [Op::Update(u)] => if let (Node::Table(new), Node::Table(old)) = (u.new, u.old) {
                return new.update(ctx, old);
            },
            _ => { },
        }
        let merged = ops.len() > 1;
        let mut renamed_from = HashSet::new();
        let mut renames = vec![];
        for op in ops {
            if let Op::Update(u) = op {
                let (from, to) = match (u.new, u.old) {
                    (Node::Table(new), Node::Table(old)) => (
                        format!("table {}", old.def.id),
                        format!("table {}", new.def.id),
                    ),
                    (Node::Field(new), Node::Field(old)) => (
                        format!("column {}.{}", old.table_id, old.def.id),
                        format!("column {}.{}", new.table_id, new.def.id),
                    ),
                    (Node::Index(new), Node::Index(old)) => (
                        format!("index {}.{}", old.table_id, old.def.id),
                        format!("index {}.{}", new.table_id, new.def.id),
                    ),
                    (Node::Constraint(new), Node::Constraint(old)) => (
                        format!("constraint {}.{}", old.table_id, old.def.id),
                        format!("constraint {}.{}", new.table_id, new.def.id),
                    ),
                    _ => continue,
                };
                if from != to {
                    renamed_from.insert(from.clone());
                    renames.push((from, to));
                }
            }
        }
        if merged && renames.iter().any(|(_, to)| renamed_from.contains(to)) {
            ctx
                .errs
                .err(
                    &rpds::vector![],
                    format!(
                        "These renames swap names, which can't be done safely in one migration step; rename through a temporary name in an intermediate version instead: {}",
                        renames.iter().map(|(from, to)| format!("{} to {}", from, to)).collect::<Vec<_>>().join(", ")
                    ),
                );
            return;
        }
        let no_table = String::new();
        let mut table_id = None;
        let mut statements = vec![];
        for op in ops {
            let (op_table_id, op_statements) = match op {
                Op::Create(Node::Field(x)) => (&x.table_id, x.create_clauses(ctx)),
                Op::Create(Node::Index(x)) => (&x.table_id, x.create_clauses(ctx, merged)),
                Op::Create(Node::Constraint(x)) => (&x.table_id, x.create_clauses(ctx, merged)),
                Op::Delete(Node::Field(x)) => (&x.table_id, x.delete_clauses()),
                Op::Delete(Node::Index(x)) => (&x.table_id, x.delete_clauses()),
                Op::Delete(Node::Constraint(x)) => (&x.table_id, x.delete_clauses()),
                Op::Update(u) => match (u.new, u.old) {
                    (Node::Field(new), Node::Field(old)) => (&new.table_id, new.update_clauses(ctx, old, merged)),
                    (Node::Index(new), Node::Index(old)) => (&new.table_id, new.update_clauses(old)),
                    (Node::Constraint(new), Node::Constraint(old)) => (
                        &new.table_id,
                        new.update_clauses(ctx, old, merged),
                    ),
                    _ => (&no_table, vec![]),
                },
                _ => (&no_table, vec![]),
            };
            if op_table_id.is_empty() || *table_id.get_or_insert(op_table_id) != op_table_id {
                ctx
                    .errs
                    .err(
                        &rpds::vector![],
                        format!(
                            "These schema changes depend on each other and MariaDB can only make them together within one table, split them across versions: {}",
                            ops.iter().map(|op| {
                                let (action, node) = match op {
                                    Op::Create(n) => ("create", *n),
                                    Op::Delete(n) => ("delete", *n),
                                    Op::Update(u) => ("update", u.new),
                                };
                                return match node {
                                    Node::Table(x) => format!("{} table {}", action, x.def.id),
                                    Node::Field(x) => format!("{} field {}.{}", action, x.table_id, x.def.id),
                                    Node::Constraint(x) => format!(
                                        "{} constraint {}.{}",
                                        action,
                                        x.table_id,
                                        x.def.id
                                    ),
                                    Node::Index(x) => format!("{} index {}.{}", action, x.table_id, x.def.id),
                                };
                            }).collect::<Vec<_>>().join(", ")
                        ),
                    );
                return;
            }
            statements.extend(op_statements);
        }
        let Some(table_id) = table_id else {
            return;
        };
        if merged {
            let mut clauses = vec![];
            for clause in statements.into_iter().flatten() {
                if !clauses.contains(&clause) {
                    clauses.push(clause);
                }
            }
            statements = vec![clauses];
        }
        for clauses in statements {
            let mut stmt = Tokens::new_mariadb();
            stmt.s("alter table").id(table_id);
            for (i, clause) in clauses.iter().enumerate() {
                if i > 0 {
                    stmt.s(",");
                }
                stmt.s(clause);
            }
            ctx.statements.push(stmt.to_string());
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
        return match self {
            Node::Table(x) => x.create_coalesce(other),
            _ => Some(other),
        };
    }

    fn delete_coalesce(&mut self, other: Self) -> Option<Self> {
        return match self {
            Node::Table(x) => x.delete_coalesce(other),
            _ => Some(other),
        };
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
            Node::Constraint(x) => match &x.def.type_ {
                ConstraintType::PrimaryKey(_) => vec![
                    SqlName::Index(scope(&x.table_id, &x.table_renamed_from), "PRIMARY".to_string()),
                ],
                ConstraintType::ForeignKey(_) => vec![
                    SqlName::ForeignKey(x.def.id.clone()),
                    SqlName::Index(scope(&x.table_id, &x.table_renamed_from), x.def.id.clone()),
                ],
            },
            Node::Index(x) => vec![SqlName::Index(scope(&x.table_id, &x.table_renamed_from), x.def.id.clone())],
        };
    }

    fn ordering(new: &[(&Self, Option<usize>)], ops: &[Op<'_, Self>]) -> Vec<(usize, usize)> {
        #[derive(Default)]
        struct Column {
            kept_key: bool,
            needed_before: bool,
            add_keys: Vec<usize>,
            remove_keys: Vec<usize>,
            add_need: Vec<usize>,
            remove_need: Vec<usize>,
        }

        let mut columns: BTreeMap<(String, String), Column> = BTreeMap::new();
        for (node, op) in new {
            if let Some(column) = node.leading_column_new() {
                if op.map(|op| matches!(ops.get(op).unwrap(), Op::Update(_))).unwrap_or(true) {
                    columns.entry(column).or_default().kept_key = true;
                }
            }
            if let (Node::Field(x), None) = (node, op) {
                let column =
                    (
                        x.table_renamed_from.clone().unwrap_or_else(|| x.table_id.clone()),
                        x.def.renamed_from.clone().unwrap_or_else(|| x.def.id.clone()),
                    );
                columns.entry(column).or_default().needed_before |= x.def.type_.auto_increment;
            }
        }
        for (i, op) in ops.iter().enumerate() {
            match op {
                Op::Create(Node::Field(x)) => {
                    if x.def.type_.auto_increment {
                        let column =
                            (
                                x.table_renamed_from.clone().unwrap_or_else(|| x.table_id.clone()),
                                x.def.renamed_from.clone().unwrap_or_else(|| x.def.id.clone()),
                            );
                        columns.entry(column).or_default().add_need.push(i);
                    }
                },
                Op::Delete(Node::Field(x)) => {
                    if x.def.type_.auto_increment {
                        let column = columns.entry((x.table_id.clone(), x.def.id.clone())).or_default();
                        column.needed_before = true;
                        column.remove_need.push(i);
                    }
                },
                Op::Update(u) => {
                    if let (Node::Field(new), Node::Field(old)) = (u.new, u.old) {
                        let column = columns.entry((old.table_id.clone(), old.def.id.clone())).or_default();
                        column.needed_before |= old.def.type_.auto_increment;
                        if old.def.type_.auto_increment && !new.def.type_.auto_increment {
                            column.remove_need.push(i);
                        }
                        if !old.def.type_.auto_increment && new.def.type_.auto_increment {
                            column.add_need.push(i);
                        }
                    }
                },
                Op::Create(node) => {
                    if let Some(column) = node.leading_column_new() {
                        columns.entry(column).or_default().add_keys.push(i);
                    }
                },
                Op::Delete(node) => {
                    let column = match node {
                        Node::Index(x) => x.def.fields.first().map(|f| (x.table_id.clone(), f.clone())),
                        Node::Constraint(x) => match &x.def.type_ {
                            ConstraintType::PrimaryKey(pk) => pk
                                .fields
                                .first()
                                .map(|f| (x.table_id.clone(), f.clone())),
                            ConstraintType::ForeignKey(_) => None,
                        },
                        _ => None,
                    };
                    if let Some(column) = column {
                        columns.entry(column).or_default().remove_keys.push(i);
                    }
                },
            }
        }
        let mut out = vec![];
        for column in columns.values() {
            if column.kept_key {
                continue;
            }
            if column.needed_before {
                for remove in &column.remove_keys {
                    for add in column.add_keys.iter().chain(column.remove_need.iter()) {
                        out.push((*add, *remove));
                    }
                }
            }
            for need in &column.add_need {
                for add in &column.add_keys {
                    out.push((*add, *need));
                }
            }
        }
        return out;
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
