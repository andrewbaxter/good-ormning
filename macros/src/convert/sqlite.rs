use sqlparser::ast as sql;
use std::collections::HashSet;
use crate::GoodQueryInput;
use good_ormning::sqlite::{
    query::{
        expr::{
            Expr,
            BinOp,
        },
        select::{
            Select,
            NamedSelectSource,
            JoinSource,
            Join,
            JoinType,
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
        Type,
    },
    Query,
};
use good_ormning::QueryResCount;
use super::{
    param_type_to_sqlite_type,
    sql_type_to_sqlite_type,
};

pub fn convert_query(
    input: &GoodQueryInput,
    statement: &sql::Statement,
    custom_types: &std::collections::BTreeMap<String, good_ormning::sqlite::schema::custom_type::CustomType>,
) -> Query {
    let mut used_params = HashSet::new();
    let body: Box<dyn QueryBody> = match statement {
        sql::Statement::Query(q) => Box::new(convert_select_query(input, q, &mut used_params, custom_types)),
        sql::Statement::Insert(insert) => Box::new(
            convert_insert(input, insert, &mut used_params, custom_types),
        ),
        sql::Statement::Update { table, assignments, selection, returning, .. } => Box::new(
            convert_update(input, table, assignments, selection, returning, &mut used_params, custom_types),
        ),
        sql::Statement::Delete(delete) => Box::new(
            convert_delete(input, delete, &mut used_params, custom_types),
        ),
        _ => unimplemented!("Unsupported statement type: {:?}", statement),
    };
    for (ident, _) in &input.params {
        if !used_params.contains(&ident.to_string()) {
            panic!("Parameter {} not used in query", ident);
        }
    }
    Query {
        name: "unnamed".to_string(),
        body,
        // Dummy
        res_count: QueryResCount::Many,
        res_name: None,
    }
}

fn convert_select_query(
    input: &GoodQueryInput,
    q: &sql::Query,
    used_params: &mut HashSet<String>,
    custom_types: &std::collections::BTreeMap<String, good_ormning::sqlite::schema::custom_type::CustomType>,
) -> Select {
    match &*q.body {
        sql::SetExpr::Select(s) => {
            let mut sel = convert_select(input, q, s, used_params, custom_types);
            if let Some(with) = &q.with {
                let mut ctes = vec![];
                for cte in &with.cte_tables {
                    let name = cte.alias.name.value.clone();
                    let query = convert_select_query(input, &cte.query, used_params, custom_types);
                    let mut builder = CteBuilder::new(name, Box::new(query.clone()));
                    if !cte.alias.columns.is_empty() {
                        for col in &cte.alias.columns {
                            builder = builder.column(
                                col.value.clone(),
                                good_ormning::sqlite::types::type_i32().build(),
                            );
                        }
                    } else {
                        for r in &query.returning {
                            let r_name = r.rename.clone().unwrap_or_else(|| "column".to_string());
                            builder = builder.column(r_name, good_ormning::sqlite::types::type_i32().build());
                        }
                    }
                    ctes.push(good_ormning::sqlite::query::utils::Cte::from(builder));
                }
                sel.with = Some(With {
                    recursive: with.recursive,
                    ctes,
                });
            }
            sel
        },
        sql::SetExpr::SetOperation { left, op, right, .. } => {
            let mut l = match &**left {
                sql::SetExpr::Select(s) => convert_select(input, q, s, used_params, custom_types),
                _ => unimplemented!("Nested set operations on left"),
            };
            let r = match &**right {
                sql::SetExpr::Select(s) => convert_select(input, q, s, used_params, custom_types),
                _ => unimplemented!("Nested set operations on right"),
            };
            let operator = match op {
                sql::SetOperator::Union => good_ormning::sqlite::query::select_body::SelectJunctionOperator::Union,
                sql::SetOperator::Intersect => good_ormning::sqlite::query::select_body::SelectJunctionOperator::Intersect,
                sql::SetOperator::Except => good_ormning::sqlite::query::select_body::SelectJunctionOperator::Except,
                _ => unimplemented!("Set operator not supported: {:?}", op),
            };
            l.junction.push(good_ormning::sqlite::query::select_body::SelectJunction {
                op: operator,
                body: Box::new(r),
            });
            l
        },
        _ => unimplemented!("Select body type not supported"),
    }
}

fn get_table_ref(name: &sql::ObjectName) -> TableRef {
    TableRef(name.0.last().expect("ObjectName should not be empty").value.clone())
}

fn convert_returning(
    input: &GoodQueryInput,
    returning: &Option<Vec<sql::SelectItem>>,
    used_params: &mut HashSet<String>,
    custom_types: &std::collections::BTreeMap<String, good_ormning::sqlite::schema::custom_type::CustomType>,
) -> Vec<Returning> {
    let mut out = vec![];
    if let Some(items) = returning {
        for item in items {
            match item {
                sql::SelectItem::UnnamedExpr(e) => out.push(Returning {
                    e: convert_expr(input, e, used_params, custom_types),
                    rename: None,
                }),
                sql::SelectItem::ExprWithAlias { expr, alias } => out.push(Returning {
                    e: convert_expr(input, expr, used_params, custom_types),
                    rename: Some(alias.value.clone()),
                }),
                _ => unimplemented!("Unsupported returning item"),
            }
        }
    }
    out
}

fn convert_insert(
    input: &GoodQueryInput,
    insert: &sql::Insert,
    used_params: &mut HashSet<String>,
    custom_types: &std::collections::BTreeMap<String, good_ormning::sqlite::schema::custom_type::CustomType>,
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
                    values.push((field, convert_expr(input, expr, used_params, custom_types)));
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
                        updates.push((field, convert_expr(input, &a.value, used_params, custom_types)));
                    }
                    let conflict = if let Some(target) = &oc.conflict_target {
                        match target {
                            sql::ConflictTarget::Columns(idents) => idents
                                .iter()
                                .map(|id| FieldRef {
                                    table_id: table.0.clone(),
                                    field_id: id.value.clone(),
                                })
                                .collect(),
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
        match insert.or {
            Some(sql::SqliteOnConflict::Ignore) => Some(InsertConflict::DoNothing),
            _ => None,
        }
    };
    Insert {
        table,
        values,
        on_conflict,
        returning: convert_returning(input, &insert.returning, used_params, custom_types),
    }
}

fn convert_update(
    input: &GoodQueryInput,
    table: &sql::TableWithJoins,
    assignments: &[sql::Assignment],
    selection: &Option<sql::Expr>,
    returning: &Option<Vec<sql::SelectItem>>,
    used_params: &mut HashSet<String>,
    custom_types: &std::collections::BTreeMap<String, good_ormning::sqlite::schema::custom_type::CustomType>,
) -> Update {
    let table_ref = match &table.relation {
        sql::TableFactor::Table { name, .. } => get_table_ref(name),
        _ => unimplemented!("Update table factor"),
    };
    let mut values = vec![];
    for a in assignments {
        let target_name = match &a.target {
            sql::AssignmentTarget::ColumnName(name) => name.0.last().unwrap().value.clone(),
            _ => unimplemented!("Assignment target"),
        };
        let field = FieldRef {
            table_id: table_ref.0.clone(),
            field_id: target_name,
        };
        values.push((field, convert_expr(input, &a.value, used_params, custom_types)));
    }
    Update {
        table: table_ref,
        values,
        where_: selection.as_ref().map(|e| convert_expr(input, e, used_params, custom_types)),
        returning: convert_returning(input, returning, used_params, custom_types),
    }
}

fn convert_delete(
    input: &GoodQueryInput,
    delete: &sql::Delete,
    used_params: &mut HashSet<String>,
    custom_types: &std::collections::BTreeMap<String, good_ormning::sqlite::schema::custom_type::CustomType>,
) -> Delete {
    let relation = match &delete.from {
        sql::FromTable::WithFromKeyword(f) => &f[0].relation,
        sql::FromTable::WithoutKeyword(f) => &f[0].relation,
    };
    let table_ref = match relation {
        sql::TableFactor::Table { name, .. } => get_table_ref(name),
        _ => unimplemented!("Delete table factor"),
    };
    Delete {
        table: table_ref,
        where_: delete.selection.as_ref().map(|e| convert_expr(input, e, used_params, custom_types)),
        returning: convert_returning(input, &delete.returning, used_params, custom_types),
    }
}

fn convert_select(
    input: &GoodQueryInput,
    q: &sql::Query,
    s: &sql::Select,
    used_params: &mut HashSet<String>,
    custom_types: &std::collections::BTreeMap<String, good_ormning::sqlite::schema::custom_type::CustomType>,
) -> Select {
    let table = match &s.from[0].relation {
        sql::TableFactor::Table { name, alias, args, .. } => {
            if let Some(args) = args {
                 let name_str = name.to_string().to_lowercase();
                 if name_str == "rarray" {
                     if !args.args.is_empty() {
                         if let sql::FunctionArg::Unnamed(sql::FunctionArgExpr::Expr(e)) = &args.args[0] {
                             let expr = convert_expr(input, e, used_params, custom_types);
                             return Select {
                                 with: None,
                                 table: NamedSelectSource {
                                     source: JoinSource::Func("__good_ormning_rarray".to_string(), vec![expr]),
                                     alias: alias.as_ref().map(|a| a.name.value.clone()),
                                 },
                                 returning: convert_returning(input, &Some(s.projection.clone()), used_params, custom_types),
                                 junction: vec![],
                                 join: vec![],
                                 where_: s.selection.as_ref().map(|e| convert_expr(input, e, used_params, custom_types)),
                                 group: match &s.group_by {
                                     sql::GroupByExpr::All(_) => unimplemented!("Group by all"),
                                     sql::GroupByExpr::Expressions(exprs, _) => exprs
                                         .iter()
                                         .map(|e| convert_expr(input, e, used_params, custom_types))
                                         .collect(),
                                 },
                                 order: vec![],
                                 limit: None,
                             };
                         }
                     }
                 }
            }
            NamedSelectSource {
                source: JoinSource::Table(get_table_ref(name)),
                alias: Some(
                    alias
                        .as_ref()
                        .map(|a| a.name.value.clone())
                        .unwrap_or_else(|| get_table_ref(name).0.clone()),
                ),
            }
        },
        _ => unimplemented!("Select table factor"),
    };
    let mut join = vec![];
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
            _ => unimplemented!("Join table factor"),
        };
        let type_ = match j.join_operator {
            sql::JoinOperator::LeftOuter(_) => JoinType::Left,
            sql::JoinOperator::Inner(_) => JoinType::Inner,
            _ => unimplemented!("Join type: {:?}", j.join_operator),
        };
        let on = match &j.join_operator {
            sql::JoinOperator::LeftOuter(constraint)
            | sql::JoinOperator::Inner(constraint) => match constraint {
                sql::JoinConstraint::On(e) => convert_expr(input, e, used_params, custom_types),
                _ => unimplemented!("Join constraint"),
            },
            _ => unreachable!(),
        };
        join.push(Join {
            source: source,
            type_,
            on,
        });
    }
    let mut order = vec![];
    if let Some(order_by) = &q.order_by {
        for o in &order_by.exprs {
            let e = convert_expr(input, &o.expr, used_params, custom_types);
            let dir = match o.asc {
                Some(true) | None => Order::Asc,
                Some(false) => Order::Desc,
            };
            order.push((e, dir));
        }
    }
    Select {
        with: None,
        table,
        returning: convert_returning(input, &Some(s.projection.clone()), used_params, custom_types),
        junction: vec![],
        join,
        where_: s.selection.as_ref().map(|e| convert_expr(input, e, used_params, custom_types)),
        group: match &s.group_by {
            sql::GroupByExpr::All(_) => unimplemented!("Group by all"),
            sql::GroupByExpr::Expressions(exprs, _) => exprs
                .iter()
                .map(|e| convert_expr(input, e, used_params, custom_types))
                .collect(),
        },
        order,
        limit: q.limit.as_ref().map(|e| convert_expr(input, e, used_params, custom_types)),
    }
}

fn convert_expr(
    input: &GoodQueryInput,
    e: &sql::Expr,
    used_params: &mut HashSet<String>,
    custom_types: &std::collections::BTreeMap<String, good_ormning::sqlite::schema::custom_type::CustomType>,
) -> Expr {
    match e {
        sql::Expr::Identifier(ident) => {
            Expr::Field(FieldRef {
                table_id: "".into(),
                field_id: ident.value.clone(),
            })
        },
        sql::Expr::CompoundIdentifier(idents) => {
            let table_id = idents[0].value.clone();
            let id = idents[1].value.clone();
            Expr::Field(FieldRef {
                table_id,
                field_id: id,
            })
        },
        sql::Expr::Value(v) => {
            match v {
                sql::Value::Number(n, _) => {
                    if let Ok(i) = n.parse::<i32>() {
                        Expr::LitI32(i)
                    } else if let Ok(i) = n.parse::<i64>() {
                        Expr::LitI64(i)
                    } else {
                        unimplemented!("Number parsing")
                    }
                },
                sql::Value::SingleQuotedString(s) => Expr::LitString(s.clone()),
                sql::Value::Boolean(b) => Expr::LitBool(*b),
                sql::Value::Placeholder(p) => {
                    let placeholder_name = p.replace("$", "").replace("?", "");
                    let param_name = if let Ok(idx) = placeholder_name.parse::<usize>() {
                        format!("p{}", idx - 1)
                    } else {
                        placeholder_name
                    };
                    used_params.insert(param_name.clone());
                    let pt = input
                        .params
                        .iter()
                        .find(|(ident, _)| ident.to_string() == param_name)
                        .map(|(_, pt)| pt)
                        .unwrap_or_else(|| panic!("Parameter {} not found in params section", param_name));
                    Expr::Param {
                        name: param_name,
                        type_: param_type_to_sqlite_type(pt, custom_types),
                    }
                },
                sql::Value::Null => Expr::LitNull(SimpleType {
                    type_: SimpleSimpleType::I32,
                    custom: None,
                }),
                _ => unimplemented!("Value type: {:?}", v),
            }
        },
        sql::Expr::BinaryOp { left, op, right } => {
            let l = convert_expr(input, left, used_params, custom_types);
            let r = convert_expr(input, right, used_params, custom_types);
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
                _ => unimplemented!("Binary operator: {:?}", op),
            };
            Expr::BinOp {
                left: Box::new(l),
                op: o,
                right: Box::new(r),
            }
        },
        sql::Expr::Nested(expr) => convert_expr(input, expr, used_params, custom_types),
        sql::Expr::Cast { expr, data_type, .. } => {
            let e = convert_expr(input, expr, used_params, custom_types);
            let t = sql_type_to_sqlite_type(data_type, custom_types);
            Expr::Cast(Box::new(e), t)
        },
        sql::Expr::InSubquery { expr, subquery, negated } => {
            let l = convert_expr(input, expr, used_params, custom_types);
            let r = convert_select_query(input, subquery, used_params, custom_types);
            Expr::BinOp {
                left: Box::new(l),
                op: if *negated {
                    BinOp::NotIn
                } else {
                    BinOp::In
                },
                right: Box::new(Expr::Select(Box::new(r))),
            }
        },
        sql::Expr::InList { expr, list, negated } => {
            let l = convert_expr(input, expr, used_params, custom_types);
            let r = Expr::LitArray(
                list.iter().map(|e| convert_expr(input, e, used_params, custom_types)).collect(),
            );
            Expr::BinOp {
                left: Box::new(l),
                op: if *negated {
                    BinOp::NotIn
                } else {
                    BinOp::In
                },
                right: Box::new(r),
            }
        },
        sql::Expr::Function(f) => {
            let name = f.name.to_string().to_lowercase();
            let mut args = vec![];
            if let sql::FunctionArguments::List(list) = &f.args {
                for arg in &list.args {
                    match arg {
                        sql::FunctionArg::Unnamed(sql::FunctionArgExpr::Expr(e)) => args.push(
                            convert_expr(input, e, used_params, custom_types),
                        ),
                        _ => unimplemented!("Function argument type not supported"),
                    }
                }
            }

            if let Some(over) = &f.over {
                let arg = if args.is_empty() {
                     Expr::LitI32(0) // DUMMY
                } else {
                     args.pop().unwrap()
                };
                let e = match name.as_str() {
                    "sum" => fn_sum(arg),
                    "count" => fn_count(arg),
                    "min" => fn_min(arg),
                    "max" => fn_max(arg),
                    "avg" => fn_avg(arg),
                    _ => unimplemented!("Function {} not supported in window", name),
                };
                match over {
                    sql::WindowType::WindowSpec(spec) => {
                        let mut partition_by = vec![];
                        for p in &spec.partition_by {
                            partition_by.push(convert_expr(input, p, used_params, custom_types));
                        }
                        let mut order_by = vec![];
                        for o in &spec.order_by {
                            let expr = convert_expr(input, &o.expr, used_params, custom_types);
                            let dir = match o.asc {
                                Some(true) | None => Order::Asc,
                                Some(false) => Order::Desc,
                            };
                            order_by.push((expr, dir));
                        }
                        return Expr::Window {
                            expr: Box::new(e),
                            partition_by,
                            order_by,
                        };
                    },
                    sql::WindowType::NamedWindow(_) => unimplemented!("Named windows not supported"),
                }
            }

            if args.len() != 1 {
                unimplemented!("Only 1-argument functions supported");
            }
            let expr = args.pop().expect("args should have exactly 1 element");
            match name.as_str() {
                "sum" => fn_sum(expr),
                "count" => fn_count(expr),
                "min" => fn_min(expr),
                "max" => fn_max(expr),
                "avg" => fn_avg(expr),
                _ => unimplemented!("Function {} not supported", name),
            }
        },
        _ => unimplemented!("Expression type not supported: {:?}", e),
    }
}
