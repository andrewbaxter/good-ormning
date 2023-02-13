use crate::pg::schema::field::Field;
use super::expr::{
    Expr,
    BinOp,
};

/// Generates a field element for instert and update statements, to set a field
/// from a parameter of the same type.
pub fn set_field(f: &Field) -> (Field, Expr) {
    (f.clone(), Expr::Param {
        name: f.id.clone(),
        type_: f.type_.type_.clone(),
    })
}

/// Generates an expression checking for equality of a field and a parameter and
/// the same type.
pub fn field_eq(f: &Field) -> Expr {
    Expr::BinOp {
        left: Box::new(Expr::Field(f.clone())),
        op: BinOp::Equals,
        right: Box::new(Expr::Param {
            name: f.id.clone(),
            type_: f.type_.type_.clone(),
        }),
    }
}

/// Shortcut for AND expressions.
pub fn expr_and(l: Expr, r: Expr) -> Expr {
    Expr::BinOp {
        left: Box::new(l),
        op: BinOp::And,
        right: Box::new(r),
    }
}
