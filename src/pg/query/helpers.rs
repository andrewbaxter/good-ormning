use crate::pg::schema::field::Field;
use super::expr::{
    Expr,
    BinOp,
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
pub fn eq_field(param_name: impl Into<String>, f: &Field) -> Expr {
    Expr::BinOp {
        left: Box::new(Expr::Field(f.clone())),
        op: BinOp::Equals,
        right: Box::new(Expr::Param {
            name: param_name.into(),
            type_: f.type_.type_.clone(),
        }),
    }
}

/// Generates an expression selecting field values greater than a corresponding
/// parameter
pub fn gt_field(param_name: impl Into<String>, f: &Field) -> Expr {
    Expr::BinOp {
        left: Box::new(Expr::Field(f.clone())),
        op: BinOp::GreaterThan,
        right: Box::new(Expr::Param {
            name: param_name.into(),
            type_: f.type_.type_.clone(),
        }),
    }
}

/// Generates an expression selecting field values greater than or equal to a
/// corresponding parameter
pub fn gte_field(param_name: impl Into<String>, f: &Field) -> Expr {
    Expr::BinOp {
        left: Box::new(Expr::Field(f.clone())),
        op: BinOp::GreaterThanEqualTo,
        right: Box::new(Expr::Param {
            name: param_name.into(),
            type_: f.type_.type_.clone(),
        }),
    }
}

/// Generates an expression selecting field values greater than a corresponding
/// parameter
pub fn lt_field(param_name: impl Into<String>, f: &Field) -> Expr {
    Expr::BinOp {
        left: Box::new(Expr::Field(f.clone())),
        op: BinOp::LessThan,
        right: Box::new(Expr::Param {
            name: param_name.into(),
            type_: f.type_.type_.clone(),
        }),
    }
}

/// Generates an expression selecting field values greater than or equal to a
/// corresponding parameter
pub fn lte_field(param_name: impl Into<String>, f: &Field) -> Expr {
    Expr::BinOp {
        left: Box::new(Expr::Field(f.clone())),
        op: BinOp::LessThanEqualTo,
        right: Box::new(Expr::Param {
            name: param_name.into(),
            type_: f.type_.type_.clone(),
        }),
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
