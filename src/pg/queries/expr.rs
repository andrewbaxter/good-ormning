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
use std::collections::HashMap;
use crate::{
    pg::{
        types::{
            Type,
            SimpleSimpleType,
            SimpleType,
        },
        queries::utils::QueryBody,
        schema::{
            field::FieldId,
            table::TableId,
        },
        utils::sanitize,
    },
    utils::{
        Tokens,
        Errs,
    },
};
use super::{
    utils::PgQueryCtx,
    select::Select,
};

#[derive(Clone, Debug)]
pub enum Expr {
    // A null value needs a type for type checking purposes. It will always be trated as an
    // optional value.
    LitNull(SimpleType),
    LitBool(bool),
    LitI32(i32),
    LitI64(i64),
    LitU32(u32),
    LitF32(f32),
    LitF64(f64),
    LitString(String),
    LitBytes(Vec<u8>),
    LitUtcTime(DateTime<Utc>),
    /// A query parameter. This will become a parameter to the generated Rust function with
    /// the specified `name` and `type_`.
    Param {
        name: String,
        type_: Type,
    },
    /// This evaluates to the value of a field in the query main or joined tables. If you've
    /// aliased tables or field names, you'll have to instantiate `FieldId` yourself with the
    /// appropriate values. For synthetic values like function results you may need a `FieldId`
    ///  with an empty `TableId` (`""`).
    Field(FieldId),
    BinOp {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },
    /// This is the same as `BinOp` but allows chaining multiple expressions with the same
    /// operator. This can be useful if you have many successive `AND`s or similar.
    BinOpChain {
        op: BinOp,
        exprs: Vec<Expr>,
    },
    PrefixOp {
        op: PrefixOp,
        right: Box<Expr>,
    },
    /// Represents a call to an SQL function, like `collate()`. You must provide the type of
    /// the result since we don't have a table of functions and their return types at present.
    Call {
        func: String,
        type_: Type,
        args: Vec<Expr>,
    },
    /// A sub SELECT query.
    Select(Box<Select>),
    /// This is a synthetic expression, saying to treat the result of the expression as having
    /// the specified type. Use this for casting between primitive types and Rust new-types
    /// for instance.
    Cast(Box<Expr>, Type),
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct ExprValName {
    pub field: FieldId,
    pub name: String,
}

impl ExprValName {
    pub(crate) fn local(name: String) -> Self {
        ExprValName {
            field: FieldId(TableId("".into()), name.clone()),
            name: name,
        }
    }

    pub(crate) fn empty() -> Self {
        ExprValName {
            field: FieldId(TableId("".into()), "".into()),
            name: "".into(),
        }
    }

    pub(crate) fn field(f: FieldId, name: String) -> Self {
        ExprValName {
            field: f,
            name: name,
        }
    }
}

pub struct ExprType(pub(crate) Vec<(ExprValName, Type)>);

impl ExprType {
    pub fn assert_scalar(&self, errs: &mut Errs) -> Option<(ExprValName, Type)> {
        if self.0.len() != 1 {
            errs.err(
                format!("Select outputs must be scalars, but got result with more than one field: {}", self.0.len()),
            );
            return None;
        }
        Some(self.0[0].clone())
    }
}

pub fn check_general_same_type(ctx: &mut PgQueryCtx, left: &Type, right: &Type) {
    if left.opt != right.opt {
        ctx.errs.err(format!("Operator arms have differing optionality"));
    }

    #[derive(Debug)]
    #[samevariant(GeneralTypePairs)]
    enum GeneralType {
        Bool,
        Numeric,
        Blob,
    }

    fn general_type(t: &Type) -> GeneralType {
        match t.type_.type_ {
            SimpleSimpleType::Auto => GeneralType::Numeric,
            SimpleSimpleType::U32 => GeneralType::Numeric,
            SimpleSimpleType::I32 => GeneralType::Numeric,
            SimpleSimpleType::I64 => GeneralType::Numeric,
            SimpleSimpleType::F32 => GeneralType::Numeric,
            SimpleSimpleType::F64 => GeneralType::Numeric,
            SimpleSimpleType::Bool => GeneralType::Bool,
            SimpleSimpleType::String => GeneralType::Blob,
            SimpleSimpleType::Bytes => GeneralType::Blob,
            SimpleSimpleType::UtcTime => GeneralType::Numeric,
        }
    }

    match GeneralTypePairs::pairs(&general_type(left), &general_type(right)) {
        GeneralTypePairs::Nonmatching(left, right) => {
            ctx.errs.err(format!("Operator arms have incompatible types: {:?} and {:?}", left, right));
        },
        _ => { },
    }
}

pub(crate) fn check_general_same(ctx: &mut PgQueryCtx, left: &ExprType, right: &ExprType) {
    if left.0.len() != right.0.len() {
        ctx
            .errs
            .err(
                format!(
                    "Operator arms record type lengths don't match: left has {} fields and right has {}",
                    left.0.len(),
                    right.0.len()
                ),
            );
    } else if left.0.len() == 1 && right.0.len() == 1 {
        check_general_same_type(ctx, &left.0[0].1, &left.0[0].1);
    } else {
        for (i, (left, right)) in left.0.iter().zip(right.0.iter()).enumerate() {
            ctx.errs.push_ctx(vec![("Record pair", i.to_string())]);
            check_general_same_type(ctx, &left.1, &right.1);
            ctx.errs.pop_ctx();
        }
    }
}

pub(crate) fn check_same(errs: &mut Errs, left: &ExprType, right: &ExprType) -> Option<Type> {
    errs.push_ctx(vec![("expr", "left".into())]);
    let left = match left.assert_scalar(errs) {
        Some(t) => t,
        None => {
            return None;
        },
    };
    errs.pop_ctx();
    errs.push_ctx(vec![("expr", "right".into())]);
    let right = match right.assert_scalar(errs) {
        Some(t) => t,
        None => {
            return None;
        },
    };
    errs.pop_ctx();
    if left.1.opt != right.1.opt {
        errs.err(
            format!(
                "Expected same types, but left nullability is {} but right nullability is {}",
                left.1.opt,
                right.1.opt
            ),
        );
    }
    if left.1.type_.custom != right.1.type_.custom {
        errs.err(
            format!(
                "Expected same types, but left rust type is {:?} while right rust type is {:?}",
                left.1.type_.custom,
                right.1.type_.custom
            ),
        );
    }
    if left.1.type_.type_ != right.1.type_.type_ {
        errs.err(
            format!(
                "Expected same types, but left base type is {:?} while right base type is {:?}",
                left.1.type_.type_,
                right.1.type_.type_
            ),
        );
    }
    Some(left.1.clone())
}

pub(crate) fn check_bool(ctx: &mut PgQueryCtx, a: &ExprType) {
    let t = match a.assert_scalar(&mut ctx.errs) {
        Some(t) => t,
        None => {
            return;
        },
    };
    if t.1.opt {
        ctx.errs.err(format!("Expected bool type but is nullable: got {:?}", t));
    }
    if !matches!(t.1.type_.type_, SimpleSimpleType::Bool) {
        ctx.errs.err(format!("Expected bool but type is non-bool: got {:?}", t.1.type_.type_));
    }
}

pub(crate) fn check_assignable(errs: &mut Errs, a: &Type, b: &ExprType) {
    check_same(errs, &ExprType(vec![(ExprValName::empty(), a.clone())]), b);
}

impl Expr {
    pub(crate) fn build(
        &self,
        ctx: &mut PgQueryCtx,
        scope: &HashMap<FieldId, (String, Type)>,
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
            scope: &HashMap<FieldId, (String, Type)>,
            op: &BinOp,
            exprs: &Vec<Expr>,
        ) -> (ExprType, Tokens) {
            if exprs.len() < 2 {
                ctx.errs.err(format!("Binary ops must have at least two operands, but got {}", exprs.len()));
            }
            let mut res = vec![];
            for e in exprs {
                res.push(e.build(ctx, scope));
            }
            let t = match op {
                BinOp::Plus | BinOp::Minus | BinOp::Multiply | BinOp::Divide => {
                    let base = res.get(0).unwrap();
                    let t = match check_same(&mut ctx.errs, &base.0, &res.get(0).unwrap().0) {
                        Some(t) => t,
                        None => {
                            return (ExprType(vec![]), Tokens::new());
                        },
                    };
                    for res in &res[2..] {
                        match check_same(&mut ctx.errs, &base.0, &res.0) {
                            Some(_) => { },
                            None => {
                                return (ExprType(vec![]), Tokens::new());
                            },
                        };
                    }
                    t
                },
                BinOp::And | BinOp::Or => {
                    for res in &res {
                        check_bool(ctx, &res.0);
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
                    check_general_same(ctx, &base.0, &res.get(0).unwrap().0);
                    for res in &res[2..] {
                        check_general_same(ctx, &base.0, &res.0);
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
                BinOp::Equals => "==",
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
            Expr::LitU32(x) => {
                let mut out = Tokens::new();
                out.s(&x.to_string());
                return empty_type!(out, SimpleSimpleType::U32);
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
                let i = ctx.query_args.len();
                ctx.query_args.push(quote!(#x));
                out.s(&format!("${}", i + 1));
                return empty_type!(out, SimpleSimpleType::String);
            },
            Expr::LitBytes(x) => {
                let mut out = Tokens::new();
                let i = ctx.query_args.len();
                let h = hex::encode(&x);
                ctx.query_args.push(quote!(hex_literal::hex!(#h)));
                out.s(&format!("${}", i + 1));
                return empty_type!(out, SimpleSimpleType::Bytes);
            },
            Expr::LitUtcTime(d) => {
                let mut out = Tokens::new();
                let i = ctx.query_args.len();
                let d = d.to_rfc3339();
                ctx.query_args.push(quote!(#d));
                out.s(&format!("${}", i + 1));
                return empty_type!(out, SimpleSimpleType::UtcTime);
            },
            Expr::Param { name: x, type_: t } => {
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
                        let rust_type = match t.type_.type_ {
                            SimpleSimpleType::Auto => quote!(i64),
                            SimpleSimpleType::U32 => quote!(u32),
                            SimpleSimpleType::I32 => quote!(i32),
                            SimpleSimpleType::I64 => quote!(i64),
                            SimpleSimpleType::F32 => quote!(f32),
                            SimpleSimpleType::F64 => quote!(f64),
                            SimpleSimpleType::Bool => quote!(bool),
                            SimpleSimpleType::String => quote!(&str),
                            SimpleSimpleType::Bytes => quote!(&[u8]),
                            SimpleSimpleType::UtcTime => quote!(chrono:: DateTime < chrono:: Utc >),
                        };
                        let ident = format_ident!("{}", sanitize(x).1);
                        let (mut rust_type, mut rust_forward) = if let Some(custom) = &t.type_.custom {
                            let custom_ident = match syn::parse_str::<Path>(custom.as_str()) {
                                Ok(p) => p,
                                Err(e) => {
                                    ctx.errs.err(format!("Couldn't parse custom type {}: {:?}", custom, e));
                                    return (ExprType(vec![]), Tokens::new());
                                },
                            }.to_token_stream();
                            let forward = quote!(#ident.to_sql());
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
                    ctx.errs.err(e);
                }
                out.s(&format!("${}", i + 1));
                return (ExprType(vec![(ExprValName::local(x.clone()), t.clone())]), out);
            },
            Expr::Field(x) => {
                let t = match scope.get(x) {
                    Some(t) => t.clone(),
                    None => {
                        ctx
                            .errs
                            .err(
                                format!(
                                    "Expression references {} but this field isn't available here (available fields: {:?})",
                                    x,
                                    scope
                                        .iter()
                                        .map(|e| format!("{}.{} ({})", e.0.0.0, e.0.1, e.1.0))
                                        .collect::<Vec<String>>()
                                ),
                            );
                        return (ExprType(vec![]), Tokens::new());
                    },
                };
                let mut out = Tokens::new();
                out.id(&x.0.0).s(".").id(&x.1);
                return (ExprType(vec![(ExprValName::field(x.clone(), t.0), t.1.clone())]), out);
            },
            Expr::BinOp { left, op, right } => {
                return do_bin_op(ctx, scope, op, &vec![left.as_ref().clone(), right.as_ref().clone()]);
            },
            Expr::BinOpChain { op, exprs } => {
                return do_bin_op(ctx, scope, op, exprs);
            },
            Expr::PrefixOp { op, right } => {
                let mut out = Tokens::new();
                let res = right.build(ctx, scope);
                let (op_text, op_type) = match op {
                    PrefixOp::Not => {
                        check_bool(ctx, &res.0);
                        ("not", SimpleSimpleType::Bool)
                    },
                };
                out.s(op_text).s(&res.1.to_string());
                return empty_type!(out, op_type);
            },
            Expr::Call { func, type_, args } => {
                let mut out = Tokens::new();
                out.s(func);
                out.s("(");
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        out.s(",");
                    }
                    arg.build(ctx, scope);
                }
                out.s(")");
                return (ExprType(vec![(ExprValName::empty(), type_.clone())]), out);
            },
            Expr::Select(s) => {
                return s.build(ctx);
            },
            Expr::Cast(e, t) => {
                let out = e.build(ctx, scope);
                let got_t = match out.0.assert_scalar(&mut ctx.errs) {
                    Some(t) => t,
                    None => {
                        return (ExprType(vec![]), Tokens::new());
                    },
                };
                check_general_same_type(ctx, t, &got_t.1);
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
