use sqlparser::ast as sql;
use quote::quote;
use crate::GoodQueryInput;
use good_ormning::sqlite::{
    query::{
        expr::{
            Expr,
            BinOp,
            Binding,
        },
        select::{
            Select,
            NamedSelectSource,
            JoinSource,
        },
        insert::Insert,
        update::Update,
        delete::Delete,
        utils::{
            Returning,
            QueryBody,
        },
        helpers::*,
    },
    schema::{
        table::TableRef,
        field::FieldRef,
    },
    types::{
        Type,
        SimpleType,
        SimpleSimpleType,
    },
    Query,
};
use good_ormning::QueryResCount;

pub fn convert_query(input: &GoodQueryInput, statement: &sql::Statement) -> Query {
    let body: Box<dyn QueryBody> = match statement {
        sql::Statement::Query(q) => Box::new(convert_select(input, q)),
        sql::Statement::Insert(insert) => Box::new(convert_insert(input, insert)),
        sql::Statement::Update { table, assignments, selection, returning, .. } => Box::new(
            convert_update(input, table, assignments, selection, returning),
        ),
        sql::Statement::Delete(delete) => Box::new(convert_delete(input, delete)),
        _ => unimplemented!("Unsupported statement type: {:?}", statement),
    };
    Query {
        name: "unnamed".to_string(),
        body,
        // Dummy
        res_count: QueryResCount::Many,
        res_name: None,
    }
}

fn get_table_ref(name: &sql::ObjectName) -> TableRef {
    TableRef(name.0.last().unwrap().value.clone())
}

fn convert_returning(input: &GoodQueryInput, returning: &Option<Vec<sql::SelectItem>>) -> Vec<Returning> {
    let mut out = vec![];
    if let Some(items) = returning {
        for item in items {
            match item {
                sql::SelectItem::UnnamedExpr(e) => out.push(Returning {
                    e: convert_expr(input, e),
                    rename: None,
                }),
                sql::SelectItem::ExprWithAlias { expr, alias } => out.push(Returning {
                    e: convert_expr(input, expr),
                    rename: Some(alias.value.clone()),
                }),
                _ => unimplemented!("Unsupported returning item"),
            }
        }
    }
    out
}

fn convert_insert(input: &GoodQueryInput, insert: &sql::Insert) -> Insert {
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
                    values.push((field, convert_expr(input, expr)));
                }
            }
        } else {
            unimplemented!("Only values supported for insert")
        }
    }
    Insert {
        table,
        values,
        // implement if needed
        on_conflict: None,
        returning: convert_returning(input, &insert.returning),
    }
}

fn convert_update(
    input: &GoodQueryInput,
    table: &sql::TableWithJoins,
    assignments: &[sql::Assignment],
    selection: &Option<sql::Expr>,
    returning: &Option<Vec<sql::SelectItem>>,
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
        values.push((field, convert_expr(input, &a.value)));
    }
    Update {
        table: table_ref,
        values,
        where_: selection.as_ref().map(|e| convert_expr(input, e)),
        returning: convert_returning(input, returning),
    }
}

fn convert_delete(input: &GoodQueryInput, delete: &sql::Delete) -> Delete {
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
        where_: delete.selection.as_ref().map(|e| convert_expr(input, e)),
        returning: convert_returning(input, &delete.returning),
    }
}

fn convert_select(input: &GoodQueryInput, q: &sql::Query) -> Select {
    match &*q.body {
        sql::SetExpr::Select(s) => {
            let table = match &s.from[0].relation {
                sql::TableFactor::Table { name, alias, .. } => {
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
            Select {
                with: None,
                table,
                returning: convert_returning(input, &Some(s.projection.clone())),
                junction: vec![],
                join: vec![],
                where_: s.selection.as_ref().map(|e| convert_expr(input, e)),
                group: match &s.group_by {
                    sql::GroupByExpr::All(_) => unimplemented!("Group by all"),
                    sql::GroupByExpr::Expressions(exprs, _) => exprs
                        .iter()
                        .map(|e| convert_expr(input, e))
                        .collect(),
                },
                order: vec![],
                limit: q.limit.as_ref().map(|e| convert_expr(input, e)),
            }
        },
        _ => unimplemented!("Select body"),
    }
}

fn convert_expr(input: &GoodQueryInput, e: &sql::Expr) -> Expr {
    match e {
        sql::Expr::Identifier(ident) => {
            Expr::Field(good_ormning::sqlite::schema::field::FieldRef {
                table_id: "".into(),
                field_id: ident.value.clone(),
            })
        },
        sql::Expr::CompoundIdentifier(idents) => {
            let table_id = idents[0].value.clone();
            let id = idents[1].value.clone();
            Expr::Field(good_ormning::sqlite::schema::field::FieldRef {
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
                    let num_str = p.replace("$", "").replace("?", "");
                    if let Ok(idx) = num_str.parse::<usize>() {
                        let (ident, ty) = &input.params[idx - 1];
                        let type_str = quote!(#ty).to_string().replace(" ", "");
                        let mut is_opt = type_str.contains("Option<");
                        let inner_type = if is_opt {
                            type_str.replace("Option<", "").replace(">", "")
                        } else {
                            type_str.clone()
                        };
                        let (simple_type, custom) = match inner_type.as_str() {
                            "i32" => (SimpleSimpleType::I32, None),
                            "i64" => (SimpleSimpleType::I64, None),
                            "u32" => (SimpleSimpleType::U32, None),
                            "bool" => (SimpleSimpleType::Bool, None),
                            "String" | "&str" => (SimpleSimpleType::String, None),
                            "chrono::DateTime<chrono::Utc>" => (SimpleSimpleType::UtcTimeSChrono, None),
                            "jiff::Timestamp" => (SimpleSimpleType::UtcTimeSJiff, None),
                            _ => (SimpleSimpleType::I32, Some(inner_type)),
                        };
                        Expr::Param {
                            name: ident.to_string(),
                            type_: Type {
                                type_: SimpleType {
                                    type_: simple_type,
                                    custom: custom,
                                },
                                opt: is_opt,
                                arr: false,
                            },
                        }
                    } else {
                        unimplemented!("Placeholder parsing: {}", p)
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
            let l = convert_expr(input, left);
            let r = convert_expr(input, right);
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
        sql::Expr::Function(f) => {
            let name = f.name.to_string().to_lowercase();
            let mut args = vec![];
            if let sql::FunctionArguments::List(list) = &f.args {
                for arg in &list.args {
                    match arg {
                        sql::FunctionArg::Unnamed(sql::FunctionArgExpr::Expr(e)) => args.push(
                            convert_expr(input, e),
                        ),
                        _ => unimplemented!("Function argument type not supported"),
                    }
                }
            }
            if args.len() != 1 {
                unimplemented!("Only 1-argument functions supported");
            }
            let expr = args.pop().unwrap();
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
