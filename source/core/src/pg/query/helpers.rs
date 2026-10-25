use {
    crate::pg::{
        FieldHandle,
        types::{
            SimpleSimpleType,
            SimpleType,
            Type,
        },
    },
    std::rc::Rc,
    super::expr::{
        BinOp,
        check_general_same_type,
        ComputeType,
        Expr,
        ExprType,
        ExprValName,
    },
};

/// Generates an expression checking for equality of a field and a parameter and
/// the same type.
pub fn eq_field(param_name: impl Into<String>, f: &FieldHandle) -> Expr {
    return Expr::BinOp {
        left: Box::new(Expr::Field(f.to_ref())),
        op: BinOp::Equals,
        right: Box::new(field_param(param_name, f)),
    };
}

/// Shortcut for chain AND expressions.
pub fn expr_and(exprs: Vec<Expr>) -> Expr {
    return Expr::BinOpChain {
        op: BinOp::And,
        exprs: exprs,
    };
}

/// Shortcut for chain OR expressions.
pub fn expr_or(exprs: Vec<Expr>) -> Expr {
    return Expr::BinOpChain {
        op: BinOp::Or,
        exprs: exprs,
    };
}

/// Generates a param matching a field in name in type
pub fn field_param(param_name: impl Into<String>, f: &FieldHandle) -> Expr {
    let version = f.table.version.0.borrow();
    let type_ =
        version.as_ref().unwrap().tables.get(&f.table.id).unwrap().fields.get(&f.id).unwrap().type_.type_.clone();
    return Expr::Param {
        name: param_name.into(),
        type_: type_,
    };
}

pub fn fn_avg(expr: Expr) -> Expr {
    return Expr::Call {
        func: "avg".to_string(),
        args: vec![expr],
        compute_type: ComputeType(Rc::new(|_, _, args| {
            let t = match args.first().and_then(|a| a.0.first()) {
                Some(t) => t.1.clone(),
                None => {
                    return ExprType(vec![]);
                },
            };
            let mut out_t = t;
            out_t.opt = true;
            return ExprType(vec![(ExprValName::empty(), out_t)]);
        })),
        filter: None,
        distinct: false,
        order_by: vec![],
    }
}

pub fn fn_count(expr: Expr) -> Expr {
    return Expr::Call {
        func: "count".to_string(),
        args: vec![expr],
        compute_type: ComputeType(Rc::new(|_, _, _| {
            return ExprType(vec![(ExprValName::empty(), Type {
                type_: SimpleType {
                    type_: SimpleSimpleType::I64,
                    custom: None,
                },
                opt: false,
                arr: false,
            })]);
        })),
        filter: None,
        distinct: false,
        order_by: vec![],
    }
}

pub fn fn_dense_rank() -> Expr {
    return Expr::Call {
        func: "dense_rank".to_string(),
        args: vec![],
        compute_type: ComputeType(Rc::new(|_, _, _| {
            return ExprType(vec![(ExprValName::empty(), Type {
                type_: SimpleType {
                    type_: SimpleSimpleType::I64,
                    custom: None,
                },
                opt: false,
                arr: false,
            })]);
        })),
        filter: None,
        distinct: false,
        order_by: vec![],
    }
}

pub fn fn_max(expr: Expr) -> Expr {
    return Expr::Call {
        func: "max".to_string(),
        args: vec![expr],
        compute_type: ComputeType(Rc::new(|_, _, args| {
            let t = match args.first().and_then(|a| a.0.first()) {
                Some(t) => t.1.clone(),
                None => {
                    return ExprType(vec![]);
                },
            };
            let mut out_t = t;
            out_t.opt = true;
            return ExprType(vec![(ExprValName::empty(), out_t)]);
        })),
        filter: None,
        distinct: false,
        order_by: vec![],
    }
}

pub fn fn_min(expr: Expr) -> Expr {
    return Expr::Call {
        func: "min".to_string(),
        args: vec![expr],
        compute_type: ComputeType(Rc::new(|_, _, args| {
            let t = match args.first().and_then(|a| a.0.first()) {
                Some(t) => t.1.clone(),
                None => {
                    return ExprType(vec![]);
                },
            };
            let mut out_t = t;
            out_t.opt = true;
            return ExprType(vec![(ExprValName::empty(), out_t)]);
        })),
        filter: None,
        distinct: false,
        order_by: vec![],
    }
}

pub fn fn_rank() -> Expr {
    return Expr::Call {
        func: "rank".to_string(),
        args: vec![],
        compute_type: ComputeType(Rc::new(|_, _, _| {
            return ExprType(vec![(ExprValName::empty(), Type {
                type_: SimpleType {
                    type_: SimpleSimpleType::I64,
                    custom: None,
                },
                opt: false,
                arr: false,
            })]);
        })),
        filter: None,
        distinct: false,
        order_by: vec![],
    }
}

pub fn fn_row_number() -> Expr {
    return Expr::Call {
        func: "row_number".to_string(),
        args: vec![],
        compute_type: ComputeType(Rc::new(|_, _, _| {
            return ExprType(vec![(ExprValName::empty(), Type {
                type_: SimpleType {
                    type_: SimpleSimpleType::I64,
                    custom: None,
                },
                opt: false,
                arr: false,
            })]);
        })),
        filter: None,
        distinct: false,
        order_by: vec![],
    }
}

pub fn fn_sum(expr: Expr) -> Expr {
    return Expr::Call {
        func: "sum".to_string(),
        args: vec![expr],
        compute_type: ComputeType(Rc::new(|_, _, args| {
            let t = match args.first().and_then(|a| a.0.first()) {
                Some(t) => t.1.clone(),
                None => {
                    return ExprType(vec![]);
                },
            };
            let mut out_t = t;
            match out_t.type_.type_ {
                SimpleSimpleType::AnyInt |
                SimpleSimpleType::I16 |
                SimpleSimpleType::I32 |
                SimpleSimpleType::I64 |
                SimpleSimpleType::Auto => {
                    out_t.type_.type_ = SimpleSimpleType::I64;
                },
                _ => { },
            }
            out_t.opt = true;
            return ExprType(vec![(ExprValName::empty(), out_t)]);
        })),
        filter: None,
        distinct: false,
        order_by: vec![],
    }
}

/// Generates an expression selecting field values greater than a corresponding
/// parameter
pub fn gt_field(param_name: impl Into<String>, f: &FieldHandle) -> Expr {
    return Expr::BinOp {
        left: Box::new(Expr::Field(f.to_ref())),
        op: BinOp::GreaterThan,
        right: Box::new(field_param(param_name, f)),
    };
}

/// Generates an expression selecting field values greater than or equal to a
/// corresponding parameter
pub fn gte_field(param_name: impl Into<String>, f: &FieldHandle) -> Expr {
    return Expr::BinOp {
        left: Box::new(Expr::Field(f.to_ref())),
        op: BinOp::GreaterThanEqualTo,
        right: Box::new(field_param(param_name, f)),
    };
}

/// Generates an expression selecting field values greater than a corresponding
/// parameter
pub fn lt_field(param_name: impl Into<String>, f: &FieldHandle) -> Expr {
    return Expr::BinOp {
        left: Box::new(Expr::Field(f.to_ref())),
        op: BinOp::LessThan,
        right: Box::new(field_param(param_name, f)),
    };
}

/// Generates an expression selecting field values greater than or equal to a
/// corresponding parameter
pub fn lte_field(param_name: impl Into<String>, f: &FieldHandle) -> Expr {
    return Expr::BinOp {
        left: Box::new(Expr::Field(f.to_ref())),
        op: BinOp::LessThanEqualTo,
        right: Box::new(field_param(param_name, f)),
    };
}

/// Generates a field element for instert and update statements, to set a field
/// from a parameter of the same type.
pub fn set_field(param_name: impl Into<String>, f: &FieldHandle) -> (FieldHandle, Expr) {
    return (f.clone(), field_param(param_name, f));
}

/// `count(distinct x)`
pub fn fn_count_distinct(expr: Expr) -> Expr {
    let mut e = fn_count(expr);
    if let Expr::Call { distinct: ref mut d, .. } = e {
        *d = true;
    }
    return e;
}

/// `coalesce(a, b, ...)` - the result is the unified type of all arguments, and is
/// optional only if all arguments are optional.
pub fn fn_coalesce(args: Vec<Expr>) -> Expr {
    return Expr::Call {
        func: "coalesce".to_string(),
        args: args,
        compute_type: ComputeType(Rc::new(|ctx, path, args| {
            let mut out_t: Option<Type> = None;
            let mut all_opt = true;
            for (i, a) in args.iter().enumerate() {
                let Some(t) = a.assert_scalar(&mut ctx.errs, &path.push_back(format!("Arg {}", i))) else {
                    continue;
                };
                all_opt = all_opt && t.opt;
                out_t = Some(match out_t {
                    Some(prev) => check_general_same_type(ctx, &path.push_back(format!("Arg {}", i)), &prev, &t),
                    None => t,
                });
            }
            let Some(mut out_t) = out_t else {
                ctx.errs.err(path, "coalesce requires at least one argument".to_string());
                return ExprType(vec![]);
            };
            out_t.opt = all_opt;
            return ExprType(vec![(ExprValName::empty(), out_t)]);
        })),
        filter: None,
        distinct: false,
        order_by: vec![],
    }
}

/// `nullif(a, b)` - the result has the type of `a`, and is optional.
pub fn fn_nullif(a: Expr, b: Expr) -> Expr {
    return Expr::Call {
        func: "nullif".to_string(),
        args: vec![a, b],
        compute_type: ComputeType(Rc::new(|ctx, path, args| {
            if args.len() != 2 {
                ctx.errs.err(path, "nullif requires exactly 2 arguments".to_string());
                return ExprType(vec![]);
            }
            let Some(a) = args[0].assert_scalar(&mut ctx.errs, &path.push_back("Arg 0".to_string())) else {
                return ExprType(vec![]);
            };
            let Some(b) = args[1].assert_scalar(&mut ctx.errs, &path.push_back("Arg 1".to_string())) else {
                return ExprType(vec![]);
            };
            check_general_same_type(ctx, path, &a, &b);
            let mut out_t = a;
            out_t.opt = true;
            return ExprType(vec![(ExprValName::empty(), out_t)]);
        })),
        filter: None,
        distinct: false,
        order_by: vec![],
    }
}

/// A function taking a single string argument and returning a string (`lower`,
/// `upper`, `trim`, `ltrim`, `rtrim`), with the same optionality as the argument.
pub fn fn_str_to_str(func: &str, expr: Expr) -> Expr {
    return Expr::Call {
        func: func.to_string(),
        args: vec![expr],
        compute_type: ComputeType(Rc::new(|ctx, path, args| {
            let Some(t) = args.first().and_then(|a| a.assert_scalar(&mut ctx.errs, path)) else {
                return ExprType(vec![]);
            };
            if !matches!(t.type_.type_, SimpleSimpleType::String | SimpleSimpleType::AnyNull) {
                ctx.errs.err(path, format!("This function requires a string argument, but got {:?}", t.type_.type_));
            }
            return ExprType(vec![(ExprValName::empty(), Type {
                type_: SimpleType {
                    type_: SimpleSimpleType::String,
                    custom: None,
                },
                opt: t.opt,
                arr: false,
            })]);
        })),
        filter: None,
        distinct: false,
        order_by: vec![],
    }
}

/// `length(x)` - works on strings and bytes, returns an integer.
pub fn fn_length(expr: Expr) -> Expr {
    return Expr::Call {
        func: "length".to_string(),
        args: vec![expr],
        compute_type: ComputeType(Rc::new(|ctx, path, args| {
            let Some(t) = args.first().and_then(|a| a.assert_scalar(&mut ctx.errs, path)) else {
                return ExprType(vec![]);
            };
            if !matches!(
                t.type_.type_,
                SimpleSimpleType::String | SimpleSimpleType::Bytes | SimpleSimpleType::AnyNull
            ) {
                ctx
                    .errs
                    .err(path, format!("length requires a string or bytes argument, but got {:?}", t.type_.type_));
            }
            return ExprType(vec![(ExprValName::empty(), Type {
                type_: SimpleType {
                    type_: SimpleSimpleType::I32,
                    custom: None,
                },
                opt: t.opt,
                arr: false,
            })]);
        })),
        filter: None,
        distinct: false,
        order_by: vec![],
    }
}

/// `abs(x)` - same type as the argument.
pub fn fn_abs(expr: Expr) -> Expr {
    return Expr::Call {
        func: "abs".to_string(),
        args: vec![expr],
        compute_type: ComputeType(Rc::new(|ctx, path, args| {
            let Some(t) = args.first().and_then(|a| a.assert_scalar(&mut ctx.errs, path)) else {
                return ExprType(vec![]);
            };
            if !matches!(
                t.type_.type_,
                SimpleSimpleType::AnyInt | SimpleSimpleType::AnyFloat | SimpleSimpleType::I16 | SimpleSimpleType::I32 |
                    SimpleSimpleType::I64 |
                    SimpleSimpleType::U32 |
                    SimpleSimpleType::F32 |
                    SimpleSimpleType::F64 |
                    SimpleSimpleType::AnyNull
            ) {
                ctx.errs.err(path, format!("abs requires a numeric argument, but got {:?}", t.type_.type_));
            }
            return ExprType(vec![(ExprValName::empty(), t)]);
        })),
        filter: None,
        distinct: false,
        order_by: vec![],
    }
}

/// How a function's result optionality is determined from its arguments.
#[derive(Clone, Copy, Debug)]
pub enum FnOpt {
    /// Never null
    Never,
    /// Null if any argument is null
    AnyArg,
    /// May always be null (e.g. aggregates over no rows)
    Always,
}

/// A function with a fixed return type. Argument types aren't checked.
pub fn fn_fixed(name: &str, args: Vec<Expr>, ret: SimpleSimpleType, opt: FnOpt) -> Expr {
    return Expr::Call {
        func: name.to_string(),
        args: args,
        compute_type: ComputeType(Rc::new(move |ctx, path, args| {
            let mut any_opt = false;
            for (i, a) in args.iter().enumerate() {
                if let Some(t) = a.assert_scalar(&mut ctx.errs, &path.push_back(format!("Arg {}", i))) {
                    any_opt = any_opt || t.opt;
                }
            }
            return ExprType(vec![(ExprValName::empty(), Type {
                type_: SimpleType {
                    type_: ret.clone(),
                    custom: None,
                },
                opt: match opt {
                    FnOpt::Never => false,
                    FnOpt::AnyArg => any_opt,
                    FnOpt::Always => true,
                },
                arr: false,
            })]);
        })),
        filter: None,
        distinct: false,
        order_by: vec![],
    }
}

/// A function whose result has the same type as its first argument (like `abs`,
/// `round`), optional if any argument is optional.
pub fn fn_same_as_first(name: &str, args: Vec<Expr>) -> Expr {
    return Expr::Call {
        func: name.to_string(),
        args: args,
        compute_type: ComputeType(Rc::new(|ctx, path, args| {
            let mut any_opt = false;
            let mut first = None;
            for (i, a) in args.iter().enumerate() {
                if let Some(t) = a.assert_scalar(&mut ctx.errs, &path.push_back(format!("Arg {}", i))) {
                    any_opt = any_opt || t.opt;
                    if first.is_none() {
                        first = Some(t);
                    }
                }
            }
            let Some(mut out_t) = first else {
                ctx.errs.err(path, "This function requires at least one argument".to_string());
                return ExprType(vec![]);
            };
            out_t.opt = any_opt;
            return ExprType(vec![(ExprValName::empty(), out_t)]);
        })),
        filter: None,
        distinct: false,
        order_by: vec![],
    }
}
