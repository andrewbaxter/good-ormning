use crate::pg::{
    FieldHandle,
};
use super::expr::{
    Expr,
    BinOp,
};

/// Generates a field element for instert and update statements, to set a field
/// from a parameter of the same type.
pub fn set_field(param_name: impl Into<String>, f: &FieldHandle) -> (FieldHandle, Expr) {
    (f.clone(), field_param(param_name, f))
}

/// Generates a param matching a field in name in type
pub fn field_param(param_name: impl Into<String>, f: &FieldHandle) -> Expr {
    let version = f.table.version.0.borrow();
    let type_ = version.tables.get(&f.table.schema_id).unwrap().fields.get(&f.schema_id).unwrap().type_.type_.clone();
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
