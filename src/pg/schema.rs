use std::{
    fmt::{
        Display,
        Debug,
    },
};
use enum_dispatch::enum_dispatch;
use samevariant::samevariant;
use crate::{
    utils::Tokens,
    graphmigrate::Comparison,
};
use super::{
    types::{
        SimpleType,
        to_sql_type,
        SimpleSimpleType,
    },
    Version,
};

pub(crate) struct PgMigrateCtx<'a> {
    pub errs: &'a mut Vec<String>,
    pub ver: &'a Version,
    pub statements: Vec<String>,
}

pub(crate) type MigrateNode = crate::graphmigrate::Node<Node, Id>;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum FieldType {
    NonOpt(SimpleType, String),
    Opt(SimpleType),
}

impl FieldType {
    fn simple(&self) -> &SimpleType {
        match self {
            FieldType::NonOpt(t, _) => t,
            FieldType::Opt(t) => t,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct TableId(pub String);

impl Display for TableId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct FieldId(pub TableId, pub String);

impl Display for FieldId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&format!("{}.{}", self.0, self.1), f)
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct TableConstraintId(pub TableId, pub String);

impl Display for TableConstraintId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&format!("{}.constraint {}", self.0, self.1), f)
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct TableIndexId(pub TableId, pub String);

impl Display for TableIndexId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&format!("{}.index {}", self.0, self.1), f)
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub enum Id {
    Table(TableId),
    Field(FieldId),
    TableConstraint(TableConstraintId),
    TableIndex(TableIndexId),
}

#[derive(Clone)]
pub struct TableDef {
    pub name: String,
}

#[derive(Clone)]
pub struct NodeTable_ {
    pub id: TableId,
    pub def: TableDef,
    pub fields: Vec<(FieldId, FieldDef)>,
}

impl NodeData for NodeTable_ {
    fn update(&self, _ctx: &mut PgMigrateCtx, _old: &Self) {
        unreachable!()
    }
}

impl NodeDataDispatch for NodeTable_ {
    fn create_coalesce(&mut self, other: &Node) -> bool {
        match other {
            Node::Field(f) if f.id.0 == self.id => {
                self.fields.push((f.id.clone(), f.def.clone()));
                true
            },
            _ => false,
        }
    }

    fn delete_coalesce(&mut self, other: &Node) -> bool {
        match other {
            Node::Field(f) if f.id.0 == self.id => true,
            Node::TableConstraint(e) if e.id.0 == self.id => true,
            Node::TableIndex(e) if e.id.0 == self.id => true,
            _ => false,
        }
    }

    fn create(&self, ctx: &mut PgMigrateCtx) {
        let mut stmt = Tokens::new();
        stmt.s("create table").id(&self.id.0).s("(");
        for (i, f) in self.fields.iter().enumerate() {
            if i > 0 {
                stmt.s(",");
            }
            stmt.id(&f.0.1);
            match &f.1.type_ {
                FieldType::NonOpt(t, _) => stmt.s(&format!("{} not null", to_sql_type(t))),
                FieldType::Opt(t) => stmt.s(to_sql_type(&t)),
            };
        }
        stmt.s(")");
        ctx.statements.push(stmt.to_string());
    }

    fn delete(&self, ctx: &mut PgMigrateCtx) {
        ctx.statements.push(Tokens::new().s("drop table").id(&self.id.0).to_string());
    }
}

#[derive(Clone)]
pub struct FieldDef {
    pub name: String,
    pub type_: FieldType,
}

#[derive(Clone, PartialEq)]
pub struct PrimaryKeyDef {
    pub fields: Vec<FieldId>,
}

#[derive(Clone, PartialEq)]
pub struct ForeignKeyDef {
    pub fields: Vec<(FieldId, FieldId)>,
}

#[derive(Clone, PartialEq)]
pub enum TableConstraintTypeDef {
    PrimaryKey(PrimaryKeyDef),
    ForeignKey(ForeignKeyDef),
}

#[derive(Clone)]
pub struct TableConstraintDef {
    pub type_: TableConstraintTypeDef,
}

#[derive(Clone)]
pub struct TableIndexDef {
    pub field_ids: Vec<FieldId>,
    pub unique: bool,
}

#[derive(Clone)]
pub(crate) struct NodeField_ {
    pub id: FieldId,
    pub def: FieldDef,
}

impl NodeData for NodeField_ {
    fn update(&self, ctx: &mut PgMigrateCtx, old: &Self) {
        match (&self.def.type_, &old.def.type_) {
            (FieldType::NonOpt(t, d), FieldType::NonOpt(old_t, old_d)) => todo!(),
            (FieldType::NonOpt(t, _), FieldType::Opt(old_t)) => todo!(),
            (FieldType::Opt(t), FieldType::NonOpt(old_t, _)) => todo!(),
            (FieldType::Opt(t), FieldType::Opt(old_t)) => {
                ctx
                    .statements
                    .push(
                        Tokens::new()
                            .s("alter table")
                            .id(&self.id.0.0)
                            .s("alter column")
                            .id(&self.id.1)
                            .s("set type")
                            .s(to_sql_type(t))
                            .to_string(),
                    );
            },
        }
    }
}

impl NodeDataDispatch for NodeField_ {
    fn create(&self, ctx: &mut PgMigrateCtx) {
        if matches!(self.def.type_.simple().type_, SimpleSimpleType::Auto) {
            ctx.errs.push(format!("Auto (serial) fields can't be added after table creation"));
        }
        let mut stmt = Tokens::new();
        stmt
            .s("alter table")
            .id(&self.id.0.0)
            .s("add column")
            .id(&self.id.1)
            .s(to_sql_type(self.def.type_.simple()));
        if let FieldType::NonOpt(_, d) = &self.def.type_ {
            stmt.s("not null default");
            stmt.s(d);
        }
        ctx.statements.push(stmt.to_string());
        if let FieldType::NonOpt(_, _) = &self.def.type_ {
            let mut stmt = Tokens::new();
            stmt.s("alter table").id(&self.id.0.0).s("alter column").id(&self.id.1).s("drop default");
        }
    }

    fn delete(&self, ctx: &mut PgMigrateCtx) {
        ctx
            .statements
            .push(Tokens::new().s("alter table").id(&self.id.0.0).s("drop column").id(&self.id.1).to_string());
    }

    fn create_coalesce(&mut self, _other: &Node) -> bool {
        false
    }

    fn delete_coalesce(&mut self, _other: &Node) -> bool {
        false
    }
}

#[derive(Clone)]
pub(crate) struct NodeTableConstraint_ {
    pub id: TableConstraintId,
    pub def: TableConstraintDef,
}

impl NodeDataDispatch for NodeTableConstraint_ {
    fn create_coalesce(&mut self, _other: &Node) -> bool {
        false
    }

    fn create(&self, ctx: &mut PgMigrateCtx) {
        let mut stmt = Tokens::new();
        stmt.s("alter table").id(&self.id.0.0).s("add constraint").id(&self.id.1);
        match &self.def.type_ {
            TableConstraintTypeDef::PrimaryKey(x) => {
                stmt.s("primary key (").f(|t| {
                    for (i, id) in x.fields.iter().enumerate() {
                        if i > 0 {
                            t.s(",");
                        }
                        t.id(&id.1);
                    }
                }).s(")");
            },
            TableConstraintTypeDef::ForeignKey(x) => {
                stmt.s("foreign key (").f(|t| {
                    for (i, id) in x.fields.iter().enumerate() {
                        if i > 0 {
                            t.s(",");
                        }
                        t.id(&id.1.1);
                    }
                }).s(") references ").f(|t| {
                    for (i, id) in x.fields.iter().enumerate() {
                        if i == 0 {
                            t.id(&id.1.0.0).s("(");
                        } else {
                            t.s(",");
                        }
                        t.id(&id.1.1);
                    }
                }).s(")");
            },
        }
        ctx.statements.push(stmt.to_string());
    }

    fn delete_coalesce(&mut self, _other: &Node) -> bool {
        false
    }

    fn delete(&self, ctx: &mut PgMigrateCtx) {
        ctx
            .statements
            .push(
                Tokens::new().s("alter table").id(&self.id.0.0).s("drop constraint").id(&self.id.1).to_string(),
            );
    }
}

impl NodeData for NodeTableConstraint_ {
    fn update(&self, _ctx: &mut PgMigrateCtx, _old: &Self) {
        unreachable!()
    }
}

#[derive(Clone)]
pub(crate) struct NodeTableIndex_ {
    pub id: TableIndexId,
    pub def: TableIndexDef,
}

impl NodeDataDispatch for NodeTableIndex_ {
    fn create_coalesce(&mut self, _other: &Node) -> bool {
        false
    }

    fn create(&self, ctx: &mut PgMigrateCtx) {
        ctx.statements.push(Tokens::new().s("create").f(|t| {
            if self.def.unique {
                t.s("unique");
            }
        }).s("index").id(&self.id.1).s("on").id(&self.id.0.0).s("(").f(|t| {
            for (i, id) in self.def.field_ids.iter().enumerate() {
                if i > 0 {
                    t.s(",");
                }
                t.id(&id.1);
            }
        }).s(")").to_string());
    }

    fn delete_coalesce(&mut self, _other: &Node) -> bool {
        false
    }

    fn delete(&self, ctx: &mut PgMigrateCtx) {
        ctx.statements.push(Tokens::new().s("drop index").id(&self.id.1).to_string());
    }
}

impl NodeData for NodeTableIndex_ {
    fn update(&self, _ctx: &mut PgMigrateCtx, _old: &Self) {
        unreachable!()
    }
}

#[enum_dispatch]
trait NodeDataDispatch {
    fn create_coalesce(&mut self, other: &Node) -> bool;
    fn create(&self, ctx: &mut PgMigrateCtx);
    fn delete_coalesce(&mut self, other: &Node) -> bool;
    fn delete(&self, ctx: &mut PgMigrateCtx);
}

trait NodeData: NodeDataDispatch {
    fn update(&self, ctx: &mut PgMigrateCtx, old: &Self);
}

#[derive(Clone)]
#[enum_dispatch(NodeDataDispatch)]
#[samevariant(PairwiseNode)]
pub enum Node {
    Table(NodeTable_),
    Field(NodeField_),
    TableConstraint(NodeTableConstraint_),
    TableIndex(NodeTableIndex_),
}

impl crate::graphmigrate::NodeData for Node {
    type O = PgMigrateCtx<'static>;

    fn compare(&self, other: &Self) -> Comparison {
        match PairwiseNode::pairs(self, other) {
            PairwiseNode::Table(_, _) => Comparison::DoNothing,
            PairwiseNode::Field(current, old) => if current.def.type_ == old.def.type_ {
                Comparison::DoNothing
            } else {
                Comparison::Update
            },
            PairwiseNode::TableConstraint(current, old) => {
                if current.def.type_ == old.def.type_ {
                    Comparison::DoNothing
                } else {
                    Comparison::DeleteCreate
                }
            },
            PairwiseNode::TableIndex(current, old) => {
                if current.def.field_ids == old.def.field_ids {
                    Comparison::DoNothing
                } else {
                    Comparison::DeleteCreate
                }
            },
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
        match PairwiseNode::pairs(self, old) {
            PairwiseNode::Table(current, old) => current.update(ctx, &old),
            PairwiseNode::Field(current, old) => current.update(ctx, &old),
            PairwiseNode::TableConstraint(current, old) => current.update(ctx, &old),
            PairwiseNode::TableIndex(current, old) => current.update(ctx, &old),
            PairwiseNode::Nonmatching(_, _) => unreachable!(),
        }
    }

    fn create_coalesce(&mut self, other: &Self) -> bool {
        NodeDataDispatch::create_coalesce(self, other)
    }

    fn delete_coalesce(&mut self, other: &Self) -> bool {
        NodeDataDispatch::delete_coalesce(self, other)
    }
}
