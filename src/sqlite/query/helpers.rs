use {
    super::expr::{
        BinOp,
        ComputeType,
        Expr,
        Binding,
    },
    crate::sqlite::{
        schema::field::Field,
        types::{
            SimpleSimpleType,
            SimpleType,
            Type,
        },
    },
    flowcontrol::shed,
};

/// Generates a field element for instert and update statements, to set a field
/// from a parameter of the same type.
pub fn set_field(param_name: impl Into<String>, f: &Field) -> (Field, Expr) {
    (f.clone(), field_param(param_name, f))
}

/// Generates a param matching a field in name in type
pub fn field_param(param_name: impl Into<String>, f: &Field) -> Expr {
    Expr::Param {
        name: param_name.into(),
        type_: f.type_.type_.clone(),
    }
}

/// Generates an expression checking for equality of a field and a parameter and
/// the same type.
pub fn expr_field_eq(param_name: impl Into<String>, f: &Field) -> Expr {
    Expr::BinOp {
        left: Box::new(Expr::Binding(Binding::field(f))),
        op: BinOp::Equals,
        right: Box::new(Expr::Param {
            name: param_name.into(),
            type_: f.type_.type_.clone(),
        }),
    }
}

/// Generates an expression selecting field values greater than a corresponding
/// parameter
pub fn expr_field_gt(param_name: impl Into<String>, f: &Field) -> Expr {
    Expr::BinOp {
        left: Box::new(Expr::Binding(Binding::field(f))),
        op: BinOp::GreaterThan,
        right: Box::new(Expr::Param {
            name: param_name.into(),
            type_: f.type_.type_.clone(),
        }),
    }
}

/// Generates an expression selecting field values greater than or equal to a
/// corresponding parameter
pub fn expr_field_gte(param_name: impl Into<String>, f: &Field) -> Expr {
    Expr::BinOp {
        left: Box::new(Expr::Binding(Binding::field(f))),
        op: BinOp::GreaterThanEqualTo,
        right: Box::new(Expr::Param {
            name: param_name.into(),
            type_: f.type_.type_.clone(),
        }),
    }
}

/// Generates an expression selecting field values greater than a corresponding
/// parameter
pub fn expr_field_lt(param_name: impl Into<String>, f: &Field) -> Expr {
    Expr::BinOp {
        left: Box::new(Expr::Binding(Binding::field(f))),
        op: BinOp::LessThan,
        right: Box::new(Expr::Param {
            name: param_name.into(),
            type_: f.type_.type_.clone(),
        }),
    }
}

/// Generates an expression selecting field values greater than or equal to a
/// corresponding parameter
pub fn expr_field_lte(param_name: impl Into<String>, f: &Field) -> Expr {
    Expr::BinOp {
        left: Box::new(Expr::Binding(Binding::field(f))),
        op: BinOp::LessThanEqualTo,
        right: Box::new(Expr::Param {
            name: param_name.into(),
            type_: f.type_.type_.clone(),
        }),
    }
}

/// Shortcut for AND expressions.
pub fn expr_and(exprs: Vec<Expr>) -> Expr {
    Expr::BinOpChain {
        op: BinOp::And,
        exprs: exprs,
    }
}

#[cfg(feature = "chrono")]
pub fn as_utc(expr: Expr) -> Expr {
    return Expr::Call {
        func: "strftime".to_string(),
        args: vec![Expr::LitString("%Y-%m-%dT%H:%M:%f".to_string()), expr],
        compute_type: ComputeType::new(|ctx, path, args| {
            shed!{
                let arg = args.get(1).unwrap();
                let Some(type_) = arg.0.iter().next() else {
                    break;
                };
                if !matches!(type_.1.type_.type_, SimpleSimpleType::FixedOffsetTimeMs) {
                    ctx
                        .errs
                        .err(
                            path,
                            format!(
                                "This method only operates on fixed-offset timestamps, but the argument is of type {:?}",
                                type_.1.type_.type_
                            ),
                        );
                }
            };
            return Some(Type {
                type_: SimpleType {
                    type_: SimpleSimpleType::UtcTimeMs,
                    custom: None,
                },
                opt: false,
                array: false,
            });
        }),
    }
}

pub fn fn_min(expr: Expr) -> Expr {
    return Expr::Call {
        func: "min".to_string(),
        args: vec![expr],
        compute_type: ComputeType::new(|ctx, path, args| {
            let Some(t) = args.get(0).unwrap().assert_scalar(&mut ctx.errs, path) else {
                return None;
            };
            return Some(t.1);
        }),
    }
}

pub fn fn_max(expr: Expr) -> Expr {
    return Expr::Call {
        func: "max".to_string(),
        args: vec![expr],
        compute_type: ComputeType::new(|ctx, path, args| {
            let Some(t) = args.get(0).unwrap().assert_scalar(&mut ctx.errs, path) else {
                return None;
            };
            return Some(t.1);
        }),
    }
}

pub fn fn_avg(expr: Expr) -> Expr {
    return Expr::Call {
        func: "avg".to_string(),
        args: vec![expr],
        compute_type: ComputeType::new(|ctx, path, args| {
            let Some(t) = args.get(0).unwrap().assert_scalar(&mut ctx.errs, path) else {
                return None;
            };
            return Some(t.1);
        }),
    }
}

pub fn fn_count(expr: Expr) -> Expr {
    return Expr::Call {
        func: "count".to_string(),
        args: vec![expr],
        compute_type: ComputeType::new(|_ctx, _path, _args| {
            return Some(Type {
                type_: SimpleType {
                    type_: SimpleSimpleType::I64,
                    custom: None,
                },
                opt: false,
                array: false,
            });
        }),
    }
}
