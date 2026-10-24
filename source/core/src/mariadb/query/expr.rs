use crate::{
    mariadb::{
        types::{
            Type,
            to_rust_types,
            unify_types,
            SimpleSimpleType,
            SimpleType,
            Temporal,
            temporal,
        },
        query::utils::{
            PARAM_LIST,
            PARAM_MARKER,
            MariadbQueryCtx,
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
use {
    quote::{
        ToTokens,
        format_ident,
        quote,
    },
    serde::{
        Deserialize,
        Serialize,
    },
    std::{
        collections::HashMap,
        fmt::Display,
        rc::Rc,
    },
    super::select::{
        Order,
        Select,
    },
};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum BinOp {
    And,
    BitwiseAnd,
    BitwiseOr,
    BitwiseShiftLeft,
    BitwiseShiftRight,
    BitwiseXor,
    Divide,
    Equals,
    GreaterThan,
    GreaterThanEqualTo,
    In,
    IntDivide,
    Is,
    IsNot,
    LessThan,
    LessThanEqualTo,
    Like,
    Minus,
    Mod,
    Multiply,
    NotEquals,
    NotIn,
    NullSafeEquals,
    Or,
    Plus,
    NotRegexp,
    Regexp,
    Xor,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ExprParent {
    Other,
    Comparison,
    InList,
    Interval,
}

pub fn check_assignable(
    errs: &mut Errs,
    path: &rpds::Vector<String>,
    left: &Type,
    right: &ExprType,
    source: Option<&Expr>,
) {
    let Some(right) = right.assert_scalar(errs, path) else {
        return;
    };
    let computed = source.map(is_computed).unwrap_or(false);
    let literal = match source {
        Some(Expr::LitI16(v)) => Some(*v as i128),
        Some(Expr::LitI32(v)) => Some(*v as i128),
        Some(Expr::LitAnyInt(v) | Expr::LitI64(v)) => Some(*v as i128),
        Some(Expr::LitU32(v)) => Some(*v as i128),
        Some(Expr::LitU64(v)) => Some(*v as i128),
        _ => None,
    };
    let assignable = match (numeric_class(&left.type_.type_), numeric_class(&right.type_.type_), literal) {
        (Some(_), Some(_), Some(v)) => match int_width(&left.type_.type_).map(|(signed, bits)| match bits {
            1 => (true, 8),
            _ => (signed, bits),
        }) {
            Some((true, bits)) => v >= -(1i128 << (bits - 1)) && v < (1i128 << (bits - 1)),
            Some((false, bits)) => v >= 0 && v < (1i128 << bits),
            None => numeric_assignable(&left.type_.type_, &right.type_.type_, computed),
        },
        (Some(_), Some(_), None) => numeric_assignable(&left.type_.type_, &right.type_.type_, computed),
        _ => unify_types(&left.type_.type_, &right.type_.type_).is_some() &&
            !(temporal(&left.type_.type_) == Some(Temporal::Date) &&
                temporal(&right.type_.type_) == Some(Temporal::Datetime)),
    };
    if !assignable {
        if computed {
            errs.err(
                path,
                format!(
                    "Expression has type {:?} which is not assignable to {:?}; use `cast` to convert it",
                    right.type_.type_,
                    left.type_.type_
                ),
            );
        } else {
            errs.err(
                path,
                format!(
                    "Expression has type {:?} which is not assignable to {:?} without losing values",
                    right.type_.type_,
                    left.type_.type_
                ),
            );
        }
    }
    if left.type_.custom != right.type_.custom && right.type_.type_ != SimpleSimpleType::AnyNull {
        errs.err(
            path,
            format!(
                "Expression has custom type {:?} which is not assignable to custom type {:?}",
                right.type_.custom,
                left.type_.custom
            ),
        );
    }
    if !left.opt && right.opt {
        errs.err(path, "Expression is optional but destination is not".to_string());
    }
}

pub fn date_unit(unit: &str) -> bool {
    return matches!(unit.to_ascii_lowercase().as_str(), "day" | "week" | "month" | "quarter" | "year" | "year_month");
}

fn int_width(t: &SimpleSimpleType) -> Option<(bool, u32)> {
    return match t {
        SimpleSimpleType::Bool => Some((false, 1)),
        SimpleSimpleType::I8 => Some((true, 8)),
        SimpleSimpleType::I16 => Some((true, 16)),
        SimpleSimpleType::I32 => Some((true, 32)),
        SimpleSimpleType::I64 => Some((true, 64)),
        SimpleSimpleType::U8 => Some((false, 8)),
        SimpleSimpleType::U16 => Some((false, 16)),
        SimpleSimpleType::U32 => Some((false, 32)),
        SimpleSimpleType::U64 => Some((false, 64)),
        _ => None,
    };
}

fn numeric_assignable(left: &SimpleSimpleType, right: &SimpleSimpleType, computed: bool) -> bool {
    if *right == SimpleSimpleType::AnyInt {
        return true;
    }
    if *right == SimpleSimpleType::AnyDecimal {
        return !matches!(numeric_class(left), Some(NumericClass::Signed | NumericClass::Unsigned));
    }
    if *right == SimpleSimpleType::AnyFloat {
        return numeric_class(left) == Some(NumericClass::Float);
    }
    if let Some((right_signed, right_bits)) = int_width(right) {
        if let Some((left_signed, left_bits)) = int_width(left) {
            let lossless =
                (left_signed == right_signed && left_bits >= right_bits) ||
                    (left_signed && !right_signed && left_bits > right_bits);
            return lossless || computed;
        }
        return match left {
            SimpleSimpleType::Decimal => true,
            SimpleSimpleType::F32 => right_bits <= 16 || computed,
            SimpleSimpleType::F64 => right_bits <= 32 || computed,
            _ => false,
        };
    }
    return match (left, right) {
        (SimpleSimpleType::F64, SimpleSimpleType::F32) => true,
        (SimpleSimpleType::F32, SimpleSimpleType::F64) => computed,
        (SimpleSimpleType::F32 | SimpleSimpleType::F64, SimpleSimpleType::Decimal) => computed,
        (SimpleSimpleType::Decimal, SimpleSimpleType::Decimal) => true,
        _ => left == right,
    };
}

fn is_computed(e: &Expr) -> bool {
    return match e {
        Expr::BinOp { op, .. } | Expr::BinOpChain { op, .. } => matches!(
            op,
            BinOp::Plus | BinOp::Minus | BinOp::Multiply | BinOp::Divide | BinOp::IntDivide | BinOp::Mod |
                BinOp::BitwiseAnd |
                BinOp::BitwiseOr |
                BinOp::BitwiseXor |
                BinOp::BitwiseShiftLeft |
                BinOp::BitwiseShiftRight
        ),
        Expr::PrefixOp { op, .. } => matches!(op, PrefixOp::Minus | PrefixOp::BitwiseNot),
        Expr::Cast { .. } | Expr::Extract { .. } => true,
        Expr::Call { func, args, .. } => match func.as_str() {
            "coalesce" |
            "ifnull" |
            "nullif" |
            "if" |
            "greatest" |
            "least" |
            "min" |
            "max" |
            "first_value" |
            "last_value" |
            "nth_value" |
            "lag" |
            "lead" => args
                .iter()
                .any(is_computed),
            _ => true,
        },
        Expr::Case { conditions, else_, .. } => conditions.iter().any(|(_, e)| is_computed(e)) ||
            else_.as_ref().map(|e| is_computed(e)).unwrap_or(false),
        Expr::Paren(e) | Expr::Collate(e, _) => is_computed(e),
        Expr::Window { expr, .. } => is_computed(expr),
        Expr::Select(s) => s.returning.len() == 1 && is_computed(&s.returning[0].e),
        _ => false,
    };
}

pub fn check_bool(ctx: &mut MariadbQueryCtx, path: &rpds::Vector<String>, t: &ExprType) {
    let Some(t) = t.assert_scalar(&mut ctx.errs, path) else {
        return;
    };
    if !matches!(
        t.type_.type_,
        SimpleSimpleType::Bool | SimpleSimpleType::AnyNull | SimpleSimpleType::AnyInt | SimpleSimpleType::AnyFloat |
            SimpleSimpleType::AnyDecimal |
            SimpleSimpleType::I8 |
            SimpleSimpleType::I16 |
            SimpleSimpleType::I32 |
            SimpleSimpleType::I64 |
            SimpleSimpleType::U8 |
            SimpleSimpleType::U16 |
            SimpleSimpleType::U32 |
            SimpleSimpleType::U64 |
            SimpleSimpleType::F32 |
            SimpleSimpleType::F64 |
            SimpleSimpleType::Decimal
    ) {
        ctx.errs.err(path, format!("Expected a bool or number but got {:?}", t.type_.type_));
    }
}

pub fn check_general_same(
    ctx: &mut MariadbQueryCtx,
    path: &rpds::Vector<String>,
    left: &ExprType,
    right: &ExprType,
) -> ExprType {
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
        return left.clone();
    } else if left.0.len() == 1 && right.0.len() == 1 {
        return ExprType(
            vec![(left.0[0].0.clone(), check_general_same_type(ctx, path, &left.0[0].1, &right.0[0].1))],
        );
    } else {
        let mut out = vec![];
        for (i, (left, right)) in left.0.iter().zip(right.0.iter()).enumerate() {
            out.push(
                (
                    left.0.clone(),
                    check_general_same_type(ctx, &path.push_back(format!("Record pair {}", i)), &left.1, &right.1),
                ),
            );
        }
        return ExprType(out);
    }
}

pub fn check_general_same_type(
    ctx: &mut MariadbQueryCtx,
    path: &rpds::Vector<String>,
    left: &Type,
    right: &Type,
) -> Type {
    let Some(unified) =
        unify_types(&left.type_.type_, &right.type_.type_).or_else(|| (|| -> Option<SimpleSimpleType> {
            let (l, r) = (&left.type_.type_, &right.type_.type_);
            let rank = |t: &SimpleSimpleType| match t {
                SimpleSimpleType::Bool | SimpleSimpleType::I8 | SimpleSimpleType::U8 => 1,
                SimpleSimpleType::I16 | SimpleSimpleType::U16 => 2,
                SimpleSimpleType::I32 | SimpleSimpleType::U32 => 3,
                _ => 4,
            };
            let class = numeric_class(l)?.max(numeric_class(r)?);
            return Some(match class {
                NumericClass::Float => SimpleSimpleType::F64,
                NumericClass::Decimal => SimpleSimpleType::Decimal,
                _ => {
                    let signed =
                        numeric_class(l) == Some(NumericClass::Signed) ||
                            numeric_class(r) == Some(NumericClass::Signed);
                    let mixed = numeric_class(l) != numeric_class(r);
                    if mixed && (*l == SimpleSimpleType::U64 || *r == SimpleSimpleType::U64) {
                        return Some(SimpleSimpleType::Decimal);
                    }
                    let rank = rank(l).max(rank(r)) + if mixed {
                        1
                    } else {
                        0
                    };
                    match (signed, rank) {
                        (true, 1) => SimpleSimpleType::I8,
                        (true, 2) => SimpleSimpleType::I16,
                        (true, 3) => SimpleSimpleType::I32,
                        (true, _) => SimpleSimpleType::I64,
                        (false, 1) => SimpleSimpleType::U8,
                        (false, 2) => SimpleSimpleType::U16,
                        (false, 3) => SimpleSimpleType::U32,
                        (false, _) => SimpleSimpleType::U64,
                    }
                },
            });
        })()) else {
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
            return left.clone();
        };
    if let (Some(l), Some(r)) = (&left.type_.custom, &right.type_.custom) {
        if l != r {
            ctx
                .errs
                .err(
                    path,
                    format!("Operator arms custom types don't match: left has type {} and right has {}", l, r),
                );
        }
    }
    let mut out = unified_type(left, right, unified);
    out.opt = left.opt || right.opt;
    return out;
}

pub fn check_same(errs: &mut Errs, path: &rpds::Vector<String>, left: &ExprType, right: &ExprType) -> Option<Type> {
    let left = left.assert_scalar(errs, &path.push_back("Left".into()))?;
    let right = right.assert_scalar(errs, &path.push_back("Right".into()))?;
    if left.type_.custom.is_some() && right.type_.custom.is_some() && left.type_.custom != right.type_.custom {
        errs.err(
            path,
            format!(
                "Operator arms custom types don't match: left has type {:?} and right has {:?}",
                left.type_.custom,
                right.type_.custom
            ),
        );
    }
    let Some(unified) = unify_types(&left.type_.type_, &right.type_.type_) else {
        errs.err(
            path,
            format!(
                "Operator arms types don't match: left has type {:?} and right has {:?}",
                left.type_.type_,
                right.type_.type_
            ),
        );
        return Some(left.clone());
    };
    let mut out = unified_type(&left, &right, unified);
    out.opt = left.opt || right.opt;
    return Some(out);
}

pub struct ComputeType(pub Rc<dyn Fn(&mut MariadbQueryCtx, &rpds::Vector<String>, &[ExprType]) -> ExprType>);

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

#[derive(Clone, Debug)]
pub enum Expr {
    Between {
        e: Box<Expr>,
        negated: bool,
        low: Box<Expr>,
        high: Box<Expr>,
    },
    BinOp {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },
    BinOpChain {
        op: BinOp,
        exprs: Vec<Expr>,
    },
    Call {
        func: String,
        args: Vec<Expr>,
        compute_type: ComputeType,
        distinct: bool,
    },
    Case {
        operand: Option<Box<Expr>>,
        conditions: Vec<(Expr, Expr)>,
        else_: Option<Box<Expr>>,
    },
    Cast {
        expr: Box<Expr>,
        type_: Type,
        type_args: Vec<u32>,
    },
    Collate(Box<Expr>, String),
    Exists(Box<super::select::Select>),
    Extract {
        unit: String,
        expr: Box<Expr>,
    },
    GroupConcat {
        distinct: bool,
        args: Vec<Expr>,
        order_by: Vec<(Expr, Order)>,
        separator: Option<String>,
        limit: Option<Box<Expr>>,
    },
    Field(FieldRef),
    Interval {
        value: Box<Expr>,
        unit: String,
    },
    Like {
        expr: Box<Expr>,
        negated: bool,
        pattern: Box<Expr>,
        escape: Option<Box<Expr>>,
    },
    LitAnyDecimal(String),
    LitAnyFloat(f64),
    LitAnyInt(i64),
    LitArray(Vec<Expr>),
    LitBool(bool),
    LitBytes(Vec<u8>),
    LitF32(f32),
    LitF64(f64),
    LitI16(i16),
    LitI32(i32),
    LitI64(i64),
    LitNull(SimpleType),
    LitDate(String),
    LitString(String),
    LitTime(String),
    LitTimestamp(String),
    LitU32(u32),
    LitU64(u64),
    #[cfg(feature = "chrono")]
    LitDatetimeChrono(chrono::NaiveDateTime),
    #[cfg(feature = "jiff")]
    LitDatetimeJiff(jiff::civil::DateTime),
    Param {
        name: String,
        type_: Type,
    },
    ParamList {
        name: String,
        types: Vec<Type>,
    },
    Paren(Box<Expr>),
    PrefixOp {
        op: PrefixOp,
        right: Box<Expr>,
    },
    Select(Box<Select>),
    Star,
    Trim {
        where_: Option<TrimWhere>,
        what: Option<Box<Expr>>,
        expr: Box<Expr>,
    },
    Window {
        expr: Box<Expr>,
        over: WindowOver,
    },
}

fn param_rust(
    ctx: &mut MariadbQueryCtx,
    path: &rpds::Vector<String>,
    name: &str,
    type_: &Type,
) -> Option<(proc_macro2::Ident, proc_macro2::TokenStream, proc_macro2::TokenStream)> {
    let rust_types = to_rust_types(&ctx.errs, path, &type_.type_.type_, ctx.default_time_library);
    let custom_trait_ident = rust_types.custom_trait;
    let ident = format_ident!("{}", sanitize_ident(name).1);
    let (mut rust_type, mut rust_forward) = if let Some(custom) = &type_.type_.custom {
        let custom_ident = match syn::parse_str::<syn::Path>(custom.as_str()) {
            Ok(p) => p,
            Err(e) => {
                ctx.errs.err(path, format!("Couldn't parse custom type {}: {:?}", custom, e));
                return None;
            },
        }.to_token_stream();
        let forward = quote!(< #custom_ident as #custom_trait_ident < #custom_ident >>:: to_sql(& #ident));
        (quote!(& #custom_ident), forward)
    } else {
        (rust_types.arg_type, quote!(#ident))
    };
    if type_.opt {
        rust_type = quote!(Option < #rust_type >);
        rust_forward = quote!(#ident.map(| #ident | #rust_forward));
    }
    return Some((ident, rust_type, rust_forward));
}

impl Expr {
    pub fn build(
        &self,
        ctx: &mut MariadbQueryCtx,
        path: &rpds::Vector<String>,
        scope: &HashMap<ExprValName, Type>,
    ) -> (ExprType, Tokens) {
        let parent = std::mem::replace(&mut ctx.expr_parent, ExprParent::Other);
        return match self {
            Expr::LitArray(v) => {
                let mut out = Tokens::new_mariadb();
                let mut res_types = vec![];
                if !matches!(parent, ExprParent::Comparison | ExprParent::InList) {
                    ctx
                        .errs
                        .err(
                            path,
                            "MariaDB doesn't have arrays; lists can only be used with `in` or for comparing rows".to_string(),
                        );
                }
                if v.is_empty() {
                    ctx.errs.err(path, "MariaDB doesn't allow empty lists".to_string());
                }
                out.s("(");
                for (i, e) in v.iter().enumerate() {
                    if i > 0 {
                        out.s(",");
                    }
                    if parent == ExprParent::InList {
                        ctx.expr_parent = ExprParent::Comparison;
                    }
                    let res = e.build(ctx, path, scope);
                    out.s(&res.1.to_string());
                    res_types.push(res.0);
                }
                out.s(")");
                return (ExprType(res_types.into_iter().flat_map(|t| t.0).collect()), out);
            },
            Expr::LitNull(t) => {
                let mut out = Tokens::new_mariadb();
                out.s("null");
                return (ExprType(vec![(ExprValName::empty(), Type {
                    type_: t.clone(),
                    opt: true,
                })]), out);
            },
            Expr::LitBool(b) => {
                let mut out = Tokens::new_mariadb();
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
                })]), out);
            },
            Expr::LitAnyInt(v) => {
                let mut out = Tokens::new_mariadb();
                out.s(&v.to_string());
                return (ExprType(vec![(ExprValName::empty(), Type {
                    type_: SimpleType {
                        type_: if *v < 0 && i32::try_from(*v).is_ok() {
                            SimpleSimpleType::I32
                        } else if i32::try_from(*v).is_ok() {
                            SimpleSimpleType::AnyInt
                        } else {
                            SimpleSimpleType::I64
                        },
                        custom: None,
                    },
                    opt: false,
                })]), out);
            },
            Expr::LitAnyDecimal(v) => {
                let mut out = Tokens::new_mariadb();
                out.s(v);
                return (ExprType(vec![(ExprValName::empty(), Type {
                    type_: SimpleType {
                        type_: SimpleSimpleType::AnyDecimal,
                        custom: None,
                    },
                    opt: false,
                })]), out);
            },
            Expr::LitAnyFloat(v) => {
                let mut out = Tokens::new_mariadb();
                if !v.is_finite() {
                    ctx.errs.err(path, format!("MariaDB doesn't have NaN or infinite floats, but got {}", v));
                }
                out.s(&format!("{:e}", v));
                return (ExprType(vec![(ExprValName::empty(), Type {
                    type_: SimpleType {
                        type_: SimpleSimpleType::AnyFloat,
                        custom: None,
                    },
                    opt: false,
                })]), out);
            },
            Expr::LitI16(v) => {
                let mut out = Tokens::new_mariadb();
                out.s(&v.to_string());
                return (ExprType(vec![(ExprValName::empty(), Type {
                    type_: SimpleType {
                        type_: SimpleSimpleType::I16,
                        custom: None,
                    },
                    opt: false,
                })]), out);
            },
            Expr::LitI32(v) => {
                let mut out = Tokens::new_mariadb();
                out.s(&v.to_string());
                return (ExprType(vec![(ExprValName::empty(), Type {
                    type_: SimpleType {
                        type_: SimpleSimpleType::I32,
                        custom: None,
                    },
                    opt: false,
                })]), out);
            },
            Expr::LitI64(v) => {
                let mut out = Tokens::new_mariadb();
                out.s(&v.to_string());
                return (ExprType(vec![(ExprValName::empty(), Type {
                    type_: SimpleType {
                        type_: SimpleSimpleType::I64,
                        custom: None,
                    },
                    opt: false,
                })]), out);
            },
            Expr::LitU32(v) => {
                let mut out = Tokens::new_mariadb();
                out.s(&v.to_string());
                return (ExprType(vec![(ExprValName::empty(), Type {
                    type_: SimpleType {
                        type_: SimpleSimpleType::U32,
                        custom: None,
                    },
                    opt: false,
                })]), out);
            },
            Expr::LitU64(v) => {
                let mut out = Tokens::new_mariadb();
                out.s(&v.to_string());
                return (ExprType(vec![(ExprValName::empty(), Type {
                    type_: SimpleType {
                        type_: SimpleSimpleType::U64,
                        custom: None,
                    },
                    opt: false,
                })]), out);
            },
            Expr::LitF32(v) => {
                let mut out = Tokens::new_mariadb();
                if !v.is_finite() {
                    ctx.errs.err(path, format!("MariaDB doesn't have NaN or infinite floats, but got {}", v));
                }
                out.s(&format!("cast({:e} as float)", v));
                return (ExprType(vec![(ExprValName::empty(), Type {
                    type_: SimpleType {
                        type_: SimpleSimpleType::F32,
                        custom: None,
                    },
                    opt: false,
                })]), out);
            },
            Expr::LitF64(v) => {
                let mut out = Tokens::new_mariadb();
                if !v.is_finite() {
                    ctx.errs.err(path, format!("MariaDB doesn't have NaN or infinite floats, but got {}", v));
                }
                out.s(&format!("{:e}", v));
                return (ExprType(vec![(ExprValName::empty(), Type {
                    type_: SimpleType {
                        type_: SimpleSimpleType::F64,
                        custom: None,
                    },
                    opt: false,
                })]), out);
            },
            Expr::LitString(v) => {
                let mut out = Tokens::new_mariadb();
                if v.contains(PARAM_MARKER) {
                    ctx.errs.err(path, "String literals can't contain the character \\u{1}".to_string());
                }
                if v.contains('\\') {
                    out.s(&format!("_utf8mb4 x'{}'", hex::encode(v)));
                } else {
                    out.s(&format!("'{}'", v.replace("'", "''")));
                }
                return (ExprType(vec![(ExprValName::empty(), Type {
                    type_: SimpleType {
                        type_: SimpleSimpleType::String,
                        custom: None,
                    },
                    opt: false,
                })]), out);
            },
            Expr::LitBytes(v) => {
                let mut out = Tokens::new_mariadb();
                out.s(&format!("x'{}'", hex::encode(v)));
                return (ExprType(vec![(ExprValName::empty(), Type {
                    type_: SimpleType {
                        type_: SimpleSimpleType::Bytes,
                        custom: None,
                    },
                    opt: false,
                })]), out);
            },
            #[cfg(feature = "chrono")]
            Expr::LitDatetimeChrono(v) => {
                let mut out = Tokens::new_mariadb();
                out.s(&format!("timestamp '{}'", v.format("%Y-%m-%d %H:%M:%S%.6f")));
                return (ExprType(vec![(ExprValName::empty(), Type {
                    type_: SimpleType {
                        type_: SimpleSimpleType::DatetimeChrono,
                        custom: None,
                    },
                    opt: false,
                })]), out);
            },
            #[cfg(feature = "jiff")]
            Expr::LitDatetimeJiff(v) => {
                let mut out = Tokens::new_mariadb();
                out.s(&format!("timestamp '{}'", v.strftime("%Y-%m-%d %H:%M:%S%.6f")));
                return (ExprType(vec![(ExprValName::empty(), Type {
                    type_: SimpleType {
                        type_: SimpleSimpleType::DatetimeJiff,
                        custom: None,
                    },
                    opt: false,
                })]), out);
            },
            Expr::Param { name, type_ } => {
                let mut out = Tokens::new_mariadb();
                let path = path.push_back(format!("Param ({})", name));
                if ctx.list_params.contains_key(name) {
                    ctx.errs.err(&path, format!("Parameter {} is used both as a list and as a single value", name));
                }
                let forward = match ctx.rust_arg_lookup.get(name) {
                    Some((prev_t, forward)) => {
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
                        forward.clone()
                    },
                    None => {
                        let Some((ident, rust_type, rust_forward)) = param_rust(ctx, &path, name, type_) else {
                            return (ExprType(vec![]), Tokens::new_mariadb());
                        };
                        let rust_forward = quote!(good_ormning:: runtime:: mariadb:: to_value(#rust_forward));
                        ctx.rust_args.push((name.clone(), quote!(#ident: #rust_type)));
                        ctx.rust_arg_lookup.insert(name.clone(), (type_.clone(), rust_forward.clone()));
                        rust_forward
                    },
                };
                ctx.query_args.push(forward);
                let marker = format!("{}{}{}", PARAM_MARKER, ctx.query_args.len() - 1, PARAM_MARKER);
                if type_.type_.type_ == SimpleSimpleType::Decimal {
                    out.s("cast(").s(&marker).s("as decimal(65, 30))");
                } else {
                    out.s(&marker);
                }
                return (ExprType(vec![(ExprValName::local(name.clone()), type_.clone())]), out);
            },
            Expr::ParamList { name, types } => {
                let mut out = Tokens::new_mariadb();
                let path = path.push_back(format!("Param list ({})", name));
                if parent != ExprParent::InList {
                    ctx
                        .errs
                        .err(
                            &path,
                            "List parameters can only be used as the list in `in (...)` or `not in (...)`".to_string(),
                        );
                }
                if types.is_empty() {
                    ctx.errs.err(&path, "List parameters need at least one element type".to_string());
                    return (ExprType(vec![]), Tokens::new_mariadb());
                }
                if ctx.rust_arg_lookup.contains_key(name) {
                    ctx.errs.err(&path, format!("Parameter {} is used both as a list and as a single value", name));
                }
                let forward = match ctx.list_params.get(name) {
                    Some((prev_types, forward)) => {
                        if types != prev_types {
                            ctx
                                .errs
                                .err(
                                    &path,
                                    format!(
                                        "Parameter {} specified with multiple types: {:?}, {:?}",
                                        name,
                                        types,
                                        prev_types
                                    ),
                                );
                        }
                        forward.clone()
                    },
                    None => {
                        let mut idents = vec![];
                        let mut rust_types = vec![];
                        let mut forwards = vec![];
                        for (i, type_) in types.iter().enumerate() {
                            let element_name = if types.len() == 1 {
                                name.clone()
                            } else {
                                format!("{}_{}", name, i)
                            };
                            let Some((ident, rust_type, rust_forward)) =
                                param_rust(ctx, &path, &element_name, type_) else {
                                    return (ExprType(vec![]), Tokens::new_mariadb());
                                };
                            idents.push(ident);
                            rust_types.push(rust_type);
                            forwards.push(quote!(good_ormning:: runtime:: mariadb:: to_value(#rust_forward)));
                        }
                        let ident = format_ident!("{}", sanitize_ident(name).1);
                        let template = types.iter().map(|t| if t.type_.type_ == SimpleSimpleType::Decimal {
                            "cast(? as decimal(65, 30))"
                        } else {
                            "?"
                        }).collect::<Vec<_>>();
                        let width = types.len();
                        let (rust_type, template) = if width == 1 {
                            (quote!(#(#rust_types) *), template.join(""))
                        } else {
                            (quote!((#(#rust_types,) *)), format!("({})", template.join(", ")))
                        };
                        let pattern = if width == 1 {
                            quote!(#(#idents) *)
                        } else {
                            quote!((#(#idents,) *))
                        };
                        let forward =
                            quote!(
                                #ident.iter().cloned().flat_map(| #pattern |[#(#forwards), *]).collect::< Vec < _ >>(),
                                #template,
                                #width
                            );
                        ctx.rust_args.push((name.clone(), quote!(#ident: &[#rust_type])));
                        ctx.list_params.insert(name.clone(), (types.clone(), forward.clone()));
                        forward
                    },
                };
                ctx.query_args.push(forward);
                out.s(&format!("{}{}{}{}", PARAM_MARKER, PARAM_LIST, ctx.query_args.len() - 1, PARAM_MARKER));
                return (
                    ExprType(types.iter().map(|t| (ExprValName::local(name.clone()), t.clone())).collect()),
                    out,
                );
            },
            Expr::Field(x) => {
                let name = ExprValName::field(x);
                let mut found = scope.get(&name).cloned();
                if found.is_none() {
                    for outer in ctx.outer_scopes.iter().rev() {
                        if let Some(t) = outer.get(&name) {
                            found = Some(t.clone());
                            break;
                        }
                    }
                }
                let t = match found {
                    Some(t) => t.clone(),
                    None => {
                        if x.table_id.is_empty() {
                            let mut found_list = vec![];
                            for (k, v) in scope {
                                if k.id == x.field_id {
                                    found_list.push(v.clone());
                                }
                            }
                            if found_list.is_empty() {
                                for outer in ctx.outer_scopes.iter().rev() {
                                    for (k, v) in outer {
                                        if k.id == x.field_id {
                                            found_list.push(v.clone());
                                        }
                                    }
                                    if !found_list.is_empty() {
                                        break;
                                    }
                                }
                            }
                            if found_list.len() == 1 {
                                found_list[0].clone()
                            } else if found_list.len() > 1 {
                                ctx
                                    .errs
                                    .err(
                                        path,
                                        format!("Field {:?} is ambiguous (found in multiple tables)", x.field_id),
                                    );
                                return (ExprType(vec![]), Tokens::new_mariadb());
                            } else {
                                ctx.errs.err(path, format!("Field {:?} not found in any table", x.field_id));
                                return (ExprType(vec![]), Tokens::new_mariadb());
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
                            return (ExprType(vec![]), Tokens::new_mariadb());
                        }
                    },
                };
                let mut out = Tokens::new_mariadb();
                if x.table_id.is_empty() {
                    out.id(&x.field_id);
                } else {
                    let table_id = ctx.table_aliases.get(&x.table_id).map(|tr| tr.0.clone());
                    let table_info = match &table_id {
                        Some(table_id) => ctx.tables.get(&TableRef(table_id.clone())),
                        None => ctx.tables.get(&TableRef(x.table_id.clone())),
                    };
                    let (sql_table, sql_field) = match table_info {
                        Some(table_info) => {
                            let field_info = table_info.fields.get(&FieldRef {
                                table_id: table_id.clone().unwrap_or_else(|| x.table_id.clone()),
                                field_id: x.field_id.clone(),
                            });
                            (if table_id.is_some() {
                                x.table_id.clone()
                            } else {
                                table_info.sql_name.clone()
                            }, field_info.map(|f| f.sql_name.clone()).unwrap_or_else(|| x.field_id.clone()))
                        },
                        None => (x.table_id.clone(), x.field_id.clone()),
                    };
                    out.id(&sql_table).s(".").id(&sql_field);
                }
                return (ExprType(vec![(name, t.clone())]), out);
            },
            Expr::BinOp { left, op, right } => {
                let mut out = Tokens::new_mariadb();
                let token;
                match op {
                    BinOp::In | BinOp::NotIn => {
                        let token = match op {
                            BinOp::In => "in",
                            BinOp::NotIn => "not in",
                            _ => unreachable!(),
                        };
                        ctx.expr_parent = ExprParent::Comparison;
                        let (left_t, left_tokens) = left.build(ctx, &path.push_back("Operand 0".into()), scope);
                        ctx.expr_parent = ExprParent::InList;
                        let (right_t, right_tokens) = right.build(ctx, &path.push_back("Operand 1".into()), scope);
                        if matches!(**right, Expr::ParamList { .. }) && right_t.0.len() != left_t.0.len() {
                            ctx
                                .errs
                                .err(
                                    path,
                                    format!(
                                        "The list parameter has {} values per element but the left side has {}",
                                        right_t.0.len(),
                                        left_t.0.len()
                                    ),
                                );
                        } else if !left_t.0.is_empty() && !right_t.0.is_empty() {
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
                            opt: left_t.0.iter().chain(right_t.0.iter()).any(|(_, t)| t.opt),
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
                        token = "^";
                    },
                    BinOp::BitwiseShiftLeft => {
                        token = "<<";
                    },
                    BinOp::BitwiseShiftRight => {
                        token = ">>";
                    },
                    BinOp::NullSafeEquals => {
                        token = "<=>";
                    },
                    BinOp::IntDivide => {
                        token = "div";
                    },
                    BinOp::Xor => {
                        token = "xor";
                    },
                    BinOp::Regexp => {
                        token = "regexp";
                    },
                    BinOp::NotRegexp => {
                        token = "not regexp";
                    },
                }
                let is_tuple_cmp =
                    matches!(
                        op,
                        BinOp::Equals | BinOp::NotEquals | BinOp::LessThan | BinOp::LessThanEqualTo |
                            BinOp::GreaterThan |
                            BinOp::GreaterThanEqualTo |
                            BinOp::NullSafeEquals
                    );
                let operand_parent = if is_tuple_cmp {
                    ExprParent::Comparison
                } else if matches!(op, BinOp::Plus | BinOp::Minus) {
                    ExprParent::Interval
                } else {
                    ExprParent::Other
                };
                if *op != BinOp::Minus {
                    ctx.expr_parent = operand_parent;
                }
                let mut l_res = left.build(ctx, &path.push_back("Bin op left".into()), scope);
                ctx.expr_parent = operand_parent;
                let mut r_res = right.build(ctx, &path.push_back("Bin op right".into()), scope);
                for (e, res) in [(&**left, &mut l_res), (&**right, &mut r_res)] {
                    if matches!(e, Expr::BinOp { .. } | Expr::BinOpChain { .. }) {
                        let mut wrapped = Tokens::new_mariadb();
                        wrapped.s("(").s(&res.1.to_string()).s(")");
                        res.1 = wrapped;
                    }
                }
                if matches!(
                    op,
                    BinOp::Plus | BinOp::Minus | BinOp::Multiply | BinOp::Divide | BinOp::IntDivide | BinOp::Mod
                ) {
                    for res in [&mut l_res, &mut r_res] {
                        for (_, t) in &mut res.0.0 {
                            if t.type_.type_ == SimpleSimpleType::Bool {
                                t.type_ = SimpleType {
                                    type_: SimpleSimpleType::AnyInt,
                                    custom: None,
                                };
                            }
                        }
                    }
                }
                let interval_side = match (&**left, &**right) {
                    (Expr::Interval { unit, .. }, _) => Some((&r_res.0, unit)),
                    (_, Expr::Interval { unit, .. }) => Some((&l_res.0, unit)),
                    _ => None,
                };
                let right_expr = &**right;
                let t =
                    if let Some((time_t, unit)) =
                        interval_side.filter(|_| matches!(op, BinOp::Plus | BinOp::Minus)) {
                        time_t.assert_scalar(&mut ctx.errs, path).map(|mut t| {
                            t.opt = true;
                            match temporal(&t.type_.type_) {
                                Some(Temporal::Date) if !date_unit(unit) => {
                                    t.type_ = SimpleType {
                                        type_: SimpleSimpleType::AnyDatetime,
                                        custom: None,
                                    };
                                },
                                Some(_) => { },
                                None => if !matches!(
                                    t.type_.type_,
                                    SimpleSimpleType::String | SimpleSimpleType::AnyNull
                                ) {
                                    ctx
                                        .errs
                                        .err(
                                            path,
                                            format!(
                                                "Interval arithmetic needs a date, datetime, time, or string, but got {:?}",
                                                t.type_.type_
                                            ),
                                        );
                                },
                            }
                            return t;
                        })
                    } else if matches!(op, BinOp::And | BinOp::Or | BinOp::Xor) {
                        check_bool(ctx, &path.push_back("Bin op left".into()), &l_res.0);
                        check_bool(ctx, &path.push_back("Bin op right".into()), &r_res.0);
                        Some(Type {
                            type_: SimpleType {
                                type_: SimpleSimpleType::Bool,
                                custom: None,
                            },
                            opt: l_res.0.0.iter().chain(r_res.0.0.iter()).any(|(_, t)| t.opt),
                        })
                    } else if is_tuple_cmp && (l_res.0.0.len() > 1 || r_res.0.0.len() > 1) {
                        if !l_res.0.0.is_empty() && !r_res.0.0.is_empty() {
                            if l_res.0.0.len() == r_res.0.0.len() {
                                check_general_same(ctx, path, &l_res.0, &r_res.0);
                            } else {
                                ctx
                                    .errs
                                    .err(
                                        path,
                                        format!(
                                            "Operator {:?} arms record type lengths don't match: left has {} fields and right has {}",
                                            op,
                                            l_res.0.0.len(),
                                            r_res.0.0.len()
                                        ),
                                    );
                            }
                        }
                        None
                    } else if matches!(
                        op,
                        BinOp::Plus | BinOp::Minus | BinOp::Multiply | BinOp::Divide | BinOp::IntDivide | BinOp::Mod |
                            BinOp::BitwiseAnd |
                            BinOp::BitwiseOr |
                            BinOp::BitwiseXor |
                            BinOp::BitwiseShiftLeft |
                            BinOp::BitwiseShiftRight
                    ) &&
                        l_res
                            .0
                            .0
                            .iter()
                            .chain(r_res.0.0.iter())
                            .any(
                                |(_, t)| numeric_class(&t.type_.type_).is_none() &&
                                    t.type_.type_ != SimpleSimpleType::AnyNull,
                            ) {
                        ctx
                            .errs
                            .err(
                                path,
                                format!(
                                    "Operator {:?} needs numbers, but got {:?} and {:?}; MariaDB would convert these to numbers (for dates use functions like `datediff`, `timediff` or `date_add`)",
                                    op,
                                    l_res.0.0.iter().map(|(_, t)| &t.type_.type_).collect::<Vec<_>>(),
                                    r_res.0.0.iter().map(|(_, t)| &t.type_.type_).collect::<Vec<_>>()
                                ),
                            );
                        None
                    } else if let Some(t) = (|| -> Option<Type> {
                        let ([(_, left)], [(_, right)]) = (l_res.0.0.as_slice(), r_res.0.0.as_slice()) else {
                            return None;
                        };
                        let left_class = numeric_class(&left.type_.type_)?;
                        let right_class = numeric_class(&right.type_.type_)?;
                        if let (Some(l), Some(r)) = (&left.type_.custom, &right.type_.custom) {
                            if l != r {
                                ctx
                                    .errs
                                    .err(
                                        path,
                                        format!(
                                            "Operator arms custom types don't match: left has type {} and right has {}",
                                            l,
                                            r
                                        ),
                                    );
                            }
                        }
                        let class = left_class.max(right_class);
                        let int_type =
                            if left_class == NumericClass::Unsigned || right_class == NumericClass::Unsigned {
                                SimpleSimpleType::U64
                            } else {
                                SimpleSimpleType::I64
                            };
                        let any_opt = left.opt || right.opt;
                        let (type_, opt) = match op {
                            BinOp::Plus | BinOp::Minus | BinOp::Multiply => (match class {
                                NumericClass::Signed | NumericClass::Unsigned => int_type,
                                NumericClass::Decimal => SimpleSimpleType::Decimal,
                                NumericClass::Float => SimpleSimpleType::F64,
                            }, any_opt),
                            BinOp::Divide => (match class {
                                NumericClass::Float => SimpleSimpleType::F64,
                                _ => SimpleSimpleType::Decimal,
                            }, any_opt || !nonzero_literal(right_expr)),
                            BinOp::Mod => (match class {
                                NumericClass::Signed | NumericClass::Unsigned => int_type,
                                NumericClass::Decimal => SimpleSimpleType::Decimal,
                                NumericClass::Float => SimpleSimpleType::F64,
                            }, any_opt || !nonzero_literal(right_expr)),
                            BinOp::IntDivide => (match class {
                                NumericClass::Unsigned => SimpleSimpleType::U64,
                                _ => int_type,
                            }, any_opt || !nonzero_literal(right_expr)),
                            BinOp::BitwiseAnd |
                            BinOp::BitwiseOr |
                            BinOp::BitwiseXor |
                            BinOp::BitwiseShiftLeft |
                            BinOp::BitwiseShiftRight => (
                                SimpleSimpleType::U64,
                                any_opt,
                            ),
                            BinOp::Equals |
                            BinOp::NotEquals |
                            BinOp::LessThan |
                            BinOp::LessThanEqualTo |
                            BinOp::GreaterThan |
                            BinOp::GreaterThanEqualTo |
                            BinOp::NullSafeEquals |
                            BinOp::Is |
                            BinOp::IsNot => (
                                SimpleSimpleType::Bool,
                                any_opt,
                            ),
                            _ => return None,
                        };
                        return Some(Type {
                            type_: SimpleType {
                                type_: type_,
                                custom: None,
                            },
                            opt: opt,
                        });
                    })() {
                        Some(t)
                    } else {
                        check_same(&mut ctx.errs, path, &l_res.0, &r_res.0)
                    };
                out.s(&l_res.1.to_string()).s(token).s(&r_res.1.to_string());
                let any_opt = l_res.0.0.iter().chain(r_res.0.0.iter()).any(|(_, t)| t.opt);
                if matches!(op, BinOp::Is | BinOp::IsNot) && !matches!(**right, Expr::LitBool(_) | Expr::LitNull(_)) {
                    ctx
                        .errs
                        .err(path, "MariaDB only allows `true`, `false`, `unknown` or `null` after `is`".to_string());
                }
                if matches!(op, BinOp::Is | BinOp::IsNot | BinOp::NullSafeEquals) {
                    for (null_side, other_t) in [(&**right, &l_res.0), (&**left, &r_res.0)] {
                        if matches!(null_side, Expr::LitNull(_)) && !other_t.0.is_empty() &&
                            other_t.0.iter().all(|(_, t)| !t.opt) {
                            ctx
                                .errs
                                .err(
                                    path,
                                    format!(
                                        "Comparing a non-optional value with null using {:?}, this is always the same result",
                                        op
                                    ),
                                );
                        }
                    }
                }
                let mut res_t = t.unwrap_or(Type {
                    type_: SimpleType {
                        type_: SimpleSimpleType::I32,
                        custom: None,
                    },
                    opt: false,
                });
                match op {
                    BinOp::Is | BinOp::IsNot | BinOp::NullSafeEquals => {
                        res_t = Type {
                            type_: SimpleType {
                                type_: SimpleSimpleType::Bool,
                                custom: None,
                            },
                            opt: false,
                        };
                    },
                    BinOp::Equals |
                    BinOp::NotEquals |
                    BinOp::LessThan |
                    BinOp::LessThanEqualTo |
                    BinOp::GreaterThan |
                    BinOp::GreaterThanEqualTo |
                    BinOp::Like |
                    BinOp::Regexp |
                    BinOp::NotRegexp => {
                        res_t = Type {
                            type_: SimpleType {
                                type_: SimpleSimpleType::Bool,
                                custom: None,
                            },
                            opt: any_opt,
                        };
                    },
                    BinOp::Xor => {
                        res_t = Type {
                            type_: SimpleType {
                                type_: SimpleSimpleType::Bool,
                                custom: None,
                            },
                            opt: any_opt,
                        };
                    },
                    BinOp::In | BinOp::NotIn => unreachable!(),
                    _ => { },
                }
                return (ExprType(vec![(ExprValName::empty(), res_t)]), out);
            },
            Expr::BinOpChain { op, exprs } => {
                let mut out = Tokens::new_mariadb();
                let token = match op {
                    BinOp::And => "and",
                    BinOp::Or => "or",
                    BinOp::Xor => "xor",
                    _ => {
                        ctx
                            .errs
                            .err(path, format!("Operator chains are only supported for and/or/xor, not {:?}", op));
                        return (ExprType(vec![]), Tokens::new_mariadb());
                    },
                };
                let mut any_opt = false;
                for (i, e) in exprs.iter().enumerate() {
                    if i > 0 {
                        out.s(token);
                    }
                    let res = e.build(ctx, &path.push_back(format!("Chain element {}", i)), scope);
                    check_bool(ctx, &path.push_back(format!("Chain element {}", i)), &res.0);
                    if matches!(e, Expr::BinOp { .. } | Expr::BinOpChain { .. }) {
                        out.s("(").s(&res.1.to_string()).s(")");
                    } else {
                        out.s(&res.1.to_string());
                    }
                    any_opt = any_opt || res.0.0.iter().any(|(_, t)| t.opt);
                }
                return (ExprType(vec![(ExprValName::empty(), Type {
                    type_: SimpleType {
                        type_: SimpleSimpleType::Bool,
                        custom: None,
                    },
                    opt: any_opt,
                })]), out);
            },
            Expr::PrefixOp { op, right } => {
                let mut out = Tokens::new_mariadb();
                let token = match op {
                    PrefixOp::Not => "not",
                    PrefixOp::BitwiseNot => "~",
                    PrefixOp::Minus => "-",
                };
                let mut res = right.build(ctx, &path.push_back("Prefix op".into()), scope);
                match op {
                    PrefixOp::Not => {
                        check_bool(ctx, path, &res.0);
                        for (_, t) in &mut res.0.0 {
                            t.type_ = SimpleType {
                                type_: SimpleSimpleType::Bool,
                                custom: None,
                            };
                        }
                    },
                    PrefixOp::BitwiseNot => {
                        for (_, t) in &mut res.0.0 {
                            if numeric_class(&t.type_.type_).is_none() && t.type_.type_ != SimpleSimpleType::AnyNull {
                                ctx.errs.err(path, format!("`~` needs a number, but got {:?}", t.type_.type_));
                            }
                            t.type_ = SimpleType {
                                type_: SimpleSimpleType::U64,
                                custom: None,
                            };
                        }
                    },
                    PrefixOp::Minus => {
                        for (_, t) in &mut res.0.0 {
                            if numeric_class(&t.type_.type_).is_none() && t.type_.type_ != SimpleSimpleType::AnyNull {
                                ctx.errs.err(path, format!("Negation needs a number, but got {:?}", t.type_.type_));
                            }
                            let negated = match t.type_.type_ {
                                SimpleSimpleType::AnyInt => Some(SimpleSimpleType::I32),
                                SimpleSimpleType::Bool |
                                SimpleSimpleType::I8 |
                                SimpleSimpleType::I16 |
                                SimpleSimpleType::I32 |
                                SimpleSimpleType::U8 |
                                SimpleSimpleType::U16 |
                                SimpleSimpleType::U32 |
                                SimpleSimpleType::U64 => Some(
                                    SimpleSimpleType::I64,
                                ),
                                SimpleSimpleType::F32 => Some(SimpleSimpleType::F64),
                                _ => None,
                            };
                            if let Some(negated) = negated {
                                t.type_ = SimpleType {
                                    type_: negated,
                                    custom: None,
                                };
                            }
                        }
                    },
                }
                out.s(token);
                if matches!(**right, Expr::BinOp { .. } | Expr::BinOpChain { .. }) {
                    out.s("(").s(&res.1.to_string()).s(")");
                } else {
                    out.s(&res.1.to_string());
                }
                return (res.0, out);
            },
            Expr::Call { func, args, compute_type, distinct } => {
                let mut out = Tokens::new_mariadb();
                out.s(&format!("{}(", func));
                if *distinct {
                    out.s("distinct");
                }
                let mut arg_types = vec![];
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        out.s(",");
                    }
                    if i == 1 && matches!(func.as_str(), "date_add" | "date_sub" | "adddate" | "subdate") {
                        ctx.expr_parent = ExprParent::Interval;
                    }
                    let (t, tokens) = arg.build(ctx, &path.push_back(format!("Call arg {}", i)), scope);
                    out.s(&tokens.to_string());
                    arg_types.push(t);
                }
                out.s(")");
                return (compute_type.0(ctx, path, &arg_types), out);
            },
            Expr::Window { expr, over } => {
                let mut out = Tokens::new_mariadb();
                let (t, tokens) = expr.build(ctx, &path.push_back("Window expr".into()), scope);
                out.s(&tokens.to_string()).s("over");
                match over {
                    WindowOver::Named(name) => {
                        out.id(name);
                    },
                    WindowOver::Spec(spec) => {
                        out.s("(").s(&spec.build(ctx, &path.push_back("Over".into()), scope).to_string()).s(")");
                    },
                }
                return (t, out);
            },
            Expr::Select(s) => {
                let mut out = Tokens::new_mariadb();
                out.s("(");
                let in_set_op = parent == ExprParent::InList;
                ctx.outer_scopes.push(scope.clone());
                let (mut t, tokens) = s.build(ctx, &path.push_back("Subselect".into()), crate::QueryResCount::Many);
                ctx.outer_scopes.pop();
                out.s(&tokens.to_string()).s(")");
                if !in_set_op {
                    for (_, t) in t.0.iter_mut() {
                        t.opt = true;
                    }
                }
                return (t, out);
            },
            Expr::Cast { expr: e, type_: t, type_args } => {
                let mut out = Tokens::new_mariadb();
                let (got_t, tokens) = e.build(ctx, &path.push_back("Cast".into()), scope);
                let max_args = match &t.type_.type_ {
                    SimpleSimpleType::AnyDecimal | SimpleSimpleType::Decimal => 2,
                    SimpleSimpleType::String | SimpleSimpleType::Bytes => 1,
                    t if matches!(temporal(t), Some(Temporal::Datetime | Temporal::Time)) => 1,
                    _ => 0,
                };
                if type_args.len() > max_args {
                    ctx
                        .errs
                        .err(
                            path,
                            format!(
                                "Cast to {:?} takes at most {} arguments but got {}",
                                t.type_.type_,
                                max_args,
                                type_args.len()
                            ),
                        );
                }
                if matches!(t.type_.type_, SimpleSimpleType::AnyDecimal | SimpleSimpleType::Decimal) &&
                    type_args.is_empty() {
                    ctx
                        .errs
                        .err(
                            path,
                            "Cast to decimal needs a precision (and usually a scale), since MariaDB's default `decimal(10,0)` rounds to an integer".to_string(),
                        );
                }
                let got_t = got_t.assert_scalar(&mut ctx.errs, &path.push_back("Cast".into()));
                if matches!(
                    t.type_.type_,
                    SimpleSimpleType::Bool | SimpleSimpleType::I8 | SimpleSimpleType::I16 | SimpleSimpleType::I32 |
                        SimpleSimpleType::U8 |
                        SimpleSimpleType::U16 |
                        SimpleSimpleType::U32
                ) {
                    ctx
                        .errs
                        .err(
                            path,
                            format!(
                                "MariaDB can't cast to {:?}, integer casts (`signed`, `unsigned`) produce 64-bit integers",
                                t.type_.type_
                            ),
                        );
                }
                out
                    .s("cast(")
                    .s(&tokens.to_string())
                    .s("as")
                    .s(crate::mariadb::types::to_sql_cast_type(&t.type_.type_));
                if !type_args.is_empty() {
                    out.s("(");
                    for (i, a) in type_args.iter().enumerate() {
                        if i > 0 {
                            out.s(",");
                        }
                        out.s(&a.to_string());
                    }
                    out.s(")");
                }
                out.s(")");
                let mut out_t = t.clone();
                let to_time = temporal(&t.type_.type_).is_some();
                out_t.opt = out_t.opt || got_t.map(|t| t.opt).unwrap_or(false) || to_time;
                return (ExprType(vec![(ExprValName::empty(), out_t)]), out);
            },
            Expr::GroupConcat { distinct, args, order_by, separator, limit } => {
                let mut out = Tokens::new_mariadb();
                out.s("group_concat(");
                let mut binary = false;
                if *distinct {
                    out.s("distinct");
                }
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        out.s(",");
                    }
                    let (arg_t, arg_tokens) = arg.build(ctx, &path.push_back(format!("Arg {}", i)), scope);
                    if let Some(t) = arg_t.assert_scalar(&mut ctx.errs, &path.push_back(format!("Arg {}", i))) {
                        binary = binary || t.type_.type_ == SimpleSimpleType::Bytes;
                    }
                    out.s(&arg_tokens.to_string());
                }
                if !order_by.is_empty() {
                    out.s("order by");
                    for (i, (e, o)) in order_by.iter().enumerate() {
                        if i > 0 {
                            out.s(",");
                        }
                        let (_, e_tokens) = e.build(ctx, &path.push_back(format!("Order by {}", i)), scope);
                        out.s(&e_tokens.to_string());
                        match o {
                            Order::Asc => {
                                out.s("asc");
                            },
                            Order::Desc => {
                                out.s("desc");
                            },
                        }
                    }
                }
                if let Some(separator) = separator {
                    if separator.contains('\\') {
                        out.s("separator").s(&format!("x'{}'", hex::encode(separator)));
                    } else {
                        let (_, separator_tokens) =
                            Expr::LitString(
                                separator.clone(),
                            ).build(ctx, &path.push_back("Separator".into()), scope);
                        out.s("separator").s(&separator_tokens.to_string());
                    }
                }
                if let Some(limit) = limit {
                    let path = path.push_back("Limit".into());
                    super::utils::check_limit_arg(ctx, &path, limit);
                    let (limit_t, limit_tokens) = limit.build(ctx, &path, scope);
                    super::utils::check_limit_type(ctx, &path, &limit_t);
                    out.s("limit").s(&limit_tokens.to_string());
                }
                out.s(")");
                return (ExprType(vec![(ExprValName::empty(), Type {
                    type_: SimpleType {
                        type_: if binary {
                            SimpleSimpleType::Bytes
                        } else {
                            SimpleSimpleType::String
                        },
                        custom: None,
                    },
                    opt: true,
                })]), out);
            },
            Expr::Extract { unit, expr } => {
                let mut out = Tokens::new_mariadb();
                let (t, tokens) = expr.build(ctx, &path.push_back("Extract".into()), scope);
                out.s("extract(").s(unit).s("from").s(&tokens.to_string()).s(")");
                return (ExprType(vec![(ExprValName::empty(), Type {
                    type_: SimpleType {
                        type_: SimpleSimpleType::I64,
                        custom: None,
                    },
                    opt: t.0.iter().any(|(_, t)| t.opt),
                })]), out);
            },
            Expr::Interval { value, unit } => {
                if parent != ExprParent::Interval {
                    ctx
                        .errs
                        .err(
                            path,
                            "MariaDB only allows `interval` as the second argument of `date_add`-style functions, or added to or subtracted from a date (`x + interval ...`, `interval ... + x`, `x - interval ...`)".to_string(),
                        );
                }
                let mut out = Tokens::new_mariadb();
                let (t, tokens) = value.build(ctx, &path.push_back("Interval".into()), scope);
                out.s("interval").s(&tokens.to_string()).s(unit);
                return (ExprType(vec![(ExprValName::empty(), Type {
                    type_: SimpleType {
                        type_: SimpleSimpleType::AnyNull,
                        custom: None,
                    },
                    opt: t.0.iter().any(|(_, t)| t.opt),
                })]), out);
            },
            Expr::LitTimestamp(v) | Expr::LitDate(v) | Expr::LitTime(v) => {
                let mut out = Tokens::new_mariadb();
                let (_, tokens) = Expr::LitString(v.clone()).build(ctx, path, scope);
                let (keyword, type_) = match self {
                    Expr::LitDate(_) => ("date", SimpleSimpleType::AnyDate),
                    Expr::LitTime(_) => ("time", SimpleSimpleType::AnyTime),
                    _ => ("timestamp", SimpleSimpleType::AnyDatetime),
                };
                out.s(keyword).s(&tokens.to_string());
                return (ExprType(vec![(ExprValName::empty(), Type {
                    type_: SimpleType {
                        type_: type_,
                        custom: None,
                    },
                    opt: false,
                })]), out);
            },
            Expr::Star => {
                let mut out = Tokens::new_mariadb();
                out.s("*");
                return (ExprType(vec![]), out);
            },
            Expr::Trim { where_, what, expr } => {
                let mut out = Tokens::new_mariadb();
                out.s("trim(");
                let mut opt = false;
                match where_ {
                    Some(TrimWhere::Both) => {
                        out.s("both");
                    },
                    Some(TrimWhere::Leading) => {
                        out.s("leading");
                    },
                    Some(TrimWhere::Trailing) => {
                        out.s("trailing");
                    },
                    None => { },
                }
                if let Some(what) = what {
                    let (what_t, what_tokens) = what.build(ctx, &path.push_back("Trim characters".into()), scope);
                    opt = opt || what_t.0.iter().any(|(_, t)| t.opt);
                    out.s(&what_tokens.to_string());
                }
                if where_.is_some() || what.is_some() {
                    out.s("from");
                }
                let (expr_t, expr_tokens) = expr.build(ctx, &path.push_back("Trim".into()), scope);
                opt = opt || expr_t.0.iter().any(|(_, t)| t.opt);
                let binary = expr_t.0.iter().any(|(_, t)| t.type_.type_ == SimpleSimpleType::Bytes);
                out.s(&expr_tokens.to_string()).s(")");
                return (ExprType(vec![(ExprValName::empty(), Type {
                    type_: SimpleType {
                        type_: if binary {
                            SimpleSimpleType::Bytes
                        } else {
                            SimpleSimpleType::String
                        },
                        custom: None,
                    },
                    opt: opt,
                })]), out);
            },
            Expr::Exists(s) => {
                let mut out = Tokens::new_mariadb();
                out.s("exists (");
                ctx.outer_scopes.push(scope.clone());
                let (_, tokens) = s.build(ctx, &path.push_back("Exists".into()), crate::QueryResCount::Many);
                ctx.outer_scopes.pop();
                out.s(&tokens.to_string()).s(")");
                return (ExprType(vec![(ExprValName::empty(), Type {
                    type_: SimpleType {
                        type_: SimpleSimpleType::Bool,
                        custom: None,
                    },
                    opt: false,
                })]), out);
            },
            Expr::Collate(e, s) => {
                let mut out = Tokens::new_mariadb();
                let (t, tokens) = e.build(ctx, &path.push_back("Collate".into()), scope);
                out.s(&tokens.to_string()).s("collate").id(s);
                return (t, out);
            },
            Expr::Like { expr, negated, pattern, escape } => {
                let mut out = Tokens::new_mariadb();
                let (t_expr, tokens_expr) = expr.build(ctx, &path.push_back("Like expr".into()), scope);
                let (t_pattern, tokens_pattern) = pattern.build(ctx, &path.push_back("Like pattern".into()), scope);
                for (t, name) in [(&t_expr, "Like expr"), (&t_pattern, "Like pattern")] {
                    if let Some(got_t) = t.assert_scalar(&mut ctx.errs, &path.push_back(name.into())) {
                        if !matches!(
                            got_t.type_.type_,
                            SimpleSimpleType::String | SimpleSimpleType::Bytes | SimpleSimpleType::AnyNull
                        ) {
                            ctx
                                .errs
                                .err(
                                    &path.push_back(name.into()),
                                    format!("Expected a string or bytes but got {:?}", got_t.type_.type_),
                                );
                        }
                    }
                }
                let want_t = Type {
                    type_: SimpleType {
                        type_: SimpleSimpleType::String,
                        custom: None,
                    },
                    opt: false,
                };
                let mut any_opt = t_expr.0.iter().chain(t_pattern.0.iter()).any(|(_, t)| t.opt);
                out.s(&tokens_expr.to_string()).s(if *negated {
                    "not like"
                } else {
                    "like"
                }).s(&tokens_pattern.to_string());
                if let Some(escape) = escape {
                    let (t_escape, tokens_escape) =
                        escape.build(ctx, &path.push_back("Like escape".into()), scope);
                    if let Some(got_t) =
                        t_escape.assert_scalar(&mut ctx.errs, &path.push_back("Like escape".into())) {
                        check_general_same_type(ctx, &path.push_back("Like escape".into()), &got_t, &want_t);
                    }
                    any_opt = any_opt || t_escape.0.iter().any(|(_, t)| t.opt);
                    out.s("escape").s(&tokens_escape.to_string());
                }
                return (ExprType(vec![(ExprValName::empty(), Type {
                    type_: SimpleType {
                        type_: SimpleSimpleType::Bool,
                        custom: None,
                    },
                    opt: any_opt,
                })]), out);
            },
            Expr::Between { e, negated, low, high } => {
                let mut out = Tokens::new_mariadb();
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
                let mut res_t = crate::mariadb::types::type_bool().build();
                res_t.opt = t.0.iter().chain(t_low.0.iter()).chain(t_high.0.iter()).any(|(_, t)| t.opt);
                (ExprType(vec![(ExprValName::empty(), res_t)]), out)
            },
            Expr::Case { operand, conditions, else_ } => {
                let mut out = Tokens::new_mariadb();
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
                        res_type =
                            Some(
                                check_general_same(ctx, &path.push_back(format!("Case result {}", i)), res_t, &r_t),
                            );
                    } else {
                        res_type = Some(r_t);
                    }
                    out.s(&res_tokens.to_string());
                }
                if let Some(else_) = else_ {
                    out.s("else");
                    let (else_t, else_tokens) = else_.build(ctx, &path.push_back("Case else".into()), scope);
                    if let Some(res_t) = &res_type {
                        res_type =
                            Some(check_general_same(ctx, &path.push_back("Case else".into()), res_t, &else_t));
                    } else {
                        res_type = Some(else_t);
                    }
                    out.s(&else_tokens.to_string());
                } else if let Some(res_t) = &mut res_type {
                    for (_, t) in &mut res_t.0 {
                        t.opt = true;
                    }
                }
                out.s("end");
                match res_type {
                    Some(t) => (t, out),
                    None => {
                        ctx.errs.err(path, "Case must have at least one branch".to_string());
                        (ExprType(vec![]), out)
                    },
                }
            },
            Expr::Paren(e) => {
                let mut out = Tokens::new_mariadb();
                let (t, tokens) = e.build(ctx, &path.push_back("Paren".into()), scope);
                out.s("(").s(&tokens.to_string()).s(")");
                return (t, out);
            },
        };
    }
}

impl From<SerialExpr> for Expr {
    fn from(s: SerialExpr) -> Self {
        return match s {
            SerialExpr::LitArray(v) => Expr::LitArray(v.into_iter().map(Expr::from).collect()),
            SerialExpr::LitNull(t) => Expr::LitNull(t),
            SerialExpr::LitBool(b) => Expr::LitBool(b),
            SerialExpr::LitAnyDecimal(v) => Expr::LitAnyDecimal(v),
            SerialExpr::LitAnyInt(v) => Expr::LitAnyInt(v),
            SerialExpr::LitI16(v) => Expr::LitI16(v),
            SerialExpr::LitI32(v) => Expr::LitI32(v),
            SerialExpr::LitI64(v) => Expr::LitI64(v),
            SerialExpr::LitU32(v) => Expr::LitU32(v),
            SerialExpr::LitF32(v) => Expr::LitF32(v),
            SerialExpr::LitF64(v) => Expr::LitF64(v),
            SerialExpr::LitString(v) => Expr::LitString(v),
            SerialExpr::LitDate(v) => Expr::LitDate(v),
            SerialExpr::LitTime(v) => Expr::LitTime(v),
            SerialExpr::LitTimestamp(v) => Expr::LitTimestamp(v),
            SerialExpr::LitBytes(v) => Expr::LitBytes(v),
            #[cfg(feature = "chrono")]
            SerialExpr::LitDatetimeChrono(v) => Expr::LitDatetimeChrono(v),
            #[cfg(feature = "jiff")]
            SerialExpr::LitDatetimeJiff(v) => Expr::LitDatetimeJiff(v),
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
            SerialExpr::Cast { expr, type_, type_args } => Expr::Cast {
                expr: Box::new(Expr::from(*expr)),
                type_: type_,
                type_args: type_args,
            },
            SerialExpr::Collate(e, s) => Expr::Collate(Box::new(Expr::from(*e)), s),
            SerialExpr::Like { expr, negated, pattern, escape } => Expr::Like {
                expr: Box::new(Expr::from(*expr)),
                negated: negated,
                pattern: Box::new(Expr::from(*pattern)),
                escape: escape.map(|e| Box::new(Expr::from(*e))),
            },
            SerialExpr::Between { e, negated, low, high } => Expr::Between {
                e: Box::new(Expr::from(*e)),
                negated: negated,
                low: Box::new(Expr::from(*low)),
                high: Box::new(Expr::from(*high)),
            },
        };
    }
}

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
        return Some(self.0[0].1.clone());
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct ExprValName {
    pub id: String,
    pub table_id: String,
}

impl ExprValName {
    pub fn empty() -> Self {
        return ExprValName {
            table_id: "".into(),
            id: "".into(),
        };
    }

    pub fn field(f: &FieldRef) -> Self {
        return ExprValName {
            table_id: f.table_id.clone(),
            id: f.field_id.clone(),
        };
    }

    pub fn local(name: String) -> Self {
        return ExprValName {
            table_id: "".into(),
            id: name,
        };
    }

    pub fn with_alias(&self, s: &str) -> ExprValName {
        return ExprValName {
            table_id: s.into(),
            id: self.id.clone(),
        };
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

#[derive(Clone, Debug)]
pub enum TrimWhere {
    Both,
    Leading,
    Trailing,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum PrefixOp {
    BitwiseNot,
    Minus,
    Not,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum SerialExpr {
    Between {
        e: Box<SerialExpr>,
        negated: bool,
        low: Box<SerialExpr>,
        high: Box<SerialExpr>,
    },
    BinOp {
        left: Box<SerialExpr>,
        op: BinOp,
        right: Box<SerialExpr>,
    },
    BinOpChain {
        op: BinOp,
        exprs: Vec<SerialExpr>,
    },
    Cast {
        expr: Box<SerialExpr>,
        type_: Type,
        type_args: Vec<u32>,
    },
    Collate(Box<SerialExpr>, String),
    Like {
        expr: Box<SerialExpr>,
        negated: bool,
        pattern: Box<SerialExpr>,
        escape: Option<Box<SerialExpr>>,
    },
    LitAnyDecimal(String),
    LitAnyInt(i64),
    LitArray(Vec<SerialExpr>),
    LitBool(bool),
    LitBytes(Vec<u8>),
    LitF32(f32),
    LitF64(f64),
    LitI16(i16),
    LitI32(i32),
    LitI64(i64),
    LitNull(SimpleType),
    LitDate(String),
    LitString(String),
    LitTime(String),
    LitTimestamp(String),
    LitU32(u32),
    #[cfg(feature = "chrono")]
    LitDatetimeChrono(chrono::NaiveDateTime),
    #[cfg(feature = "jiff")]
    LitDatetimeJiff(jiff::civil::DateTime),
    PrefixOp {
        op: PrefixOp,
        right: Box<SerialExpr>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SerialWindowFrame {
    pub end: Option<SerialWindowFrameBound>,
    pub start: SerialWindowFrameBound,
    pub type_: SerialWindowFrameType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SerialWindowFrameBound {
    CurrentRow,
    Following(Box<SerialExpr>),
    Preceding(Box<SerialExpr>),
    UnboundedFollowing,
    UnboundedPreceding,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SerialWindowFrameType {
    Range,
    Rows,
}

pub fn numeric_class(t: &SimpleSimpleType) -> Option<NumericClass> {
    return match t {
        SimpleSimpleType::AnyInt |
        SimpleSimpleType::Bool |
        SimpleSimpleType::I8 |
        SimpleSimpleType::I16 |
        SimpleSimpleType::I32 |
        SimpleSimpleType::I64 => Some(
            NumericClass::Signed,
        ),
        SimpleSimpleType::U8 | SimpleSimpleType::U16 | SimpleSimpleType::U32 | SimpleSimpleType::U64 => Some(
            NumericClass::Unsigned,
        ),
        SimpleSimpleType::AnyDecimal | SimpleSimpleType::Decimal => Some(NumericClass::Decimal),
        SimpleSimpleType::AnyFloat | SimpleSimpleType::F32 | SimpleSimpleType::F64 => Some(NumericClass::Float),
        _ => None,
    };
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NumericClass {
    Signed,
    Unsigned,
    Decimal,
    Float,
}

fn nonzero_literal(e: &Expr) -> bool {
    return match e {
        Expr::LitAnyInt(v) | Expr::LitI64(v) => *v != 0,
        Expr::LitI32(v) => *v != 0,
        Expr::LitI16(v) => *v != 0,
        Expr::LitU32(v) => *v != 0,
        Expr::LitU64(v) => *v != 0,
        Expr::LitAnyFloat(v) | Expr::LitF64(v) => *v != 0.,
        Expr::LitF32(v) => *v != 0.,
        Expr::LitAnyDecimal(v) => v.parse::<f64>().map(|v| v != 0.).unwrap_or(false),
        _ => false,
    };
}

fn unified_type(left: &Type, right: &Type, unified: SimpleSimpleType) -> Type {
    if unified == left.type_.type_ {
        return left.clone();
    } else if unified == right.type_.type_ {
        return right.clone();
    } else {
        let mut out = left.clone();
        out.type_ = SimpleType {
            custom: left.type_.custom.clone().or_else(|| right.type_.custom.clone()),
            type_: unified,
        };
        return out;
    }
}

#[derive(Clone, Debug)]
pub enum WindowOver {
    Named(String),
    Spec(WindowSpec),
}

#[derive(Clone, Debug)]
pub struct WindowSpec {
    pub base: Option<String>,
    pub partition_by: Vec<Expr>,
    pub order_by: Vec<(Expr, Order)>,
    pub frame: Option<WindowFrame>,
}

impl WindowSpec {
    pub fn build(
        &self,
        ctx: &mut MariadbQueryCtx,
        path: &rpds::Vector<String>,
        scope: &HashMap<ExprValName, Type>,
    ) -> Tokens {
        let mut out = Tokens::new_mariadb();
        if let Some(base) = &self.base {
            out.id(base);
        }
        let mut first = self.base.is_none();
        if !self.partition_by.is_empty() {
            out.s("partition by");
            for (i, p) in self.partition_by.iter().enumerate() {
                if i > 0 {
                    out.s(",");
                }
                let (_, tokens) = p.build(ctx, &path.push_back(format!("Partition by {}", i)), scope);
                out.s(&tokens.to_string());
            }
            first = false;
        }
        if !self.order_by.is_empty() {
            if !first {
                out.s(" ");
            }
            out.s("order by");
            for (i, (e, o)) in self.order_by.iter().enumerate() {
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
        if let Some(frame) = &self.frame {
            if !first {
                out.s(" ");
            }
            out.s(&frame.build(ctx, &path.push_back("Frame".into()), scope).to_string());
        }
        return out;
    }
}

#[derive(Clone, Debug)]
pub struct WindowFrame {
    pub end: Option<WindowFrameBound>,
    pub start: WindowFrameBound,
    pub type_: WindowFrameType,
}

impl WindowFrame {
    pub fn build(
        &self,
        ctx: &mut MariadbQueryCtx,
        path: &rpds::Vector<String>,
        scope: &HashMap<ExprValName, Type>,
    ) -> Tokens {
        let mut out = Tokens::new_mariadb();
        match self.type_ {
            WindowFrameType::Rows => {
                out.s("rows");
            },
            WindowFrameType::Range => {
                out.s("range");
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
        return out;
    }
}

impl From<SerialWindowFrame> for WindowFrame {
    fn from(s: SerialWindowFrame) -> Self {
        return WindowFrame {
            type_: WindowFrameType::from(s.type_),
            start: WindowFrameBound::from(s.start),
            end: s.end.map(WindowFrameBound::from),
        };
    }
}

#[derive(Clone, Debug)]
pub enum WindowFrameBound {
    CurrentRow,
    Following(Box<Expr>),
    Preceding(Box<Expr>),
    UnboundedFollowing,
    UnboundedPreceding,
}

impl WindowFrameBound {
    pub fn build(
        &self,
        ctx: &mut MariadbQueryCtx,
        path: &rpds::Vector<String>,
        scope: &HashMap<ExprValName, Type>,
    ) -> Tokens {
        let mut out = Tokens::new_mariadb();
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
        return out;
    }
}

impl From<SerialWindowFrameBound> for WindowFrameBound {
    fn from(s: SerialWindowFrameBound) -> Self {
        return match s {
            SerialWindowFrameBound::UnboundedPreceding => WindowFrameBound::UnboundedPreceding,
            SerialWindowFrameBound::Preceding(e) => WindowFrameBound::Preceding(Box::new(Expr::from(*e))),
            SerialWindowFrameBound::CurrentRow => WindowFrameBound::CurrentRow,
            SerialWindowFrameBound::Following(e) => WindowFrameBound::Following(Box::new(Expr::from(*e))),
            SerialWindowFrameBound::UnboundedFollowing => WindowFrameBound::UnboundedFollowing,
        };
    }
}

#[derive(Clone, Debug)]
pub enum WindowFrameType {
    Range,
    Rows,
}

impl From<SerialWindowFrameType> for WindowFrameType {
    fn from(s: SerialWindowFrameType) -> Self {
        return match s {
            SerialWindowFrameType::Rows => WindowFrameType::Rows,
            SerialWindowFrameType::Range => WindowFrameType::Range,
        };
    }
}
