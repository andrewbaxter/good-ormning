use {
    crate::{
        GoodQueryInput,
        ParamType,
    },
    good_ormning_core::{
        QueryResCount,
        sqlite::{
            Query,
            query::{
                delete::Delete,
                expr::{
                    BinOp,
                    Expr,
                    PrefixOp,
                },
                helpers::{
                    *,
                    FnOpt,
                },
                insert::{
                    Insert,
                    InsertConflict,
                    InsertRows,
                    InsertSource,
                },
                select::{
                    Join,
                    JoinSource,
                    JoinType,
                    NamedSelectSource,
                    Order,
                    Select,
                },
                update::Update,
                utils::{
                    CteBuilder,
                    QueryBody,
                    Returning,
                    SqliteTableInfo,
                },
            },
            schema::{
                custom_type::CustomType as SqliteCustomType,
                field::FieldRef,
                table::TableRef,
            },
            types::{
                SimpleSimpleType,
                SimpleSimpleType as SqliteSimpleSimpleType,
                SimpleType,
                SimpleType as SqliteSimpleType,
                Type as SqliteType,
            },
        },
    },
    sqlparser::ast as sql,
    std::collections::{
        BTreeMap,
        HashSet,
    },
};

fn convert_delete(
    input: &GoodQueryInput,
    delete: &sql::Delete,
    used_params: &mut HashSet<String>,
    custom_types: &std::collections::BTreeMap<String, good_ormning_core::sqlite::schema::custom_type::CustomType>,
    field_lookup: &std::collections::HashMap<TableRef, SqliteTableInfo>,
) -> Delete {
    let from = match &delete.from {
        sql::FromTable::WithFromKeyword(f) => f,
        sql::FromTable::WithoutKeyword(f) => f,
    };
    if from.len() != 1 || !from[0].joins.is_empty() {
        panic!("Delete must have exactly one table, use a subquery in the `where` for other tables");
    }
    let (table_ref, alias) = match &from[0].relation {
        sql::TableFactor::Table { name, alias, .. } => {
            (get_table_ref(name), alias.as_ref().map(|a| a.name.value.clone()))
        },
        _ => panic!("Unsupported: Delete table factor: {:?}", from[0].relation),
    };
    if delete.using.is_some() {
        panic!("`delete ... using` is not supported by SQLite, use a subquery in the `where`");
    }
    return Delete {
        with: None,
        alias: alias.clone(),
        using: vec![],
        table: table_ref.clone(),
        where_: delete.selection.as_ref().map(|e| convert_expr(input, e, used_params, custom_types, field_lookup)),
        returning: convert_returning(
            input,
            &delete.returning,
            used_params,
            custom_types,
            field_lookup,
            &[(alias.unwrap_or_else(|| table_ref.0.clone()), Some(table_ref.clone()))],
        ),
        index_hint: None,
    };
}

fn convert_expr(
    input: &GoodQueryInput,
    e: &sql::Expr,
    used_params: &mut HashSet<String>,
    custom_types: &std::collections::BTreeMap<String, good_ormning_core::sqlite::schema::custom_type::CustomType>,
    field_lookup: &std::collections::HashMap<TableRef, SqliteTableInfo>,
) -> Expr {
    match e {
        sql::Expr::Identifier(ident) => {
            return Expr::Field(FieldRef {
                table_id: "".into(),
                field_id: ident.value.clone(),
            });
        },
        sql::Expr::CompoundIdentifier(idents) => {
            let table_id = idents[0].value.clone();
            let id = idents[1].value.clone();
            return Expr::Field(FieldRef {
                table_id,
                field_id: id,
            });
        },
        sql::Expr::Value(v) => {
            match &v.value {
                sql::Value::Number(n, _) => {
                    if let Ok(i) = n.parse::<i64>() {
                        return Expr::LitAnyInt(i);
                    } else if let Ok(f) = n.parse::<f64>() {
                        return Expr::LitAnyFloat(f);
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
                sql::Value::Placeholder(p) => {
                    let placeholder_name = p.trim_start_matches(|c| c == '$' || c == '?' || c == ':' || c == '@');
                    let param_name = if let Ok(idx) = placeholder_name.parse::<usize>() {
                        format!("p{}", idx)
                    } else {
                        placeholder_name.to_string()
                    };
                    used_params.insert(param_name.clone());
                    let pt =
                        input
                            .param_types
                            .iter()
                            .find(|(ident, _)| {
                                let s = ident.to_string();
                                let s = s.trim_start_matches("r#");
                                return s == param_name;
                            })
                            .map(|(_, pt)| pt)
                            .unwrap_or_else(|| panic!("Parameter {} not found in params section", param_name));
                    return Expr::Param {
                        name: param_name,
                        type_: param_type_to_sqlite_type(pt, custom_types),
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
                sql::BinaryOperator::And => BinOp::And,
                sql::BinaryOperator::Or => BinOp::Or,
                sql::BinaryOperator::Plus => BinOp::Plus,
                sql::BinaryOperator::Minus => BinOp::Minus,
                sql::BinaryOperator::Multiply => BinOp::Multiply,
                sql::BinaryOperator::Divide => BinOp::Divide,
                sql::BinaryOperator::StringConcat => BinOp::StringConcat,
                sql::BinaryOperator::Modulo => BinOp::Mod,
                sql::BinaryOperator::BitwiseAnd => BinOp::BitwiseAnd,
                sql::BinaryOperator::BitwiseOr => BinOp::BitwiseOr,
                sql::BinaryOperator::BitwiseXor => BinOp::BitwiseXor,
                sql::BinaryOperator::PGBitwiseShiftLeft => BinOp::BitwiseShiftLeft,
                sql::BinaryOperator::PGBitwiseShiftRight => BinOp::BitwiseShiftRight,
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
                sql::UnaryOperator::Minus => PrefixOp::Minus,
                sql::UnaryOperator::Not => PrefixOp::Not,
                sql::UnaryOperator::PGBitwiseNot => PrefixOp::BitwiseNot,
                _ => panic!("Unsupported: UnaryOperator: {:?}", op),
            };
            return Expr::PrefixOp {
                op,
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
            let res = Expr::Like {
                expr: Box::new(l),
                pattern: Box::new(r),
                escape,
                glob: false,
            };
            if *negated {
                return Expr::PrefixOp {
                    op: PrefixOp::Not,
                    right: Box::new(res),
                };
            } else {
                return res;
            }
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
        sql::Expr::IsDistinctFrom(left, right) => {
            return Expr::BinOp {
                left: Box::new(convert_expr(input, left, used_params, custom_types, field_lookup)),
                op: BinOp::IsDistinctFrom,
                right: Box::new(convert_expr(input, right, used_params, custom_types, field_lookup)),
            };
        },
        sql::Expr::IsNotDistinctFrom(left, right) => {
            return Expr::BinOp {
                left: Box::new(convert_expr(input, left, used_params, custom_types, field_lookup)),
                op: BinOp::IsNotDistinctFrom,
                right: Box::new(convert_expr(input, right, used_params, custom_types, field_lookup)),
            };
        },
        sql::Expr::Array(sql::Array { elem, .. }) => {
            return Expr::LitArray(
                elem.iter().map(|e| convert_expr(input, e, used_params, custom_types, field_lookup)).collect(),
            );
        },
        sql::Expr::Tuple(exprs) => {
            return Expr::LitArray(
                exprs.iter().map(|e| convert_expr(input, e, used_params, custom_types, field_lookup)).collect(),
            );
        },
        sql::Expr::Nested(expr) => {
            return Expr::Paren(Box::new(convert_expr(input, expr, used_params, custom_types, field_lookup)));
        },
        sql::Expr::Cast { expr, data_type, .. } => {
            let e = convert_expr(input, expr, used_params, custom_types, field_lookup);
            let t = sql_type_to_sqlite_type(data_type, custom_types);
            return Expr::Cast(Box::new(e), t);
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
            let r =
                Expr::LitArray(
                    list.iter().map(|e| convert_expr(input, e, used_params, custom_types, field_lookup)).collect(),
                );
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
            if let sql::FunctionArguments::List(list) = &f.args {
                distinct = matches!(list.duplicate_treatment, Some(sql::DuplicateTreatment::Distinct));
                for arg in &list.args {
                    match arg {
                        sql::FunctionArg::Unnamed(sql::FunctionArgExpr::Expr(e)) => args.push(
                            convert_expr(input, e, used_params, custom_types, field_lookup),
                        ),
                        sql::FunctionArg::Unnamed(sql::FunctionArgExpr::Wildcard) => {
                            args.push(Expr::LitI32(1));
                        },
                        _ => panic!("Unsupported: FunctionArg: {:?}", arg),
                    }
                }
            }
            let filter =
                f
                    .filter
                    .as_ref()
                    .map(|e| Box::new(convert_expr(input, e, used_params, custom_types, field_lookup)));
            if let Some(over) = &f.over {
                let mut e = match name.as_str() {
                    "sum" | "count" | "min" | "max" | "avg" => convert_function(&name, args, distinct),
                    "row_number" => fn_row_number(),
                    "rank" => fn_rank(),
                    "dense_rank" => fn_dense_rank(),
                    _ => panic!("Unsupported window function: {}", name),
                };
                if let Expr::Call { filter: ref mut f_opt, .. } = e {
                    *f_opt = filter;
                }
                match over {
                    sql::WindowType::WindowSpec(spec) => {
                        let mut partition_by = vec![];
                        for p in &spec.partition_by {
                            partition_by.push(convert_expr(input, p, used_params, custom_types, field_lookup));
                        }
                        let mut order_by = vec![];
                        for o in &spec.order_by {
                            let expr = convert_expr(input, &o.expr, used_params, custom_types, field_lookup);
                            let dir = match o.options.asc {
                                Some(true) | None => Order::Asc,
                                Some(false) => Order::Desc,
                            };
                            order_by.push((expr, dir));
                        }
                        let frame = if let Some(sql_frame) = &spec.window_frame {
                            let start =
                                convert_window_frame_bound(
                                    input,
                                    &sql_frame.start_bound,
                                    used_params,
                                    custom_types,
                                    field_lookup,
                                );
                            let end =
                                sql_frame
                                    .end_bound
                                    .as_ref()
                                    .map(
                                        |b| convert_window_frame_bound(
                                            input,
                                            b,
                                            used_params,
                                            custom_types,
                                            field_lookup,
                                        ),
                                    );
                            Some(good_ormning_core::sqlite::query::expr::WindowFrame {
                                type_: match sql_frame.units {
                                    sql::WindowFrameUnits::Rows => good_ormning_core
                                    ::sqlite
                                    ::query
                                    ::expr
                                    ::WindowFrameType
                                    ::Rows,
                                    sql::WindowFrameUnits::Range => good_ormning_core
                                    ::sqlite
                                    ::query
                                    ::expr
                                    ::WindowFrameType
                                    ::Range,
                                    sql::WindowFrameUnits::Groups => good_ormning_core
                                    ::sqlite
                                    ::query
                                    ::expr
                                    ::WindowFrameType
                                    ::Groups,
                                },
                                start,
                                end,
                                exclude: None,
                            })
                        } else {
                            None
                        };
                        return Expr::Window {
                            expr: Box::new(e),
                            partition_by,
                            order_by,
                            frame,
                        };
                    },
                    sql::WindowType::NamedWindow(_) => panic!("Named windows are not supported"),
                }
            }
            let mut e = convert_function(&name, args, distinct);
            if let Expr::Call { filter: ref mut f_opt, .. } = e {
                *f_opt = filter;
            }
            return e;
        },
        sql::Expr::Substring { expr, substring_from, substring_for, .. } => {
            let mut args = vec![convert_expr(input, expr, used_params, custom_types, field_lookup)];
            match substring_from {
                Some(from) => {
                    args.push(convert_expr(input, from, used_params, custom_types, field_lookup));
                },
                None => {
                    args.push(Expr::LitAnyInt(1));
                },
            }
            if let Some(for_) = substring_for {
                args.push(convert_expr(input, for_, used_params, custom_types, field_lookup));
            }
            return fn_fixed("substr", args, SimpleSimpleType::String, FnOpt::AnyArg);
        },
        sql::Expr::Trim { expr, trim_where, trim_what, trim_characters } => {
            if trim_characters.is_some() {
                panic!("Unsupported trim syntax: {}", e);
            }
            let name = match trim_where {
                None | Some(sql::TrimWhereField::Both) => "trim",
                Some(sql::TrimWhereField::Leading) => "ltrim",
                Some(sql::TrimWhereField::Trailing) => "rtrim",
            };
            let mut args = vec![convert_expr(input, expr, used_params, custom_types, field_lookup)];
            if let Some(what) = trim_what {
                args.push(convert_expr(input, what, used_params, custom_types, field_lookup));
            }
            return fn_fixed(name, args, SimpleSimpleType::String, FnOpt::AnyArg);
        },
        sql::Expr::Position { expr, r#in } => {
            // `position(x in y)` is `strpos(y, x)`/`instr(y, x)`
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
            return fn_fixed(name, args, SimpleSimpleType::F64, FnOpt::AnyArg);
        },
        _ => panic!("Unsupported: Expr type: {:?}", e),
    }
}

/// Convert a call to a known function. Any function not listed here is unsupported
/// because its return type isn't known.
fn convert_function(name: &str, mut args: Vec<Expr>, distinct: bool) -> Expr {
    let arg1 = |args: &mut Vec<Expr>, name: &str| -> Expr {
        if args.len() != 1 {
            panic!("Function {} requires exactly 1 argument, but got {}", name, args.len());
        }
        return args.pop().unwrap();
    };
    let mut e = match name {
        "sum" => fn_sum(arg1(&mut args, name)),
        "count" => fn_count(arg1(&mut args, name)),
        "min" | "max" if args.len() == 1 => {
            if name == "min" {
                fn_min(arg1(&mut args, name))
            } else {
                fn_max(arg1(&mut args, name))
            }
        },
        "avg" => fn_avg(arg1(&mut args, name)),
        "coalesce" | "ifnull" | "min" | "max" => {
            let mut e = fn_coalesce(args);
            if let Expr::Call { func: ref mut f, .. } = e {
                *f = name.to_string();
            }
            e
        },
        "nullif" => {
            if args.len() != 2 {
                panic!("Function nullif requires exactly 2 arguments, but got {}", args.len());
            }
            let b = args.pop().unwrap();
            let a = args.pop().unwrap();
            fn_nullif(a, b)
        },
        "lower" | "upper" | "trim" | "ltrim" | "rtrim" => fn_str_to_str(name, arg1(&mut args, name)),
        "length" => fn_length(arg1(&mut args, name)),
        "abs" => fn_abs(arg1(&mut args, name)),
        "round" |
        "floor" |
        "ceil" |
        "ceiling" |
        "mod" |
        "sqrt" |
        "power" |
        "pow" |
        "exp" |
        "ln" |
        "log" |
        "log2" |
        "log10" |
        "julianday" |
        "total" => fn_fixed(
            name,
            args,
            SimpleSimpleType::F64,
            FnOpt::AnyArg,
        ),
        "sign" => fn_same_as_first(name, args),
        "random" | "last_insert_rowid" | "changes" | "total_changes" => {
            fn_fixed(name, args, SimpleSimpleType::I64, FnOpt::Never)
        },
        "unixepoch" | "instr" | "unicode" => fn_fixed(name, args, SimpleSimpleType::I64, FnOpt::AnyArg),
        "substr" |
        "substring" |
        "replace" |
        "concat" |
        "concat_ws" |
        "hex" |
        "quote" |
        "printf" |
        "format" |
        "char" |
        "strftime" |
        "date" |
        "time" |
        "datetime" |
        "timediff" |
        "json_extract" |
        "json" |
        "typeof" => {
            fn_fixed(name, args, SimpleSimpleType::String, FnOpt::AnyArg)
        },
        "group_concat" | "string_agg" => fn_fixed(name, args, SimpleSimpleType::String, FnOpt::Always),
        "iif" => {
            if args.len() != 3 {
                panic!("Function iif requires exactly 3 arguments, but got {}", args.len());
            }
            let no = args.pop().unwrap();
            let yes = args.pop().unwrap();
            let cond = args.pop().unwrap();
            Expr::Case {
                operand: None,
                conditions: vec![(cond, yes)],
                else_: Some(Box::new(no)),
            }
        },
        _ => panic!(
            "Unsupported function: {} (supported functions: sum, count, min, max, avg, total, group_concat, string_agg, coalesce, ifnull, nullif, iif, lower, upper, trim, ltrim, rtrim, length, substr, substring, replace, concat, concat_ws, hex, quote, printf, format, char, instr, unicode, typeof, abs, round, floor, ceil, mod, sign, sqrt, power, exp, ln, log, log2, log10, random, strftime, date, time, datetime, timediff, julianday, unixepoch, json, json_extract, last_insert_rowid, changes, total_changes, row_number, rank, dense_rank)",
            name
        ),
    };
    if distinct {
        if !matches!(name, "sum" | "count" | "min" | "max" | "avg" | "total" | "group_concat" | "string_agg") {
            panic!("`distinct` is only supported in aggregate functions, not {}", name);
        }
        if let Expr::Call { distinct: ref mut d, .. } = e {
            *d = true;
        }
    }
    return e;
}

fn convert_insert(
    input: &GoodQueryInput,
    insert: &sql::Insert,
    used_params: &mut HashSet<String>,
    custom_types: &std::collections::BTreeMap<String, good_ormning_core::sqlite::schema::custom_type::CustomType>,
    field_lookup: &std::collections::HashMap<TableRef, SqliteTableInfo>,
) -> Insert {
    let table = get_table_ref(match &insert.table {
        sqlparser::ast::TableObject::TableName(n) => n,
        _ => panic!("Unsupported insert table object: {:?}", insert.table),
    });
    let columns: Vec<FieldRef> = insert.columns.iter().map(|c| FieldRef {
        table_id: table.0.clone(),
        field_id: c.value.clone(),
    }).collect();
    let source = if let Some(q) = &insert.source {
        match &*q.body {
            sql::SetExpr::Values(v) => {
                let mut rows = vec![];
                for row in &v.rows {
                    if row.len() != columns.len() {
                        panic!("Insert row has {} values but {} columns were specified", row.len(), columns.len());
                    }
                    rows.push(
                        row
                            .iter()
                            .map(|expr| convert_expr(input, expr, used_params, custom_types, field_lookup))
                            .collect::<Vec<_>>(),
                    );
                }
                if rows.len() == 1 {
                    InsertSource::Values(columns.iter().cloned().zip(rows.pop().unwrap().into_iter()).collect())
                } else {
                    InsertSource::Rows(InsertRows {
                        columns: columns,
                        rows: rows,
                    })
                }
            },
            _ => {
                InsertSource::Select {
                    columns: columns,
                    select: convert_select_query(input, q, used_params, custom_types, field_lookup),
                }
            },
        }
    } else {
        InsertSource::Values(vec![])
    };
    let on_conflict = if let Some(on) = &insert.on {
        match on {
            sql::OnInsert::OnConflict(oc) => {
                let conflict = match &oc.conflict_target {
                    Some(sql::ConflictTarget::Columns(idents)) => idents.iter().map(|id| FieldRef {
                        table_id: table.0.clone(),
                        field_id: id.value.clone(),
                    }).collect(),
                    Some(sql::ConflictTarget::OnConstraint(c)) => panic!(
                        "`on conflict on constraint` is not supported, use a list of columns instead: {}",
                        c
                    ),
                    None => vec![],
                };
                match &oc.action {
                    sql::OnConflictAction::DoNothing => {
                        if conflict.is_empty() {
                            Some(InsertConflict::DoNothing)
                        } else {
                            Some(InsertConflict::DoNothingOn { conflict: conflict })
                        }
                    },
                    sql::OnConflictAction::DoUpdate(du) => {
                        if let Some(sel) = &du.selection {
                            panic!("`on conflict do update ... where` is not supported: {}", sel);
                        }
                        let mut updates = vec![];
                        for a in &du.assignments {
                            let target_name = match &a.target {
                                sql::AssignmentTarget::ColumnName(name) => match name.0.last().unwrap() {
                                    sqlparser::ast::ObjectNamePart::Identifier(i) => i.value.clone(),
                                },
                                _ => panic!("Unsupported assignment target: {:?}", a.target),
                            };
                            let field = FieldRef {
                                table_id: table.0.clone(),
                                field_id: target_name,
                            };
                            updates.push(
                                (field, convert_expr(input, &a.value, used_params, custom_types, field_lookup)),
                            );
                        }
                        Some(InsertConflict::DoUpdate {
                            conflict: conflict,
                            set: updates,
                        })
                    },
                }
            },
            sql::OnInsert::DuplicateKeyUpdate(_) => panic!("`on duplicate key update` is not supported"),
            _ => panic!("Unsupported `on` clause in insert: {:?}", on),
        }
    } else {
        match insert.or {
            None => None,
            Some(sql::SqliteOnConflict::Ignore) => Some(InsertConflict::OrIgnore),
            Some(sql::SqliteOnConflict::Replace) => Some(InsertConflict::OrReplace),
            Some(sql::SqliteOnConflict::Abort) => Some(InsertConflict::OrAbort),
            Some(sql::SqliteOnConflict::Fail) => Some(InsertConflict::OrFail),
            Some(sql::SqliteOnConflict::Rollback) => Some(InsertConflict::OrRollback),
        }
    };
    return Insert {
        table: table.clone(),
        source: source,
        on_conflict: on_conflict,
        returning: convert_returning(
            input,
            &insert.returning,
            used_params,
            custom_types,
            field_lookup,
            &[(table.0.clone(), Some(table.clone()))],
        ),
    };
}

fn convert_on(
    input: &GoodQueryInput,
    constraint: &sql::JoinConstraint,
    left_alias: Option<&str>,
    right_alias: Option<&str>,
    used_params: &mut HashSet<String>,
    custom_types: &std::collections::BTreeMap<String, good_ormning_core::sqlite::schema::custom_type::CustomType>,
    field_lookup: &std::collections::HashMap<TableRef, SqliteTableInfo>,
) -> Expr {
    match constraint {
        sql::JoinConstraint::On(e) => {
            return convert_expr(input, e, used_params, custom_types, field_lookup);
        },
        sql::JoinConstraint::Using(cols) => {
            // `using (a, b)` is `on left.a = right.a and left.b = right.b`
            let Some(left_alias) = left_alias else {
                panic!("`join ... using` needs a table on the left side");
            };
            let Some(right_alias) = right_alias else {
                panic!("`join ... using` needs a table or alias on the right side");
            };
            let mut conds = vec![];
            for c in cols {
                let col = match c.0.last().unwrap() {
                    sqlparser::ast::ObjectNamePart::Identifier(i) => i.value.clone(),
                };
                conds.push(Expr::BinOp {
                    left: Box::new(Expr::Field(FieldRef {
                        table_id: left_alias.to_string(),
                        field_id: col.clone(),
                    })),
                    op: BinOp::Equals,
                    right: Box::new(Expr::Field(FieldRef {
                        table_id: right_alias.to_string(),
                        field_id: col,
                    })),
                });
            }
            if conds.len() == 1 {
                return conds.pop().unwrap();
            }
            return Expr::BinOpChain {
                op: BinOp::And,
                exprs: conds,
            };
        },
        sql::JoinConstraint::Natural => {
            panic!("Natural joins aren't supported, use `join ... using (...)` or `on`")
        },
        sql::JoinConstraint::None => {
            panic!("JOIN requires an ON clause")
        },
    }
}

pub fn convert_query(
    input: &GoodQueryInput,
    statement: &sql::Statement,
    custom_types: &std::collections::BTreeMap<String, good_ormning_core::sqlite::schema::custom_type::CustomType>,
    field_lookup: &std::collections::HashMap<TableRef, SqliteTableInfo>,
) -> Query {
    let mut used_params = HashSet::new();
    let body: Box<dyn QueryBody> = match statement {
        sql::Statement::Query(q) => {
            if let sql::SetExpr::Delete(sql::Statement::Delete(d)) = &*q.body {
                let mut del = convert_delete(input, d, &mut used_params, custom_types, field_lookup);
                if let Some(with) = &q.with {
                    del.with = Some(convert_with(input, with, &mut used_params, custom_types, field_lookup));
                }
                Box::new(del)
            } else {
                Box::new(convert_select_query(input, q, &mut used_params, custom_types, field_lookup))
            }
        },
        sql::Statement::Insert(insert) => Box::new(
            convert_insert(input, insert, &mut used_params, custom_types, field_lookup),
        ),
        sql::Statement::Update { table, assignments, from, selection, returning, or } => {
            if let Some(or) = or {
                panic!("`update or {}` is not supported", or);
            }
            Box::new(
                convert_update(
                    input,
                    table,
                    assignments,
                    from,
                    selection,
                    returning,
                    &mut used_params,
                    custom_types,
                    field_lookup,
                ),
            )
        },
        sql::Statement::Delete(delete) => Box::new(
            convert_delete(input, delete, &mut used_params, custom_types, field_lookup),
        ),
        _ => panic!("Unsupported: Statement type: {:?}", statement),
    };
    for (ident, _) in &input.param_types {
        let s = ident.to_string();
        let s = s.trim_start_matches("r#");
        if !used_params.contains(s) {
            panic!("Parameter {} not used in query", ident);
        }
    }
    return Query {
        name: "unnamed".to_string(),
        body,
        // Dummy
        res_count: QueryResCount::Many,
        res_name: None,
    };
}

fn convert_returning(
    input: &GoodQueryInput,
    returning: &Option<Vec<sql::SelectItem>>,
    used_params: &mut HashSet<String>,
    custom_types: &std::collections::BTreeMap<String, good_ormning_core::sqlite::schema::custom_type::CustomType>,
    field_lookup: &std::collections::HashMap<TableRef, SqliteTableInfo>,
    tables: &[(String, Option<TableRef>)],
) -> Vec<Returning> {
    // Expand `*` for a table into its fields, in declaration order
    let expand = |out: &mut Vec<Returning>, alias: &str, table_ref: &Option<TableRef>| {
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
                sql::SelectItem::Wildcard(_) => {
                    if tables.is_empty() {
                        panic!("Can't use a wildcard in a select with no tables");
                    }
                    for (alias, table_ref) in tables {
                        expand(&mut out, alias, table_ref);
                    }
                },
                sql::SelectItem::QualifiedWildcard(sql::SelectItemQualifiedWildcardKind::ObjectName(name), _) => {
                    let alias = match name.0.last().unwrap() {
                        sqlparser::ast::ObjectNamePart::Identifier(i) => i.value.clone(),
                    };
                    match tables.iter().find(|(a, _)| *a == alias) {
                        Some((alias, table_ref)) => {
                            expand(&mut out, alias, table_ref);
                        },
                        None => {
                            panic!("No table or alias {} in query for wildcard", alias);
                        },
                    }
                },
                sql::SelectItem::QualifiedWildcard(sql::SelectItemQualifiedWildcardKind::Expr(_), _) => panic!(
                    "Qualified wildcard with expression is not supported by SQLite"
                ),
            }
        }
    }
    return out;
}

/// Order/limit/offset of the outermost query.
fn convert_order_limit(
    input: &GoodQueryInput,
    q: &sql::Query,
    used_params: &mut HashSet<String>,
    custom_types: &std::collections::BTreeMap<String, good_ormning_core::sqlite::schema::custom_type::CustomType>,
    field_lookup: &std::collections::HashMap<TableRef, SqliteTableInfo>,
) -> (Vec<(Expr, Order)>, Option<Expr>, Option<Expr>) {
    let mut order = vec![];
    if let Some(order_by) = &q.order_by {
        let exprs = match &order_by.kind {
            sqlparser::ast::OrderByKind::Expressions(exprs) => exprs,
            _ => panic!("Unsupported order by kind"),
        };
        for o in exprs {
            let e = convert_expr(input, &o.expr, used_params, custom_types, field_lookup);
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

fn convert_select(
    input: &GoodQueryInput,
    q: &sql::Query,
    s: &sql::Select,
    used_params: &mut HashSet<String>,
    custom_types: &std::collections::BTreeMap<String, good_ormning_core::sqlite::schema::custom_type::CustomType>,
    field_lookup: &std::collections::HashMap<TableRef, SqliteTableInfo>,
) -> Select {
    let table = if s.from.is_empty() {
        NamedSelectSource {
            source: JoinSource::Empty,
            alias: None,
            index_hint: None,
        }
    } else {
        convert_table_factor_sqlite(input, &s.from[0].relation, used_params, custom_types, field_lookup)
    };
    let mut join = vec![];
    if !s.from.is_empty() {
        join = convert_sql_joins_sqlite(input, &s.from[0], used_params, custom_types, field_lookup);

        // Implicit cross joins (`from a, b`)
        for extra in &s.from[1..] {
            let source = if extra.joins.is_empty() {
                convert_table_factor_sqlite(input, &extra.relation, used_params, custom_types, field_lookup)
            } else {
                NamedSelectSource {
                    source: JoinSource::NestedJoin(
                        Box::new(
                            convert_table_factor_sqlite(
                                input,
                                &extra.relation,
                                used_params,
                                custom_types,
                                field_lookup,
                            ),
                        ),
                        convert_sql_joins_sqlite(input, extra, used_params, custom_types, field_lookup),
                    ),
                    alias: None,
                    index_hint: None,
                }
            };
            join.push(Join {
                source: source,
                type_: JoinType::Cross,
                on: None,
            });
        }
    }

    // Tables for wildcard expansion
    let mut tables: Vec<(String, Option<TableRef>)> = vec![];
    for source in Iterator::chain([&table].into_iter(), join.iter().map(|j| &j.source)) {
        let Some(alias) = &source.alias else {
            continue;
        };
        tables.push((alias.clone(), match &source.source {
            JoinSource::Table(tr) => Some(tr.clone()),
            _ => None,
        }));
    }
    let (order, limit, offset) = convert_order_limit(input, q, used_params, custom_types, field_lookup);
    return Select {
        with: None,
        table: table.clone(),
        returning: convert_returning(
            input,
            &Some(s.projection.clone()),
            used_params,
            custom_types,
            field_lookup,
            &tables,
        ),
        junction: vec![],
        join,
        where_: s.selection.as_ref().map(|e| convert_expr(input, e, used_params, custom_types, field_lookup)),
        group: match &s.group_by {
            sql::GroupByExpr::All(_) => panic!("GROUP BY ALL is not supported by SQLite"),
            sql::GroupByExpr::Expressions(exprs, _) => exprs
                .iter()
                .map(|e| convert_expr(input, e, used_params, custom_types, field_lookup))
                .collect(),
        },
        having: s.having.as_ref().map(|e| convert_expr(input, e, used_params, custom_types, field_lookup)),
        order,
        limit,
        offset,
        distinct: match &s.distinct {
            None => false,
            Some(sql::Distinct::Distinct) => true,
            Some(sql::Distinct::On(_)) => panic!("`distinct on` is not supported by SQLite"),
        },
    };
}

fn convert_select_query(
    input: &GoodQueryInput,
    q: &sql::Query,
    used_params: &mut HashSet<String>,
    custom_types: &std::collections::BTreeMap<String, good_ormning_core::sqlite::schema::custom_type::CustomType>,
    field_lookup: &std::collections::HashMap<TableRef, SqliteTableInfo>,
) -> Select {
    let mut sel = convert_select_query_expr(input, q, &q.body, used_params, custom_types, field_lookup);
    if let Some(with) = &q.with {
        sel.with = Some(convert_with(input, with, used_params, custom_types, field_lookup));
    }
    return sel;
}

fn convert_select_query_expr(
    input: &GoodQueryInput,
    q: &sql::Query,
    expr: &sql::SetExpr,
    used_params: &mut HashSet<String>,
    custom_types: &std::collections::BTreeMap<String, good_ormning_core::sqlite::schema::custom_type::CustomType>,
    field_lookup: &std::collections::HashMap<TableRef, SqliteTableInfo>,
) -> Select {
    match expr {
        sql::SetExpr::Select(s) => return convert_select(input, q, s, used_params, custom_types, field_lookup),
        sql::SetExpr::SetOperation { left, op, right, set_quantifier } => {
            // Order/limit apply to the whole compound select, not the arms
            let inner_q = sql::Query {
                order_by: None,
                limit_clause: None,
                ..q.clone()
            };
            let mut l = convert_select_query_expr(input, &inner_q, left, used_params, custom_types, field_lookup);
            let r = convert_select_query_expr(input, &inner_q, right, used_params, custom_types, field_lookup);
            let (order, limit, offset) = convert_order_limit(input, q, used_params, custom_types, field_lookup);
            l.order = order;
            l.limit = limit;
            l.offset = offset;
            let operator = match op {
                sql::SetOperator::Union => {
                    if matches!(set_quantifier, sql::SetQuantifier::All) {
                        good_ormning_core::sqlite::query::select_body::SelectJunctionOperator::UnionAll
                    } else {
                        good_ormning_core::sqlite::query::select_body::SelectJunctionOperator::Union
                    }
                },
                sql::SetOperator::Intersect => {
                    if matches!(set_quantifier, sql::SetQuantifier::All) {
                        panic!("INTERSECT ALL is not supported by SQLite")
                    } else {
                        good_ormning_core::sqlite::query::select_body::SelectJunctionOperator::Intersect
                    }
                },
                sql::SetOperator::Minus => {
                    if matches!(set_quantifier, sql::SetQuantifier::All) {
                        panic!("MINUS ALL is not supported by SQLite")
                    } else {
                        good_ormning_core::sqlite::query::select_body::SelectJunctionOperator::Except
                    }
                },
                sql::SetOperator::Except => {
                    if matches!(set_quantifier, sql::SetQuantifier::All) {
                        panic!("EXCEPT ALL is not supported by SQLite")
                    } else {
                        good_ormning_core::sqlite::query::select_body::SelectJunctionOperator::Except
                    }
                },
            };
            l.junction.push(good_ormning_core::sqlite::query::select_body::SelectJunction {
                op: operator,
                body: Box::new(r),
            });
            return l;
        },
        _ => panic!("Unsupported: SetExpr type: {:?}", expr),
    }
}

fn convert_sql_joins_sqlite(
    input: &GoodQueryInput,
    from: &sql::TableWithJoins,
    used_params: &mut HashSet<String>,
    custom_types: &std::collections::BTreeMap<String, good_ormning_core::sqlite::schema::custom_type::CustomType>,
    field_lookup: &std::collections::HashMap<TableRef, SqliteTableInfo>,
) -> Vec<Join> {
    let mut out = vec![];
    let mut prev_alias =
        convert_table_factor_sqlite(input, &from.relation, used_params, custom_types, field_lookup).alias;
    for j in &from.joins {
        let source = convert_table_factor_sqlite(input, &j.relation, used_params, custom_types, field_lookup);
        let on = |constraint: &sql::JoinConstraint, used_params: &mut HashSet<String>| -> Expr {
            return convert_on(
                input,
                constraint,
                prev_alias.as_deref(),
                source.alias.as_deref(),
                used_params,
                custom_types,
                field_lookup,
            );
        };
        let (type_, on) = match &j.join_operator {
            sql::JoinOperator::Join(constraint) | sql::JoinOperator::Inner(constraint) => {
                (JoinType::Inner, Some(on(constraint, used_params)))
            },
            sql::JoinOperator::Left(constraint) | sql::JoinOperator::LeftOuter(constraint) => {
                (JoinType::Left, Some(on(constraint, used_params)))
            },
            sql::JoinOperator::Right(constraint) | sql::JoinOperator::RightOuter(constraint) => {
                (JoinType::Right, Some(on(constraint, used_params)))
            },
            sql::JoinOperator::FullOuter(constraint) => {
                (JoinType::Full, Some(on(constraint, used_params)))
            },
            sql::JoinOperator::CrossJoin => (JoinType::Cross, None),
            _ => {
                panic!("Join operator is not supported by SQLite: {:?}", j.join_operator)
            },
        };
        prev_alias = source.alias.clone();
        out.push(Join {
            source: source,
            type_,
            on: on,
        });
    }
    return out;
}

fn convert_table_factor_sqlite(
    input: &GoodQueryInput,
    tf: &sql::TableFactor,
    used_params: &mut HashSet<String>,
    custom_types: &std::collections::BTreeMap<String, good_ormning_core::sqlite::schema::custom_type::CustomType>,
    field_lookup: &std::collections::HashMap<TableRef, SqliteTableInfo>,
) -> NamedSelectSource {
    match tf {
        sql::TableFactor::Table { name, alias, args, .. } => {
            if let Some(args) = args {
                // Table-valued function
                let name_str = name.to_string().to_lowercase();
                if name_str != "rarray" {
                    panic!("Unsupported table-valued function: {}", name_str);
                }
                let exprs = args.args.iter().map(|a| match a {
                    sql::FunctionArg::Unnamed(sql::FunctionArgExpr::Expr(e)) => {
                        convert_expr(input, e, used_params, custom_types, field_lookup)
                    },
                    _ => panic!("Unsupported function argument: {:?}", a),
                }).collect();
                return NamedSelectSource {
                    source: JoinSource::Func("__good_ormning_rarray".to_string(), exprs),
                    alias: alias.as_ref().map(|a| a.name.value.clone()),
                    index_hint: None,
                };
            }
            NamedSelectSource {
                source: JoinSource::Table(get_table_ref(name)),
                alias: Some(
                    alias.as_ref().map(|a| a.name.value.clone()).unwrap_or_else(|| get_table_ref(name).0.clone()),
                ),
                index_hint: None,
            }
        },
        sql::TableFactor::Derived { subquery, alias, .. } => NamedSelectSource {
            source: JoinSource::Subsel(
                Box::new(convert_select_query(input, subquery, used_params, custom_types, field_lookup)),
            ),
            alias: alias.as_ref().map(|a| a.name.value.clone()),
            index_hint: None,
        },
        sql::TableFactor::Function { name, args, alias, .. } => NamedSelectSource {
            source: JoinSource::Func(name.to_string(), args.iter().map(|a| match a {
                sql::FunctionArg::Unnamed(sql::FunctionArgExpr::Expr(e)) => {
                    convert_expr(input, e, used_params, custom_types, field_lookup)
                },
                _ => panic!("Unsupported: FunctionArg: {:?}", a),
            }).collect()),
            alias: alias.as_ref().map(|a| a.name.value.clone()),
            index_hint: None,
        },
        sql::TableFactor::NestedJoin { table_with_joins, alias } => {
            let base =
                convert_table_factor_sqlite(
                    input,
                    &table_with_joins.relation,
                    used_params,
                    custom_types,
                    field_lookup,
                );
            let joins = convert_sql_joins_sqlite(input, table_with_joins, used_params, custom_types, field_lookup);
            NamedSelectSource {
                source: JoinSource::NestedJoin(Box::new(base), joins),
                alias: alias.as_ref().map(|a| a.name.value.clone()),
                index_hint: None,
            }
        },
        sql::TableFactor::TableFunction { .. } => panic!("Unsupported: TableFunction"),
        sql::TableFactor::UNNEST { .. } => panic!("UNNEST is not supported by SQLite"),
        sql::TableFactor::JsonTable { .. } => panic!("JSON_TABLE is not supported by SQLite"),
        sql::TableFactor::Pivot { .. } => panic!("PIVOT is not supported by SQLite"),
        sql::TableFactor::Unpivot { .. } => panic!("UNPIVOT is not supported by SQLite"),
        sql::TableFactor::MatchRecognize { .. } => panic!("MATCH_RECOGNIZE is not supported by SQLite"),
        sql::TableFactor::OpenJsonTable { .. } => panic!("OPENJSON is not supported by SQLite"),
        sql::TableFactor::XmlTable { .. } => panic!("XMLTABLE is not supported by SQLite"),
    }
}

fn convert_update(
    input: &GoodQueryInput,
    table: &sql::TableWithJoins,
    assignments: &[sql::Assignment],
    from: &Option<sql::UpdateTableFromKind>,
    selection: &Option<sql::Expr>,
    returning: &Option<Vec<sql::SelectItem>>,
    used_params: &mut HashSet<String>,
    custom_types: &std::collections::BTreeMap<String, good_ormning_core::sqlite::schema::custom_type::CustomType>,
    field_lookup: &std::collections::HashMap<TableRef, SqliteTableInfo>,
) -> Update {
    if !table.joins.is_empty() {
        panic!("Joins on the updated table aren't supported, use `update ... from`");
    }
    let (table_ref, alias) = match &table.relation {
        sql::TableFactor::Table { name, alias, .. } => {
            (get_table_ref(name), alias.as_ref().map(|a| a.name.value.clone()))
        },
        _ => panic!("Unsupported: Update table factor: {:?}", table.relation),
    };
    let mut from_sources = vec![];
    if let Some(from) = from {
        let tables = match from {
            sql::UpdateTableFromKind::BeforeSet(t) | sql::UpdateTableFromKind::AfterSet(t) => t,
        };
        for t in tables {
            if !t.joins.is_empty() {
                panic!("Joins in `update ... from` aren't supported, list the tables separated by commas instead");
            }
            from_sources.push(
                convert_table_factor_sqlite(input, &t.relation, used_params, custom_types, field_lookup),
            );
        }
    }
    let mut values = vec![];
    for a in assignments {
        let target_name = match &a.target {
            sql::AssignmentTarget::ColumnName(name) => match name.0.last().unwrap() {
                sqlparser::ast::ObjectNamePart::Identifier(i) => i.value.clone(),
            },
            _ => panic!("Unsupported: AssignmentTarget: {:?}", a.target),
        };
        let field = FieldRef {
            table_id: table_ref.0.clone(),
            field_id: target_name,
        };
        values.push((field, convert_expr(input, &a.value, used_params, custom_types, field_lookup)));
    }
    return Update {
        alias: alias.clone(),
        from: from_sources,
        table: table_ref.clone(),
        values,
        where_: selection.as_ref().map(|e| convert_expr(input, e, used_params, custom_types, field_lookup)),
        returning: convert_returning(
            input,
            returning,
            used_params,
            custom_types,
            field_lookup,
            &[(alias.unwrap_or_else(|| table_ref.0.clone()), Some(table_ref.clone()))],
        ),
        index_hint: None,
    };
}

fn convert_window_frame_bound(
    input: &GoodQueryInput,
    b: &sql::WindowFrameBound,
    used_params: &mut HashSet<String>,
    custom_types: &std::collections::BTreeMap<String, good_ormning_core::sqlite::schema::custom_type::CustomType>,
    field_lookup: &std::collections::HashMap<TableRef, SqliteTableInfo>,
) -> good_ormning_core::sqlite::query::expr::WindowFrameBound {
    match b {
        sql::WindowFrameBound::Preceding(e) => match e {
            Some(e) => {
                return good_ormning_core::sqlite::query::expr::WindowFrameBound::Preceding(
                    Box::new(convert_expr(input, e, used_params, custom_types, field_lookup)),
                );
            },
            None => {
                return good_ormning_core::sqlite::query::expr::WindowFrameBound::UnboundedPreceding;
            },
        },
        sql::WindowFrameBound::CurrentRow => return good_ormning_core
        ::sqlite
        ::query
        ::expr
        ::WindowFrameBound
        ::CurrentRow,
        sql::WindowFrameBound::Following(e) => match e {
            Some(e) => {
                return good_ormning_core::sqlite::query::expr::WindowFrameBound::Following(
                    Box::new(convert_expr(input, e, used_params, custom_types, field_lookup)),
                );
            },
            None => {
                return good_ormning_core::sqlite::query::expr::WindowFrameBound::UnboundedFollowing;
            },
        },
    }
}

fn convert_with(
    input: &GoodQueryInput,
    with: &sql::With,
    used_params: &mut HashSet<String>,
    custom_types: &std::collections::BTreeMap<String, good_ormning_core::sqlite::schema::custom_type::CustomType>,
    field_lookup: &std::collections::HashMap<TableRef, SqliteTableInfo>,
) -> good_ormning_core::sqlite::query::utils::With {
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
        ctes.push(good_ormning_core::sqlite::query::utils::Cte::from(builder));
    }
    return good_ormning_core::sqlite::query::utils::With {
        recursive: with.recursive,
        ctes,
    };
}

fn get_table_ref(name: &sql::ObjectName) -> TableRef {
    return TableRef(match name.0.last().expect("ObjectName should not be empty") {
        sqlparser::ast::ObjectNamePart::Identifier(i) => i.value.clone(),
    });
}

pub fn param_type_to_sqlite_type(pt: &ParamType, custom_types: &BTreeMap<String, SqliteCustomType>) -> SqliteType {
    let (simple_type, custom) = match pt.base.as_str() {
        "i16" => (SqliteSimpleSimpleType::I16, None),
        "i32" => (SqliteSimpleSimpleType::I32, None),
        "i64" => (SqliteSimpleSimpleType::I64, None),
        "u32" => (SqliteSimpleSimpleType::U32, None),
        "f32" => (SqliteSimpleSimpleType::F32, None),
        "f64" => (SqliteSimpleSimpleType::F64, None),
        "bool" => (SqliteSimpleSimpleType::Bool, None),
        "string" => (SqliteSimpleSimpleType::String, None),
        "bytes" => (SqliteSimpleSimpleType::Bytes, None),
        #[cfg(feature = "chrono")]
        "utctime_s_chrono" => (SqliteSimpleSimpleType::UtcTimeSChrono, None),
        #[cfg(feature = "chrono")]
        "utctime_ms_chrono" => (SqliteSimpleSimpleType::UtcTimeMsChrono, None),
        #[cfg(feature = "jiff")]
        "utctime_s_jiff" => (SqliteSimpleSimpleType::UtcTimeSJiff, None),
        #[cfg(feature = "jiff")]
        "utctime_ms_jiff" => (SqliteSimpleSimpleType::UtcTimeMsJiff, None),
        "auto" => (SqliteSimpleSimpleType::Auto, None),
        _ => {
            if let Some(ct) = custom_types.get(&pt.base) {
                (ct.base_type.type_.type_.clone(), Some(ct.rust_type.clone()))
            } else {
                (SqliteSimpleSimpleType::I32, Some(pt.base.clone()))
            }
        },
    };
    return SqliteType {
        type_: SqliteSimpleType {
            type_: simple_type,
            custom,
        },
        opt: pt.opt,
        arr: pt.arr,
    };
}

pub fn sql_type_to_sqlite_type(
    t: &sqlparser::ast::DataType,
    custom_types: &BTreeMap<String, SqliteCustomType>,
) -> SqliteType {
    let (simple_type, custom) = match t {
        sqlparser::ast::DataType::SmallInt(_) => (SqliteSimpleSimpleType::I16, None),
        sqlparser::ast::DataType::Int(_) | sqlparser::ast::DataType::Integer(_) => (
            SqliteSimpleSimpleType::I32,
            None,
        ),
        sqlparser::ast::DataType::BigInt(_) => (SqliteSimpleSimpleType::I64, None),
        sqlparser::ast::DataType::Float(_) | sqlparser::ast::DataType::Real => (SqliteSimpleSimpleType::F32, None),
        sqlparser::ast::DataType::DoublePrecision => (SqliteSimpleSimpleType::F64, None),
        sqlparser::ast::DataType::Boolean => (SqliteSimpleSimpleType::Bool, None),
        sqlparser::ast::DataType::Text | sqlparser::ast::DataType::Varchar(_) => (
            SqliteSimpleSimpleType::String,
            None,
        ),
        sqlparser::ast::DataType::Binary(_) |
        sqlparser::ast::DataType::Varbinary(_) |
        sqlparser::ast::DataType::Blob(_) => (
            SqliteSimpleSimpleType::Bytes,
            None,
        ),
        sqlparser::ast::DataType::Timestamp(..) => {
            #[cfg(feature = "chrono")]
            {
                (SqliteSimpleSimpleType::UtcTimeSChrono, None)
            }
            #[cfg(not(feature = "chrono"))]
            {
                (SqliteSimpleSimpleType::I32, None)
            }
        },
        sqlparser::ast::DataType::Custom(name, ..) => {
            let name_str = name.to_string();
            if let Some(ct) = custom_types.get(&name_str) {
                (ct.base_type.type_.type_.clone(), Some(ct.rust_type.clone()))
            } else {
                (SqliteSimpleSimpleType::I32, Some(name_str))
            }
        },
        _ => (SqliteSimpleSimpleType::I32, None),
    };
    return SqliteType {
        type_: SqliteSimpleType {
            type_: simple_type,
            custom,
        },
        opt: false,
        arr: false,
    };
}
