use {
    good_ormning_core::mariadb::{
        Version,
        query::expr::SerialExpr,
        schema::{
            constraint::{
                Constraint,
                ConstraintType,
                ForeignKeyAction,
                ForeignKeyDef,
                PrimaryKeyDef,
            },
            field::{
                Field,
                FieldType,
                TypeVariant,
            },
            index::Index,
            table::Table,
        },
        types::{
            SimpleSimpleType,
            SimpleType,
            Temporal,
            Type,
            temporal,
        },
    },
    loga::ResultContext,
    mysql_async::{
        Conn,
        prelude::Queryable,
    },
    std::collections::{
        BTreeMap,
        HashMap,
        HashSet,
    },
};

pub async fn read_schema(conn: &mut Conn, prefer_chrono: bool) -> Result<(Version, Vec<String>), loga::Error> {
    let mut failures = vec![];
    let table_names: Vec<String> = conn.query(
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             table_name
           from
             information_schema.tables
           where
             table_schema = database()
             and table_type = 'BASE TABLE'
             and table_name not like '\_\_good\_%'
           order by
             table_name
           "#,
    ).await.context("Querying table list")?;
    let mut tables: BTreeMap<String, Table> = BTreeMap::new();
    for table_name in &table_names {
        let col_rows:
            Vec<
                (
                    String,
                    String,
                    String,
                    String,
                    String,
                    Option<u64>,
                    Option<u64>,
                    Option<u64>,
                    Option<String>,
                    Option<u64>,
                ),
            > =
            conn.exec(
                //# genemichaels-external: sql-formatter-mariadb
                r#"select
                 column_name,
                 data_type,
                 column_type,
                 is_nullable,
                 extra,
                 character_maximum_length,
                 numeric_precision,
                 numeric_scale,
                 column_default,
                 datetime_precision
               from
                 information_schema.columns
               where
                 table_schema = database()
                 and table_name = ?
               order by
                 ordinal_position
               "#,
                (table_name,),
            ).await.context("Querying columns")?;
        let mut fields: BTreeMap<String, Field> = BTreeMap::new();
        for (
            col_name,
            data_type,
            column_type,
            is_nullable,
            extra,
            max_len,
            numeric_precision,
            numeric_scale,
            column_default,
            datetime_precision,
        ) in col_rows {
            let unsigned = column_type.contains("unsigned");
            let extra_rest = extra.to_lowercase().replace("auto_increment", "");
            if !extra_rest.trim().is_empty() {
                failures.push(
                    format!(
                        "Column {:?} of table {:?} has unsupported attributes {:?}",
                        col_name,
                        table_name,
                        extra_rest.trim()
                    ),
                );
                continue;
            }
            if unsigned && matches!(data_type.as_str(), "decimal" | "float" | "double") {
                failures.push(
                    format!(
                        "Column {:?} of table {:?} is an unsigned {}, which isn't supported",
                        col_name,
                        table_name,
                        data_type
                    ),
                );
                continue;
            }
            let mut field_max_len = None;
            let mut field_precision = None;
            let mut field_scale = None;
            let mut variant = None;
            let sst = match (data_type.as_str(), unsigned) {
                ("tinyint", false) if column_type.starts_with("tinyint(1)") => SimpleSimpleType::Bool,
                ("tinyint", false) => SimpleSimpleType::I8,
                ("tinyint", true) => SimpleSimpleType::U8,
                ("smallint", false) => SimpleSimpleType::I16,
                ("smallint", true) => SimpleSimpleType::U16,
                ("mediumint", false) => {
                    variant = Some(TypeVariant::Mediumint);
                    SimpleSimpleType::I32
                },
                ("mediumint", true) => {
                    variant = Some(TypeVariant::Mediumint);
                    SimpleSimpleType::U32
                },
                ("int", false) => SimpleSimpleType::I32,
                ("int", true) => SimpleSimpleType::U32,
                ("bigint", false) => SimpleSimpleType::I64,
                ("bigint", true) => SimpleSimpleType::U64,
                ("float", _) => SimpleSimpleType::F32,
                ("double", _) => SimpleSimpleType::F64,
                ("decimal", _) => {
                    field_precision = numeric_precision.map(|p| p as u8);
                    field_scale = numeric_scale.map(|s| s as u8);
                    SimpleSimpleType::Decimal
                },
                ("char", _) => {
                    variant = Some(TypeVariant::Char);
                    field_max_len = max_len.map(|l| l as u32);
                    SimpleSimpleType::String
                },
                ("varchar", _) => {
                    field_max_len = max_len.map(|l| l as u32);
                    SimpleSimpleType::String
                },
                ("tinytext", _) => {
                    variant = Some(TypeVariant::TinyText);
                    SimpleSimpleType::String
                },
                ("text", _) => {
                    variant = Some(TypeVariant::Text);
                    SimpleSimpleType::String
                },
                ("mediumtext", _) => {
                    variant = Some(TypeVariant::MediumText);
                    SimpleSimpleType::String
                },
                ("longtext", _) => SimpleSimpleType::String,
                ("binary", _) => {
                    variant = Some(TypeVariant::Binary);
                    field_max_len = max_len.map(|l| l as u32);
                    SimpleSimpleType::Bytes
                },
                ("varbinary", _) => {
                    field_max_len = max_len.map(|l| l as u32);
                    SimpleSimpleType::Bytes
                },
                ("tinyblob", _) => {
                    variant = Some(TypeVariant::TinyBlob);
                    SimpleSimpleType::Bytes
                },
                ("blob", _) => {
                    variant = Some(TypeVariant::Blob);
                    SimpleSimpleType::Bytes
                },
                ("mediumblob", _) => {
                    variant = Some(TypeVariant::MediumBlob);
                    SimpleSimpleType::Bytes
                },
                ("longblob", _) => SimpleSimpleType::Bytes,
                ("datetime" | "timestamp", _) => {
                    if data_type == "timestamp" {
                        variant = Some(TypeVariant::Timestamp);
                    }
                    field_precision = datetime_precision.filter(|p| *p > 0).map(|p| p as u8);
                    if prefer_chrono {
                        SimpleSimpleType::DatetimeChrono
                    } else {
                        SimpleSimpleType::DatetimeJiff
                    }
                },
                ("date", _) => {
                    if prefer_chrono {
                        SimpleSimpleType::DateChrono
                    } else {
                        SimpleSimpleType::DateJiff
                    }
                },
                ("time", _) => {
                    field_precision = datetime_precision.filter(|p| *p > 0).map(|p| p as u8);
                    if prefer_chrono {
                        SimpleSimpleType::TimeChrono
                    } else {
                        SimpleSimpleType::TimeJiff
                    }
                },
                ("year", _) => {
                    variant = Some(TypeVariant::Year);
                    SimpleSimpleType::U16
                },
                _ => {
                    failures.push(
                        format!(
                            "Column {:?} of table {:?} has unsupported SQL type {:?}",
                            col_name,
                            table_name,
                            column_type
                        ),
                    );
                    continue;
                },
            };
            let default = match column_default.as_deref() {
                None | Some("NULL") => None,
                Some(d) => {
                    if d.len() >= 2 && d.starts_with('\'') && d.ends_with('\'') {
                        let mut text = String::new();
                        let mut chars = d[1 .. d.len() - 1].chars();
                        while let Some(c) = chars.next() {
                            match c {
                                '\\' => {
                                    match chars.next() {
                                        Some('n') => text.push('\n'),
                                        Some('r') => text.push('\r'),
                                        Some('t') => text.push('\t'),
                                        Some('0') => text.push('\0'),
                                        Some(c) => text.push(c),
                                        None => { },
                                    }
                                },
                                '\'' => {
                                    chars.next();
                                    text.push('\'');
                                },
                                c => text.push(c),
                            }
                        }
                        match temporal(&sst) {
                            _ if sst == SimpleSimpleType::Bytes => Some(SerialExpr::LitBytes(text.into_bytes())),
                            None => Some(SerialExpr::LitString(text)),
                            Some(_) if text.starts_with("0000-00-00") => {
                                failures.push(
                                    format!(
                                        "Column {:?} of table {:?} has a zero date default, which can't be represented",
                                        col_name,
                                        table_name
                                    ),
                                );
                                continue;
                            },
                            Some(Temporal::Date) => Some(SerialExpr::LitDate(text)),
                            Some(Temporal::Time) => Some(SerialExpr::LitTime(text)),
                            Some(Temporal::Datetime) => Some(SerialExpr::LitTimestamp(text)),
                        }
                    } else if let Ok(i) = d.parse::<i64>() {
                        if sst == SimpleSimpleType::Bool {
                            Some(SerialExpr::LitBool(i != 0))
                        } else {
                            Some(SerialExpr::LitAnyInt(i))
                        }
                    } else if d.parse::<f64>().is_ok() {
                        Some(SerialExpr::LitAnyDecimal(d.to_string()))
                    } else {
                        failures.push(
                            format!(
                                "Column {:?} of table {:?} has default {:?} which isn't a literal, so it can't be imported",
                                col_name,
                                table_name,
                                d
                            ),
                        );
                        continue;
                    }
                },
            };
            fields.insert(col_name.clone(), Field {
                id: col_name.clone(),
                renamed_from: None,
                type_: FieldType {
                    type_: Type {
                        type_: SimpleType {
                            type_: sst,
                            custom: None,
                        },
                        opt: is_nullable == "YES",
                    },
                    auto_increment: extra.contains("auto_increment"),
                    default: default,
                    max_len: field_max_len,
                    precision: field_precision,
                    scale: field_scale,
                    variant: variant,
                },
            });
        }
        let check_names: Vec<String> = conn.exec(
            //# genemichaels-external: sql-formatter-mariadb
            r#"select
                 constraint_name
               from
                 information_schema.check_constraints
               where
                 constraint_schema = database()
                 and table_name = ?
               "#,
            (table_name,),
        ).await.context("Querying check constraints")?;
        for check_name in check_names {
            failures.push(format!("Check constraint {:?} on table {:?} isn't supported", check_name, table_name));
        }
        let mut constraints: BTreeMap<String, Constraint> = BTreeMap::new();
        let pk_fields: Vec<String> = conn.exec(
            //# genemichaels-external: sql-formatter-mariadb
            r#"select
                 column_name
               from
                 information_schema.key_column_usage
               where
                 table_schema = database()
                 and table_name = ?
                 and constraint_name = 'PRIMARY'
               order by
                 ordinal_position
               "#,
            (table_name,),
        ).await.context("Querying primary key")?;
        if !pk_fields.is_empty() {
            constraints.insert("PRIMARY".to_string(), Constraint {
                id: "PRIMARY".to_string(),
                renamed_from: None,
                type_: ConstraintType::PrimaryKey(PrimaryKeyDef { fields: pk_fields }),
            });
        }
        let fk_rows: Vec<(String, String, String, String, String, String)> = conn.exec(
            //# genemichaels-external: sql-formatter-mariadb
            r#"select
                 k.constraint_name,
                 k.column_name,
                 k.referenced_table_name,
                 k.referenced_column_name,
                 r.delete_rule,
                 r.update_rule
               from
                 information_schema.key_column_usage k
                 join information_schema.referential_constraints r on r.constraint_schema = k.constraint_schema
                 and r.constraint_name = k.constraint_name
                 and r.table_name = k.table_name
               where
                 k.table_schema = database()
                 and k.table_name = ?
                 and k.referenced_table_name is not null
               order by
                 k.constraint_name,
                 k.ordinal_position
               "#,
            (table_name,),
        ).await.context("Querying foreign keys")?;
        let mut fk_map: BTreeMap<String, (String, String, String, Vec<(String, String)>)> = BTreeMap::new();
        for (constraint_name, local_col, remote_table, remote_col, delete_rule, update_rule) in fk_rows {
            fk_map
                .entry(constraint_name)
                .or_insert_with(|| (remote_table, delete_rule, update_rule, vec![]))
                .3
                .push((local_col, remote_col));
        }
        for (fk_name, (remote_table, delete_rule, update_rule, col_pairs)) in &fk_map {
            let mut actions = vec![];
            for rule in [delete_rule, update_rule] {
                actions.push(match rule.as_str() {
                    "CASCADE" => Some(ForeignKeyAction::Cascade),
                    "SET NULL" => Some(ForeignKeyAction::SetNull),
                    "NO ACTION" => Some(ForeignKeyAction::NoAction),
                    "RESTRICT" => None,
                    other => {
                        failures.push(
                            format!(
                                "Foreign key {:?} on table {:?} has unsupported action {:?}",
                                fk_name,
                                table_name,
                                other
                            ),
                        );
                        None
                    },
                });
            }
            constraints.insert(fk_name.clone(), Constraint {
                id: fk_name.clone(),
                renamed_from: None,
                type_: ConstraintType::ForeignKey(ForeignKeyDef {
                    remote_table: remote_table.clone(),
                    fields: col_pairs.clone(),
                    on_update: actions.pop().unwrap(),
                    on_delete: actions.pop().unwrap(),
                }),
            });
        }
        let idx_rows: Vec<(String, i64, String, Option<u64>, String, Option<String>)> = conn.exec(
            //# genemichaels-external: sql-formatter-mariadb
            r#"select
                 index_name,
                 non_unique,
                 column_name,
                 sub_part,
                 index_type,
                 collation
               from
                 information_schema.statistics
               where
                 table_schema = database()
                 and table_name = ?
                 and index_name <> 'PRIMARY'
               order by
                 index_name,
                 seq_in_index
               "#,
            (table_name,),
        ).await.context("Querying indices")?;
        let mut idx_map: HashMap<String, (bool, Vec<String>, Option<String>)> = HashMap::new();
        for (index_name, non_unique, column_name, sub_part, index_type, collation) in idx_rows {
            let entry = idx_map.entry(index_name).or_insert_with(|| (non_unique == 0, vec![], None));
            if sub_part.is_some() {
                entry.2 = Some("prefix indices aren't supported".to_string());
            }
            if collation.as_deref() == Some("D") {
                entry.2 = Some("descending index columns aren't supported".to_string());
            }
            if index_type == "FULLTEXT" || index_type == "SPATIAL" {
                entry.2 = Some(format!("{} indices aren't supported", index_type.to_lowercase()));
            }
            entry.1.push(column_name);
        }
        let mut indices: BTreeMap<String, Index> = BTreeMap::new();
        for (index_name, (unique, cols, unsupported)) in idx_map {
            if fk_map.contains_key(&index_name) {
                continue;
            }
            if let Some(unsupported) = unsupported {
                failures.push(format!("Index {:?} on table {:?}: {}", index_name, table_name, unsupported));
                continue;
            }
            indices.insert(index_name.clone(), Index {
                id: index_name,
                renamed_from: None,
                fields: cols,
                unique: unique,
            });
        }
        tables.insert(table_name.clone(), Table {
            id: table_name.clone(),
            renamed_from: None,
            fields: fields,
            indices: indices,
            constraints: constraints,
        });
    }
    let field_ids =
        tables
            .iter()
            .map(|(t_id, t)| (t_id.clone(), t.fields.keys().cloned().collect::<HashSet<_>>()))
            .collect::<HashMap<_, _>>();
    let has_field =
        |table: &String, field: &String| field_ids.get(table).map(|f| f.contains(field)).unwrap_or(false);
    for (table_id, table) in &mut tables {
        table.constraints.retain(|c_id, c| {
            let complete = match &c.type_ {
                ConstraintType::PrimaryKey(x) => x.fields.iter().all(|f| has_field(table_id, f)),
                ConstraintType::ForeignKey(x) => x
                    .fields
                    .iter()
                    .all(|(l, r)| has_field(table_id, l) && has_field(&x.remote_table, r)),
            };
            if !complete {
                failures.push(
                    format!("Constraint {:?} on table {:?} uses a column that wasn't imported", c_id, table_id),
                );
            }
            return complete;
        });
        table.indices.retain(|i_id, i| {
            let complete = i.fields.iter().all(|f| has_field(table_id, f));
            if !complete {
                failures.push(
                    format!("Index {:?} on table {:?} uses a column that wasn't imported", i_id, table_id),
                );
            }
            return complete;
        });
    }
    return Ok((Version {
        tables: tables,
        custom_types: BTreeMap::new(),
    }, failures));
}
