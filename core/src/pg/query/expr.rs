use serde::{
    Serialize,
    Deserialize,
};
#[cfg(feature = "chrono")]
use chrono::FixedOffset;
use {
    quote::{
        ToTokens,
        format_ident,
        quote,
    },
    std::{
        collections::HashMap,
        fmt::Display,
        rc::Rc,
    },
};
#[cfg(feature = "chrono")]
use chrono::{
    DateTime,
    Utc,
};
use crate::{
    pg::{
        types::{
            Type,
            to_rust_types,
            SimpleSimpleType,
            SimpleType,
        },
        query::utils::{
            PgQueryCtx,
            QueryBody,
        },
        schema::{
            field::FieldRef,
            table::TableRef,
        },
    },
    utils::{
        Tokens,
        sanitize_ident,
        Errs,
    },
};
#[cfg(feature = "jiff")]
use jiff::Timestamp;
use super::select::{
    Select,
    Order,
};

#[derive(Clone)]
pub struct ExprType(pub Vec<(ExprValName, Type)>);

impl ExprType {
    pub fn assert_scalar(&self, errs: &mut Errs, path: &rpds::Vector<String>) -> Option<Type> {
        if self.0.len() != 1 {
            errs.err(
                path,
                format!("Select outputs must be scalars, but got result with more than one field: {}", self.0.len()),
            );
            return None;
        }
        Some(self.0[0].1.clone())
    }
}

pub struct ComputeType(pub Rc<dyn Fn(&mut PgQueryCtx, &rpds::Vector<String>, &[ExprType]) -> ExprType>);

impl Clone for ComputeType {
    fn clone(&self) -> Self {
        return ComputeType(self.0.clone());
    }
}

impl std::fmt::Debug for ComputeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return f.write_str("ComputeType");
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SerialWindowFrameType {
    Rows,
    Range,
    Groups,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SerialWindowFrameBound {
    UnboundedPreceding,
    Preceding(Box<SerialExpr>),
    CurrentRow,
    Following(Box<SerialExpr>),
    UnboundedFollowing,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SerialWindowFrameExclude {
    CurrentRow,
    Group,
    Ties,
    NoOthers,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SerialWindowFrame {
    pub type_: SerialWindowFrameType,
    pub start: SerialWindowFrameBound,
    pub end: Option<SerialWindowFrameBound>,
    pub exclude: Option<SerialWindowFrameExclude>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SerialExpr {
    LitArray(Vec<SerialExpr>),
    LitNull(SimpleType),
    LitBool(bool),
    LitAuto(i64),
    LitI16(i16),
    LitI32(i32),
    LitI64(i64),
    LitU32(u32),
    LitF32(f32),
    LitF64(f64),
    LitString(String),
    LitBytes(Vec<u8>),
    #[cfg(feature = "chrono")]
    LitUtcTimeChrono(DateTime<Utc>),
    #[cfg(feature = "chrono")]
    LitFixedOffsetTimeChrono(DateTime<FixedOffset>),
    #[cfg(feature = "jiff")]
    LitUtcTimeJiff(Timestamp),
    BinOp {
        left: Box<SerialExpr>,
        op: BinOp,
        right: Box<SerialExpr>,
    },
    BinOpChain {
        op: BinOp,
        exprs: Vec<SerialExpr>,
    },
    PrefixOp {
        op: PrefixOp,
        right: Box<SerialExpr>,
    },
    Cast(Box<SerialExpr>, Type),
    Collate(Box<SerialExpr>, String),
    Like {
        expr: Box<SerialExpr>,
        pattern: Box<SerialExpr>,
        escape: Option<Box<SerialExpr>>,
        ilike: bool,
    },
    Between {
        e: Box<SerialExpr>,
        negated: bool,
        low: Box<SerialExpr>,
        high: Box<SerialExpr>,
    },
}

impl From<SerialWindowFrameType> for WindowFrameType {
    fn from(s: SerialWindowFrameType) -> Self {
        match s {
            SerialWindowFrameType::Rows => WindowFrameType::Rows,
            SerialWindowFrameType::Range => WindowFrameType::Range,
            SerialWindowFrameType::Groups => WindowFrameType::Groups,
        }
    }
}

impl From<SerialWindowFrameBound> for WindowFrameBound {
    fn from(s: SerialWindowFrameBound) -> Self {
        match s {
            SerialWindowFrameBound::UnboundedPreceding => WindowFrameBound::UnboundedPreceding,
            SerialWindowFrameBound::Preceding(e) => WindowFrameBound::Preceding(Box::new(Expr::from(*e))),
            SerialWindowFrameBound::CurrentRow => WindowFrameBound::CurrentRow,
            SerialWindowFrameBound::Following(e) => WindowFrameBound::Following(Box::new(Expr::from(*e))),
            SerialWindowFrameBound::UnboundedFollowing => WindowFrameBound::UnboundedFollowing,
        }
    }
}

impl From<SerialWindowFrameExclude> for WindowFrameExclude {
    fn from(s: SerialWindowFrameExclude) -> Self {
        match s {
            SerialWindowFrameExclude::CurrentRow => WindowFrameExclude::CurrentRow,
            SerialWindowFrameExclude::Group => WindowFrameExclude::Group,
            SerialWindowFrameExclude::Ties => WindowFrameExclude::Ties,
            SerialWindowFrameExclude::NoOthers => WindowFrameExclude::NoOthers,
        }
    }
}

impl From<SerialWindowFrame> for WindowFrame {
    fn from(s: SerialWindowFrame) -> Self {
        WindowFrame {
            type_: WindowFrameType::from(s.type_),
            start: WindowFrameBound::from(s.start),
            end: s.end.map(WindowFrameBound::from),
            exclude: s.exclude.map(WindowFrameExclude::from),
        }
    }
}

impl From<SerialExpr> for Expr {
    fn from(s: SerialExpr) -> Self {
        match s {
            SerialExpr::LitArray(v) => Expr::LitArray(v.into_iter().map(Expr::from).collect()),
            SerialExpr::LitNull(t) => Expr::LitNull(t),
            SerialExpr::LitBool(b) => Expr::LitBool(b),
            SerialExpr::LitAuto(v) => Expr::LitAuto(v),
            SerialExpr::LitI16(v) => Expr::LitI16(v),
            SerialExpr::LitI32(v) => Expr::LitI32(v),
            SerialExpr::LitI64(v) => Expr::LitI64(v),
            SerialExpr::LitU32(v) => Expr::LitU32(v),
            SerialExpr::LitF32(v) => Expr::LitF32(v),
            SerialExpr::LitF64(v) => Expr::LitF64(v),
            SerialExpr::LitString(v) => Expr::LitString(v),
            SerialExpr::LitBytes(v) => Expr::LitBytes(v),
            #[cfg(feature = "chrono")]
            SerialExpr::LitUtcTimeChrono(v) => Expr::LitUtcTimeChrono(v),
            #[cfg(feature = "chrono")]
            SerialExpr::LitFixedOffsetTimeChrono(v) => Expr::LitFixedOffsetTimeChrono(v),
            #[cfg(feature = "jiff")]
            SerialExpr::LitUtcTimeJiff(v) => Expr::LitUtcTimeJiff(v),
            SerialExpr::BinOp { left, op, right } => Expr::BinOp {
                left: Box::new(Expr::from(*left)),
                op: op,
                right: Box::new(Expr::from(*right)),
            },
            SerialExpr::BinOpChain { op, exprs } => Expr::BinOpChain {
                op: op,
                exprs: exprs.into_iter().map(Expr::from).collect(),
            },
            SerialExpr::PrefixOp { op, right } => Expr::PrefixOp {
                op: op,
                right: Box::new(Expr::from(*right)),
            },
            SerialExpr::Cast(e, t) => Expr::Cast(Box::new(Expr::from(*e)), t),
            SerialExpr::Collate(e, s) => Expr::Collate(Box::new(Expr::from(*e)), s),
            SerialExpr::Like { expr, pattern, escape, ilike } => Expr::Like {
                expr: Box::new(Expr::from(*expr)),
                pattern: Box::new(Expr::from(*pattern)),
                escape: escape.map(|e| Box::new(Expr::from(*e))),
                ilike,
            },
            SerialExpr::Between { e, negated, low, high } => Expr::Between {
                e: Box::new(Expr::from(*e)),
                negated,
                low: Box::new(Expr::from(*low)),
                high: Box::new(Expr::from(*high)),
            },
        }
    }
}

#[derive(Clone, Debug)]
pub enum WindowFrameType {
    Rows,
    Range,
    Groups,
}

#[derive(Clone, Debug)]
pub enum WindowFrameBound {
    UnboundedPreceding,
    Preceding(Box<Expr>),
    CurrentRow,
    Following(Box<Expr>),
    UnboundedFollowing,
}

#[derive(Clone, Debug)]
pub enum WindowFrameExclude {
    CurrentRow,
    Group,
    Ties,
    NoOthers,
}

#[derive(Clone, Debug)]
pub struct WindowFrame {
    pub type_: WindowFrameType,
    pub start: WindowFrameBound,
    pub end: Option<WindowFrameBound>,
    pub exclude: Option<WindowFrameExclude>,
}

#[derive(Clone, Debug)]
pub enum Expr {
    LitArray(Vec<Expr>),
    // A null value needs a type for type checking purposes. It will always be trated
    // as an optional value.
    LitNull(SimpleType),
    LitBool(bool),
    LitAuto(i64),
    LitI16(i16),
    LitI32(i32),
    LitI64(i64),
    LitU32(u32),
    LitF32(f32),
    LitF64(f64),
    LitString(String),
    LitBytes(Vec<u8>),
    #[cfg(feature = "chrono")]
    LitUtcTimeChrono(DateTime<Utc>),
    #[cfg(feature = "chrono")]
    LitFixedOffsetTimeChrono(DateTime<FixedOffset>),
    #[cfg(feature = "jiff")]
    LitUtcTimeJiff(Timestamp),
    /// A query parameter. This will become a parameter to the generated Rust function
    /// with the specified `name` and `type_`.
    Param {
        name: String,
        type_: Type,
    },
    /// This evaluates to the value of a field in the query main or joined tables. If
    /// you've aliased tables or field names, you'll have to instantiate `FieldId`
    /// yourself with the appropriate values. For synthetic values like function
    /// results you may need a `FieldId` with an empty `TableId` (`""`).
    Field(FieldRef),
    BinOp {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },
    BinOpChain {
        op: BinOp,
        exprs: Vec<Expr>,
    },
    PrefixOp {
        op: PrefixOp,
        right: Box<Expr>,
    },
    Call {
        func: String,
        args: Vec<Expr>,
        compute_type: ComputeType,
        filter: Option<Box<Expr>>,
    },
    Window {
        expr: Box<Expr>,
        partition_by: Vec<Expr>,
        order_by: Vec<(Expr, Order)>,
        frame: Option<WindowFrame>,
    },
    /// A sub SELECT query.
    Select(Box<Select>),
    /// This is a synthetic expression, saying to treat the result of the expression as
    /// having the specified type. Use this for casting between primitive types and
    /// Rust new-types for instance.
    Cast(Box<Expr>, Type),
    Exists(Box<super::select::Select>),
    Collate(Box<Expr>, String),
    Like {
        expr: Box<Expr>,
        pattern: Box<Expr>,
        escape: Option<Box<Expr>>,
        ilike: bool,
    },
    Between {
        e: Box<Expr>,
        negated: bool,
        low: Box<Expr>,
        high: Box<Expr>,
    },
    Case {
        operand: Option<Box<Expr>>,
        conditions: Vec<(Expr, Expr)>,
        else_: Option<Box<Expr>>,
    },
}

#[derive(Clone, Hash, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct ExprValName {
    pub table_id: String,
    pub id: String,
}

impl ExprValName {
    pub fn local(name: String) -> Self {
        ExprValName {
            table_id: "".into(),
            id: name,
        }
    }

    pub fn empty() -> Self {
        ExprValName {
            table_id: "".into(),
            id: "".into(),
        }
    }

    pub fn field(f: &FieldRef) -> Self {
        ExprValName {
            table_id: f.table_id.clone(),
            id: f.field_id.clone(),
        }
    }

    pub fn with_alias(&self, s: &str) -> ExprValName {
        ExprValName {
            table_id: s.into(),
            id: self.id.clone(),
        }
    }
}

impl Display for ExprValName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.table_id.is_empty() {
            return Display::fmt(&self.id, f);
        } else {
            return Display::fmt(&format!("{}.{}", self.table_id, self.id), f);
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BinOp {
    Plus,
    Minus,
    Multiply,
    Divide,
    And,
    Or,
    Equals,
    NotEquals,
    Is,
    IsNot,
    LessThan,
    LessThanEqualTo,
    GreaterThan,
    GreaterThanEqualTo,
    In,
    NotIn,
    Like,
    ILike,
    StringConcat,
    Mod,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    BitwiseShiftLeft,
    BitwiseShiftRight,
    IsDistinctFrom,
    IsNotDistinctFrom,
    Glob,
    Regexp,
    Match,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PrefixOp {
    Not,
    BitwiseNot,
    Minus,
}

pub fn check_same(errs: &mut Errs, path: &rpds::Vector<String>, left: &ExprType, right: &ExprType) -> Option<Type> {
    let left = left.assert_scalar(errs, &path.push_back("Left".into()))?;
    let right = right.assert_scalar(errs, &path.push_back("Right".into()))?;
    if left.opt != right.opt {
        errs.err(
            path,
            format!("Operator arms optionality don't match: left has {} and right has {}", left.opt, right.opt),
        );
    }
    if left.type_.custom != right.type_.custom {
        errs.err(
            path,
            format!(
                "Operator arms custom types don't match: left has type {:?} and right has {:?}",
                left.type_.custom,
                right.type_.custom
            ),
        );
    }
    if left.type_.type_ != right.type_.type_ {
        errs.err(
            path,
            format!(
                "Operator arms types don't match: left has type {:?} and right has {:?}",
                left.type_.type_,
                right.type_.type_
            ),
        );
    }
    Some(left.clone())
}

pub fn check_bool(ctx: &mut PgQueryCtx, path: &rpds::Vector<String>, t: &ExprType) {
    let Some(t) = t.assert_scalar(&mut ctx.errs, path) else {
        return;
    };
    if t.opt {
        ctx.errs.err(path, "Expected non-optional bool but got optional bool".to_string());
    }
    if !matches!(t.type_.type_, SimpleSimpleType::Bool) {
        ctx.errs.err(path, format!("Expected bool but type is non-bool: got {:?}", t.type_.type_));
    }
}

pub fn check_assignable(errs: &mut Errs, path: &rpds::Vector<String>, left: &Type, right: &ExprType) {
    let Some(right) = right.assert_scalar(errs, path) else {
        return;
    };
    if left.type_.type_ != right.type_.type_ {
        errs.err(
            path,
            format!("Expression has type {:?} which is not assignable to {:?}", right.type_.type_, left.type_.type_),
        );
    }
    if !left.opt && right.opt {
        errs.err(path, "Expression is optional but destination is not".to_string());
    }
}

pub fn check_general_same(ctx: &mut PgQueryCtx, path: &rpds::Vector<String>, left: &ExprType, right: &ExprType) {
    if left.0.len() != right.0.len() {
        ctx
            .errs
            .err(
                path,
                format!(
                    "Operator arms record type lengths don't match: left has {} fields and right has {}",
                    left.0.len(),
                    right.0.len()
                ),
            );
    } else if left.0.len() == 1 && right.0.len() == 1 {
        check_general_same_type(ctx, path, &left.0[0].1, &right.0[0].1);
    } else {
        for (i, (left, right)) in left.0.iter().zip(right.0.iter()).enumerate() {
            check_general_same_type(ctx, &path.push_back(format!("Record pair {}", i)), &left.1, &right.1);
        }
    }
}

pub fn check_general_same_type(ctx: &mut PgQueryCtx, path: &rpds::Vector<String>, left: &Type, right: &Type) {
    if left.type_.type_ != right.type_.type_ {
        ctx
            .errs
            .err(
                path,
                format!(
                    "Operator arms types don't match: left has type {:?} and right has {:?}",
                    left.type_.type_,
                    right.type_.type_
                ),
            );
    }
}

impl WindowFrame {
    pub fn build(
        &self,
        ctx: &mut PgQueryCtx,
        path: &rpds::Vector<String>,
        scope: &HashMap<ExprValName, Type>,
    ) -> Tokens {
        let mut out = Tokens::new();
        match self.type_ {
            WindowFrameType::Rows => {
                out.s("rows");
            },
            WindowFrameType::Range => {
                out.s("range");
            },
            WindowFrameType::Groups => {
                out.s("groups");
            },
        }
        if let Some(end) = &self.end {
            out.s("between");
            out.s(&self.start.build(ctx, &path.push_back("Start".into()), scope).to_string());
            out.s("and");
            out.s(&end.build(ctx, &path.push_back("End".into()), scope).to_string());
        } else {
            out.s(&self.start.build(ctx, &path.push_back("Start".into()), scope).to_string());
        }
        if let Some(exclude) = &self.exclude {
            out.s("exclude");
            match exclude {
                WindowFrameExclude::CurrentRow => {
                    out.s("current row");
                },
                WindowFrameExclude::Group => {
                    out.s("group");
                },
                WindowFrameExclude::Ties => {
                    out.s("ties");
                },
                WindowFrameExclude::NoOthers => {
                    out.s("no others");
                },
            }
        }
        out
    }
}

impl WindowFrameBound {
    pub fn build(
        &self,
        ctx: &mut PgQueryCtx,
        path: &rpds::Vector<String>,
        scope: &HashMap<ExprValName, Type>,
    ) -> Tokens {
        let mut out = Tokens::new();
        match self {
            WindowFrameBound::UnboundedPreceding => {
                out.s("unbounded preceding");
            },
            WindowFrameBound::Preceding(e) => {
                let (_, tokens) = e.build(ctx, path, scope);
                out.s(&tokens.to_string()).s("preceding");
            },
            WindowFrameBound::CurrentRow => {
                out.s("current row");
            },
            WindowFrameBound::Following(e) => {
                let (_, tokens) = e.build(ctx, path, scope);
                out.s(&tokens.to_string()).s("following");
            },
            WindowFrameBound::UnboundedFollowing => {
                out.s("unbounded following");
            },
        }
        out
    }
}

impl Expr {
    pub fn build(
        &self,
        ctx: &mut PgQueryCtx,
        path: &rpds::Vector<String>,
        scope: &HashMap<ExprValName, Type>,
    ) -> (ExprType, Tokens) {
        match self {
            Expr::LitArray(v) => {
                let mut out = Tokens::new();
                let mut res_types = vec![];
                let is_in = matches!(ctx.op_stack.last(), Some(BinOp::In | BinOp::NotIn));
                if is_in {
                    out.s("(");
                } else {
                    out.s("array [");
                }
                for (i, e) in v.iter().enumerate() {
                    if i > 0 {
                        out.s(",");
                    }
                    let res = e.build(ctx, path, scope);
                    out.s(&res.1.to_string());
                    res_types.push(res.0);
                }
                if is_in {
                    out.s(")");
                } else {
                    out.s("]");
                }
                let mut out_type = None;
                for (i, t) in res_types.iter().enumerate() {
                    if let Some(prev) = &out_type {
                        check_general_same(ctx, &path.push_back(format!("Array element {}", i)), prev, t);
                    } else {
                        out_type = Some(t.clone());
                    }
                }
                return (out_type.unwrap_or(ExprType(vec![])), out);
            },
            Expr::LitNull(t) => {
                let mut out = Tokens::new();
                out.s("null");
                return (ExprType(vec![(ExprValName::empty(), Type {
                    type_: t.clone(),
                    opt: true,
                    arr: false,
                })]), out);
            },
            Expr::LitBool(b) => {
                let mut out = Tokens::new();
                out.s(if *b {
                    "true"
                } else {
                    "false"
                });
                return (ExprType(vec![(ExprValName::empty(), Type {
                    type_: SimpleType {
                        type_: SimpleSimpleType::Bool,
                        custom: None,
                    },
                    opt: false,
                    arr: false,
                })]), out);
            },
            Expr::LitAuto(v) => {
                let mut out = Tokens::new();
                out.s(&v.to_string());
                return (ExprType(vec![(ExprValName::empty(), Type {
                    type_: SimpleType {
                        type_: SimpleSimpleType::Auto,
                        custom: None,
                    },
                    opt: false,
                    arr: false,
                })]), out);
            },
            Expr::LitI16(v) => {
                let mut out = Tokens::new();
                out.s(&v.to_string());
                return (ExprType(vec![(ExprValName::empty(), Type {
                    type_: SimpleType {
                        type_: SimpleSimpleType::I16,
                        custom: None,
                    },
                    opt: false,
                    arr: false,
                })]), out);
            },
            Expr::LitI32(v) => {
                let mut out = Tokens::new();
                out.s(&v.to_string());
                return (ExprType(vec![(ExprValName::empty(), Type {
                    type_: SimpleType {
                        type_: SimpleSimpleType::I32,
                        custom: None,
                    },
                    opt: false,
                    arr: false,
                })]), out);
            },
            Expr::LitI64(v) => {
                let mut out = Tokens::new();
                out.s(&v.to_string());
                return (ExprType(vec![(ExprValName::empty(), Type {
                    type_: SimpleType {
                        type_: SimpleSimpleType::I64,
                        custom: None,
                    },
                    opt: false,
                    arr: false,
                })]), out);
            },
            Expr::LitU32(v) => {
                let mut out = Tokens::new();
                out.s(&v.to_string());
                return (ExprType(vec![(ExprValName::empty(), Type {
                    type_: SimpleType {
                        type_: SimpleSimpleType::U32,
                        custom: None,
                    },
                    opt: false,
                    arr: false,
                })]), out);
            },
            Expr::LitF32(v) => {
                let mut out = Tokens::new();
                out.s(&v.to_string());
                return (ExprType(vec![(ExprValName::empty(), Type {
                    type_: SimpleType {
                        type_: SimpleSimpleType::F32,
                        custom: None,
                    },
                    opt: false,
                    arr: false,
                })]), out);
            },
            Expr::LitF64(v) => {
                let mut out = Tokens::new();
                out.s(&v.to_string());
                return (ExprType(vec![(ExprValName::empty(), Type {
                    type_: SimpleType {
                        type_: SimpleSimpleType::F64,
                        custom: None,
                    },
                    opt: false,
                    arr: false,
                })]), out);
            },
            Expr::LitString(v) => {
                let mut out = Tokens::new();
                out.s(&format!("'{}'", v.replace("'", "''")));
                return (ExprType(vec![(ExprValName::empty(), Type {
                    type_: SimpleType {
                        type_: SimpleSimpleType::String,
                        custom: None,
                    },
                    opt: false,
                    arr: false,
                })]), out);
            },
            Expr::LitBytes(v) => {
                let mut out = Tokens::new();
                out.s(&format!("'\\x{}'", hex::encode(v)));
                return (ExprType(vec![(ExprValName::empty(), Type {
                    type_: SimpleType {
                        type_: SimpleSimpleType::Bytes,
                        custom: None,
                    },
                    opt: false,
                    arr: false,
                })]), out);
            },
            #[cfg(feature = "chrono")]
            Expr::LitUtcTimeChrono(v) => {
                let mut out = Tokens::new();
                out.s(&format!("'{}'", v.to_rfc3339()));
                return (ExprType(vec![(ExprValName::empty(), Type {
                    type_: SimpleType {
                        type_: SimpleSimpleType::UtcTimeSChrono,
                        custom: None,
                    },
                    opt: false,
                    arr: false,
                })]), out);
            },
            #[cfg(feature = "chrono")]
            Expr::LitFixedOffsetTimeChrono(v) => {
                let mut out = Tokens::new();
                out.s(&format!("'{}'", v.to_rfc3339()));
                return (ExprType(vec![(ExprValName::empty(), Type {
                    type_: SimpleType {
                        type_: SimpleSimpleType::FixedOffsetTimeChrono,
                        custom: None,
                    },
                    opt: false,
                    arr: false,
                })]), out);
            },
            #[cfg(feature = "jiff")]
            Expr::LitUtcTimeJiff(v) => {
                let mut out = Tokens::new();
                out.s(&format!("'{}'", v.to_string()));
                return (ExprType(vec![(ExprValName::empty(), Type {
                    type_: SimpleType {
                        type_: SimpleSimpleType::UtcTimeSJiff,
                        custom: None,
                    },
                    opt: false,
                    arr: false,
                })]), out);
            },
            Expr::Param { name, type_ } => {
                let mut out = Tokens::new();
                let path = path.push_back(format!("Param ({})", name));
                let i = match ctx.rust_arg_lookup.entry(name.clone()) {
                    std::collections::hash_map::Entry::Occupied(e) => {
                        let (i, prev_t) = e.get();
                        if type_ != prev_t {
                            ctx
                                .errs
                                .err(
                                    &path,
                                    format!(
                                        "Parameter {} specified with multiple types: {:?}, {:?}",
                                        name,
                                        type_,
                                        prev_t
                                    ),
                                );
                        }
                        *i
                    },
                    std::collections::hash_map::Entry::Vacant(e) => {
                        let i = ctx.rust_args.len() + 1;
                        e.insert((i, type_.clone()));
                        let rust_types = to_rust_types(&type_.type_.type_);
                        let custom_trait_ident = rust_types.custom_trait;
                        let rust_type = rust_types.arg_type;
                        let ident = format_ident!("{}", sanitize_ident(name).1);
                        let (mut rust_type, mut rust_forward) = if let Some(custom) = &type_.type_.custom {
                            let custom_ident = match syn::parse_str::<syn::Path>(custom.as_str()) {
                                Ok(p) => p,
                                Err(e) => {
                                    ctx.errs.err(&path, format!("Couldn't parse custom type {}: {:?}", custom, e));
                                    return (ExprType(vec![]), Tokens::new());
                                },
                            }.to_token_stream();
                            let forward =
                                quote!(< #custom_ident as #custom_trait_ident < #custom_ident >>:: to_sql(& #ident));
                            (quote!(& #custom_ident), forward)
                        } else {
                            (rust_type, quote!(#ident))
                        };
                        if type_.opt {
                            rust_type = quote!(Option < #rust_type >);
                            rust_forward = quote!(#ident.map(| #ident | #rust_forward));
                        }
                        if type_.arr {
                            rust_type = quote!(Vec < #rust_type >);
                        }
                        ctx.rust_args.push(quote!(#ident: #rust_type));
                        ctx.query_args.push(quote!(#rust_forward));
                        i
                    },
                };
                out.s(&format!("${}", i));
                return (ExprType(vec![(ExprValName::local(name.clone()), type_.clone())]), out);
            },
            Expr::Field(x) => {
                let name = ExprValName::field(x);
                let t = match scope.get(&name) {
                    Some(t) => t.clone(),
                    None => {
                        if x.table_id.is_empty() {
                            let mut found = vec![];
                            for (k, v) in scope {
                                if k.id == x.field_id {
                                    found.push(v.clone());
                                }
                            }
                            if found.len() == 1 {
                                found[0].clone()
                            } else if found.len() > 1 {
                                ctx
                                    .errs
                                    .err(
                                        path,
                                        format!("Field {:?} is ambiguous (found in multiple tables)", x.field_id),
                                    );
                                return (ExprType(vec![]), Tokens::new());
                            } else {
                                ctx.errs.err(path, format!("Field {:?} not found in any table", x.field_id));
                                return (ExprType(vec![]), Tokens::new());
                            }
                        } else {
                            ctx
                                .errs
                                .err(
                                    path,
                                    format!(
                                        "Expression references {:?} but this field isn't available here (available fields: {:?})",
                                        x,
                                        scope.iter().map(|e| e.0.to_string()).collect::<Vec<String>>()
                                    ),
                                );
                            return (ExprType(vec![]), Tokens::new());
                        }
                    },
                };
                let mut out = Tokens::new();
                if x.table_id.is_empty() {
                    out.id(&x.field_id);
                } else {
                    let table_info =
                        ctx
                            .tables
                            .get(&TableRef(x.table_id.clone()))
                            .unwrap_or_else(|| panic!("Table {:?} not found in context", x.table_id));
                    let field_info =
                        table_info
                            .fields
                            .get(x)
                            .unwrap_or_else(|| panic!("Field {:?} not found in table {:?}", x.field_id, x.table_id));
                    out.id(&table_info.sql_name).s(".").id(&field_info.sql_name);
                }
                return (ExprType(vec![(name, t.clone())]), out);
            },
            Expr::BinOp { left, op, right } => {
                let mut out = Tokens::new();
                let token;
                match op {
                    BinOp::In | BinOp::NotIn => {
                        let token = match op {
                            BinOp::In => "in",
                            BinOp::NotIn => "not in",
                            _ => unreachable!(),
                        };
                        ctx.op_stack.push(op.clone());
                        let (left_t, left_tokens) = left.build(ctx, &path.push_back("Operand 0".into()), scope);
                        let (right_t, right_tokens) = right.build(ctx, &path.push_back("Operand 1".into()), scope);
                        ctx.op_stack.pop();
                        if !left_t.0.is_empty() && !right_t.0.is_empty() {
                            if right_t.0.len() == left_t.0.len() {
                                check_general_same(ctx, path, &left_t, &right_t);
                            } else if !left_t.0.is_empty() && right_t.0.len() % left_t.0.len() == 0 {
                                for i in 0 .. (right_t.0.len() / left_t.0.len()) {
                                    let start = i * left_t.0.len();
                                    let end = start + left_t.0.len();
                                    let sub_right_t = ExprType(right_t.0[start .. end].to_vec());
                                    check_general_same(
                                        ctx,
                                        &path.push_back(format!("Right set element {}", i)),
                                        &left_t,
                                        &sub_right_t,
                                    );
                                }
                            } else {
                                ctx
                                    .errs
                                    .err(
                                        path,
                                        format!(
                                            "Operator {:?} arms record type lengths don't match: left has {} fields and right has {}",
                                            op,
                                            left_t.0.len(),
                                            right_t.0.len()
                                        ),
                                    );
                            }
                        }
                        out.s(&left_tokens.to_string()).s(token).s(&right_tokens.to_string());
                        return (ExprType(vec![(ExprValName::empty(), Type {
                            type_: SimpleType {
                                type_: SimpleSimpleType::Bool,
                                custom: None,
                            },
                            opt: false,
                            arr: false,
                        })]), out);
                    },
                    BinOp::Plus => {
                        token = "+";
                    },
                    BinOp::Minus => {
                        token = "-";
                    },
                    BinOp::Multiply => {
                        token = "*";
                    },
                    BinOp::Divide => {
                        token = "/";
                    },
                    BinOp::And => {
                        token = "and";
                    },
                    BinOp::Or => {
                        token = "or";
                    },
                    BinOp::Equals => {
                        token = "=";
                    },
                    BinOp::NotEquals => {
                        token = "<>";
                    },
                    BinOp::Is => {
                        token = "is";
                    },
                    BinOp::IsNot => {
                        token = "is not";
                    },
                    BinOp::LessThan => {
                        token = "<";
                    },
                    BinOp::LessThanEqualTo => {
                        token = "<=";
                    },
                    BinOp::GreaterThan => {
                        token = ">";
                    },
                    BinOp::GreaterThanEqualTo => {
                        token = ">=";
                    },
                    BinOp::Like => {
                        token = "like";
                    },
                    BinOp::ILike => {
                        token = "ilike";
                    },
                    BinOp::StringConcat => {
                        token = "||";
                    },
                    BinOp::Mod => {
                        token = "%";
                    },
                    BinOp::BitwiseAnd => {
                        token = "&";
                    },
                    BinOp::BitwiseOr => {
                        token = "|";
                    },
                    BinOp::BitwiseXor => {
                        token = "#";
                    },
                    BinOp::BitwiseShiftLeft => {
                        token = "<<";
                    },
                    BinOp::BitwiseShiftRight => {
                        token = ">>";
                    },
                    BinOp::IsDistinctFrom => {
                        token = "is distinct from";
                    },
                    BinOp::IsNotDistinctFrom => {
                        token = "is not distinct from";
                    },
                    BinOp::Glob => {
                        token = "glob";
                    },
                    BinOp::Regexp => {
                        token = "regexp";
                    },
                    BinOp::Match => {
                        token = "match";
                    },
                }
                let l_res = left.build(ctx, &path.push_back("Bin op left".into()), scope);
                let r_res = right.build(ctx, &path.push_back("Bin op right".into()), scope);
                let t = check_same(&mut ctx.errs, path, &l_res.0, &r_res.0);
                out.s(&l_res.1.to_string()).s(token).s(&r_res.1.to_string());
                let mut res_t = t.unwrap_or(Type {
                    type_: SimpleType {
                        type_: SimpleSimpleType::I32,
                        custom: None,
                    },
                    opt: false,
                    arr: false,
                });
                match op {
                    BinOp::Equals |
                    BinOp::NotEquals |
                    BinOp::Is |
                    BinOp::IsNot |
                    BinOp::LessThan |
                    BinOp::LessThanEqualTo |
                    BinOp::GreaterThan |
                    BinOp::GreaterThanEqualTo |
                    BinOp::Like |
                    BinOp::ILike |
                    BinOp::IsDistinctFrom |
                    BinOp::IsNotDistinctFrom |
                    BinOp::Glob |
                    BinOp::Regexp |
                    BinOp::Match => {
                        res_t = Type {
                            type_: SimpleType {
                                type_: SimpleSimpleType::Bool,
                                custom: None,
                            },
                            opt: false,
                            arr: false,
                        };
                    },
                    BinOp::StringConcat => {
                        res_t = Type {
                            type_: SimpleType {
                                type_: SimpleSimpleType::String,
                                custom: None,
                            },
                            opt: false,
                            arr: false,
                        };
                    },
                    BinOp::In | BinOp::NotIn => unreachable!(),
                    _ => { },
                }
                return (ExprType(vec![(ExprValName::empty(), res_t)]), out);
            },
            Expr::BinOpChain { op, exprs } => {
                let mut out = Tokens::new();
                let token = match op {
                    BinOp::And => "and",
                    BinOp::Or => "or",
                    _ => panic!("Chain only supported for and/or"),
                };
                let mut out_t = None;
                for (i, e) in exprs.iter().enumerate() {
                    if i > 0 {
                        out.s(token);
                    }
                    let res = e.build(ctx, &path.push_back(format!("Chain element {}", i)), scope);
                    check_bool(ctx, &path.push_back(format!("Chain element {}", i)), &res.0);
                    out.s(&res.1.to_string());
                    out_t = Some(res.0);
                }
                return (out_t.unwrap_or(ExprType(vec![])), out);
            },
            Expr::PrefixOp { op, right } => {
                let mut out = Tokens::new();
                let token = match op {
                    PrefixOp::Not => "not",
                    PrefixOp::BitwiseNot => "~",
                    PrefixOp::Minus => "-",
                };
                let res = right.build(ctx, &path.push_back("Prefix op".into()), scope);
                if matches!(op, PrefixOp::Not) {
                    check_bool(ctx, path, &res.0);
                }
                out.s(token).s(&res.1.to_string());
                return (res.0, out);
            },
            Expr::Call { func, args, compute_type, filter } => {
                let mut out = Tokens::new();
                out.s(func).s("(");
                let mut arg_types = vec![];
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        out.s(",");
                    }
                    let (t, tokens) = arg.build(ctx, &path.push_back(format!("Call arg {}", i)), scope);
                    out.s(&tokens.to_string());
                    arg_types.push(t);
                }
                out.s(")");
                if let Some(filter) = filter {
                    out.s("filter (where");
                    let (t, tokens) = filter.build(ctx, &path.push_back("Filter".into()), scope);
                    check_bool(ctx, &path.push_back("Filter".into()), &t);
                    out.s(&tokens.to_string()).s(")");
                }
                return (compute_type.0(ctx, path, &arg_types), out);
            },
            Expr::Window { expr, partition_by, order_by, frame } => {
                let mut out = Tokens::new();
                let (t, tokens) = expr.build(ctx, &path.push_back("Window expr".into()), scope);
                out.s(&tokens.to_string()).s("over (");
                let mut first = true;
                if !partition_by.is_empty() {
                    out.s("partition by");
                    for (i, p) in partition_by.iter().enumerate() {
                        if i > 0 {
                            out.s(",");
                        }
                        let (_, tokens) = p.build(ctx, &path.push_back(format!("Partition by {}", i)), scope);
                        out.s(&tokens.to_string());
                    }
                    first = false;
                }
                if !order_by.is_empty() {
                    if !first {
                        out.s(" ");
                    }
                    out.s("order by");
                    for (i, (e, o)) in order_by.iter().enumerate() {
                        if i > 0 {
                            out.s(",");
                        }
                        let (_, tokens): (ExprType, Tokens) =
                            e.build(ctx, &path.push_back(format!("Order by {}", i)), scope);
                        out.s(&tokens.to_string());
                        match o {
                            Order::Asc => {
                                out.s("asc");
                            },
                            Order::Desc => {
                                out.s("desc");
                            },
                        }
                    }
                    first = false;
                }
                if let Some(frame) = frame {
                    if !first {
                        out.s(" ");
                    }
                    out.s(&frame.build(ctx, &path.push_back("Frame".into()), scope).to_string());
                }
                out.s(")");
                return (t, out);
            },
            Expr::Select(s) => {
                let mut out = Tokens::new();
                out.s("(");
                let (t, tokens) = s.build(ctx, &path.push_back("Subselect".into()), crate::QueryResCount::Many);
                out.s(&tokens.to_string()).s(")");
                return (t, out);
            },
            Expr::Cast(e, t) => {
                let mut out = Tokens::new();
                let (got_t, tokens) = e.build(ctx, &path.push_back("Cast".into()), scope);
                check_general_same(ctx, path, &got_t, &ExprType(vec![(ExprValName::empty(), t.clone())]));
                out
                    .s("(")
                    .s(&tokens.to_string())
                    .s("::")
                    .s(crate::pg::types::to_sql_type(&t.type_.type_))
                    .s(")");
                return (ExprType(vec![(ExprValName::empty(), t.clone())]), out);
            },
            Expr::Exists(s) => {
                let mut out = Tokens::new();
                out.s("exists (");
                let (_, tokens) = s.build(ctx, &path.push_back("Exists".into()), crate::QueryResCount::Many);
                out.s(&tokens.to_string()).s(")");
                return (ExprType(vec![(ExprValName::empty(), Type {
                    type_: SimpleType {
                        type_: SimpleSimpleType::Bool,
                        custom: None,
                    },
                    opt: false,
                    arr: false,
                })]), out);
            },
            Expr::Collate(e, s) => {
                let mut out = Tokens::new();
                let (t, tokens) = e.build(ctx, &path.push_back("Collate".into()), scope);
                out.s(&tokens.to_string()).s("collate").s(&format!("\"{}\"", s.replace("\"", "\"\"")));
                return (t, out);
            },
            Expr::Like { expr, pattern, escape, ilike } => {
                let mut out = Tokens::new();
                let (t_expr, tokens_expr) = expr.build(ctx, &path.push_back("Like expr".into()), scope);
                let (t_pattern, tokens_pattern) = pattern.build(ctx, &path.push_back("Like pattern".into()), scope);
                let want_t = Type {
                    type_: SimpleType {
                        type_: SimpleSimpleType::String,
                        custom: None,
                    },
                    opt: false,
                    arr: false,
                };
                if let Some(got_t) = t_expr.assert_scalar(&mut ctx.errs, &path.push_back("Like expr".into())) {
                    check_general_same_type(ctx, &path.push_back("Like expr".into()), &got_t, &want_t);
                }
                if let Some(got_t) = t_pattern.assert_scalar(&mut ctx.errs, &path.push_back("Like pattern".into())) {
                    check_general_same_type(ctx, &path.push_back("Like pattern".into()), &got_t, &want_t);
                }
                out.s(&tokens_expr.to_string());
                if *ilike {
                    out.s("ilike");
                } else {
                    out.s("like");
                }
                out.s(&tokens_pattern.to_string());
                if let Some(escape) = escape {
                    let (t_escape, tokens_escape) =
                        escape.build(ctx, &path.push_back("Like escape".into()), scope);
                    if let Some(got_t) =
                        t_escape.assert_scalar(&mut ctx.errs, &path.push_back("Like escape".into())) {
                        check_general_same_type(ctx, &path.push_back("Like escape".into()), &got_t, &want_t);
                    }
                    out.s("escape").s(&tokens_escape.to_string());
                }
                return (ExprType(vec![(ExprValName::empty(), Type {
                    type_: SimpleType {
                        type_: SimpleSimpleType::Bool,
                        custom: None,
                    },
                    opt: false,
                    arr: false,
                })]), out);
            },
            Expr::Between { e, negated, low, high } => {
                let mut out = Tokens::new();
                let (t, e_tokens) = e.build(ctx, &path.push_back("Between expr".into()), scope);
                let (t_low, low_tokens) = low.build(ctx, &path.push_back("Between low".into()), scope);
                let (t_high, high_tokens) = high.build(ctx, &path.push_back("Between high".into()), scope);
                check_general_same(ctx, path, &t, &t_low);
                check_general_same(ctx, path, &t, &t_high);
                out.s(&e_tokens.to_string());
                if *negated {
                    out.s("not");
                }
                out.s("between").s(&low_tokens.to_string()).s("and").s(&high_tokens.to_string());
                (ExprType(vec![(ExprValName::empty(), crate::pg::types::type_bool().build())]), out)
            },
            Expr::Case { operand, conditions, else_ } => {
                let mut out = Tokens::new();
                out.s("case");
                let op_type = if let Some(operand) = operand {
                    let (t, op_tokens) = operand.build(ctx, &path.push_back("Case operand".into()), scope);
                    out.s(&op_tokens.to_string());
                    Some(t)
                } else {
                    None
                };
                let mut res_type = None;
                for (i, (cond, res)) in conditions.iter().enumerate() {
                    out.s("when");
                    let (cond_t, cond_tokens) =
                        cond.build(ctx, &path.push_back(format!("Case condition {}", i)), scope);
                    if let Some(op_t) = &op_type {
                        check_general_same(ctx, &path.push_back(format!("Case condition {}", i)), op_t, &cond_t);
                    } else {
                        check_bool(ctx, &path.push_back(format!("Case condition {}", i)), &cond_t);
                    }
                    out.s(&cond_tokens.to_string()).s("then");
                    let (r_t, res_tokens) = res.build(ctx, &path.push_back(format!("Case result {}", i)), scope);
                    if let Some(res_t) = &res_type {
                        check_general_same(ctx, &path.push_back(format!("Case result {}", i)), res_t, &r_t);
                    } else {
                        res_type = Some(r_t);
                    }
                    out.s(&res_tokens.to_string());
                }
                if let Some(else_) = else_ {
                    out.s("else");
                    let (else_t, else_tokens) = else_.build(ctx, &path.push_back("Case else".into()), scope);
                    if let Some(res_t) = &res_type {
                        check_general_same(ctx, &path.push_back("Case else".into()), res_t, &else_t);
                    } else {
                        res_type = Some(else_t);
                    }
                    out.s(&else_tokens.to_string());
                }
                out.s("end");
                (res_type.expect("Case must have at least one branch"), out)
            },
        }
    }
}
