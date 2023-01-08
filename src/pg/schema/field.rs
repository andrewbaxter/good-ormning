use std::{
    fmt::{
        Display,
        Debug,
    },
    collections::HashMap,
};
use crate::{
    utils::Tokens,
    pg::{
        types::{
            to_sql_type,
            SimpleSimpleType,
            SimpleType,
            Type,
        },
        queries::{
            expr::{
                Expr,
                check_same,
                ExprType,
                ExprValName,
            },
            utils::PgQueryCtx,
        },
    },
    graphmigrate::Comparison,
};
use super::{
    table::TableId,
    utils::{
        NodeData,
        PgMigrateCtx,
        NodeDataDispatch,
    },
    node::Node_,
};

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct FieldId(pub TableId, pub String);

impl Display for FieldId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&format!("{}.{}", self.0, self.1), f)
    }
}

#[derive(Clone, Debug)]
pub enum FieldType {
    NonOpt(SimpleType, Option<Expr>),
    Opt(SimpleType),
}

pub struct FieldBuilder {
    t: SimpleSimpleType,
    default_: Option<Expr>,
    opt: bool,
    custom: Option<String>,
}

impl FieldBuilder {
    fn new(t: SimpleSimpleType) -> FieldBuilder {
        FieldBuilder {
            t: t,
            opt: false,
            default_: None,
            custom: None,
        }
    }

    pub fn opt(mut self) -> FieldBuilder {
        self.default_ = None;
        self
    }

    pub fn migrate_default(mut self, expr: Expr) -> FieldBuilder {
        if self.opt {
            panic!("Optional fields can't have defaults.");
        }
        self.default_ = Some(expr);
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
        if self.opt {
            FieldType::Opt(t)
        } else {
            FieldType::NonOpt(t, self.default_)
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

#[derive(Clone)]
pub struct FieldDef {
    pub name: String,
    pub type_: FieldType,
}

#[derive(Clone)]
pub(crate) struct NodeField_ {
    pub id: FieldId,
    pub def: FieldDef,
}

impl NodeField_ {
    pub fn compare(&self, other: &Self) -> Comparison {
        match (&self.def.type_, &other.def.type_) {
            (FieldType::NonOpt(t, _), FieldType::NonOpt(old_t, _)) => if t != old_t {
                Comparison::Update
            } else {
                Comparison::DoNothing
            },
            (FieldType::NonOpt(_, _), FieldType::Opt(_)) => Comparison::Update,
            (FieldType::Opt(_), FieldType::NonOpt(_, _)) => Comparison::Update,
            (FieldType::Opt(t), FieldType::Opt(old_t)) => if t != old_t {
                Comparison::Update
            } else {
                Comparison::DoNothing
            },
        }
    }
}

impl NodeData for NodeField_ {
    fn update(&self, ctx: &mut PgMigrateCtx, old: &Self) {
        let new_t = match (&self.def.type_, &old.def.type_) {
            (FieldType::NonOpt(t, _), FieldType::NonOpt(old_t, _)) => {
                if t != old_t {
                    Some(t)
                } else {
                    None
                }
            },
            (FieldType::NonOpt(t, _), FieldType::Opt(old_t)) => {
                if t != old_t {
                    Some(t)
                } else {
                    None
                }
            },
            (FieldType::Opt(t), FieldType::NonOpt(old_t, _)) => {
                if t != old_t {
                    Some(t)
                } else {
                    None
                }
            },
            (FieldType::Opt(t), FieldType::Opt(_)) => {
                Some(t)
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
        if let FieldType::NonOpt(t, d) = &self.def.type_ {
            if let Some(e) = d {
                stmt.s("not null default");
                let qctx_fields = HashMap::new();
                let mut qctx = PgQueryCtx::new(&mut ctx.errs, &qctx_fields);
                let e_res = e.build(&mut qctx, &HashMap::new());
                check_same(&mut qctx.errs, &ExprType(vec![(ExprValName {
                    table: "".into(),
                    field: "".into(),
                }, Type {
                    type_: t.clone(),
                    opt: false,
                })]), &e_res.0);
                if !qctx.rust_args.is_empty() {
                    qctx
                        .errs
                        .err(
                            format!(
                                "Default expressions must not have any parameters, but this has {} parameters",
                                qctx.rust_args.len()
                            ),
                        );
                }
                stmt.s(&e_res.1.to_string());
            } else {
                ctx.errs.err(format!("New column missing default"));
            }
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
