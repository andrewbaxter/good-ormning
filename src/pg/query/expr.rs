use chrono::FixedOffset;
#[cfg(feature = "chrono")]
use chrono::{
    DateTime,
    Utc,
};
use quote::{
    quote,
    format_ident,
    ToTokens,
};
use samevariant::samevariant;
use syn::Path;
use std::{
    collections::HashMap,
    fmt::Display,
    rc::Rc,
};
use crate::{
    pg::{
        types::{
            Type,
            SimpleSimpleType,
            SimpleType,
            to_rust_types,
        },
        query::utils::QueryBody,
        schema::{
            field::{
                Field,
            },
        },
        QueryResCount,
    },
    utils::{
        Tokens,
        Errs,
        sanitize_ident,
    },
};
use super::{
    utils::PgQueryCtx,
    select::Select,
};

/// This is used for function expressions, to check the argument types and compute
/// a result type from them.  See readme for details.
#[derive(Clone)]
pub struct ComputeType(Rc<dyn Fn(&mut PgQueryCtx, &rpds::Vector<String>, Vec<ExprType>) -> Option<Type>>);

impl ComputeType {
    pub fn new(
        f: impl Fn(&mut PgQueryCtx, &rpds::Vector<String>, Vec<ExprType>) -> Option<Type> + 'static,
    ) -> ComputeType {
        return ComputeType(Rc::new(f));
    }
}

impl std::fmt::Debug for ComputeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return f.write_str("ComputeType");
    }
}

#[derive(Clone, Debug)]
pub enum Expr {
    LitArray(Vec<Expr>),
    // A null value needs a type for type checking purposes. It will always be trated
    // as an optional value.
    LitNull(SimpleType),
    LitBool(bool),
    LitAuto(i64),
    LitI32(i32),
    LitI64(i64),
    LitF32(f32),
    LitF64(f64),
    LitString(String),
    LitBytes(Vec<u8>),
    #[cfg(feature = "chrono")]
    LitUtcTime(DateTime<Utc>),
    #[cfg(feature = "chrono")]
    LitFixedOffsetTime(DateTime<FixedOffset>),
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
    Field(Field),
    BinOp {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },
    /// This is the same as `BinOp` but allows chaining multiple expressions with the
    /// same operator. This can be useful if you have many successive `AND`s or similar.
    BinOpChain {
        op: BinOp,
        exprs: Vec<Expr>,
    },
    PrefixOp {
        op: PrefixOp,
        right: Box<Expr>,
    },
    /// Represents a call to an SQL function, like `collate()`. You must provide a
    /// helper to check and determine type of the result since we don't have a table of
    /// functions and their return types at present.
    Call {
        func: String,
        args: Vec<Expr>,
        compute_type: ComputeType,
    },
    /// A sub SELECT query.
    Select(Box<Select>),
    /// This is a synthetic expression, saying to treat the result of the expression as
    /// having the specified type. Use this for casting between primitive types and
    /// Rust new-types for instance.
    Cast(Box<Expr>, Type),
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct ExprValName {
    pub table_id: String,
    pub id: String,
}

impl ExprValName {
    pub(crate) fn local(name: String) -> Self {
        ExprValName {
            table_id: "".into(),
            id: name,
        }
    }

    pub(crate) fn empty() -> Self {
        ExprValName {
            table_id: "".into(),
            id: "".into(),
        }
    }

    pub(crate) fn field(f: &Field) -> Self {
        ExprValName {
            table_id: f.table.id.clone(),
            id: f.id.clone(),
        }
    }

    pub(crate) fn with_alias(&self, s: &str) -> ExprValName {
        ExprValName {
            table_id: s.into(),
            id: self.id.clone(),
        }
    }
}

impl Display for ExprValName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&format!("{}.{}", self.table_id, self.id), f)
    }
}

pub struct ExprType(pub Vec<(ExprValName, Type)>);

impl ExprType {
    pub fn assert_scalar(&self, errs: &mut Errs, path: &rpds::Vector<String>) -> Option<(ExprValName, Type)> {
        if self.0.len() != 1 {
            errs.err(
                path,
                format!("Select outputs must be scalars, but got result with more than one field: {}", self.0.len()),
            );
            return None;
        }
        Some(self.0[0].clone())
    }
}

#[derive(Debug)]
#[samevariant(GeneralTypePairs)]
pub(crate) enum GeneralType {
    Bool,
    Numeric,
    Blob,
}

pub(crate) fn general_type(t: &Type) -> GeneralType {
    match t.type_.type_ {
        SimpleSimpleType::Auto => GeneralType::Numeric,
        SimpleSimpleType::I32 => GeneralType::Numeric,
        SimpleSimpleType::I64 => GeneralType::Numeric,
        SimpleSimpleType::F32 => GeneralType::Numeric,
        SimpleSimpleType::F64 => GeneralType::Numeric,
        SimpleSimpleType::Bool => GeneralType::Bool,
        SimpleSimpleType::String => GeneralType::Blob,
        SimpleSimpleType::Bytes => GeneralType::Blob,
        #[cfg(feature = "chrono")]
        SimpleSimpleType::UtcTime => GeneralType::Numeric,
        #[cfg(feature = "chrono")]
        SimpleSimpleType::FixedOffsetTime => GeneralType::Numeric,
    }
}

pub fn check_general_same_type(ctx: &mut PgQueryCtx, path: &rpds::Vector<String>, left: &Type, right: &Type) {
    if left.opt != right.opt {
        ctx.errs.err(path, format!("Operator arms have differing optionality"));
    }
    match GeneralTypePairs::pairs(&general_type(left), &general_type(right)) {
        GeneralTypePairs::Nonmatching(left, right) => {
            ctx.errs.err(path, format!("Operator arms have incompatible types: {:?} and {:?}", left, right));
        },
        _ => { },
    }
}

pub(crate) fn check_general_same(
    ctx: &mut PgQueryCtx,
    path: &rpds::Vector<String>,
    left: &ExprType,
    right: &ExprType,
) {
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
        check_general_same_type(ctx, path, &left.0[0].1, &left.0[0].1);
    } else {
        for (i, (left, right)) in left.0.iter().zip(right.0.iter()).enumerate() {
            check_general_same_type(ctx, &path.push_back(format!("Record pair {}", i)), &left.1, &right.1);
        }
    }
}

pub(crate) fn check_same(
    errs: &mut Errs,
    path: &rpds::Vector<String>,
    left: &ExprType,
    right: &ExprType,
) -> Option<Type> {
    let left = match left.assert_scalar(errs, &path.push_back("Left".into())) {
        Some(t) => t,
        None => {
            return None;
        },
    };
    let right = match right.assert_scalar(errs, &path.push_back("Right".into())) {
        Some(t) => t,
        None => {
            return None;
        },
    };
    if left.1.opt != right.1.opt {
        errs.err(
            path,
            format!(
                "Expected same types, but left nullability is {} but right nullability is {}",
                left.1.opt,
                right.1.opt
            ),
        );
    }
    if left.1.type_.custom != right.1.type_.custom {
        errs.err(
            path,
            format!(
                "Expected same types, but left rust type is {:?} while right rust type is {:?}",
                left.1.type_.custom,
                right.1.type_.custom
            ),
        );
    }
    if left.1.type_.type_ != right.1.type_.type_ {
        errs.err(
            path,
            format!(
                "Expected same types, but left base type is {:?} while right base type is {:?}",
                left.1.type_.type_,
                right.1.type_.type_
            ),
        );
    }
    Some(left.1.clone())
}

pub(crate) fn check_bool(ctx: &mut PgQueryCtx, path: &rpds::Vector<String>, a: &ExprType) {
    let t = match a.assert_scalar(&mut ctx.errs, path) {
        Some(t) => t,
        None => {
            return;
        },
    };
    if t.1.opt {
        ctx.errs.err(path, format!("Expected bool type but is nullable: got {:?}", t));
    }
    if !matches!(t.1.type_.type_, SimpleSimpleType::Bool) {
        ctx.errs.err(path, format!("Expected bool but type is non-bool: got {:?}", t.1.type_.type_));
    }
}

pub(crate) fn check_assignable(errs: &mut Errs, path: &rpds::Vector<String>, a: &Type, b: &ExprType) {
    check_same(errs, path, &ExprType(vec![(ExprValName::empty(), a.clone())]), b);
}

impl Expr {
    pub(crate) fn build(
        &self,
        ctx: &mut PgQueryCtx,
        path: &rpds::Vector<String>,
        scope: &HashMap<ExprValName, Type>,
    ) -> (ExprType, Tokens) {
        macro_rules! empty_type{
            ($o: expr, $t: expr) => {
                (ExprType(vec![(ExprValName::empty(), Type {
                    type_: SimpleType {
                        type_: $t,
                        custom: None,
                    },
                    opt: false,
                })]), $o)
            };
        }

        fn do_bin_op(
            ctx: &mut PgQueryCtx,
            path: &rpds::Vector<String>,
            scope: &HashMap<ExprValName, Type>,
            op: &BinOp,
            exprs: &Vec<Expr>,
        ) -> (ExprType, Tokens) {
            if exprs.len() < 2 {
                ctx.errs.err(path, format!("Binary ops must have at least two operands, but got {}", exprs.len()));
            }
            let mut res = vec![];
            for (i, e) in exprs.iter().enumerate() {
                res.push(e.build(ctx, &path.push_back(format!("Operand {}", i)), scope));
            }
            let t = match op {
                BinOp::Plus | BinOp::Minus | BinOp::Multiply | BinOp::Divide => {
                    let base = res.get(0).unwrap();
                    let t =
                        match check_same(
                            &mut ctx.errs,
                            &path.push_back(format!("Operands 0, 1")),
                            &base.0,
                            &res.get(0).unwrap().0,
                        ) {
                            Some(t) => t,
                            None => {
                                return (ExprType(vec![]), Tokens::new());
                            },
                        };
                    for (i, res) in res.iter().enumerate().skip(2) {
                        match check_same(
                            &mut ctx.errs,
                            &path.push_back(format!("Operands 0, {}", i)),
                            &base.0,
                            &res.0,
                        ) {
                            Some(_) => { },
                            None => {
                                return (ExprType(vec![]), Tokens::new());
                            },
                        };
                    }
                    t
                },
                BinOp::And | BinOp::Or => {
                    for (i, res) in res.iter().enumerate() {
                        check_bool(ctx, &path.push_back(format!("Operand {}", i)), &res.0);
                    }
                    Type {
                        type_: SimpleType {
                            type_: SimpleSimpleType::Bool,
                            custom: None,
                        },
                        opt: false,
                    }
                },
                BinOp::Equals |
                BinOp::NotEquals |
                BinOp::Is |
                BinOp::IsNot |
                BinOp::LessThan |
                BinOp::LessThanEqualTo |
                BinOp::GreaterThan |
                BinOp::GreaterThanEqualTo => {
                    let base = res.get(0).unwrap();
                    check_general_same(
                        ctx,
                        &path.push_back(format!("Operands 0, 1")),
                        &base.0,
                        &res.get(1).unwrap().0,
                    );
                    for (i, res) in res.iter().enumerate().skip(2) {
                        check_general_same(ctx, &path.push_back(format!("Operands 0, {}", i)), &base.0, &res.0);
                    }
                    Type {
                        type_: SimpleType {
                            type_: SimpleSimpleType::Bool,
                            custom: None,
                        },
                        opt: false,
                    }
                },
            };
            let token = match op {
                BinOp::Plus => "+",
                BinOp::Minus => "-",
                BinOp::Multiply => "*",
                BinOp::Divide => "/",
                BinOp::And => "and",
                BinOp::Or => "or",
                BinOp::Equals => "=",
                BinOp::NotEquals => "!=",
                BinOp::Is => "is",
                BinOp::IsNot => "is not",
                BinOp::LessThan => "<",
                BinOp::LessThanEqualTo => "<=",
                BinOp::GreaterThan => ">",
                BinOp::GreaterThanEqualTo => ">=",
            };
            let mut out = Tokens::new();
            out.s("(");
            for (i, res) in res.iter().enumerate() {
                if i > 0 {
                    out.s(token);
                }
                out.s(&res.1.to_string());
            }
            out.s(")");
            (ExprType(vec![(ExprValName::empty(), t)]), out)
        }

        match self {
            Expr::LitArray(t) => {
                let mut out = Tokens::new();
                let mut child_types = vec![];
                out.s("(");
                for (i, child) in t.iter().enumerate() {
                    if i > 0 {
                        out.s(", ");
                    }
                    let (child_type, child_tokens) = child.build(ctx, path, scope);
                    out.s(&child_tokens.to_string());
                    child_types.extend(child_type.0);
                }
                out.s(")");
                return (ExprType(child_types), out);
            },
            Expr::LitNull(t) => {
                let mut out = Tokens::new();
                out.s("null");
                return (ExprType(vec![(ExprValName::empty(), Type {
                    type_: t.clone(),
                    opt: true,
                })]), out);
            },
            Expr::LitBool(x) => {
                let mut out = Tokens::new();
                out.s(if *x {
                    "true"
                } else {
                    "false"
                });
                return empty_type!(out, SimpleSimpleType::Bool);
            },
            Expr::LitAuto(x) => {
                let mut out = Tokens::new();
                out.s(&x.to_string());
                return empty_type!(out, SimpleSimpleType::Auto);
            },
            Expr::LitI32(x) => {
                let mut out = Tokens::new();
                out.s(&x.to_string());
                return empty_type!(out, SimpleSimpleType::I32);
            },
            Expr::LitI64(x) => {
                let mut out = Tokens::new();
                out.s(&x.to_string());
                return empty_type!(out, SimpleSimpleType::I64);
            },
            Expr::LitF32(x) => {
                let mut out = Tokens::new();
                out.s(&x.to_string());
                return empty_type!(out, SimpleSimpleType::F32);
            },
            Expr::LitF64(x) => {
                let mut out = Tokens::new();
                out.s(&x.to_string());
                return empty_type!(out, SimpleSimpleType::F64);
            },
            Expr::LitString(x) => {
                let mut out = Tokens::new();
                out.s(&format!("'{}'", x.replace("'", "''")));
                return empty_type!(out, SimpleSimpleType::String);
            },
            Expr::LitBytes(x) => {
                let mut out = Tokens::new();
                let h = hex::encode(&x);
                out.s(&format!("x'{}'", h));
                return empty_type!(out, SimpleSimpleType::Bytes);
            },
            #[cfg(feature = "chrono")]
            Expr::LitUtcTime(d) => {
                let mut out = Tokens::new();
                let d = d.to_rfc3339();
                out.s(&format!("'{}'", d));
                return empty_type!(out, SimpleSimpleType::UtcTime);
            },
            #[cfg(feature = "chrono")]
            Expr::LitFixedOffsetTime(d) => {
                let mut out = Tokens::new();
                let d = d.to_rfc3339();
                out.s(&format!("'{}'", d));
                return empty_type!(out, SimpleSimpleType::FixedOffsetTime);
            },
            Expr::Param { name: x, type_: t } => {
                let path = path.push_back(format!("Param ({})", x));
                let mut out = Tokens::new();
                let mut errs = vec![];
                let i = match ctx.rust_arg_lookup.entry(x.clone()) {
                    std::collections::hash_map::Entry::Occupied(e) => {
                        let (i, prev_t) = e.get();
                        if t != prev_t {
                            errs.push(
                                format!("Parameter {} specified with multiple types: {:?}, {:?}", x, t, prev_t),
                            );
                        }
                        *i
                    },
                    std::collections::hash_map::Entry::Vacant(e) => {
                        let i = ctx.query_args.len();
                        e.insert((i, t.clone()));
                        let rust_types = to_rust_types(&t.type_.type_);
                        let custom_trait_ident = rust_types.custom_trait;
                        let rust_type = rust_types.arg_type;
                        let ident = format_ident!("{}", sanitize_ident(x).1);
                        let (mut rust_type, mut rust_forward) = if let Some(custom) = &t.type_.custom {
                            let custom_ident = match syn::parse_str::<Path>(custom.as_str()) {
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
                        if t.opt {
                            rust_type = quote!(Option < #rust_type >);
                            rust_forward = quote!(#ident.map(| #ident | #rust_forward));
                        }
                        ctx.rust_args.push(quote!(#ident: #rust_type));
                        ctx.query_args.push(quote!(#rust_forward));
                        i
                    },
                };
                for e in errs {
                    ctx.errs.err(&path, e);
                }
                out.s(&format!("${}", i + 1));
                return (ExprType(vec![(ExprValName::local(x.clone()), t.clone())]), out);
            },
            Expr::Field(x) => {
                let name = ExprValName::field(x);
                let t = match scope.get(&name) {
                    Some(t) => t.clone(),
                    None => {
                        ctx
                            .errs
                            .err(
                                path,
                                format!(
                                    "Expression references {} but this field isn't available here (available fields: {:?})",
                                    x,
                                    scope.iter().map(|e| e.0.to_string()).collect::<Vec<String>>()
                                ),
                            );
                        return (ExprType(vec![]), Tokens::new());
                    },
                };
                let mut out = Tokens::new();
                out.id(&x.table.id).s(".").id(&x.id);
                return (ExprType(vec![(name, t.clone())]), out);
            },
            Expr::BinOp { left, op, right } => {
                return do_bin_op(
                    ctx,
                    &path.push_back(format!("Bin op {:?}", op)),
                    scope,
                    op,
                    &vec![left.as_ref().clone(), right.as_ref().clone()],
                );
            },
            Expr::BinOpChain { op, exprs } => {
                return do_bin_op(ctx, &path.push_back(format!("Chain bin op {:?}", op)), scope, op, exprs);
            },
            Expr::PrefixOp { op, right } => {
                let path = path.push_back(format!("Prefix op {:?}", op));
                let mut out = Tokens::new();
                let res = right.build(ctx, &path, scope);
                let (op_text, op_type) = match op {
                    PrefixOp::Not => {
                        check_bool(ctx, &path, &res.0);
                        ("not", SimpleSimpleType::Bool)
                    },
                };
                out.s(op_text).s(&res.1.to_string());
                return empty_type!(out, op_type);
            },
            Expr::Call { func, args, compute_type } => {
                let mut types = vec![];
                let mut out = Tokens::new();
                out.s(func);
                out.s("(");
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        out.s(",");
                    }
                    let (arg_type, tokens) =
                        arg.build(ctx, &path.push_back(format!("Call [{}] arg {}", func, i)), scope);
                    types.push(arg_type);
                    out.s(&tokens.to_string());
                }
                out.s(")");
                let type_ = match (compute_type.0)(ctx, path, types) {
                    Some(t) => t,
                    None => {
                        return (ExprType(vec![]), Tokens::new());
                    },
                };
                return (ExprType(vec![(ExprValName::empty(), type_)]), out);
            },
            Expr::Select(s) => {
                let path = path.push_back(format!("Subselect"));
                return s.build(ctx, &path, QueryResCount::Many);
            },
            Expr::Cast(e, t) => {
                let path = path.push_back(format!("Cast"));
                let out = e.build(ctx, &path, scope);
                let got_t = match out.0.assert_scalar(&mut ctx.errs, &path) {
                    Some(t) => t,
                    None => {
                        return (ExprType(vec![]), Tokens::new());
                    },
                };
                check_general_same_type(ctx, &path, t, &got_t.1);
                return (ExprType(vec![(got_t.0, t.clone())]), out.1);
            },
        };
    }
}

#[derive(Clone, Debug)]
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
}

#[derive(Clone, Debug)]
pub enum PrefixOp {
    Not,
}
