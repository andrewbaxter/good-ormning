use std::{
    fmt::{
        Display,
        Debug,
    },
    marker::PhantomData,
};
use enum_dispatch::enum_dispatch;
use samevariant::samevariant;
use crate::{
    utils::{
        Tokens,
        Errs,
    },
    graphmigrate::Comparison,
};
use super::{
    types::{
        SimpleType,
        to_sql_type,
        SimpleSimpleType,
    },
};

pub(crate) struct PgMigrateCtx<'a> {
    pub(crate) errs: &'a mut Errs,
    pub statements: Vec<String>,
}

impl<'a> PgMigrateCtx<'a> {
    pub fn new(errs: &'a mut Errs) -> Self {
        Self {
            errs: errs,
            statements: Default::default(),
        }
    }
}

pub(crate) type MigrateNode<'a> = crate::graphmigrate::Node<Node<'a>, Id>;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum FieldType {
    NonOpt(SimpleType, String),
    Opt(SimpleType),
}

pub struct FieldBuilder {
    t: SimpleSimpleType,
    default_: Option<String>,
    custom: Option<String>,
}

impl FieldBuilder {
    fn new(t: SimpleSimpleType) -> FieldBuilder {
        FieldBuilder {
            t: t,
            default_: Some("".into()),
            custom: None,
        }
    }

    pub fn opt(mut self) -> FieldBuilder {
        self.default_ = None;
        self
    }

    pub fn migrate_default(mut self, text: impl ToString) -> FieldBuilder {
        self.default_ = Some(text.to_string());
        self
    }

    pub fn custom(mut self, type_: impl ToString) -> FieldBuilder {
        self.custom = Some(type_.to_string());
        self
    }

    pub fn build(self) -> FieldType {
        let t = SimpleType {
            custom: self.custom,
            type_: self.t,
        };
        if let Some(d) = self.default_ {
            FieldType::NonOpt(t, d)
        } else {
            FieldType::Opt(t)
        }
    }
}

pub fn field_auto() -> FieldBuilder {
    FieldBuilder::new(SimpleSimpleType::Auto)
}

pub fn field_bool() -> FieldBuilder {
    FieldBuilder::new(SimpleSimpleType::Bool)
}

pub fn field_i32() -> FieldBuilder {
    FieldBuilder::new(SimpleSimpleType::I32)
}

pub fn field_i64() -> FieldBuilder {
    FieldBuilder::new(SimpleSimpleType::I64)
}

pub fn field_u32() -> FieldBuilder {
    FieldBuilder::new(SimpleSimpleType::U32)
}

pub fn field_u64() -> FieldBuilder {
    FieldBuilder::new(SimpleSimpleType::U64)
}

pub fn field_f32() -> FieldBuilder {
    FieldBuilder::new(SimpleSimpleType::F32)
}

pub fn field_f64() -> FieldBuilder {
    FieldBuilder::new(SimpleSimpleType::F64)
}

pub fn field_str() -> FieldBuilder {
    FieldBuilder::new(SimpleSimpleType::String)
}

pub fn field_bytes() -> FieldBuilder {
    FieldBuilder::new(SimpleSimpleType::Bytes)
}

pub fn field_localtime() -> FieldBuilder {
    FieldBuilder::new(SimpleSimpleType::LocalTime)
}

pub fn field_utctime() -> FieldBuilder {
    FieldBuilder::new(SimpleSimpleType::UtcTime)
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

#[enum_dispatch]
trait NodeDataDispatch {
    fn create_coalesce(&mut self, other: &Node_) -> bool;
    fn create(&self, ctx: &mut PgMigrateCtx);
    fn delete_coalesce(&mut self, other: &Node_) -> bool;
    fn delete(&self, ctx: &mut PgMigrateCtx);
}

trait NodeData: NodeDataDispatch {
    fn update(&self, ctx: &mut PgMigrateCtx, old: &Self);
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
    fn create_coalesce(&mut self, other: &Node_) -> bool {
        match other {
            Node_::Field(f) if f.id.0 == self.id => {
                self.fields.push((f.id.clone(), f.def.clone()));
                true
            },
            _ => false,
        }
    }

    fn delete_coalesce(&mut self, other: &Node_) -> bool {
        match other {
            Node_::Field(f) if f.id.0 == self.id => true,
            Node_::Constraint(e) if e.id.0 == self.id => true,
            Node_::Index(e) if e.id.0 == self.id => true,
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
pub struct IndexDef {
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
        let (new_t, new_def) = match (&self.def.type_, &old.def.type_) {
            (FieldType::NonOpt(t, d), FieldType::NonOpt(old_t, old_d)) => {
                (if t != old_t {
                    Some(t)
                } else {
                    None
                }, if d != old_d {
                    Some(Some(d))
                } else {
                    None
                })
            },
            (FieldType::NonOpt(t, def), FieldType::Opt(old_t)) => {
                (if t != old_t {
                    Some(t)
                } else {
                    None
                }, Some(Some(def)))
            },
            (FieldType::Opt(t), FieldType::NonOpt(old_t, _)) => {
                (if t != old_t {
                    Some(t)
                } else {
                    None
                }, Some(None))
            },
            (FieldType::Opt(t), FieldType::Opt(_)) => {
                (Some(t), None)
            },
        };
        if let Some(new_t) = new_t {
            ctx
                .statements
                .push(
                    Tokens::new()
                        .s("alter table")
                        .id(&self.id.0.0)
                        .s("alter column")
                        .id(&self.id.1)
                        .s("set type")
                        .s(to_sql_type(new_t))
                        .to_string(),
                );
        }
        if let Some(new_def) = new_def {
            match new_def {
                Some(def) => {
                    ctx
                        .statements
                        .push(
                            Tokens::new()
                                .s("alter table")
                                .id(&self.id.0.0)
                                .s("alter column")
                                .id(&self.id.1)
                                .s("set default")
                                .s(&def)
                                .to_string(),
                        );
                },
                None => {
                    ctx
                        .statements
                        .push(
                            Tokens::new()
                                .s("alter table")
                                .id(&self.id.0.0)
                                .s("alter column")
                                .id(&self.id.1)
                                .s("unset default")
                                .to_string(),
                        );
                },
            }
        }
    }
}

impl NodeDataDispatch for NodeField_ {
    fn create(&self, ctx: &mut PgMigrateCtx) {
        if matches!(self.def.type_.simple().type_, SimpleSimpleType::Auto) {
            ctx.errs.err(format!("Auto (serial) fields can't be added after table creation"));
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

    fn create_coalesce(&mut self, _other: &Node_) -> bool {
        false
    }

    fn delete_coalesce(&mut self, _other: &Node_) -> bool {
        false
    }
}

#[derive(Clone)]
pub(crate) struct NodeConstraint_ {
    pub id: TableConstraintId,
    pub def: TableConstraintDef,
}

impl NodeDataDispatch for NodeConstraint_ {
    fn create_coalesce(&mut self, _other: &Node_) -> bool {
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

    fn delete_coalesce(&mut self, _other: &Node_) -> bool {
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

impl NodeData for NodeConstraint_ {
    fn update(&self, _ctx: &mut PgMigrateCtx, _old: &Self) {
        unreachable!()
    }
}

#[derive(Clone)]
pub(crate) struct NodeIndex_ {
    pub id: TableIndexId,
    pub def: IndexDef,
}

impl NodeDataDispatch for NodeIndex_ {
    fn create_coalesce(&mut self, _other: &Node_) -> bool {
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

    fn delete_coalesce(&mut self, _other: &Node_) -> bool {
        false
    }

    fn delete(&self, ctx: &mut PgMigrateCtx) {
        ctx.statements.push(Tokens::new().s("drop index").id(&self.id.1).to_string());
    }
}

impl NodeData for NodeIndex_ {
    fn update(&self, _ctx: &mut PgMigrateCtx, _old: &Self) {
        unreachable!()
    }
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
            PairwiseNode::Table(_, _) => Comparison::DoNothing,
            PairwiseNode::Field(current, old) => if current.def.type_ == old.def.type_ {
                Comparison::DoNothing
            } else {
                Comparison::Update
            },
            PairwiseNode::Constraint(current, old) => {
                if current.def.type_ == old.def.type_ {
                    Comparison::DoNothing
                } else {
                    Comparison::DeleteCreate
                }
            },
            PairwiseNode::Index(current, old) => {
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
