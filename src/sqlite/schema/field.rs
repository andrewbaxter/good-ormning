use std::{
    fmt::{
        Display,
        Debug,
    },
    collections::{
        HashMap,
        HashSet,
    },
};
use crate::{
    utils::Tokens,
    sqlite::{
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
            utils::SqliteQueryCtx,
        },
    },
    graphmigrate::Comparison,
};
use super::{
    table::TableId,
    utils::{
        SqliteNodeData,
        SqliteMigrateCtx,
        SqliteNodeDataDispatch,
    },
    node::{
        Node,
        Id,
    },
};

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct FieldId(pub TableId, pub String);

impl Display for FieldId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&format!("{}.{}", self.0, self.1), f)
    }
}

#[derive(Clone, Debug)]
pub struct FieldType {
    pub type_: Type,
    pub migration_default: Option<Expr>,
}

impl FieldType {
    /// Create a field type from the specified value type.
    pub fn with(t: &Type) -> Self {
        Self {
            type_: t.clone(),
            migration_default: None,
        }
    }

    /// Create a field type from the specified value type, and provide a migration fill value.
    pub fn with_migration(t: &Type, def: Option<Expr>) -> Self {
        if t.opt {
            panic!("Optional fields can't have defaults.");
        }
        Self {
            type_: t.clone(),
            migration_default: def,
        }
    }
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

    /// Make the field optional.
    pub fn opt(mut self) -> FieldBuilder {
        if self.default_.is_some() {
            panic!("Optional fields can't have migration fill expressions.");
        }
        self.opt = true;
        self
    }

    /// Specify an expression to use to populate the new column in existing rows. This is must
    /// be specified (only) for non-opt fields in a new version of an existing table.
    pub fn migrate_fill(mut self, expr: Expr) -> FieldBuilder {
        if self.opt {
            panic!("Optional fields can't have migration fill expressions.");
        }
        self.default_ = Some(expr);
        self
    }

    /// Use a custom Rust type for this field. This must be the full path to the type, like
    /// `crate::abcdef::MyType`.
    pub fn custom(mut self, type_: impl ToString) -> FieldBuilder {
        self.custom = Some(type_.to_string());
        self
    }

    pub fn build(self) -> FieldType {
        FieldType {
            type_: Type {
                type_: SimpleType {
                    custom: self.custom,
                    type_: self.t,
                },
                opt: self.opt,
            },
            migration_default: self.default_,
        }
    }
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

pub fn field_utctime_s() -> FieldBuilder {
    FieldBuilder::new(SimpleSimpleType::UtcTimeS)
}

pub fn field_utctime_ms() -> FieldBuilder {
    FieldBuilder::new(SimpleSimpleType::UtcTimeMs)
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
    pub fn compare(&self, other: &Self, created: &HashSet<Id>) -> Comparison {
        if created.contains(&Id::Table(self.id.0.clone())) {
            return Comparison::Recreate;
        }
        let t = &self.def.type_.type_;
        let other_t = &other.def.type_.type_;
        if t.opt != other_t.opt || t.type_.type_ != other_t.type_.type_ {
            Comparison::Update
        } else {
            Comparison::DoNothing
        }
    }

    fn path(&self) -> rpds::Vector<String> {
        rpds::vector![format!("{} ({})", self.def.name, self.id)]
    }
}

impl SqliteNodeData for NodeField_ {
    fn update(&self, ctx: &mut SqliteMigrateCtx, old: &Self) {
        let t = &self.def.type_.type_;
        let old_t = &old.def.type_.type_;
        if t.type_.type_ != old_t.type_.type_ {
            ctx.errs.err(&self.path(), format!("Column types cannot be changed in sqlite"));
        }
    }
}

impl SqliteNodeDataDispatch for NodeField_ {
    fn create(&self, ctx: &mut SqliteMigrateCtx) {
        let path = self.path();
        if &self.id.1 == "rowid" {
            return;
        }
        let mut stmt = Tokens::new();
        stmt
            .s("alter table")
            .id(&self.id.0.at(ctx.version))
            .s("add column")
            .id(&self.id.1)
            .s(to_sql_type(&self.def.type_.type_.type_.type_));
        if !self.def.type_.type_.opt {
            if let Some(d) = &self.def.type_.migration_default {
                stmt.s("not null default");
                let qctx_fields = HashMap::new();
                let mut qctx = SqliteQueryCtx::new(ctx.errs.clone(), ctx.version, &qctx_fields);
                let e_res = d.build(&mut qctx, &path, &HashMap::new());
                check_same(&mut qctx.errs, &path, &ExprType(vec![(ExprValName::empty(), Type {
                    type_: self.def.type_.type_.type_.clone(),
                    opt: false,
                })]), &e_res.0);
                if !qctx.rust_args.is_empty() {
                    qctx
                        .errs
                        .err(
                            &path,
                            format!(
                                "Default expressions must not have any parameters, but this has {} parameters",
                                qctx.rust_args.len()
                            ),
                        );
                }
                stmt.s(&e_res.1.to_string());
            } else {
                ctx.errs.err(&path, format!("New column missing default"));
            }
        }
        ctx.statements.push(stmt.to_string());
    }

    fn delete(&self, ctx: &mut SqliteMigrateCtx) {
        if &self.id.1 == "rowid" {
            return;
        }
        ctx
            .statements
            .push(
                Tokens::new()
                    .s("alter table")
                    .id(&self.id.0.at(ctx.version - 1))
                    .s("drop column")
                    .id(&self.id.1)
                    .to_string(),
            );
    }

    fn create_coalesce(&mut self, other: Node) -> Option<Node> {
        Some(other)
    }

    fn delete_coalesce(&mut self, other: Node) -> Option<Node> {
        Some(other)
    }
}
