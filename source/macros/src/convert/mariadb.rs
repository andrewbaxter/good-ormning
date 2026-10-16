use {
    crate::{
        GoodQueryInput,
    },
    good_ormning_core::{
        mariadb::{
            query::{
                expr::{
                    BinOp,
                    Expr,
                    PrefixOp,
                    TrimWhere,
                    WindowOver,
                    WindowSpec,
                },
                helpers::*,
                select::{
                    Join,
                    JoinCondition,
                    SelectFetch,
                    SelectLock,
                    JoinSource,
                    JoinType,
                    NamedSelectSource,
                    Order,
                    Select,
                },
                utils::{
                    CteBuilder,
                    MariadbTableInfo,
                    Returning,
                },
            },
            schema::{
                field::FieldRef,
                table::TableRef,
            },
            query::helpers::FnOpt,
            types::{
                SimpleSimpleType,
                SimpleSimpleType as MariadbSimpleSimpleType,
                SimpleType,
                SimpleType as MariadbSimpleType,
                Type as MariadbType,
            },
        },
    },
    sqlparser::ast as sql,
    std::collections::{
        HashMap,
        HashSet,
    },
};

pub struct ConvertState {
    pub param_locations: HashMap<(u64, u64), String>,
    pub used_params: HashSet<String>,
}

pub fn convert_expr(
    input: &GoodQueryInput,
    e: &sql::Expr,
    used_params: &mut ConvertState,
    custom_types: &std::collections::BTreeMap<String, good_ormning_core::mariadb::schema::custom_type::CustomType>,
    field_lookup: &std::collections::HashMap<TableRef, MariadbTableInfo>,
) -> Expr {
    match e {
        sql::Expr::Identifier(ident) => {
            return Expr::Field(FieldRef {
                table_id: "".into(),
                field_id: ident.value.clone(),
            });
        },
        sql::Expr::CompoundIdentifier(idents) => {
            if idents.len() != 2 {
                panic!("Only `table.column` references are supported: {}", e);
            }
            let table_id = idents[0].value.clone();
            let id = idents[1].value.clone();
            return Expr::Field(FieldRef {
                table_id: table_id,
                field_id: id,
            });
        },
        sql::Expr::Value(v) => {
            match &v.value {
                sql::Value::Number(n, _) => {
                    if let Ok(i) = n.parse::<i64>() {
                        return Expr::LitAnyInt(i);
                    } else if let Ok(i) = n.parse::<u64>() {
                        return Expr::LitU64(i);
                    } else if n.contains(['e', 'E']) {
                        return Expr::LitAnyFloat(
                            n
                                .parse::<f64>()
                                .unwrap_or_else(|e| panic!("Couldn't parse number literal {}: {}", n, e)),
                        );
                    } else if n.parse::<f64>().is_ok() {
                        return Expr::LitAnyDecimal(n.clone());
                    } else {
                        panic!("Couldn't parse number literal: {}", n)
                    }
                },
                sql::Value::SingleQuotedString(s) => return Expr::LitString(s.clone()),
                sql::Value::DoubleQuotedString(s) => return Expr::LitString(s.clone()),
                sql::Value::Boolean(b) => return Expr::LitBool(*b),
                sql::Value::HexStringLiteral(h) => {
                    return Expr::LitBytes(hex::decode(h).expect("Invalid hex string literal"));
                },
                sql::Value::Placeholder(_) => {
                    let (name, type_, arr) = convert_param(input, v, used_params, custom_types);
                    if arr {
                        panic!(
                            "Parameter {} is a list, which can only be used as the whole list in `in (?)` or `not in (?)`",
                            name
                        );
                    }
                    return Expr::Param {
                        name: name,
                        type_: type_,
                    };
                },
                sql::Value::Null => return Expr::LitNull(SimpleType {
                    type_: SimpleSimpleType::AnyNull,
                    custom: None,
                }),
                _ => panic!("Unsupported: Value type: {:?}", v),
            }
        },
        sql::Expr::BinaryOp { left, op, right } => {
            let l = convert_expr(input, left, used_params, custom_types, field_lookup);
            let r = convert_expr(input, right, used_params, custom_types, field_lookup);
            let o = match op {
                sql::BinaryOperator::Eq => BinOp::Equals,
                sql::BinaryOperator::NotEq => BinOp::NotEquals,
                sql::BinaryOperator::Lt => BinOp::LessThan,
                sql::BinaryOperator::LtEq => BinOp::LessThanEqualTo,
                sql::BinaryOperator::Gt => BinOp::GreaterThan,
                sql::BinaryOperator::GtEq => BinOp::GreaterThanEqualTo,
                sql::BinaryOperator::Spaceship => BinOp::NullSafeEquals,
                sql::BinaryOperator::And => BinOp::And,
                sql::BinaryOperator::Or => BinOp::Or,
                sql::BinaryOperator::Xor => BinOp::Xor,
                sql::BinaryOperator::Plus => BinOp::Plus,
                sql::BinaryOperator::Minus => BinOp::Minus,
                sql::BinaryOperator::Multiply => BinOp::Multiply,
                sql::BinaryOperator::Divide => BinOp::Divide,
                sql::BinaryOperator::MyIntegerDivide => BinOp::IntDivide,
                sql::BinaryOperator::Modulo => BinOp::Mod,
                sql::BinaryOperator::BitwiseAnd => BinOp::BitwiseAnd,
                sql::BinaryOperator::BitwiseOr => BinOp::BitwiseOr,
                sql::BinaryOperator::BitwiseXor => BinOp::BitwiseXor,
                sql::BinaryOperator::PGBitwiseShiftLeft => BinOp::BitwiseShiftLeft,
                sql::BinaryOperator::PGBitwiseShiftRight => BinOp::BitwiseShiftRight,
                sql::BinaryOperator::StringConcat => panic!(
                    "`||` means `or` in MariaDB (unless the `PIPES_AS_CONCAT` mode is set); use `or` or `concat(...)`"
                ),
                _ => panic!("Unsupported: BinaryOperator: {:?}", op),
            };
            return Expr::BinOp {
                left: Box::new(l),
                op: o,
                right: Box::new(r),
            };
        },
        sql::Expr::UnaryOp { op, expr } => {
            let op = match op {
                sql::UnaryOperator::Plus => return convert_expr(input, expr, used_params, custom_types, field_lookup),
                sql::UnaryOperator::Minus => {
                    if let sql::Expr::Value(sql::ValueWithSpan { value: sql::Value::Number(n, _), .. }) = &**expr {
                        if let Ok(i) = format!("-{}", n).parse::<i64>() {
                            return Expr::LitAnyInt(i);
                        }
                    }
                    PrefixOp::Minus
                },
                sql::UnaryOperator::Not => PrefixOp::Not,
                sql::UnaryOperator::PGBitwiseNot => PrefixOp::BitwiseNot,
                _ => panic!("Unsupported: UnaryOperator: {:?}", op),
            };
            return Expr::PrefixOp {
                op: op,
                right: Box::new(convert_expr(input, expr, used_params, custom_types, field_lookup)),
            };
        },
        sql::Expr::IsTrue(expr) => {
            return Expr::BinOp {
                left: Box::new(convert_expr(input, expr, used_params, custom_types, field_lookup)),
                op: BinOp::Is,
                right: Box::new(Expr::LitBool(true)),
            };
        },
        sql::Expr::IsNotTrue(expr) => {
            return Expr::BinOp {
                left: Box::new(convert_expr(input, expr, used_params, custom_types, field_lookup)),
                op: BinOp::IsNot,
                right: Box::new(Expr::LitBool(true)),
            };
        },
        sql::Expr::IsFalse(expr) => {
            return Expr::BinOp {
                left: Box::new(convert_expr(input, expr, used_params, custom_types, field_lookup)),
                op: BinOp::Is,
                right: Box::new(Expr::LitBool(false)),
            };
        },
        sql::Expr::IsNotFalse(expr) => {
            return Expr::BinOp {
                left: Box::new(convert_expr(input, expr, used_params, custom_types, field_lookup)),
                op: BinOp::IsNot,
                right: Box::new(Expr::LitBool(false)),
            };
        },
        sql::Expr::IsUnknown(expr) => {
            return Expr::BinOp {
                left: Box::new(convert_expr(input, expr, used_params, custom_types, field_lookup)),
                op: BinOp::Is,
                right: Box::new(Expr::LitNull(SimpleType {
                    type_: SimpleSimpleType::AnyNull,
                    custom: None,
                })),
            };
        },
        sql::Expr::IsNotUnknown(expr) => {
            return Expr::BinOp {
                left: Box::new(convert_expr(input, expr, used_params, custom_types, field_lookup)),
                op: BinOp::IsNot,
                right: Box::new(Expr::LitNull(SimpleType {
                    type_: SimpleSimpleType::AnyNull,
                    custom: None,
                })),
            };
        },
        sql::Expr::Between { expr, negated, low, high } => {
            return Expr::Between {
                e: Box::new(convert_expr(input, expr, used_params, custom_types, field_lookup)),
                negated: *negated,
                low: Box::new(convert_expr(input, low, used_params, custom_types, field_lookup)),
                high: Box::new(convert_expr(input, high, used_params, custom_types, field_lookup)),
            };
        },
        sql::Expr::Case { operand, conditions, else_result } => {
            let results = conditions.iter().map(|c| c.result.clone()).collect::<Vec<_>>();
            let conditions = conditions.iter().map(|c| c.condition.clone()).collect::<Vec<_>>();
            return Expr::Case {
                operand: operand
                    .as_ref()
                    .map(|e| Box::new(convert_expr(input, e, used_params, custom_types, field_lookup))),
                conditions: conditions
                    .iter()
                    .zip(results.iter())
                    .map(
                        |(c, r)| (
                            convert_expr(input, c, used_params, custom_types, field_lookup),
                            convert_expr(input, r, used_params, custom_types, field_lookup),
                        ),
                    )
                    .collect(),
                else_: else_result
                    .as_ref()
                    .map(|e| Box::new(convert_expr(input, e, used_params, custom_types, field_lookup))),
            };
        },
        sql::Expr::Like { expr, negated, pattern, escape_char, .. } => {
            let l = convert_expr(input, expr, used_params, custom_types, field_lookup);
            let r = convert_expr(input, pattern, used_params, custom_types, field_lookup);
            let escape = escape_char.as_ref().map(|c| Box::new(Expr::LitString(c.to_string())));
            return Expr::Like {
                expr: Box::new(l),
                negated: *negated,
                pattern: Box::new(r),
                escape: escape,
            };
        },
        sql::Expr::ILike { .. } => {
            panic!("MariaDB doesn't have `ilike`; `like` is case insensitive with case insensitive collations")
        },
        sql::Expr::RLike { expr, negated, pattern, .. } => {
            return Expr::BinOp {
                left: Box::new(convert_expr(input, expr, used_params, custom_types, field_lookup)),
                op: if *negated {
                    BinOp::NotRegexp
                } else {
                    BinOp::Regexp
                },
                right: Box::new(convert_expr(input, pattern, used_params, custom_types, field_lookup)),
            };
        },
        sql::Expr::IsNull(expr) => {
            return Expr::BinOp {
                left: Box::new(convert_expr(input, expr, used_params, custom_types, field_lookup)),
                op: BinOp::Is,
                right: Box::new(Expr::LitNull(SimpleType {
                    type_: SimpleSimpleType::AnyNull,
                    custom: None,
                })),
            };
        },
        sql::Expr::IsNotNull(expr) => {
            return Expr::BinOp {
                left: Box::new(convert_expr(input, expr, used_params, custom_types, field_lookup)),
                op: BinOp::IsNot,
                right: Box::new(Expr::LitNull(SimpleType {
                    type_: SimpleSimpleType::AnyNull,
                    custom: None,
                })),
            };
        },
        sql::Expr::IsDistinctFrom(..) | sql::Expr::IsNotDistinctFrom(..) => {
            panic!("MariaDB doesn't have `is [not] distinct from`, use `<=>` instead")
        },
        sql::Expr::Tuple(exprs) => {
            return Expr::LitArray(
                exprs.iter().map(|e| convert_expr(input, e, used_params, custom_types, field_lookup)).collect(),
            );
        },
        sql::Expr::Nested(expr) => {
            return Expr::Paren(Box::new(convert_expr(input, expr, used_params, custom_types, field_lookup)));
        },
        sql::Expr::Cast { kind, expr, data_type, format } => {
            if *kind != sql::CastKind::Cast || format.is_some() {
                panic!("MariaDB only supports `cast(x as type)`: {}", e);
            }
            let e = convert_expr(input, expr, used_params, custom_types, field_lookup);
            let mut type_args = vec![];
            let t = {
                let (simple_type, custom) = match data_type {
                    sqlparser::ast::DataType::Signed | sqlparser::ast::DataType::SignedInteger => (
                        MariadbSimpleSimpleType::I64,
                        None,
                    ),
                    sqlparser::ast::DataType::Unsigned | sqlparser::ast::DataType::UnsignedInteger => (
                        MariadbSimpleSimpleType::U64,
                        None,
                    ),
                    sqlparser::ast::DataType::Float(_) => (MariadbSimpleSimpleType::F32, None),
                    sqlparser::ast::DataType::Double(_) | sqlparser::ast::DataType::DoublePrecision => (
                        MariadbSimpleSimpleType::F64,
                        None,
                    ),
                    sqlparser::ast::DataType::Char(len) | sqlparser::ast::DataType::Character(len) => {
                        match len {
                            Some(sqlparser::ast::CharacterLength::IntegerLength { length, unit: None }) => {
                                type_args.push(*length as u32);
                            },
                            None => { },
                            Some(_) => panic!("Unsupported cast type for MariaDB: {}", data_type),
                        }
                        (MariadbSimpleSimpleType::String, None)
                    },
                    sqlparser::ast::DataType::Binary(len) => {
                        type_args.extend(len.map(|l| l as u32));
                        (MariadbSimpleSimpleType::Bytes, None)
                    },
                    sqlparser::ast::DataType::Datetime(precision) => {
                        type_args.extend(precision.map(|p| p as u32));
                        (MariadbSimpleSimpleType::AnyDatetime, None)
                    },
                    sqlparser::ast::DataType::Date => (MariadbSimpleSimpleType::AnyDate, None),
                    sqlparser::ast::DataType::Time(precision, sqlparser::ast::TimezoneInfo::None) => {
                        type_args.extend(precision.map(|p| p as u32));
                        (MariadbSimpleSimpleType::AnyTime, None)
                    },
                    sqlparser::ast::DataType::Decimal(info) | sqlparser::ast::DataType::Dec(info) => {
                        match info {
                            sqlparser::ast::ExactNumberInfo::None => { },
                            sqlparser::ast::ExactNumberInfo::Precision(p) => {
                                type_args.push(*p as u32);
                            },
                            sqlparser::ast::ExactNumberInfo::PrecisionAndScale(p, s) => {
                                type_args.push(*p as u32);
                                type_args.push(*s as u32);
                            },
                        }
                        (MariadbSimpleSimpleType::Decimal, None)
                    },
                    sqlparser::ast::DataType::Custom(name, ..) => {
                        let name_str = name.to_string();
                        let Some(ct) = custom_types.get(&name_str) else {
                            panic!("Unsupported cast type for MariaDB: {}", data_type);
                        };
                        (ct.base_type.type_.type_.clone(), Some(ct.rust_type.clone()))
                    },
                    _ => panic!("Unsupported cast type for MariaDB: {}", data_type),
                };
                MariadbType {
                    type_: MariadbSimpleType {
                        type_: simple_type,
                        custom: custom,
                    },
                    opt: false,
                }
            };
            return Expr::Cast {
                expr: Box::new(e),
                type_: t,
                type_args: type_args,
            };
        },
        sql::Expr::Collate { expr, collation } => {
            return Expr::Collate(
                Box::new(convert_expr(input, expr, used_params, custom_types, field_lookup)),
                collation.0.iter().map(|i| match i {
                    sqlparser::ast::ObjectNamePart::Identifier(id) => id.value.clone(),
                }).collect::<Vec<_>>().join("."),
            );
        },
        sql::Expr::InSubquery { expr, subquery, negated } => {
            let l = convert_expr(input, expr, used_params, custom_types, field_lookup);
            let dummy_q = sqlparser::ast::Query {
                with: None,
                body: subquery.clone(),
                order_by: None,
                limit_clause: None,
                fetch: None,
                locks: vec![],
                for_clause: None,
                settings: None,
                format_clause: None,
            };
            let r = convert_select_query(input, &dummy_q, used_params, custom_types, field_lookup);
            return Expr::BinOp {
                left: Box::new(l),
                op: if *negated {
                    BinOp::NotIn
                } else {
                    BinOp::In
                },
                right: Box::new(Expr::Select(Box::new(r))),
            };
        },
        sql::Expr::InList { expr, list, negated } => {
            let l = convert_expr(input, expr, used_params, custom_types, field_lookup);
            let r = match list.as_slice() {
                [sql::Expr::Value(v @ sql::ValueWithSpan { value: sql::Value::Placeholder(_), .. })] => {
                    let (name, type_, arr) = convert_param(input, v, used_params, custom_types);
                    if arr {
                        Expr::ParamList {
                            name: name,
                            type_: type_,
                        }
                    } else {
                        Expr::LitArray(vec![Expr::Param {
                            name: name,
                            type_: type_,
                        }])
                    }
                },
                _ => Expr::LitArray(
                    list.iter().map(|e| convert_expr(input, e, used_params, custom_types, field_lookup)).collect(),
                ),
            };
            return Expr::BinOp {
                left: Box::new(l),
                op: if *negated {
                    BinOp::NotIn
                } else {
                    BinOp::In
                },
                right: Box::new(r),
            };
        },
        sql::Expr::Subquery(q) => {
            return Expr::Select(Box::new(convert_select_query(input, q, used_params, custom_types, field_lookup)));
        },
        sql::Expr::Exists { subquery, negated } => {
            let res =
                Expr::Exists(
                    Box::new(convert_select_query(input, subquery, used_params, custom_types, field_lookup)),
                );
            if *negated {
                return Expr::PrefixOp {
                    op: PrefixOp::Not,
                    right: Box::new(res),
                };
            } else {
                return res;
            }
        },
        sql::Expr::Function(f) => {
            let name = f.name.to_string().to_lowercase();
            let mut args = vec![];
            let mut distinct = false;
            let mut group_concat_order = vec![];
            let mut group_concat_separator = None;
            let mut group_concat_limit = None;
            if let sql::FunctionArguments::List(list) = &f.args {
                for clause in &list.clauses {
                    match clause {
                        sql::FunctionArgumentClause::OrderBy(exprs) if name == "group_concat" => {
                            for o in exprs {
                                if o.options.nulls_first.is_some() {
                                    panic!("MariaDB doesn't support `nulls first`/`nulls last`");
                                }
                                group_concat_order.push(
                                    (
                                        convert_expr(input, &o.expr, used_params, custom_types, field_lookup),
                                        match o.options.asc {
                                            Some(true) | None => Order::Asc,
                                            Some(false) => Order::Desc,
                                        },
                                    ),
                                );
                            }
                        },
                        sql::FunctionArgumentClause::Separator(sql::Value::SingleQuotedString(sep)) if
                            name == "group_concat" => {
                            group_concat_separator = Some(sep.clone());
                        },
                        sql::FunctionArgumentClause::Limit(e) if name == "group_concat" => {
                            group_concat_limit =
                                Some(Box::new(convert_expr(input, e, used_params, custom_types, field_lookup)));
                        },
                        _ => panic!("Unsupported function argument clause in {}: {}", name, clause),
                    }
                }
                distinct = matches!(list.duplicate_treatment, Some(sql::DuplicateTreatment::Distinct));
                for arg in &list.args {
                    match arg {
                        sql::FunctionArg::Unnamed(sql::FunctionArgExpr::Expr(e)) => args.push(
                            convert_expr(input, e, used_params, custom_types, field_lookup),
                        ),
                        sql::FunctionArg::Unnamed(sql::FunctionArgExpr::Wildcard) => {
                            if name != "count" {
                                panic!("`*` is only allowed as the argument to `count`");
                            }
                            args.push(Expr::Star);
                        },
                        _ => panic!("Unsupported: FunctionArg: {:?}", arg),
                    }
                }
            }
            if f.filter.is_some() {
                panic!("MariaDB doesn't support `filter (where ...)` on aggregates");
            }
            if !f.within_group.is_empty() || f.null_treatment.is_some() ||
                !matches!(f.parameters, sql::FunctionArguments::None) {
                panic!("Unsupported function syntax: {}", f);
            }
            if name == "group_concat" {
                if f.over.is_some() {
                    panic!("MariaDB doesn't support `group_concat` as a window function");
                }
                return Expr::GroupConcat {
                    distinct: distinct,
                    args: args,
                    order_by: group_concat_order,
                    separator: group_concat_separator,
                    limit: group_concat_limit,
                };
            }
            if let Some(over) = &f.over {
                if distinct {
                    panic!("MariaDB doesn't support `distinct` in window functions");
                }
                let e = match name.as_str() {
                    "sum" | "count" | "min" | "max" | "avg" => convert_function(&name, args, distinct),
                    "row_number" => fn_row_number(),
                    "rank" => fn_rank(),
                    "dense_rank" => fn_dense_rank(),
                    "lag" | "lead" => {
                        if args.len() > 2 {
                            panic!("MariaDB's {} takes at most 2 arguments (no default)", name);
                        }
                        fn_window_value(&name, args)
                    },
                    "first_value" | "last_value" | "nth_value" => fn_window_value(&name, args),
                    "ntile" => fn_fixed(&name, args, SimpleSimpleType::I64, FnOpt::Never),
                    "percent_rank" | "cume_dist" => fn_fixed(&name, args, SimpleSimpleType::F64, FnOpt::Never),
                    "std" | "stddev" | "stddev_pop" | "stddev_samp" | "variance" | "var_pop" | "var_samp" => {
                        convert_function(&name, args, distinct)
                    },
                    _ => panic!("Unsupported window function: {}", name),
                };
                match over {
                    sql::WindowType::WindowSpec(spec) => {
                        return Expr::Window {
                            expr: Box::new(e),
                            over: WindowOver::Spec(
                                convert_window_spec(input, spec, used_params, custom_types, field_lookup),
                            ),
                        };
                    },
                    sql::WindowType::NamedWindow(name) => {
                        return Expr::Window {
                            expr: Box::new(e),
                            over: WindowOver::Named(name.value.clone()),
                        };
                    },
                }
            }
            return convert_function(&name, args, distinct);
        },
        sql::Expr::Substring { expr, substring_from, substring_for, .. } => {
            let mut args = vec![convert_expr(input, expr, used_params, custom_types, field_lookup)];
            let Some(from) = substring_from else {
                panic!("`substring` requires a start position: {}", e);
            };
            args.push(convert_expr(input, from, used_params, custom_types, field_lookup));
            if let Some(for_) = substring_for {
                args.push(convert_expr(input, for_, used_params, custom_types, field_lookup));
            }
            return fn_string("substring", args, FnOpt::AnyArg);
        },
        sql::Expr::Trim { expr, trim_where, trim_what, trim_characters } => {
            if trim_characters.is_some() {
                panic!("Unsupported trim syntax: {}", e);
            }
            return Expr::Trim {
                where_: trim_where.as_ref().map(|w| match w {
                    sql::TrimWhereField::Both => TrimWhere::Both,
                    sql::TrimWhereField::Leading => TrimWhere::Leading,
                    sql::TrimWhereField::Trailing => TrimWhere::Trailing,
                }),
                what: trim_what
                    .as_ref()
                    .map(|w| Box::new(convert_expr(input, w, used_params, custom_types, field_lookup))),
                expr: Box::new(convert_expr(input, expr, used_params, custom_types, field_lookup)),
            };
        },
        sql::Expr::Position { expr, r#in } => {
            let args =
                vec![
                    convert_expr(input, r#in, used_params, custom_types, field_lookup),
                    convert_expr(input, expr, used_params, custom_types, field_lookup),
                ];
            return fn_fixed("instr", args, SimpleSimpleType::I64, FnOpt::AnyArg);
        },
        sql::Expr::Ceil { expr, field } | sql::Expr::Floor { expr, field } => {
            if !matches!(field, sql::CeilFloorKind::DateTimeField(sql::DateTimeField::NoDateTime)) {
                panic!("Unsupported ceil/floor syntax: {}", e);
            }
            let name = match e {
                sql::Expr::Ceil { .. } => "ceil",
                _ => "floor",
            };
            let args = vec![convert_expr(input, expr, used_params, custom_types, field_lookup)];
            return fn_same_as_first(name, args);
        },
        sql::Expr::Interval(i) => {
            if i.last_field.is_some() || i.leading_precision.is_some() || i.fractional_seconds_precision.is_some() {
                panic!("Unsupported interval syntax: {}", e);
            }
            let Some(unit) = &i.leading_field else {
                panic!("MariaDB intervals need a unit: {}", e);
            };
            return Expr::Interval {
                value: Box::new(convert_expr(input, &i.value, used_params, custom_types, field_lookup)),
                unit: unit.to_string().to_lowercase(),
            };
        },
        sql::Expr::Extract { field, syntax: sql::ExtractSyntax::From, expr } => {
            return Expr::Extract {
                unit: field.to_string().to_lowercase(),
                expr: Box::new(convert_expr(input, expr, used_params, custom_types, field_lookup)),
            };
        },
        sql::Expr::TypedString { data_type, value } => {
            let sql::Value::SingleQuotedString(v) = value else {
                panic!("Unsupported typed literal: {}", e);
            };
            match data_type {
                sql::DataType::Timestamp(None, sql::TimezoneInfo::None) => return Expr::LitTimestamp(v.clone()),
                sql::DataType::Date => return Expr::LitDate(v.clone()),
                sql::DataType::Time(None, sql::TimezoneInfo::None) => return Expr::LitTime(v.clone()),
                _ => panic!("Unsupported typed literal: {}", e),
            }
        },
        _ => panic!("Unsupported: Expr type: {:?}", e),
    }
}

fn convert_param(
    input: &GoodQueryInput,
    v: &sql::ValueWithSpan,
    used_params: &mut ConvertState,
    custom_types: &std::collections::BTreeMap<String, good_ormning_core::mariadb::schema::custom_type::CustomType>,
) -> (String, MariadbType, bool) {
    let sql::Value::Placeholder(p) = &v.value else {
        panic!("Expected a parameter: {}", v);
    };
    let param_name =
        used_params
            .param_locations
            .get(&(v.span.start.line, v.span.start.column))
            .unwrap_or_else(|| panic!("Unsupported parameter `{}`", p))
            .clone();
    used_params.used_params.insert(param_name.clone());
    let pt = input.param_types.iter().find(|(ident, _)| {
        let s = ident.to_string();
        let s = s.trim_start_matches("r#");
        return s == param_name;
    }).map(|(_, pt)| pt).unwrap_or_else(|| panic!("Parameter {} not found in params section", param_name));
    let type_ = {
        let (simple_type, custom) = match pt.base.as_str() {
            "i8" => (MariadbSimpleSimpleType::I8, None),
            "i16" => (MariadbSimpleSimpleType::I16, None),
            "i32" => (MariadbSimpleSimpleType::I32, None),
            "i64" => (MariadbSimpleSimpleType::I64, None),
            "u8" => (MariadbSimpleSimpleType::U8, None),
            "u16" => (MariadbSimpleSimpleType::U16, None),
            "u32" => (MariadbSimpleSimpleType::U32, None),
            "u64" => (MariadbSimpleSimpleType::U64, None),
            "f32" => (MariadbSimpleSimpleType::F32, None),
            "f64" => (MariadbSimpleSimpleType::F64, None),
            "decimal" => (MariadbSimpleSimpleType::Decimal, None),
            "bool" => (MariadbSimpleSimpleType::Bool, None),
            "string" => (MariadbSimpleSimpleType::String, None),
            "bytes" => (MariadbSimpleSimpleType::Bytes, None),
            #[cfg(feature = "chrono")]
            "datetime_chrono" => (MariadbSimpleSimpleType::DatetimeChrono, None),
            #[cfg(feature = "jiff")]
            "datetime_jiff" => (MariadbSimpleSimpleType::DatetimeJiff, None),
            #[cfg(feature = "chrono")]
            "date_chrono" => (MariadbSimpleSimpleType::DateChrono, None),
            #[cfg(feature = "jiff")]
            "date_jiff" => (MariadbSimpleSimpleType::DateJiff, None),
            #[cfg(feature = "chrono")]
            "time_chrono" => (MariadbSimpleSimpleType::TimeChrono, None),
            #[cfg(feature = "jiff")]
            "time_jiff" => (MariadbSimpleSimpleType::TimeJiff, None),
            _ => {
                let Some(ct) = custom_types.get(&pt.base) else {
                    panic!(
                        "Parameter {} has unknown type {}; use i8, i16, i32, i64, u8, u16, u32, u64, f32, f64, decimal, bool, string, bytes, a date/datetime/time type for an enabled time library, or a custom type from the schema",
                        param_name,
                        pt.base
                    );
                };
                (ct.base_type.type_.type_.clone(), Some(ct.rust_type.clone()))
            },
        };
        MariadbType {
            type_: MariadbSimpleType {
                type_: simple_type,
                custom: custom,
            },
            opt: pt.opt,
        }
    };
    return (param_name, type_, pt.arr);
}

fn convert_function(name: &str, mut args: Vec<Expr>, distinct: bool) -> Expr {
    let arg1 = |args: &mut Vec<Expr>, name: &str| -> Expr {
        if args.len() != 1 {
            panic!("Function {} requires exactly 1 argument, but got {}", name, args.len());
        }
        return args.pop().unwrap();
    };
    let mut e = match name {
        "sum" => fn_sum(arg1(&mut args, name)),
        "count" if distinct && args.len() > 1 => fn_fixed(name, args, SimpleSimpleType::I64, FnOpt::Never),
        "count" => fn_count(arg1(&mut args, name)),
        "min" => fn_min(arg1(&mut args, name)),
        "max" => fn_max(arg1(&mut args, name)),
        "avg" => fn_avg(arg1(&mut args, name)),
        "coalesce" | "ifnull" => {
            let mut e = fn_coalesce(args);
            if let Expr::Call { func: ref mut f, .. } = e {
                *f = name.to_string();
            }
            e
        },
        "greatest" | "least" => fn_greatest_least(name, args),
        "if" => {
            if args.len() != 3 {
                panic!("Function if requires exactly 3 arguments, but got {}", args.len());
            }
            let c = args.pop().unwrap();
            let b = args.pop().unwrap();
            let a = args.pop().unwrap();
            fn_if(a, b, c)
        },
        "nullif" => {
            if args.len() != 2 {
                panic!("Function nullif requires exactly 2 arguments, but got {}", args.len());
            }
            let b = args.pop().unwrap();
            let a = args.pop().unwrap();
            fn_nullif(a, b)
        },
        "values" | "value" => {
            fn_values(arg1(&mut args, name))
        },
        "lower" | "upper" | "lcase" | "ucase" | "trim" | "ltrim" | "rtrim" => fn_str_to_str(
            name,
            arg1(&mut args, name),
        ),
        "length" | "octet_length" | "char_length" | "character_length" => {
            let mut e = fn_length(arg1(&mut args, name));
            if let Expr::Call { func: ref mut f, .. } = e {
                *f = name.to_string();
            }
            e
        },
        "abs" => fn_abs(arg1(&mut args, name)),
        "round" | "floor" | "ceil" | "ceiling" | "truncate" => fn_same_as_first(name, args),
        "mod" => fn_mod(args),
        "sign" => fn_fixed(name, args, SimpleSimpleType::I32, FnOpt::AnyArg),
        "pow" | "power" | "exp" | "sin" | "cos" | "tan" | "atan" | "atan2" | "degrees" | "radians" => fn_fixed(
            name,
            args,
            SimpleSimpleType::F64,
            FnOpt::AnyArg,
        ),
        "sqrt" | "ln" | "log" | "log2" | "log10" | "asin" | "acos" => fn_fixed(
            name,
            args,
            SimpleSimpleType::F64,
            FnOpt::Always,
        ),
        "rand" | "pi" => fn_fixed(name, args, SimpleSimpleType::F64, FnOpt::Never),
        "std" | "stddev" | "stddev_pop" | "stddev_samp" | "variance" | "var_pop" | "var_samp" => fn_fixed(
            name,
            args,
            SimpleSimpleType::F64,
            FnOpt::Always,
        ),
        "substr" |
        "substring" |
        "replace" |
        "concat" |
        "left" |
        "right" |
        "repeat" |
        "reverse" |
        "lpad" |
        "rpad" => fn_string(
            name,
            args,
            FnOpt::AnyArg,
        ),
        "space" | "hex" | "md5" | "sha1" | "sha2" | "to_base64" | "format" | "date_format" => fn_fixed(
            name,
            args,
            SimpleSimpleType::String,
            FnOpt::AnyArg,
        ),
        "concat_ws" => fn_fixed(name, args, SimpleSimpleType::String, FnOpt::FirstArg),
        "uuid" => fn_fixed(name, args, SimpleSimpleType::String, FnOpt::Never),
        "instr" | "locate" | "find_in_set" => fn_fixed(name, args, SimpleSimpleType::I64, FnOpt::AnyArg),
        "last_insert_id" => fn_fixed(name, args, SimpleSimpleType::U64, FnOpt::Never),
        "row_count" => fn_fixed(name, args, SimpleSimpleType::I64, FnOpt::Never),
        "unix_timestamp" => if args.is_empty() {
            fn_fixed(name, args, SimpleSimpleType::I64, FnOpt::Never)
        } else {
            fn_fixed(name, args, SimpleSimpleType::Decimal, FnOpt::Always)
        },
        "from_unixtime" => fn_fixed(name, args, SimpleSimpleType::AnyDatetime, FnOpt::Always),
        "date_add" | "date_sub" | "adddate" | "subdate" => fn_date_add(name, args),
        "curdate" | "current_date" | "utc_date" => fn_fixed(name, args, SimpleSimpleType::AnyDate, FnOpt::Never),
        "curtime" | "current_time" | "utc_time" => fn_fixed(name, args, SimpleSimpleType::AnyTime, FnOpt::Never),
        "date" | "last_day" | "from_days" => fn_fixed(name, args, SimpleSimpleType::AnyDate, FnOpt::AnyArgOrParsed),
        "time" | "timediff" => fn_fixed(name, args, SimpleSimpleType::AnyTime, FnOpt::AnyArgOrParsed),
        "makedate" => fn_fixed(name, args, SimpleSimpleType::AnyDate, FnOpt::Always),
        "maketime" => fn_fixed(name, args, SimpleSimpleType::AnyTime, FnOpt::Always),
        "sec_to_time" => fn_fixed(name, args, SimpleSimpleType::AnyTime, FnOpt::AnyArg),
        "year" |
        "month" |
        "day" |
        "dayofmonth" |
        "hour" |
        "minute" |
        "second" |
        "microsecond" |
        "dayofweek" |
        "dayofyear" |
        "weekday" |
        "quarter" |
        "week" |
        "weekofyear" |
        "yearweek" |
        "to_days" |
        "datediff" => fn_fixed(
            name,
            args,
            SimpleSimpleType::I32,
            FnOpt::AnyArgOrParsed,
        ),
        "dayname" | "monthname" => fn_fixed(name, args, SimpleSimpleType::String, FnOpt::AnyArgOrParsed),
        "now" | "current_timestamp" | "localtimestamp" | "localtime" | "sysdate" | "utc_timestamp" => fn_fixed(
            name,
            args,
            SimpleSimpleType::AnyDatetime,
            FnOpt::Never,
        ),
        _ => panic!(
            "Unsupported function: {} (supported functions: sum, count, min, max, avg, group_concat, std, stddev, stddev_pop, stddev_samp, variance, var_pop, var_samp, coalesce, ifnull, if, greatest, least, nullif, values, lower, upper, lcase, ucase, trim, ltrim, rtrim, length, octet_length, char_length, character_length, abs, round, floor, ceil, ceiling, truncate, mod, sign, sqrt, pow, power, exp, ln, log, log2, log10, sin, cos, tan, asin, acos, atan, atan2, degrees, radians, rand, pi, substr, substring, replace, concat, concat_ws, left, right, repeat, reverse, lpad, rpad, space, hex, md5, sha1, sha2, to_base64, format, date_format, uuid, instr, locate, find_in_set, last_insert_id, row_count, unix_timestamp, from_unixtime, date_add, date_sub, adddate, subdate, curdate, current_date, utc_date, curtime, current_time, utc_time, date, last_day, from_days, time, timediff, makedate, maketime, sec_to_time, year, month, day, dayofmonth, hour, minute, second, microsecond, dayofweek, dayofyear, weekday, quarter, week, weekofyear, yearweek, to_days, datediff, dayname, monthname, now, current_timestamp, localtimestamp, localtime, sysdate, utc_timestamp, row_number, rank, dense_rank, lag, lead, first_value, last_value)",
            name
        ),
    };
    if distinct {
        if !matches!(name, "sum" | "count" | "min" | "max" | "avg") {
            panic!("`distinct` is only supported in aggregate functions, not {}", name);
        }
        if let Expr::Call { distinct: ref mut d, .. } = e {
            *d = true;
        }
    }
    return e;
}

fn convert_window_spec(
    input: &GoodQueryInput,
    spec: &sql::WindowSpec,
    used_params: &mut ConvertState,
    custom_types: &std::collections::BTreeMap<String, good_ormning_core::mariadb::schema::custom_type::CustomType>,
    field_lookup: &std::collections::HashMap<TableRef, MariadbTableInfo>,
) -> WindowSpec {
    let mut partition_by = vec![];
    for p in &spec.partition_by {
        partition_by.push(convert_expr(input, p, used_params, custom_types, field_lookup));
    }
    let mut order_by = vec![];
    for o in &spec.order_by {
        let expr = convert_expr(input, &o.expr, used_params, custom_types, field_lookup);
        if o.options.nulls_first.is_some() {
            panic!("MariaDB doesn't support `nulls first`/`nulls last`");
        }
        let dir = match o.options.asc {
            Some(true) | None => Order::Asc,
            Some(false) => Order::Desc,
        };
        order_by.push((expr, dir));
    }
    let frame = if let Some(sql_frame) = &spec.window_frame {
        let start =
            convert_window_frame_bound(input, &sql_frame.start_bound, used_params, custom_types, field_lookup);
        let end =
            sql_frame
                .end_bound
                .as_ref()
                .map(|b| convert_window_frame_bound(input, b, used_params, custom_types, field_lookup));
        Some(good_ormning_core::mariadb::query::expr::WindowFrame {
            type_: match sql_frame.units {
                sql::WindowFrameUnits::Rows => good_ormning_core::mariadb::query::expr::WindowFrameType::Rows,
                sql::WindowFrameUnits::Range => good_ormning_core::mariadb::query::expr::WindowFrameType::Range,
                sql::WindowFrameUnits::Groups => panic!("MariaDB doesn't support `groups` window frames"),
            },
            start: start,
            end: end,
        })
    } else {
        None
    };
    return WindowSpec {
        base: spec.window_name.as_ref().map(|n| n.value.clone()),
        partition_by: partition_by,
        order_by: order_by,
        frame: frame,
    };
}

fn check_wildcard_options(options: &sql::WildcardAdditionalOptions) {
    if options.opt_ilike.is_some() || options.opt_exclude.is_some() || options.opt_except.is_some() ||
        options.opt_replace.is_some() ||
        options.opt_rename.is_some() {
        panic!("MariaDB doesn't support wildcard modifiers: {}", options);
    }
}

pub fn convert_returning(
    input: &GoodQueryInput,
    returning: &Option<Vec<sql::SelectItem>>,
    used_params: &mut ConvertState,
    custom_types: &std::collections::BTreeMap<String, good_ormning_core::mariadb::schema::custom_type::CustomType>,
    field_lookup: &std::collections::HashMap<TableRef, MariadbTableInfo>,
    tables: &[(String, Option<TableRef>)],
    merged: &[String],
    merged_away: &[(String, String)],
) -> Vec<Returning> {
    let expand = |out: &mut Vec<Returning>, alias: &str, table_ref: &Option<TableRef>, skip_merged: bool| {
        let Some(table_ref) = table_ref else {
            panic!("Can't expand wildcard for {}, it's not a table with a known schema", alias);
        };
        let table_info =
            field_lookup
                .get(table_ref)
                .unwrap_or_else(|| panic!("Table {} not found in schema version", table_ref.0));
        let mut fields: Vec<_> = table_info.fields.iter().collect();
        fields.sort_by(|a, b| a.0.field_id.cmp(&b.0.field_id));
        for (field_ref, field_info) in fields {
            if skip_merged && merged_away.contains(&(alias.to_string(), field_ref.field_id.clone())) {
                continue;
            }
            out.push(Returning {
                e: Expr::Field(FieldRef {
                    table_id: alias.to_string(),
                    field_id: field_ref.field_id.clone(),
                }),
                rename: Some(field_info.sql_name.clone()),
            });
        }
    };
    let mut out = vec![];
    if let Some(items) = returning {
        for item in items {
            match item {
                sql::SelectItem::UnnamedExpr(e) => out.push(Returning {
                    e: convert_expr(input, e, used_params, custom_types, field_lookup),
                    rename: None,
                }),
                sql::SelectItem::ExprWithAlias { expr, alias } => out.push(Returning {
                    e: convert_expr(input, expr, used_params, custom_types, field_lookup),
                    rename: Some(alias.value.clone()),
                }),
                sql::SelectItem::Wildcard(options) => {
                    check_wildcard_options(options);
                    if tables.is_empty() {
                        panic!("Can't use a wildcard in a select with no tables");
                    }
                    for col in merged {
                        out.push(Returning {
                            e: Expr::Field(FieldRef {
                                table_id: "".to_string(),
                                field_id: col.clone(),
                            }),
                            rename: Some(col.clone()),
                        });
                    }
                    for (alias, table_ref) in tables {
                        expand(&mut out, alias, table_ref, true);
                    }
                },
                sql::SelectItem::QualifiedWildcard(
                    sql::SelectItemQualifiedWildcardKind::ObjectName(name),
                    options,
                ) => {
                    check_wildcard_options(options);
                    let alias = match name.0.last().unwrap() {
                        sqlparser::ast::ObjectNamePart::Identifier(i) => i.value.clone(),
                    };
                    match tables.iter().find(|(a, _)| *a == alias) {
                        Some((alias, table_ref)) => {
                            expand(&mut out, alias, table_ref, false);
                        },
                        None => {
                            panic!("No table or alias {} in query for wildcard", alias);
                        },
                    }
                },
                sql::SelectItem::QualifiedWildcard(sql::SelectItemQualifiedWildcardKind::Expr(_), _) => panic!(
                    "Qualified wildcard with expression is not supported"
                ),
            }
        }
    }
    return out;
}

fn convert_order_limit(
    input: &GoodQueryInput,
    q: &sql::Query,
    used_params: &mut ConvertState,
    custom_types: &std::collections::BTreeMap<String, good_ormning_core::mariadb::schema::custom_type::CustomType>,
    field_lookup: &std::collections::HashMap<TableRef, MariadbTableInfo>,
) -> (Vec<(Expr, Order)>, Option<Expr>, Option<Expr>) {
    let mut order = vec![];
    if let Some(order_by) = &q.order_by {
        let exprs = match &order_by.kind {
            sqlparser::ast::OrderByKind::Expressions(exprs) => exprs,
            _ => panic!("Unsupported order by kind"),
        };
        if order_by.interpolate.is_some() {
            panic!("MariaDB doesn't support `interpolate`");
        }
        for o in exprs {
            if o.with_fill.is_some() {
                panic!("MariaDB doesn't support `with fill`");
            }
            let e = convert_expr(input, &o.expr, used_params, custom_types, field_lookup);
            if o.options.nulls_first.is_some() {
                panic!("MariaDB doesn't support `nulls first`/`nulls last`");
            }
            let dir = match o.options.asc {
                Some(true) | None => Order::Asc,
                Some(false) => Order::Desc,
            };
            order.push((e, dir));
        }
    }
    let (limit, offset) = match &q.limit_clause {
        None => (None, None),
        Some(sqlparser::ast::LimitClause::LimitOffset { limit, offset, limit_by }) => {
            if !limit_by.is_empty() {
                panic!("`limit ... by` is not supported");
            }
            (
                limit.as_ref().map(|e| convert_expr(input, e, used_params, custom_types, field_lookup)),
                offset.as_ref().map(|o| convert_expr(input, &o.value, used_params, custom_types, field_lookup)),
            )
        },
        Some(sqlparser::ast::LimitClause::OffsetCommaLimit { offset, limit }) => (
            Some(convert_expr(input, limit, used_params, custom_types, field_lookup)),
            Some(convert_expr(input, offset, used_params, custom_types, field_lookup)),
        ),
    };
    return (order, limit, offset);
}

fn convert_table_factor(
    input: &GoodQueryInput,
    tf: &sql::TableFactor,
    used_params: &mut ConvertState,
    custom_types: &std::collections::BTreeMap<String, good_ormning_core::mariadb::schema::custom_type::CustomType>,
    field_lookup: &std::collections::HashMap<TableRef, MariadbTableInfo>,
) -> NamedSelectSource {
    return match tf {
        sql::TableFactor::Table { name, alias, .. } => {
            check_plain_table(tf);
            NamedSelectSource {
                source: JoinSource::Table(get_table_ref(name)),
                alias: Some(
                    alias.as_ref().map(|a| a.name.value.clone()).unwrap_or_else(|| get_table_ref(name).0.clone()),
                ),
            }
        },
        sql::TableFactor::Derived { lateral, subquery, alias } => {
            if *lateral {
                panic!("MariaDB doesn't support lateral derived tables: {}", tf);
            }
            if alias.as_ref().map(|a| !a.columns.is_empty()).unwrap_or(false) {
                panic!("Derived table column lists aren't supported, name the columns in the subquery: {}", tf);
            }
            NamedSelectSource {
                source: JoinSource::Subsel(
                    Box::new(convert_select_query(input, subquery, used_params, custom_types, field_lookup)),
                ),
                alias: alias.as_ref().map(|a| a.name.value.clone()),
            }
        },
        sql::TableFactor::NestedJoin { .. } => panic!(
            "Parenthesized joins aren't supported yet, restructure the joins without parentheses"
        ),
        _ => panic!("Unsupported table source: {}", tf),
    };
}

pub fn check_plain_table(tf: &sql::TableFactor) {
    let sql::TableFactor::Table {
        name,
        alias,
        args,
        with_hints,
        version,
        with_ordinality,
        partitions,
        json_path,
        sample,
        index_hints,
    } =
        tf else {
            panic!("Expected a table: {}", tf);
        };
    if args.is_some() {
        panic!("Unsupported table-valued function: {}", name);
    }
    if !with_hints.is_empty() || version.is_some() || *with_ordinality || json_path.is_some() || sample.is_some() {
        panic!("Unsupported table syntax: {}", tf);
    }
    if !partitions.is_empty() {
        panic!("Partition selection isn't supported: {}", tf);
    }
    if !index_hints.is_empty() {
        panic!("Index hints aren't supported: {}", tf);
    }
    if alias.as_ref().map(|a| !a.columns.is_empty()).unwrap_or(false) {
        panic!("Table alias column lists aren't supported: {}", tf);
    }
}

pub fn convert_sql_joins(
    input: &GoodQueryInput,
    from: &sql::TableWithJoins,
    used_params: &mut ConvertState,
    custom_types: &std::collections::BTreeMap<String, good_ormning_core::mariadb::schema::custom_type::CustomType>,
    field_lookup: &std::collections::HashMap<TableRef, MariadbTableInfo>,
) -> Vec<Join> {
    let mut out = vec![];
    for j in &from.joins {
        let source = convert_table_factor(input, &j.relation, used_params, custom_types, field_lookup);
        let on = |constraint: &sql::JoinConstraint, used_params: &mut ConvertState| -> Option<JoinCondition> {
            match constraint {
                sql::JoinConstraint::On(e) => {
                    return Some(
                        JoinCondition::On(convert_expr(input, e, used_params, custom_types, field_lookup)),
                    );
                },
                sql::JoinConstraint::Using(cols) => {
                    return Some(JoinCondition::Using(cols.iter().map(|c| match c.0.last().unwrap() {
                        sqlparser::ast::ObjectNamePart::Identifier(i) => i.value.clone(),
                    }).collect()));
                },
                sql::JoinConstraint::Natural => {
                    return Some(JoinCondition::Natural);
                },
                sql::JoinConstraint::None => {
                    return None;
                },
            }
        };
        let (type_, on) = match &j.join_operator {
            sql::JoinOperator::Join(constraint) | sql::JoinOperator::Inner(constraint) => {
                (JoinType::Inner, on(constraint, used_params))
            },
            sql::JoinOperator::Left(constraint) | sql::JoinOperator::LeftOuter(constraint) => {
                (JoinType::Left, on(constraint, used_params))
            },
            sql::JoinOperator::Right(constraint) | sql::JoinOperator::RightOuter(constraint) => {
                (JoinType::Right, on(constraint, used_params))
            },
            sql::JoinOperator::CrossJoin => {
                (JoinType::Cross, None)
            },
            sql::JoinOperator::FullOuter(_) => panic!("MariaDB doesn't support full outer joins"),
            _ => {
                panic!("Join operator is not supported: {:?}", j.join_operator)
            },
        };
        out.push(Join {
            source: Box::new(source),
            type_: type_,
            condition: on,
        });
    }
    return out;
}

pub fn convert_select_query(
    input: &GoodQueryInput,
    q: &sql::Query,
    used_params: &mut ConvertState,
    custom_types: &std::collections::BTreeMap<String, good_ormning_core::mariadb::schema::custom_type::CustomType>,
    field_lookup: &std::collections::HashMap<TableRef, MariadbTableInfo>,
) -> Select {
    if q.for_clause.is_some() || q.settings.is_some() || q.format_clause.is_some() {
        panic!("Unsupported query syntax for MariaDB: {}", q);
    }
    let mut sel = convert_select_query_expr(input, q, &q.body, used_params, custom_types, field_lookup);
    if let Some(fetch) = &q.fetch {
        if fetch.percent {
            panic!("MariaDB doesn't support `fetch ... percent`");
        }
        sel.fetch = Some(SelectFetch {
            quantity: fetch
                .quantity
                .as_ref()
                .map(|e| convert_expr(input, e, used_params, custom_types, field_lookup)),
            with_ties: fetch.with_ties,
        });
    }
    match q.locks.as_slice() {
        [] => { },
        [sql::LockClause { lock_type: sql::LockType::Update, of: None, nonblock }] => {
            sel.lock = Some(match nonblock {
                None => SelectLock::ForUpdate,
                Some(sql::NonBlock::Nowait) => SelectLock::ForUpdateNowait,
                Some(sql::NonBlock::SkipLocked) => SelectLock::ForUpdateSkipLocked,
            });
        },
        _ => panic!("Unsupported locking clause, MariaDB supports `for update [nowait | skip locked]`"),
    }
    if let Some(with) = &q.with {
        sel.with = Some({
            let mut ctes = vec![];
            for cte in &with.cte_tables {
                let name = match &cte.alias.name {
                    sqlparser::ast::Ident { value, .. } => value.clone(),
                };
                let query = convert_select_query(input, &cte.query, used_params, custom_types, field_lookup);
                let mut builder = CteBuilder::new(name.clone(), Box::new(query.clone()));
                if !cte.alias.columns.is_empty() {
                    for col in &cte.alias.columns {
                        builder = builder.column_inferred(col.name.value.clone());
                    }
                } else {
                    for (i, r) in query.returning.iter().enumerate() {
                        let r_name = match (&r.rename, &r.e) {
                            (Some(n), _) => n.clone(),
                            (None, Expr::Field(f)) => f.field_id.clone(),
                            _ => panic!("CTE {} column {} needs a name (use `as`)", name, i),
                        };
                        builder = builder.column_inferred(r_name);
                    }
                }
                ctes.push(good_ormning_core::mariadb::query::utils::Cte::from(builder));
            }
            good_ormning_core::mariadb::query::utils::With {
                recursive: with.recursive,
                ctes: ctes,
            }
        });
    }
    return sel;
}

fn convert_select_query_expr(
    input: &GoodQueryInput,
    q: &sql::Query,
    expr: &sql::SetExpr,
    used_params: &mut ConvertState,
    custom_types: &std::collections::BTreeMap<String, good_ormning_core::mariadb::schema::custom_type::CustomType>,
    field_lookup: &std::collections::HashMap<TableRef, MariadbTableInfo>,
) -> Select {
    match expr {
        sql::SetExpr::Select(s) => {
            if s.top.is_some() {
                panic!("MariaDB doesn't support `top`, use `limit`");
            }
            if s.into.is_some() {
                panic!("`select ... into` isn't supported");
            }
            if !s.lateral_views.is_empty() || s.prewhere.is_some() || !s.cluster_by.is_empty() ||
                !s.distribute_by.is_empty() ||
                !s.sort_by.is_empty() ||
                s.qualify.is_some() ||
                s.value_table_mode.is_some() ||
                s.connect_by.is_some() ||
                !matches!(s.flavor, sql::SelectFlavor::Standard) {
                panic!("Unsupported select syntax for MariaDB: {}", s);
            }
            let table = if s.from.is_empty() {
                NamedSelectSource {
                    source: JoinSource::Empty,
                    alias: None,
                }
            } else {
                convert_table_factor(input, &s.from[0].relation, used_params, custom_types, field_lookup)
            };
            let mut join = vec![];
            if !s.from.is_empty() {
                join = convert_sql_joins(input, &s.from[0], used_params, custom_types, field_lookup);
                for extra in &s.from[1..] {
                    join.push(Join {
                        source: Box::new(
                            convert_table_factor(input, &extra.relation, used_params, custom_types, field_lookup),
                        ),
                        type_: JoinType::Comma,
                        condition: None,
                    });
                    join.extend(convert_sql_joins(input, extra, used_params, custom_types, field_lookup));
                }
            }
            let mut tables: Vec<(String, Option<TableRef>)> = vec![];
            for source in Iterator::chain([&table].into_iter(), join.iter().map(|j| &*j.source)) {
                let Some(alias) = &source.alias else {
                    continue;
                };
                tables.push((alias.clone(), match &source.source {
                    JoinSource::Table(tr) => Some(tr.clone()),
                    _ => None,
                }));
            }
            let source_fields = |source: &NamedSelectSource| -> Vec<String> {
                match &source.source {
                    JoinSource::Table(tr) => {
                        return field_lookup
                            .get(tr)
                            .map(|t| t.fields.keys().map(|f| f.field_id.clone()).collect())
                            .unwrap_or_default();
                    },
                    _ => return vec![],
                }
            };
            let mut merged: Vec<String> = vec![];
            let mut merged_away: Vec<(String, String)> = vec![];
            let mut left_fields: Vec<(String, String)> =
                source_fields(&table).into_iter().map(|f| (table.alias.clone().unwrap_or_default(), f)).collect();
            for j in &join {
                let right_alias = j.source.alias.clone().unwrap_or_default();
                let right_fields = source_fields(&j.source);
                let cols: Vec<String> = match &j.condition {
                    Some(JoinCondition::Using(cols)) => cols.clone(),
                    Some(JoinCondition::Natural) => right_fields
                        .iter()
                        .filter(|c| merged.contains(c) || left_fields.iter().any(|(_, l)| l == *c))
                        .cloned()
                        .collect(),
                    _ => vec![],
                };
                for col in cols {
                    if !merged.contains(&col) {
                        if let Some(left) =
                            left_fields
                                .iter()
                                .find(|(a, c)| *c == col && !merged_away.contains(&(a.clone(), c.clone()))) {
                            merged_away.push(left.clone());
                        }
                        merged.push(col.clone());
                    }
                    merged_away.push((right_alias.clone(), col));
                }
                left_fields.extend(right_fields.into_iter().map(|f| (right_alias.clone(), f)));
            }
            let (order, limit, offset) = convert_order_limit(input, q, used_params, custom_types, field_lookup);
            return Select {
                fetch: None,
                first: None,
                lock: None,
                windows: s.named_window.iter().map(|w| match &w.1 {
                    sql::NamedWindowExpr::WindowSpec(spec) => (
                        w.0.value.clone(),
                        convert_window_spec(input, spec, used_params, custom_types, field_lookup),
                    ),
                    sql::NamedWindowExpr::NamedWindow(base) => (w.0.value.clone(), WindowSpec {
                        base: Some(base.value.clone()),
                        partition_by: vec![],
                        order_by: vec![],
                        frame: None,
                    }),
                }).collect(),
                with: None,
                table: table.clone(),
                returning: convert_returning(
                    input,
                    &Some(s.projection.clone()),
                    used_params,
                    custom_types,
                    field_lookup,
                    &tables,
                    &merged,
                    &merged_away,
                ),
                join: join,
                where_: s
                    .selection
                    .as_ref()
                    .map(|e| convert_expr(input, e, used_params, custom_types, field_lookup)),
                group: match &s.group_by {
                    sql::GroupByExpr::All(_) => panic!("GROUP BY ALL is not supported by MariaDB"),
                    sql::GroupByExpr::Expressions(exprs, modifiers) => {
                        if !modifiers.is_empty() {
                            panic!("Group by modifiers aren't supported: {}", s.group_by);
                        }
                        exprs
                            .iter()
                            .map(|e| convert_expr(input, e, used_params, custom_types, field_lookup))
                            .collect()
                    },
                },
                having: s.having.as_ref().map(|e| convert_expr(input, e, used_params, custom_types, field_lookup)),
                order: order,
                limit: limit,
                offset: offset,
                distinct: match &s.distinct {
                    None => false,
                    Some(sql::Distinct::Distinct) => true,
                    Some(sql::Distinct::On(_)) => panic!("MariaDB doesn't support `distinct on`"),
                },
                junctions: vec![],
            };
        },
        sql::SetExpr::SetOperation { op, left, right, set_quantifier } => {
            let inner_q = sql::Query {
                order_by: None,
                limit_clause: None,
                ..q.clone()
            };
            let mut l = convert_select_query_expr(input, &inner_q, left, used_params, custom_types, field_lookup);
            if matches!(**left, sql::SetExpr::Query(_)) {
                l = Select {
                    fetch: None,
                    first: Some(Box::new(l)),
                    lock: None,
                    windows: vec![],
                    distinct: false,
                    group: vec![],
                    having: None,
                    join: vec![],
                    junctions: vec![],
                    limit: None,
                    offset: None,
                    order: vec![],
                    returning: vec![],
                    table: NamedSelectSource {
                        source: JoinSource::Empty,
                        alias: None,
                    },
                    where_: None,
                    with: None,
                };
            }
            let r = convert_select_query_expr(input, &inner_q, right, used_params, custom_types, field_lookup);
            let (order, limit, offset) = convert_order_limit(input, q, used_params, custom_types, field_lookup);
            l.order = order;
            l.limit = limit;
            l.offset = offset;
            let junction_op = match op {
                sql::SetOperator::Union => {
                    if matches!(set_quantifier, sql::SetQuantifier::All) {
                        good_ormning_core::mariadb::query::select_body::SelectJunctionOperator::UnionAll
                    } else {
                        good_ormning_core::mariadb::query::select_body::SelectJunctionOperator::Union
                    }
                },
                sql::SetOperator::Intersect => {
                    if matches!(set_quantifier, sql::SetQuantifier::All) {
                        good_ormning_core::mariadb::query::select_body::SelectJunctionOperator::IntersectAll
                    } else {
                        good_ormning_core::mariadb::query::select_body::SelectJunctionOperator::Intersect
                    }
                },
                sql::SetOperator::Minus => panic!("MariaDB only has `minus` in Oracle mode, use `except`"),
                sql::SetOperator::Except => {
                    if matches!(set_quantifier, sql::SetQuantifier::All) {
                        good_ormning_core::mariadb::query::select_body::SelectJunctionOperator::ExceptAll
                    } else {
                        good_ormning_core::mariadb::query::select_body::SelectJunctionOperator::Except
                    }
                },
            };
            l.junctions.push(good_ormning_core::mariadb::query::select_body::SelectJunction {
                op: junction_op,
                body: Box::new(r),
            });
            return l;
        },
        sql::SetExpr::Query(q) => {
            return convert_select_query(input, q, used_params, custom_types, field_lookup);
        },
        _ => panic!("Unsupported: SetExpr type: {:?}", expr),
    }
}

fn convert_window_frame_bound(
    input: &GoodQueryInput,
    b: &sql::WindowFrameBound,
    used_params: &mut ConvertState,
    custom_types: &std::collections::BTreeMap<String, good_ormning_core::mariadb::schema::custom_type::CustomType>,
    field_lookup: &std::collections::HashMap<TableRef, MariadbTableInfo>,
) -> good_ormning_core::mariadb::query::expr::WindowFrameBound {
    match b {
        sql::WindowFrameBound::Preceding(e) => match e {
            Some(e) => {
                return good_ormning_core::mariadb::query::expr::WindowFrameBound::Preceding(
                    Box::new(convert_expr(input, e, used_params, custom_types, field_lookup)),
                );
            },
            None => {
                return good_ormning_core::mariadb::query::expr::WindowFrameBound::UnboundedPreceding;
            },
        },
        sql::WindowFrameBound::CurrentRow => return good_ormning_core
        ::mariadb
        ::query
        ::expr
        ::WindowFrameBound
        ::CurrentRow,
        sql::WindowFrameBound::Following(e) => match e {
            Some(e) => {
                return good_ormning_core::mariadb::query::expr::WindowFrameBound::Following(
                    Box::new(convert_expr(input, e, used_params, custom_types, field_lookup)),
                );
            },
            None => {
                return good_ormning_core::mariadb::query::expr::WindowFrameBound::UnboundedFollowing;
            },
        },
    }
}

pub fn get_table_ref(name: &sql::ObjectName) -> TableRef {
    if name.0.len() != 1 {
        panic!("Tables in other databases aren't supported: {}", name);
    }
    return TableRef(match name.0.last().expect("ObjectName should not be empty") {
        sqlparser::ast::ObjectNamePart::Identifier(i) => i.value.clone(),
    });
}
