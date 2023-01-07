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
        Field,
        queries::utils::Query,
    },
    utils::Tokens,
};
use super::{
    utils::PgQueryCtx,
    select::Select,
};

pub enum Expr {
    LitBool(bool),
    LitI32(i32),
    LitI64(i64),
    LitU32(u32),
    LitU64(u64),
    LitF32(f32),
    LitF64(f64),
    LitString(String),
    Param(String, Type),
    Field(Field),
    BinOp {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },
    PrefixOp {
        op: PrefixOp,
        right: Box<Expr>,
    },
    Call {
        func: String,
        type_: Type,
        args: Vec<Box<Expr>>,
    },
    Select(Box<Select>),
    Cast(Box<Expr>, Type),
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct ExprTypeField {
    pub table: String,
    pub field: String,
}

pub struct ExprType(pub Vec<(ExprTypeField, Type)>);

impl ExprType {
    pub fn assert_scalar(&self, ctx: &mut PgQueryCtx) -> Option<(ExprTypeField, Type)> {
        if self.0.len() != 1 {
            ctx
                .errs
                .err(
                    format!(
                        "Select outputs must be scalars, but got result with more than one field: {}",
                        self.0.len()
                    ),
                );
            return None;
        }
        Some(self.0[0].clone())
    }
}

impl Expr {
    pub fn build(&self, ctx: &mut PgQueryCtx, scope: &HashMap<ExprTypeField, Type>) -> (ExprType, Tokens) {
        let mut out = Tokens::new();

        macro_rules! empty_type{
            ($t: expr) => {
                (ExprType(vec![(ExprTypeField {
                    table: "".into(),
                    field: "".into(),
                }, Type {
                    type_: SimpleType {
                        type_: $t,
                        custom: None,
                    },
                    opt: false,
                })]), out)
            };
        }

        fn check_general_same_type(ctx: &mut PgQueryCtx, left: &Type, right: &Type) {
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
                    SimpleSimpleType::U64 => GeneralType::Numeric,
                    SimpleSimpleType::I32 => GeneralType::Numeric,
                    SimpleSimpleType::I64 => GeneralType::Numeric,
                    SimpleSimpleType::F32 => GeneralType::Numeric,
                    SimpleSimpleType::F64 => GeneralType::Numeric,
                    SimpleSimpleType::Bool => GeneralType::Bool,
                    SimpleSimpleType::String => GeneralType::Blob,
                    SimpleSimpleType::Bytes => GeneralType::Blob,
                    SimpleSimpleType::LocalTime => GeneralType::Numeric,
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

        fn check_general_same(ctx: &mut PgQueryCtx, left: &ExprType, right: &ExprType) {
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
                    ctx.errs.err_ctx.push(vec![("Record pair", i.to_string())]);
                    check_general_same_type(ctx, &left.1, &right.1);
                    ctx.errs.err_ctx.pop();
                }
            }
        }

        fn check_same(ctx: &mut PgQueryCtx, left: &ExprType, right: &ExprType) -> Option<Type> {
            ctx.errs.err_ctx.push(vec![("expr", "left".into())]);
            let left = match left.assert_scalar(ctx) {
                Some(t) => t,
                None => {
                    return None;
                },
            };
            ctx.errs.err_ctx.pop();
            ctx.errs.err_ctx.push(vec![("expr", "right".into())]);
            let right = match right.assert_scalar(ctx) {
                Some(t) => t,
                None => {
                    return None;
                },
            };
            ctx.errs.err_ctx.pop();
            if left.1.opt != right.1.opt {
                ctx
                    .errs
                    .err(
                        format!(
                            "Expected same types, but left nullability is {} but right nullability is {}",
                            left.1.opt,
                            right.1.opt
                        ),
                    );
            }
            if left.1.type_.custom != right.1.type_.custom {
                ctx
                    .errs
                    .err(
                        format!(
                            "Expected same types, but left rust type is {:?} while right rust type is {:?}",
                            left.1.type_.custom,
                            right.1.type_.custom
                        ),
                    );
            }
            if left.1.type_.type_ != right.1.type_.type_ {
                ctx
                    .errs
                    .err(
                        format!(
                            "Expected same types, but left base type is {:?} while right base type is {:?}",
                            left.1.type_.type_,
                            right.1.type_.type_
                        ),
                    );
            }
            Some(left.1.clone())
        }

        fn check_bool(ctx: &mut PgQueryCtx, a: &ExprType) {
            let t = match a.assert_scalar(ctx) {
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

        match self {
            Expr::LitBool(x) => {
                out.s(if *x {
                    "true"
                } else {
                    "false"
                });
                return empty_type!(SimpleSimpleType::Bool);
            },
            Expr::LitI32(x) => {
                out.s(&x.to_string());
                return empty_type!(SimpleSimpleType::I32);
            },
            Expr::LitI64(x) => {
                out.s(&x.to_string());
                return empty_type!(SimpleSimpleType::I64);
            },
            Expr::LitU32(x) => {
                out.s(&x.to_string());
                return empty_type!(SimpleSimpleType::U32);
            },
            Expr::LitU64(x) => {
                out.s(&x.to_string());
                return empty_type!(SimpleSimpleType::U64);
            },
            Expr::LitF32(x) => {
                out.s(&x.to_string());
                return empty_type!(SimpleSimpleType::F32);
            },
            Expr::LitF64(x) => {
                out.s(&x.to_string());
                return empty_type!(SimpleSimpleType::F64);
            },
            Expr::LitString(x) => {
                out.s(&format!("'{}'", x.replace("'", "''")));
                return empty_type!(SimpleSimpleType::String);
            },
            Expr::Param(x, t) => {
                let mut errs = vec![];
                let i = match ctx.arg_lookup.entry(x.clone()) {
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
                        let i = ctx.args.len();
                        e.insert((i, t.clone()));
                        let rust_type = match t.type_.type_ {
                            SimpleSimpleType::Auto => quote!(usize),
                            SimpleSimpleType::U32 => quote!(u32),
                            SimpleSimpleType::U64 => quote!(u64),
                            SimpleSimpleType::I32 => quote!(i32),
                            SimpleSimpleType::I64 => quote!(i64),
                            SimpleSimpleType::F32 => quote!(f32),
                            SimpleSimpleType::F64 => quote!(f64),
                            SimpleSimpleType::Bool => quote!(bool),
                            SimpleSimpleType::String => quote!(&str),
                            SimpleSimpleType::Bytes => quote!(&[u8]),
                            SimpleSimpleType::LocalTime => quote!(chrono::LocalDateTime),
                            SimpleSimpleType::UtcTime => quote!(chrono:: DateTime < chrono:: Utc >),
                        };
                        let ident = format_ident!("{}", x);
                        let (mut rust_type, mut rust_forward) = if let Some(custom) = &t.type_.custom {
                            (match syn::parse_str::<Path>(custom.as_str()) {
                                Ok(p) => p,
                                Err(e) => {
                                    ctx.errs.err(format!("Couldn't parse custom type {}: {:?}", custom, e));
                                    return (ExprType(vec![]), Tokens::new());
                                },
                            }.to_token_stream(), quote!(#rust_type:: from(#ident)))
                        } else {
                            (rust_type, quote!(#ident))
                        };
                        if t.opt {
                            rust_type = quote!(Option < #rust_type >);
                            rust_forward = quote!(#ident.map(| #ident | #rust_forward));
                        }
                        ctx.args.push(quote!(#ident: #rust_type));
                        ctx.args_forward.push(rust_forward);
                        i
                    },
                };
                for e in errs {
                    ctx.errs.err(e);
                }
                out.s(&format!("${}", i));
                return (ExprType(vec![(ExprTypeField {
                    table: "".into(),
                    field: "".into(),
                }, t.clone())]), out);
            },
            Expr::Field(x) => {
                let id = ExprTypeField {
                    table: x.0.0.0.clone(),
                    field: x.0.1.clone(),
                };
                let t = match scope.get(&id) {
                    Some(t) => t.clone(),
                    None => {
                        ctx
                            .errs
                            .err(
                                format!(
                                    "Expression references {} but this field isn't available here (available fields: {:?})",
                                    x.0,
                                    scope
                                ),
                            );
                        return (ExprType(vec![]), Tokens::new());
                    },
                };
                out.id(&x.0.0.0).s(".").id(&x.0.1);
                return (ExprType(vec![(id, t.clone())]), out);
            },
            Expr::BinOp { left, op, right } => {
                out.s("(");
                let left_type = left.build(ctx, scope);
                out.s(match op {
                    BinOp::Plus => "+",
                    BinOp::Minus => "-",
                    BinOp::Multiply => "*",
                    BinOp::Divide => "/",
                    BinOp::And => "and",
                    BinOp::Or => "or",
                    BinOp::Equals => "==",
                    BinOp::NotEquals => "!=",
                    BinOp::LessThan => "<",
                    BinOp::LessThanEqualTo => "<=",
                    BinOp::GreaterThan => ">",
                    BinOp::GreaterThanEqualTo => ">=",
                });
                let right_type = right.build(ctx, scope);
                out.s(")");
                match op {
                    BinOp::Plus | BinOp::Minus | BinOp::Multiply | BinOp::Divide => {
                        let t = match check_same(ctx, &left_type.0, &right_type.0) {
                            Some(t) => t,
                            None => {
                                return (ExprType(vec![]), Tokens::new());
                            },
                        };
                        return (ExprType(vec![(ExprTypeField {
                            table: "".into(),
                            field: "".into(),
                        }, t.clone())]), out);
                    },
                    BinOp::And | BinOp::Or => {
                        check_bool(ctx, &left_type.0);
                        check_bool(ctx, &right_type.0);
                        return empty_type!(SimpleSimpleType::Bool);
                    },
                    BinOp::Equals | BinOp::NotEquals => {
                        check_general_same(ctx, &left_type.0, &right_type.0);
                        return empty_type!(SimpleSimpleType::Bool);
                    },
                    BinOp::LessThan | BinOp::LessThanEqualTo | BinOp::GreaterThan | BinOp::GreaterThanEqualTo => {
                        check_general_same(ctx, &left_type.0, &right_type.0);
                        return empty_type!(SimpleSimpleType::Bool);
                    },
                }
            },
            Expr::PrefixOp { op, right } => {
                let res = right.build(ctx, scope);
                let (op_text, op_type) = match op {
                    PrefixOp::Not => {
                        check_bool(ctx, &res.0);
                        ("not", SimpleSimpleType::Bool)
                    },
                };
                out.s(op_text).s(&res.1.to_string());
                return empty_type!(op_type);
            },
            Expr::Call { func, type_, args } => {
                out.s(func);
                out.s("(");
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        out.s(",");
                    }
                    arg.build(ctx, scope);
                }
                out.s(")");
                return (ExprType(vec![(ExprTypeField {
                    table: "".into(),
                    field: "".into(),
                }, type_.clone())]), out);
            },
            Expr::Select(s) => {
                return s.build(ctx);
            },
            Expr::Cast(e, t) => {
                let out = e.build(ctx, scope);
                if let Some(got_t) = out.0.assert_scalar(ctx) {
                    check_general_same_type(ctx, t, &got_t.1);
                }
                return (ExprType(vec![(ExprTypeField {
                    table: "".into(),
                    field: "".into(),
                }, t.clone())]), out.1);
            },
        };
    }
}

pub enum BinOp {
    Plus,
    Minus,
    Multiply,
    Divide,
    And,
    Or,
    Equals,
    NotEquals,
    LessThan,
    LessThanEqualTo,
    GreaterThan,
    GreaterThanEqualTo,
}

pub enum PrefixOp {
    Not,
}
