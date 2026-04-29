use sqlparser::ast as sql;
use std::collections::HashSet;
use crate::GoodQueryInput;
use good_ormning_core::pg::{
    query::{
        expr::{
            Expr,
            BinOp,
            PrefixOp,
        },
        select::{
            Select,
            NamedSelectSource,
            JoinSource,
            Order,
        },
        insert::{
            Insert,
            InsertConflict,
        },
        update::Update,
        delete::Delete,
        utils::{
            Returning,
            QueryBody,
            With,
            CteBuilder,
            PgTableInfo,
        },
        helpers::*,
    },
    schema::{
        table::TableRef,
        field::FieldRef,
    },
    types::{
        SimpleType,
        SimpleSimpleType,
    },
    Query,
};
use good_ormning_core::QueryResCount;
use std::collections::BTreeMap;
use good_ormning_core::pg::schema::custom_type::CustomType as PgCustomType;
use good_ormning_core::pg::types::{
    Type as PgType,
    SimpleType as PgSimpleType,
    SimpleSimpleType as PgSimpleSimpleType,
};
use crate::ParamType;

pub fn param_type_to_pg_type(pt: &ParamType, custom_types: &BTreeMap<String, PgCustomType>) -> PgType {
    let (simple_type, custom) = match pt.base.as_str() {
        "i16" => (PgSimpleSimpleType::I16, None),
        "i32" => (PgSimpleSimpleType::I32, None),
        "i64" => (PgSimpleSimpleType::I64, None),
        "u32" => (PgSimpleSimpleType::U32, None),
        "f32" => (PgSimpleSimpleType::F32, None),
        "f64" => (PgSimpleSimpleType::F64, None),
        "bool" => (PgSimpleSimpleType::Bool, None),
        "string" => (PgSimpleSimpleType::String, None),
        "bytes" => (PgSimpleSimpleType::Bytes, None),
        #[cfg(feature = "chrono")]
        "utctime_s_chrono" => (PgSimpleSimpleType::UtcTimeSChrono, None),
        #[cfg(feature = "chrono")]
        "utctime_ms_chrono" => (PgSimpleSimpleType::UtcTimeMsChrono, None),
        #[cfg(feature = "jiff")]
        "utctime_s_jiff" => (PgSimpleSimpleType::UtcTimeSJiff, None),
        #[cfg(feature = "jiff")]
        "utctime_ms_jiff" => (PgSimpleSimpleType::UtcTimeMsJiff, None),
        "auto" => (PgSimpleSimpleType::Auto, None),
        _ => {
            if let Some(ct) = custom_types.get(&pt.base) {
                (ct.base_type.type_.type_.clone(), Some(ct.rust_type.clone()))
            } else {
                (PgSimpleSimpleType::I32, Some(pt.base.clone()))
            }
        },
    };
    return PgType {
        type_: PgSimpleType {
            type_: simple_type,
            custom,
        },
        opt: pt.opt,
        arr: pt.arr,
    };
}

pub fn sql_type_to_pg_type(t: &sqlparser::ast::DataType, custom_types: &BTreeMap<String, PgCustomType>) -> PgType {
    let (simple_type, custom) = match t {
        sqlparser::ast::DataType::SmallInt(_) => (PgSimpleSimpleType::I16, None),
        sqlparser::ast::DataType::Int(_) | sqlparser::ast::DataType::Integer(_) => (PgSimpleSimpleType::I32, None),
        sqlparser::ast::DataType::BigInt(_) => (PgSimpleSimpleType::I64, None),
        sqlparser::ast::DataType::Float(_) | sqlparser::ast::DataType::Real => (PgSimpleSimpleType::F32, None),
        sqlparser::ast::DataType::DoublePrecision => (PgSimpleSimpleType::F64, None),
        sqlparser::ast::DataType::Boolean => (PgSimpleSimpleType::Bool, None),
        sqlparser::ast::DataType::Text | sqlparser::ast::DataType::Varchar(_) => (PgSimpleSimpleType::String, None),
        sqlparser::ast::DataType::Bytea |
        sqlparser::ast::DataType::Binary(_) |
        sqlparser::ast::DataType::Varbinary(_) => (
            PgSimpleSimpleType::Bytes,
            None,
        ),
        sqlparser::ast::DataType::Timestamp(_precision, _tz) => {
            // Very simplified
            #[cfg(feature = "chrono")]
            {
                (PgSimpleSimpleType::UtcTimeSChrono, None)
            }
            #[cfg(not(feature = "chrono"))]
            {
                (PgSimpleSimpleType::I32, None)
            }
        },
        sqlparser::ast::DataType::Custom(name, ..) => {
            let name_str = name.to_string();
            if let Some(ct) = custom_types.get(&name_str) {
                (ct.base_type.type_.type_.clone(), Some(ct.rust_type.clone()))
            } else {
                (PgSimpleSimpleType::I32, Some(name_str))
            }
        },
        _ => (PgSimpleSimpleType::I32, None),
    };
    return PgType {
        type_: PgSimpleType {
            type_: simple_type,
            custom,
        },
        opt: false,
        arr: false,
    };
}

pub fn convert_query(
    input: &GoodQueryInput,
    statement: &sql::Statement,
    custom_types: &std::collections::BTreeMap<String, good_ormning_core::pg::schema::custom_type::CustomType>,
    field_lookup: &std::collections::HashMap<TableRef, PgTableInfo>,
) -> Query {
    let mut used_params = HashSet::new();
    let body: Box<dyn QueryBody> = match statement {
        sql::Statement::Query(q) => Box::new(
            convert_select_query(input, q, &mut used_params, custom_types, field_lookup),
        ),
        sql::Statement::Insert(insert) => Box::new(
            convert_insert(input, insert, &mut used_params, custom_types, field_lookup),
        ),
        sql::Statement::Update { table, assignments, selection, returning, .. } => Box::new(
            convert_update(
                input,
                table,
                assignments,
                selection,
                returning,
                &mut used_params,
                custom_types,
                field_lookup,
            ),
        ),
        sql::Statement::Delete(delete) => Box::new(
            convert_delete(input, delete, &mut used_params, custom_types, field_lookup),
        ),
        _ => unimplemented!("Unsupported statement type: {:?}", statement),
    };
    for (ident, _) in &input.param_types {
        if !used_params.contains(&ident.to_string()) {
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

fn convert_select_query(
    input: &GoodQueryInput,
    q: &sql::Query,
    used_params: &mut HashSet<String>,
    custom_types: &std::collections::BTreeMap<String, good_ormning_core::pg::schema::custom_type::CustomType>,
    field_lookup: &std::collections::HashMap<TableRef, PgTableInfo>,
) -> Select {
    let mut sel = convert_select_query_expr(input, q, &q.body, used_params, custom_types, field_lookup);
    if let Some(with) = &q.with {
        let mut ctes = vec![];
        for cte in &with.cte_tables {
            let name = cte.alias.name.value.clone();
            let query = convert_select_query(input, &cte.query, used_params, custom_types, field_lookup);
            let mut builder = CteBuilder::new(name, Box::new(query.clone()));
            if !cte.alias.columns.is_empty() {
                for col in &cte.alias.columns {
                    builder = builder.column(col.value.clone(), good_ormning_core::pg::types::type_i32().build());
                }
            } else {
                for r in &query.returning {
                    let r_name = r.rename.clone().unwrap_or_else(|| "column".to_string());
                    builder = builder.column(r_name, good_ormning_core::pg::types::type_i32().build());
                }
            }
            ctes.push(good_ormning_core::pg::query::utils::Cte::from(builder));
        }
        sel.with = Some(With {
            recursive: with.recursive,
            ctes,
        });
    }
    return sel;
}

fn convert_select_query_expr(
    input: &GoodQueryInput,
    q: &sql::Query,
    expr: &sql::SetExpr,
    used_params: &mut HashSet<String>,
    custom_types: &std::collections::BTreeMap<String, good_ormning_core::pg::schema::custom_type::CustomType>,
    field_lookup: &std::collections::HashMap<TableRef, PgTableInfo>,
) -> Select {
    match expr {
        sql::SetExpr::Select(s) => return convert_select(input, q, s, used_params, custom_types, field_lookup),
        sql::SetExpr::SetOperation { op, left, right, set_quantifier } => {
            let mut l = convert_select_query_expr(input, q, left, used_params, custom_types, field_lookup);
            let r = convert_select_query_expr(input, q, right, used_params, custom_types, field_lookup);
            let junction_op = match op {
                sql::SetOperator::Union => {
                    if matches!(set_quantifier, sql::SetQuantifier::All) {
                        good_ormning_core::pg::query::select_body::SelectJunctionOperator::UnionAll
                    } else {
                        good_ormning_core::pg::query::select_body::SelectJunctionOperator::Union
                    }
                },
                sql::SetOperator::Intersect => {
                    if matches!(set_quantifier, sql::SetQuantifier::All) {
                        unimplemented!("INTERSECT ALL not supported")
                    } else {
                        good_ormning_core::pg::query::select_body::SelectJunctionOperator::Intersect
                    }
                },
                sql::SetOperator::Except => {
                    if matches!(set_quantifier, sql::SetQuantifier::All) {
                        unimplemented!("EXCEPT ALL not supported")
                    } else {
                        good_ormning_core::pg::query::select_body::SelectJunctionOperator::Except
                    }
                },
            };
            l.junctions.push(good_ormning_core::pg::query::select_body::SelectJunction {
                op: junction_op,
                body: Box::new(r),
            });
            return l;
        },
        _ => unimplemented!("Unsupported SetExpr type: {:?}", expr),
    }
}

fn get_table_ref(name: &sql::ObjectName) -> TableRef {
    return TableRef(name.0.last().expect("ObjectName should not be empty").value.clone());
}

fn convert_returning(
    input: &GoodQueryInput,
    returning: &Option<Vec<sql::SelectItem>>,
    used_params: &mut HashSet<String>,
    custom_types: &std::collections::BTreeMap<String, good_ormning_core::pg::schema::custom_type::CustomType>,
    field_lookup: &std::collections::HashMap<TableRef, PgTableInfo>,
    default_table: Option<&TableRef>,
) -> Vec<Returning> {
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
                    let table_ref = default_table.expect("Wildcard returning without default table");
                    let table_info = field_lookup.get(table_ref).expect("Table not found in field_lookup");
                    for (field_ref, field_info) in &table_info.fields {
                        out.push(Returning {
                            e: Expr::Field(field_ref.clone()),
                            rename: Some(field_info.sql_name.clone()),
                        });
                    }
                },
                sql::SelectItem::QualifiedWildcard(name, _) => {
                    let table_ref = TableRef(name.0.last().unwrap().value.clone());
                    let table_info = field_lookup.get(&table_ref).expect("Table not found in field_lookup");
                    for (field_ref, field_info) in &table_info.fields {
                        out.push(Returning {
                            e: Expr::Field(field_ref.clone()),
                            rename: Some(field_info.sql_name.clone()),
                        });
                    }
                },
            }
        }
    }
    return out;
}

fn convert_insert(
    input: &GoodQueryInput,
    insert: &sql::Insert,
    used_params: &mut HashSet<String>,
    custom_types: &std::collections::BTreeMap<String, good_ormning_core::pg::schema::custom_type::CustomType>,
    field_lookup: &std::collections::HashMap<TableRef, PgTableInfo>,
) -> Insert {
    let table = get_table_ref(&insert.table_name);
    let mut values = vec![];
    if let Some(q) = &insert.source {
        if let sql::SetExpr::Values(v) = &*q.body {
            if let Some(row) = v.rows.first() {
                for (i, expr) in row.iter().enumerate() {
                    let field = FieldRef {
                        table_id: table.0.clone(),
                        field_id: insert.columns[i].value.clone(),
                    };
                    values.push((field, convert_expr(input, expr, used_params, custom_types, field_lookup)));
                }
            }
        } else {
            unimplemented!("Only values supported for insert")
        }
    }
    let on_conflict = if let Some(on) = &insert.on {
        match on {
            sql::OnInsert::OnConflict(oc) => match &oc.action {
                sql::OnConflictAction::DoNothing => Some(InsertConflict::DoNothing),
                sql::OnConflictAction::DoUpdate(du) => {
                    let mut updates = vec![];
                    for a in &du.assignments {
                        let target_name = match &a.target {
                            sql::AssignmentTarget::ColumnName(name) => name.0.last().unwrap().value.clone(),
                            _ => unimplemented!("Assignment target in ON CONFLICT"),
                        };
                        let field = FieldRef {
                            table_id: table.0.clone(),
                            field_id: target_name,
                        };
                        updates.push(
                            (field, convert_expr(input, &a.value, used_params, custom_types, field_lookup)),
                        );
                    }
                    let conflict = if let Some(target) = &oc.conflict_target {
                        match target {
                            sql::ConflictTarget::Columns(idents) => idents.iter().map(|id| FieldRef {
                                table_id: table.0.clone(),
                                field_id: id.value.clone(),
                            }).collect(),
                            _ => vec![],
                        }
                    } else {
                        vec![]
                    };
                    Some(InsertConflict::DoUpdate {
                        conflict,
                        set: updates,
                    })
                },
            },
            _ => None,
        }
    } else {
        None
    };
    return Insert {
        table: table.clone(),
        values,
        on_conflict,
        returning: convert_returning(
            input,
            &insert.returning,
            used_params,
            custom_types,
            field_lookup,
            Some(&table),
        ),
    };
}

fn convert_update(
    input: &GoodQueryInput,
    table: &sql::TableWithJoins,
    assignments: &[sql::Assignment],
    selection: &Option<sql::Expr>,
    returning: &Option<Vec<sql::SelectItem>>,
    used_params: &mut HashSet<String>,
    custom_types: &std::collections::BTreeMap<String, good_ormning_core::pg::schema::custom_type::CustomType>,
    field_lookup: &std::collections::HashMap<TableRef, PgTableInfo>,
) -> Update {
    let table_ref = match &table.relation {
        sql::TableFactor::Table { name, .. } => get_table_ref(name),
        _ => unimplemented!("Update table factor"),
    };
    let mut sets = vec![];
    for a in assignments {
        let target_name = match &a.target {
            sql::AssignmentTarget::ColumnName(name) => name.0.last().unwrap().value.clone(),
            _ => unimplemented!("Assignment target"),
        };
        let field = FieldRef {
            table_id: table_ref.0.clone(),
            field_id: target_name,
        };
        sets.push((field, convert_expr(input, &a.value, used_params, custom_types, field_lookup)));
    }
    return Update {
        table: table_ref.clone(),
        values: sets,
        where_: selection.as_ref().map(|e| convert_expr(input, e, used_params, custom_types, field_lookup)),
        returning: convert_returning(input, returning, used_params, custom_types, field_lookup, Some(&table_ref)),
    };
}

fn convert_delete(
    input: &GoodQueryInput,
    delete: &sql::Delete,
    used_params: &mut HashSet<String>,
    custom_types: &std::collections::BTreeMap<String, good_ormning_core::pg::schema::custom_type::CustomType>,
    field_lookup: &std::collections::HashMap<TableRef, PgTableInfo>,
) -> Delete {
    let relation = match &delete.from {
        sql::FromTable::WithFromKeyword(f) => &f[0].relation,
        sql::FromTable::WithoutKeyword(f) => &f[0].relation,
    };
    let table_ref = match relation {
        sql::TableFactor::Table { name, .. } => get_table_ref(name),
        _ => unimplemented!("Delete table factor"),
    };
    return Delete {
        table: table_ref.clone(),
        where_: delete.selection.as_ref().map(|e| convert_expr(input, e, used_params, custom_types, field_lookup)),
        returning: convert_returning(
            input,
            &delete.returning,
            used_params,
            custom_types,
            field_lookup,
            Some(&table_ref),
        ),
    };
}

fn convert_select(
    input: &GoodQueryInput,
    q: &sql::Query,
    s: &sql::Select,
    used_params: &mut HashSet<String>,
    custom_types: &std::collections::BTreeMap<String, good_ormning_core::pg::schema::custom_type::CustomType>,
    field_lookup: &std::collections::HashMap<TableRef, PgTableInfo>,
) -> Select {
    let table = if s.from.is_empty() {
        NamedSelectSource {
            source: JoinSource::Empty,
            alias: None,
        }
    } else {
        match &s.from[0].relation {
            sql::TableFactor::Table { name, alias, .. } => NamedSelectSource {
                source: JoinSource::Table(get_table_ref(name)),
                alias: Some(
                    alias.as_ref().map(|a| a.name.value.clone()).unwrap_or_else(|| get_table_ref(name).0.clone()),
                ),
            },
            sql::TableFactor::Derived { subquery, alias, .. } => NamedSelectSource {
                source: JoinSource::Subsel(Box::new(convert_select_query(
                    input,
                    subquery,
                    used_params,
                    custom_types,
                    field_lookup,
                ))),
                alias: alias.as_ref().map(|a| a.name.value.clone()),
            },
            _ => unimplemented!("Select table factor"),
        }
    };
    let mut join = vec![];
    if !s.from.is_empty() {
        for j in &s.from[0].joins {
            let source = match &j.relation {
                sql::TableFactor::Table { name, alias, .. } => NamedSelectSource {
                    source: JoinSource::Table(get_table_ref(name)),
                    alias: Some(
                        alias
                            .as_ref()
                            .map(|a| a.name.value.clone())
                            .unwrap_or_else(|| get_table_ref(name).0.clone()),
                    ),
                },
                sql::TableFactor::Derived { subquery, alias, .. } => NamedSelectSource {
                    source: JoinSource::Subsel(Box::new(convert_select_query(
                        input,
                        subquery,
                        used_params,
                        custom_types,
                        field_lookup,
                    ))),
                    alias: alias.as_ref().map(|a| a.name.value.clone()),
                },
                _ => unimplemented!("Join table factor"),
            };
            let type_ = match j.join_operator {
                sql::JoinOperator::LeftOuter(_) => {
                    good_ormning_core::pg::query::select::JoinType::Left
                },
                sql::JoinOperator::Inner(_) => good_ormning_core::pg::query::select::JoinType::Inner,
                _ => unimplemented!("Join type: {:?}", j.join_operator),
            };
            let on = match &j.join_operator {
                sql::JoinOperator::LeftOuter(constraint) | sql::JoinOperator::Inner(constraint) => match constraint {
                    sql::JoinConstraint::On(e) => convert_expr(input, e, used_params, custom_types, field_lookup),
                    _ => unimplemented!("Join constraint"),
                },
                _ => unreachable!(),
            };
            join.push(good_ormning_core::pg::query::select::Join {
                source: Box::new(source),
                type_,
                on,
            });
        }
    }
    let mut order = vec![];
    if let Some(order_by) = &q.order_by {
        for o in &order_by.exprs {
            let e = convert_expr(input, &o.expr, used_params, custom_types, field_lookup);
            let dir = match o.asc {
                Some(true) | None => Order::Asc,
                Some(false) => Order::Desc,
            };
            order.push((e, dir));
        }
    }
    return Select {
        with: None,
        table: table.clone(),
        returning: convert_returning(
            input,
            &Some(s.projection.clone()),
            used_params,
            custom_types,
            field_lookup,
            Some(&match &table.source {
                JoinSource::Table(tr) => tr.clone(),
                _ => TableRef("".into()),
            }),
        ),
        join,
        where_: s.selection.as_ref().map(|e| convert_expr(input, e, used_params, custom_types, field_lookup)),
        group: match &s.group_by {
            sql::GroupByExpr::All(_) => unimplemented!("Group by all"),
            sql::GroupByExpr::Expressions(exprs, _) => exprs
                .iter()
                .map(|e| convert_expr(input, e, used_params, custom_types, field_lookup))
                .collect(),
        },
        having: s.having.as_ref().map(|e| convert_expr(input, e, used_params, custom_types, field_lookup)),
        order,
        limit: q.limit.as_ref().map(|e| convert_expr(input, e, used_params, custom_types, field_lookup)),
        distinct: s.distinct.is_some(),
        junctions: vec![],
    };
}

fn convert_expr(
    input: &GoodQueryInput,
    e: &sql::Expr,
    used_params: &mut HashSet<String>,
    custom_types: &std::collections::BTreeMap<String, good_ormning_core::pg::schema::custom_type::CustomType>,
    field_lookup: &std::collections::HashMap<TableRef, PgTableInfo>,
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
            match v {
                sql::Value::Number(n, _) => {
                    if let Ok(i) = n.parse::<i32>() {
                        return Expr::LitI32(i);
                    } else if let Ok(i) = n.parse::<i64>() {
                        return Expr::LitI64(i);
                    } else {
                        unimplemented!("Number parsing")
                    }
                },
                sql::Value::SingleQuotedString(s) => return Expr::LitString(s.clone()),
                sql::Value::Boolean(b) => return Expr::LitBool(*b),
                sql::Value::Placeholder(p) => {
                    let placeholder_name = p.replace("$", "").replace("?", "");
                    let param_name = if let Ok(idx) = placeholder_name.parse::<usize>() {
                        format!("p{}", idx)
                    } else {
                        placeholder_name
                    };
                    used_params.insert(param_name.clone());
                    let pt =
                        input
                            .param_types
                            .iter()
                            .find(|(ident, _)| ident.to_string() == param_name)
                            .map(|(_, pt)| pt)
                            .unwrap_or_else(|| panic!("Parameter {} not found in params section", param_name));
                    return Expr::Param {
                        name: param_name,
                        type_: param_type_to_pg_type(pt, custom_types),
                    };
                },
                sql::Value::Null => return Expr::LitNull(SimpleType {
                    type_: SimpleSimpleType::I32,
                    custom: None,
                }),
                _ => unimplemented!("Value type: {:?}", v),
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
                sql::BinaryOperator::PGRegexMatch => BinOp::Regexp,
                sql::BinaryOperator::PGRegexIMatch => BinOp::Regexp,
                sql::BinaryOperator::StringConcat => BinOp::StringConcat,
                sql::BinaryOperator::Modulo => BinOp::Mod,
                sql::BinaryOperator::BitwiseAnd => BinOp::BitwiseAnd,
                sql::BinaryOperator::BitwiseOr => BinOp::BitwiseOr,
                sql::BinaryOperator::BitwiseXor => BinOp::BitwiseXor,
                sql::BinaryOperator::PGBitwiseShiftLeft => BinOp::BitwiseShiftLeft,
                sql::BinaryOperator::PGBitwiseShiftRight => BinOp::BitwiseShiftRight,
                _ => unimplemented!("Binary operator: {:?}", op),
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
                _ => unimplemented!("Unary operator: {:?}", op),
            };
            return Expr::PrefixOp {
                op,
                right: Box::new(convert_expr(input, expr, used_params, custom_types, field_lookup)),
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
        sql::Expr::Case { operand, conditions, results, else_result } => {
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
                ilike: false,
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
        sql::Expr::ILike { expr, negated, pattern, escape_char, .. } => {
            let l = convert_expr(input, expr, used_params, custom_types, field_lookup);
            let r = convert_expr(input, pattern, used_params, custom_types, field_lookup);
            let escape = escape_char.as_ref().map(|c| Box::new(Expr::LitString(c.to_string())));
            let res = Expr::Like {
                expr: Box::new(l),
                pattern: Box::new(r),
                escape,
                ilike: true,
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
                    type_: SimpleSimpleType::I32,
                    custom: None,
                })),
            };
        },
        sql::Expr::IsNotNull(expr) => {
            return Expr::BinOp {
                left: Box::new(convert_expr(input, expr, used_params, custom_types, field_lookup)),
                op: BinOp::IsNot,
                right: Box::new(Expr::LitNull(SimpleType {
                    type_: SimpleSimpleType::I32,
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
        sql::Expr::Nested(expr) => return convert_expr(input, expr, used_params, custom_types, field_lookup),
        sql::Expr::Cast { expr, data_type, .. } => {
            let e = convert_expr(input, expr, used_params, custom_types, field_lookup);
            let t = sql_type_to_pg_type(data_type, custom_types);
            return Expr::Cast(Box::new(e), t);
        },
        sql::Expr::Collate { expr, collation } => {
            return Expr::Collate(
                Box::new(convert_expr(input, expr, used_params, custom_types, field_lookup)),
                collation.0.iter().map(|i| i.value.clone()).collect::<Vec<_>>().join("."),
            );
        },
        sql::Expr::InSubquery { expr, subquery, negated } => {
            let l = convert_expr(input, expr, used_params, custom_types, field_lookup);
            let r = convert_select_query(input, subquery, used_params, custom_types, field_lookup);
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
            if let sql::FunctionArguments::List(list) = &f.args {
                for arg in &list.args {
                    match arg {
                        sql::FunctionArg::Unnamed(sql::FunctionArgExpr::Expr(e)) => args.push(
                            convert_expr(input, e, used_params, custom_types, field_lookup),
                        ),
                        sql::FunctionArg::Unnamed(sql::FunctionArgExpr::Wildcard) => {
                            args.push(Expr::LitI32(1));
                        },
                        _ => unimplemented!("Function argument type not supported"),
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
                    "sum" => fn_sum(args.pop().expect("sum requires 1 arg")),
                    "count" => fn_count(args.pop().expect("count requires 1 arg")),
                    "min" => fn_min(args.pop().expect("min requires 1 arg")),
                    "max" => fn_max(args.pop().expect("max requires 1 arg")),
                    "avg" => fn_avg(args.pop().expect("avg requires 1 arg")),
                    "row_number" => fn_row_number(),
                    "rank" => fn_rank(),
                    "dense_rank" => fn_dense_rank(),
                    _ => unimplemented!("Function {} not supported in window", name),
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
                            let dir = match o.asc {
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
                            Some(good_ormning_core::pg::query::expr::WindowFrame {
                                type_: match sql_frame.units {
                                    sql::WindowFrameUnits::Rows => good_ormning_core
                                    ::pg
                                    ::query
                                    ::expr
                                    ::WindowFrameType
                                    ::Rows,
                                    sql::WindowFrameUnits::Range => good_ormning_core
                                    ::pg
                                    ::query
                                    ::expr
                                    ::WindowFrameType
                                    ::Range,
                                    sql::WindowFrameUnits::Groups => good_ormning_core
                                    ::pg
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
                    sql::WindowType::NamedWindow(_) => unimplemented!("Named windows not supported"),
                }
            }
            let mut e = match name.as_str() {
                "sum" => fn_sum(args.pop().expect("sum requires 1 arg")),
                "count" => fn_count(args.pop().expect("count requires 1 arg")),
                "min" => fn_min(args.pop().expect("min requires 1 arg")),
                "max" => fn_max(args.pop().expect("max requires 1 arg")),
                "avg" => fn_avg(args.pop().expect("avg requires 1 arg")),
                _ => unimplemented!("Function {} not supported", name),
            };
            if let Expr::Call { filter: ref mut f_opt, .. } = e {
                *f_opt = filter;
            }
            return e;
        },
        _ => unimplemented!("Expression type not supported: {:?}", e),
    }
}

fn convert_window_frame_bound(
    input: &GoodQueryInput,
    b: &sql::WindowFrameBound,
    used_params: &mut HashSet<String>,
    custom_types: &std::collections::BTreeMap<String, good_ormning_core::pg::schema::custom_type::CustomType>,
    field_lookup: &std::collections::HashMap<TableRef, PgTableInfo>,
) -> good_ormning_core::pg::query::expr::WindowFrameBound {
    match b {
        sql::WindowFrameBound::Preceding(e) => match e {
            Some(e) => {
                return good_ormning_core::pg::query::expr::WindowFrameBound::Preceding(
                    Box::new(convert_expr(input, e, used_params, custom_types, field_lookup)),
                );
            },
            None => {
                return good_ormning_core::pg::query::expr::WindowFrameBound::UnboundedPreceding;
            },
        },
        sql::WindowFrameBound::CurrentRow => return good_ormning_core::pg::query::expr::WindowFrameBound::CurrentRow,
        sql::WindowFrameBound::Following(e) => match e {
            Some(e) => {
                return good_ormning_core::pg::query::expr::WindowFrameBound::Following(
                    Box::new(convert_expr(input, e, used_params, custom_types, field_lookup)),
                );
            },
            None => {
                return good_ormning_core::pg::query::expr::WindowFrameBound::UnboundedFollowing;
            },
        },
    }
}
