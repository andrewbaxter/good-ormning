use {
    crate::mariadb::query::expr::{
        NumericClass,
        date_unit,
        numeric_class,
    },
    crate::mariadb::{
        FieldHandle,
        types::{
            SimpleSimpleType,
            SimpleType,
            Temporal,
            Type,
            temporal,
        },
    },
    std::rc::Rc,
    super::expr::{
        BinOp,
        check_bool,
        check_general_same_type,
        ComputeType,
        Expr,
        ExprType,
        ExprValName,
    },
};

pub fn eq_field(param_name: impl Into<String>, f: &FieldHandle) -> Expr {
    return Expr::BinOp {
        left: Box::new(Expr::Field(f.to_ref())),
        op: BinOp::Equals,
        right: Box::new(field_param(param_name, f)),
    };
}

pub fn expr_and(exprs: Vec<Expr>) -> Expr {
    return Expr::BinOpChain {
        op: BinOp::And,
        exprs: exprs,
    };
}

pub fn expr_or(exprs: Vec<Expr>) -> Expr {
    return Expr::BinOpChain {
        op: BinOp::Or,
        exprs: exprs,
    };
}

pub fn in_field(param_name: impl Into<String>, f: &FieldHandle) -> Expr {
    let Expr::Param { name, mut type_ } = field_param(param_name, f) else {
        unreachable!();
    };
    type_.opt = false;
    return Expr::BinOp {
        left: Box::new(Expr::Field(f.to_ref())),
        op: BinOp::In,
        right: Box::new(Expr::ParamList {
            name: name,
            type_: type_,
        }),
    };
}

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
        compute_type: ComputeType(Rc::new(|ctx, path, args| {
            let t = match args.first().and_then(|a| a.0.first()) {
                Some(t) => t.1.clone(),
                None => {
                    return ExprType(vec![]);
                },
            };
            if numeric_class(&t.type_.type_).is_none() && t.type_.type_ != SimpleSimpleType::AnyNull {
                ctx.errs.err(path, format!("avg needs a number, but got {:?}", t.type_.type_));
            }
            let mut out_t = t;
            out_t.type_ = SimpleType {
                type_: match out_t.type_.type_ {
                    SimpleSimpleType::AnyFloat | SimpleSimpleType::F32 | SimpleSimpleType::F64 => SimpleSimpleType
                    ::F64,
                    _ => SimpleSimpleType::Decimal,
                },
                custom: None,
            };
            out_t.opt = true;
            return ExprType(vec![(ExprValName::empty(), out_t)]);
        })),
        distinct: false,
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
            })]);
        })),
        distinct: false,
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
            })]);
        })),
        distinct: false,
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
        distinct: false,
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
        distinct: false,
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
            })]);
        })),
        distinct: false,
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
            })]);
        })),
        distinct: false,
    }
}

pub fn fn_sum(expr: Expr) -> Expr {
    return Expr::Call {
        func: "sum".to_string(),
        args: vec![expr],
        compute_type: ComputeType(Rc::new(|ctx, path, args| {
            let t = match args.first().and_then(|a| a.0.first()) {
                Some(t) => t.1.clone(),
                None => {
                    return ExprType(vec![]);
                },
            };
            if numeric_class(&t.type_.type_).is_none() && t.type_.type_ != SimpleSimpleType::AnyNull {
                ctx.errs.err(path, format!("sum needs a number, but got {:?}", t.type_.type_));
            }
            let mut out_t = t;
            out_t.type_ = SimpleType {
                type_: match out_t.type_.type_ {
                    SimpleSimpleType::AnyFloat | SimpleSimpleType::F32 | SimpleSimpleType::F64 => SimpleSimpleType
                    ::F64,
                    _ => SimpleSimpleType::Decimal,
                },
                custom: None,
            };
            out_t.opt = true;
            return ExprType(vec![(ExprValName::empty(), out_t)]);
        })),
        distinct: false,
    }
}

pub fn gt_field(param_name: impl Into<String>, f: &FieldHandle) -> Expr {
    return Expr::BinOp {
        left: Box::new(Expr::Field(f.to_ref())),
        op: BinOp::GreaterThan,
        right: Box::new(field_param(param_name, f)),
    };
}

pub fn gte_field(param_name: impl Into<String>, f: &FieldHandle) -> Expr {
    return Expr::BinOp {
        left: Box::new(Expr::Field(f.to_ref())),
        op: BinOp::GreaterThanEqualTo,
        right: Box::new(field_param(param_name, f)),
    };
}

pub fn lt_field(param_name: impl Into<String>, f: &FieldHandle) -> Expr {
    return Expr::BinOp {
        left: Box::new(Expr::Field(f.to_ref())),
        op: BinOp::LessThan,
        right: Box::new(field_param(param_name, f)),
    };
}

pub fn lte_field(param_name: impl Into<String>, f: &FieldHandle) -> Expr {
    return Expr::BinOp {
        left: Box::new(Expr::Field(f.to_ref())),
        op: BinOp::LessThanEqualTo,
        right: Box::new(field_param(param_name, f)),
    };
}

pub fn set_field(param_name: impl Into<String>, f: &FieldHandle) -> (FieldHandle, Expr) {
    return (f.clone(), field_param(param_name, f));
}

pub fn fn_count_distinct(expr: Expr) -> Expr {
    let mut e = fn_count(expr);
    if let Expr::Call { distinct: ref mut d, .. } = e {
        *d = true;
    }
    return e;
}

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
        distinct: false,
    }
}

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
        distinct: false,
    }
}

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
            })]);
        })),
        distinct: false,
    }
}

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
            })]);
        })),
        distinct: false,
    }
}

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
                SimpleSimpleType::AnyInt | SimpleSimpleType::AnyFloat | SimpleSimpleType::I8 | SimpleSimpleType::I16 |
                    SimpleSimpleType::I32 |
                    SimpleSimpleType::I64 |
                    SimpleSimpleType::U8 |
                    SimpleSimpleType::U16 |
                    SimpleSimpleType::U32 |
                    SimpleSimpleType::U64 |
                    SimpleSimpleType::F32 |
                    SimpleSimpleType::F64 |
                    SimpleSimpleType::AnyDecimal |
                    SimpleSimpleType::Decimal |
                    SimpleSimpleType::AnyNull
            ) {
                ctx.errs.err(path, format!("abs requires a numeric argument, but got {:?}", t.type_.type_));
            }
            let mut t = t;
            if t.type_.type_ == SimpleSimpleType::F32 {
                t.type_ = SimpleType {
                    type_: SimpleSimpleType::F64,
                    custom: None,
                };
            }
            return ExprType(vec![(ExprValName::empty(), t)]);
        })),
        distinct: false,
    }
}

#[derive(Clone, Copy, Debug)]
pub enum FnOpt {
    Never,
    AnyArg,
    FirstArg,
    Always,
    AnyArgOrParsed,
}

pub fn fn_fixed(name: &str, args: Vec<Expr>, ret: SimpleSimpleType, opt: FnOpt) -> Expr {
    return Expr::Call {
        func: name.to_string(),
        args: args,
        compute_type: ComputeType(Rc::new(move |ctx, path, args| {
            let mut any_opt = false;
            let mut first_opt = false;
            let mut parsed = false;
            for (i, a) in args.iter().enumerate() {
                if let Some(t) = a.assert_scalar(&mut ctx.errs, &path.push_back(format!("Arg {}", i))) {
                    any_opt = any_opt || t.opt;
                    parsed = parsed || temporal(&t.type_.type_).is_none();
                    if i == 0 {
                        first_opt = t.opt;
                    }
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
                    FnOpt::FirstArg => first_opt,
                    FnOpt::Always => true,
                    FnOpt::AnyArgOrParsed => any_opt || parsed,
                },
            })]);
        })),
        distinct: false,
    }
}

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
            if out_t.type_.type_ == SimpleSimpleType::F32 {
                out_t.type_ = SimpleType {
                    type_: SimpleSimpleType::F64,
                    custom: None,
                };
            }
            return ExprType(vec![(ExprValName::empty(), out_t)]);
        })),
        distinct: false,
    }
}

pub fn fn_greatest_least(name: &str, args: Vec<Expr>) -> Expr {
    return Expr::Call {
        func: name.to_string(),
        args: args,
        compute_type: ComputeType(Rc::new(|ctx, path, args| {
            let mut out_t: Option<Type> = None;
            for (i, a) in args.iter().enumerate() {
                let Some(t) = a.assert_scalar(&mut ctx.errs, &path.push_back(format!("Arg {}", i))) else {
                    continue;
                };
                out_t = Some(match out_t {
                    Some(prev) => check_general_same_type(ctx, &path.push_back(format!("Arg {}", i)), &prev, &t),
                    None => t,
                });
            }
            let Some(out_t) = out_t else {
                ctx.errs.err(path, "This function requires at least one argument".to_string());
                return ExprType(vec![]);
            };
            return ExprType(vec![(ExprValName::empty(), out_t)]);
        })),
        distinct: false,
    }
}

pub fn fn_if(cond: Expr, a: Expr, b: Expr) -> Expr {
    return Expr::Call {
        func: "if".to_string(),
        args: vec![cond, a, b],
        compute_type: ComputeType(Rc::new(|ctx, path, args| {
            if args.len() != 3 {
                ctx.errs.err(path, "if requires exactly 3 arguments".to_string());
                return ExprType(vec![]);
            }
            check_bool(ctx, &path.push_back("Arg 0".to_string()), &args[0]);
            let Some(a) = args[1].assert_scalar(&mut ctx.errs, &path.push_back("Arg 1".to_string())) else {
                return ExprType(vec![]);
            };
            let Some(b) = args[2].assert_scalar(&mut ctx.errs, &path.push_back("Arg 2".to_string())) else {
                return ExprType(vec![]);
            };
            return ExprType(vec![(ExprValName::empty(), check_general_same_type(ctx, path, &a, &b))]);
        })),
        distinct: false,
    }
}

pub fn fn_values(field: Expr) -> Expr {
    let mut e = fn_same_as_first("values", vec![field]);
    if let Expr::Call { compute_type: ref mut c, .. } = e {
        let inner = c.0.clone();
        *c = ComputeType(Rc::new(move |ctx, path, args| {
            if !ctx.in_duplicate_key_update {
                ctx
                    .errs
                    .err(
                        path,
                        "`values()` can only be used in `on duplicate key update` (elsewhere MariaDB returns null)".to_string(),
                    );
            }
            return inner(ctx, path, args);
        }));
    }
    return e;
}

pub fn fn_window_value(name: &str, args: Vec<Expr>) -> Expr {
    return Expr::Call {
        func: name.to_string(),
        args: args,
        compute_type: ComputeType(Rc::new(|ctx, path, args| {
            let Some(mut t) = args.first().and_then(|a| a.assert_scalar(&mut ctx.errs, path)) else {
                ctx.errs.err(path, "This function requires at least one argument".to_string());
                return ExprType(vec![]);
            };
            t.opt = true;
            return ExprType(vec![(ExprValName::empty(), t)]);
        })),
        distinct: false,
    }
}

pub fn fn_mod(args: Vec<Expr>) -> Expr {
    return Expr::Call {
        func: "mod".to_string(),
        args: args,
        compute_type: ComputeType(Rc::new(|ctx, path, args| {
            let mut class = None;
            let mut unsigned = false;
            for (i, a) in args.iter().enumerate() {
                let Some(t) = a.assert_scalar(&mut ctx.errs, &path.push_back(format!("Arg {}", i))) else {
                    continue;
                };
                let Some(c) = numeric_class(&t.type_.type_) else {
                    ctx.errs.err(path, format!("mod requires numeric arguments, but got {:?}", t.type_.type_));
                    continue;
                };
                unsigned = unsigned || c == NumericClass::Unsigned;
                class = class.max(Some(c));
            }
            return ExprType(vec![(ExprValName::empty(), Type {
                type_: SimpleType {
                    type_: match class {
                        Some(NumericClass::Float) => SimpleSimpleType::F64,
                        Some(NumericClass::Decimal) => SimpleSimpleType::Decimal,
                        _ => if unsigned {
                            SimpleSimpleType::U64
                        } else {
                            SimpleSimpleType::I64
                        },
                    },
                    custom: None,
                },
                opt: true,
            })]);
        })),
        distinct: false,
    }
}

pub fn fn_date_add(name: &str, args: Vec<Expr>) -> Expr {
    let date_only = match args.get(1) {
        Some(Expr::Interval { unit, .. }) => date_unit(unit),
        _ => true,
    };
    return Expr::Call {
        func: name.to_string(),
        args: args,
        compute_type: ComputeType(Rc::new(move |ctx, path, args| {
            if args.len() != 2 {
                ctx.errs.err(path, format!("This function takes 2 arguments but got {}", args.len()));
                return ExprType(vec![]);
            }
            let Some(t) = args[0].assert_scalar(&mut ctx.errs, &path.push_back("Arg 0".to_string())) else {
                return ExprType(vec![]);
            };
            let type_ = match temporal(&t.type_.type_) {
                Some(Temporal::Date) if !date_only => SimpleType {
                    type_: SimpleSimpleType::AnyDatetime,
                    custom: None,
                },
                Some(_) => t.type_,
                None => SimpleType {
                    type_: SimpleSimpleType::String,
                    custom: None,
                },
            };
            return ExprType(vec![(ExprValName::empty(), Type {
                type_: type_,
                opt: true,
            })]);
        })),
        distinct: false,
    }
}

pub fn fn_string(name: &str, args: Vec<Expr>, opt: FnOpt) -> Expr {
    return Expr::Call {
        func: name.to_string(),
        args: args,
        compute_type: ComputeType(Rc::new(move |ctx, path, args| {
            let mut any_opt = false;
            let mut first_opt = false;
            let mut binary = false;
            for (i, a) in args.iter().enumerate() {
                if let Some(t) = a.assert_scalar(&mut ctx.errs, &path.push_back(format!("Arg {}", i))) {
                    any_opt = any_opt || t.opt;
                    if i == 0 {
                        first_opt = t.opt;
                    }
                    binary = binary || t.type_.type_ == SimpleSimpleType::Bytes;
                }
            }
            return ExprType(vec![(ExprValName::empty(), Type {
                type_: SimpleType {
                    type_: if binary {
                        SimpleSimpleType::Bytes
                    } else {
                        SimpleSimpleType::String
                    },
                    custom: None,
                },
                opt: match opt {
                    FnOpt::Never => false,
                    FnOpt::AnyArg | FnOpt::AnyArgOrParsed => any_opt,
                    FnOpt::FirstArg => first_opt,
                    FnOpt::Always => true,
                },
            })]);
        })),
        distinct: false,
    }
}
