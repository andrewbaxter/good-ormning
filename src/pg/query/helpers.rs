use std::rc::Rc;
use crate::pg::{
    FieldHandle,
    types::{
        SimpleSimpleType,
        SimpleType,
        Type,
    },
};
use super::expr::{
    Expr,
    BinOp,
    ComputeType,
    ExprValName,
    ExprType,
};
use super::select::Order;

/// Generates a field element for instert and update statements, to set a field
/// from a parameter of the same type.
pub fn set_field(param_name: impl Into<String>, f: &FieldHandle) -> (FieldHandle, Expr) {
    (f.clone(), field_param(param_name, f))
}

/// Generates a param matching a field in name in type
pub fn field_param(param_name: impl Into<String>, f: &FieldHandle) -> Expr {
    let version = f.table.version.0.borrow();
    let type_ =
        version
            .as_ref()
            .unwrap()
            .tables
            .get(&f.table.id)
            .unwrap()
            .fields
            .get(&f.id)
            .unwrap()
            .type_
            .type_
            .clone();
    Expr::Param {
        name: param_name.into(),
        type_: type_,
    }
}

/// Generates an expression checking for equality of a field and a parameter and
/// the same type.
pub fn eq_field(param_name: impl Into<String>, f: &FieldHandle) -> Expr {
    Expr::BinOp {
        left: Box::new(Expr::Field(f.to_ref())),
        op: BinOp::Equals,
        right: Box::new(field_param(param_name, f)),
    }
}

/// Generates an expression selecting field values greater than a corresponding
/// parameter
pub fn gt_field(param_name: impl Into<String>, f: &FieldHandle) -> Expr {
    Expr::BinOp {
        left: Box::new(Expr::Field(f.to_ref())),
        op: BinOp::GreaterThan,
        right: Box::new(field_param(param_name, f)),
    }
}

/// Generates an expression selecting field values greater than or equal to a
/// corresponding parameter
pub fn gte_field(param_name: impl Into<String>, f: &FieldHandle) -> Expr {
    Expr::BinOp {
        left: Box::new(Expr::Field(f.to_ref())),
        op: BinOp::GreaterThanEqualTo,
        right: Box::new(field_param(param_name, f)),
    }
}

/// Generates an expression selecting field values greater than a corresponding
/// parameter
pub fn lt_field(param_name: impl Into<String>, f: &FieldHandle) -> Expr {
    Expr::BinOp {
        left: Box::new(Expr::Field(f.to_ref())),
        op: BinOp::LessThan,
        right: Box::new(field_param(param_name, f)),
    }
}

/// Generates an expression selecting field values greater than or equal to a
/// corresponding parameter
pub fn lte_field(param_name: impl Into<String>, f: &FieldHandle) -> Expr {
    Expr::BinOp {
        left: Box::new(Expr::Field(f.to_ref())),
        op: BinOp::LessThanEqualTo,
        right: Box::new(field_param(param_name, f)),
    }
}

/// Shortcut for chain AND expressions.
pub fn expr_and(exprs: Vec<Expr>) -> Expr {
    Expr::BinOpChain {
        op: BinOp::And,
        exprs: exprs,
    }
}

/// Shortcut for chain OR expressions.
pub fn expr_or(exprs: Vec<Expr>) -> Expr {
    Expr::BinOpChain {
        op: BinOp::Or,
        exprs: exprs,
    }
}

pub fn fn_min(expr: Expr) -> Expr {
    return Expr::Call {
        func: "min".to_string(),
        args: vec![expr],
        compute_type: ComputeType(Rc::new(|ctx, path, args| {
            let Some(t) = args.get(0).unwrap().assert_scalar(&mut ctx.errs, path) else {
                return ExprType(vec![]);
            };
            return ExprType(vec![(ExprValName::empty(), t)]);
        })),
    }
}

pub fn fn_max(expr: Expr) -> Expr {
    return Expr::Call {
        func: "max".to_string(),
        args: vec![expr],
        compute_type: ComputeType(Rc::new(|ctx, path, args| {
            let Some(t) = args.get(0).unwrap().assert_scalar(&mut ctx.errs, path) else {
                return ExprType(vec![]);
            };
            return ExprType(vec![(ExprValName::empty(), t)]);
        })),
    }
}

pub fn fn_avg(expr: Expr) -> Expr {
    return Expr::Call {
        func: "avg".to_string(),
        args: vec![expr],
        compute_type: ComputeType(Rc::new(|ctx, path, args| {
            let Some(t) = args.get(0).unwrap().assert_scalar(&mut ctx.errs, path) else {
                return ExprType(vec![]);
            };
            return ExprType(vec![(ExprValName::empty(), t)]);
        })),
    }
}

pub fn fn_sum(expr: Expr) -> Expr {
    return Expr::Call {
        func: "sum".to_string(),
        args: vec![expr],
        compute_type: ComputeType(Rc::new(|ctx, path, args| {
            let Some(t) = args.get(0).unwrap().assert_scalar(&mut ctx.errs, path) else {
                return ExprType(vec![]);
            };
            let mut out_t = t;
            match out_t.type_.type_ {
                SimpleSimpleType::I16 | SimpleSimpleType::I32 | SimpleSimpleType::I64 => {
                    out_t.type_.type_ = SimpleSimpleType::I64;
                },
                _ => { },
            }
            return ExprType(vec![(ExprValName::empty(), out_t)]);
        })),
    }
}

pub fn fn_count(expr: Expr) -> Expr {
    return Expr::Call {
        func: "count".to_string(),
        args: vec![expr],
        compute_type: ComputeType(Rc::new(|_ctx, _path, _args| {
            return ExprType(vec![(ExprValName::empty(), Type {
                type_: SimpleType {
                    type_: SimpleSimpleType::I64,
                    custom: None,
                },
                opt: false,
                arr: false,
            })]);
        })),
    }
}
