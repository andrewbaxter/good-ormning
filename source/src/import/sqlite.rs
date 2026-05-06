use {
    good_ormning_core::sqlite::{
        schema::{
            constraint::{
                Constraint,
                ConstraintType,
                ForeignKeyDef,
                PrimaryKeyDef,
            },
            field::{
                Field,
                FieldType,
            },
            index::Index,
            table::Table,
        },
        types::{
            SimpleSimpleType,
            SimpleType,
            Type,
        },
        Version,
    },
    loga::ResultContext,
    rusqlite::Connection,
    std::collections::{
        BTreeMap,
        HashMap,
    },
};

fn map_type(type_str: &str) -> Result<SimpleSimpleType, loga::Error> {
    let t = type_str.to_lowercase();
    if t.contains("real") || t.contains("floa") || t.contains("doub") {
        return Ok(SimpleSimpleType::F64);
    } else if t.contains("bool") {
        return Ok(SimpleSimpleType::Bool);
    } else if t.contains("blob") || t.is_empty() {
        return Ok(SimpleSimpleType::Bytes);
    } else if t.contains("char") || t.contains("clob") || t.contains("text") {
        return Ok(SimpleSimpleType::String);
    } else if t.contains("int") {
        return Ok(SimpleSimpleType::I64);
    } else {
        return Err(loga::err(format!("Unknown SQLite type: {:?}", type_str)));
    }
}

fn make_field_type(sst: SimpleSimpleType, opt: bool) -> FieldType {
    return FieldType {
        type_: Type {
            type_: SimpleType {
                type_: sst,
                custom: None,
            },
            opt: opt,
            arr: false,
        },
        migration_default: None,
    };
}

/// Read the current schema from a SQLite connection, returning a `Version` using
/// the existing good-ormning schema model. Internal field IDs match SQL column
/// names, except for rowid-alias columns which use the field ID `"rowid"`.
pub fn read_schema(conn: &Connection) -> Result<Version, loga::Error> {
    let table_names: Vec<String> = {
        let mut stmt = conn.prepare(
            //# genemichaels-external: sql-formatter-sqlite
            r#"SELECT
                 name
               FROM
                 sqlite_master
               WHERE
                 type = 'table'
                 AND name NOT LIKE '__good_%'
                 AND name NOT LIKE 'sqlite_%'
               ORDER BY
                 name
               "#,
        ).context("Preparing table list query")?;
        let mut out = vec![];
        let rows = stmt.query_map([], |r| r.get::<_, String>(0)).context("Querying table list")?;
        for row in rows {
            out.push(row.map_err(loga::err).context("Reading row from table list")?);
        }
        out
    };

    // Pass 1: build each table's fields, indexes, and raw FK data. Collect a
    // per-table map of sql_name → field_id for use when resolving FK targets.
    struct RawFk {
        fk_id: i64,
        seq: i64,
        remote_table: String,
        local_col: String,
        remote_col: String,
    }

    struct TableRaw {
        table: Table,
        sql_to_field_id: HashMap<String, String>,
        raw_fks: Vec<RawFk>,
    }

    let mut table_raws: Vec<(String, TableRaw)> = vec![];
    for table_name in &table_names {
        struct ColInfo {
            name: String,
            type_str: String,
            notnull: bool,
            pk_pos: i64,
        }

        let cols: Vec<ColInfo> = {
            let mut stmt =
                conn.prepare(&format!("PRAGMA table_info(\"{}\")", table_name)).context("Preparing table_info")?;
            let mut out = vec![];
            let rows = stmt.query_map([], |r| {
                Ok(ColInfo {
                    name: r.get(1)?,
                    type_str: r.get::<_, String>(2).unwrap_or_default(),
                    notnull: r.get::<_, i64>(3).unwrap_or(0) != 0,
                    pk_pos: r.get::<_, i64>(5).unwrap_or(0),
                })
            }).context("Querying table_info")?;
            for row in rows {
                out.push(row.map_err(loga::err).context("Reading row from table_info")?);
            }
            out
        };

        // Detect rowid alias: exactly one column with pk_pos > 0 and type "INTEGER".
        let pk_cols: Vec<&ColInfo> = cols.iter().filter(|c| c.pk_pos > 0).collect();
        let rowid_alias: Option<&ColInfo> = if pk_cols.len() == 1 && pk_cols[0].type_str.to_lowercase() == "integer" {
            Some(pk_cols[0])
        } else {
            None
        };
        let mut fields: BTreeMap<String, Field> = BTreeMap::new();

        // sql column name → the field_id used as BTreeMap key
        let mut sql_to_field_id: HashMap<String, String> = HashMap::new();
        let mut constraints: BTreeMap<String, Constraint> = BTreeMap::new();
        for col in &cols {
            let is_rowid = rowid_alias.map(|r| r.name == col.name).unwrap_or(false);
            if is_rowid {
                // Rowid alias: field_id is always "rowid", sql name is the column name.
                fields.insert("rowid".to_string(), Field {
                    id: col.name.clone(),
                    renamed_from: None,
                    type_: make_field_type(SimpleSimpleType::Auto, false),
                });
                sql_to_field_id.insert(col.name.clone(), "rowid".to_string());

                // A PK constraint is needed so good-ormning generates INTEGER PRIMARY KEY.
                constraints.insert(format!("{}_pkey", table_name), Constraint {
                    id: format!("{}_pkey", table_name),
                    renamed_from: None,
                    type_: ConstraintType::PrimaryKey(PrimaryKeyDef { fields: vec!["rowid".to_string()] }),
                });
            } else {
                let field_id = col.name.clone();
                let sst =
                    map_type(
                        &col.type_str,
                    ).context(format!("Mapping type for column {:?} of table {:?}", col.name, table_name))?;
                fields.insert(field_id.clone(), Field {
                    id: col.name.clone(),
                    renamed_from: None,
                    type_: make_field_type(sst, !col.notnull),
                });
                sql_to_field_id.insert(col.name.clone(), field_id);
            }
        }

        // Multi-column explicit PK (non-rowid).
        if rowid_alias.is_none() && pk_cols.len() > 1 {
            let mut sorted_pk: Vec<(i64, &ColInfo)> = pk_cols.iter().map(|c| (c.pk_pos, *c)).collect();
            sorted_pk.sort_by_key(|(pos, _)| *pos);
            constraints.insert(format!("{}_pkey", table_name), Constraint {
                id: format!("{}_pkey", table_name),
                renamed_from: None,
                type_: ConstraintType::PrimaryKey(
                    PrimaryKeyDef {
                        fields: sorted_pk.into_iter().map(|(_, c)| sql_to_field_id[&c.name].clone()).collect(),
                    },
                ),
            });
        }

        // Raw FK rows (resolved in pass 2 once all tables are known).
        let raw_fks: Vec<RawFk> = {
            let mut stmt =
                conn
                    .prepare(&format!("PRAGMA foreign_key_list(\"{}\")", table_name))
                    .context("Preparing foreign_key_list")?;
            let mut out = vec![];
            let rows = stmt.query_map([], |r| {
                Ok(RawFk {
                    fk_id: r.get(0)?,
                    seq: r.get(1)?,
                    remote_table: r.get(2)?,
                    local_col: r.get(3)?,
                    remote_col: r.get(4)?,
                })
            }).context("Querying foreign_key_list")?;
            for row in rows {
                out.push(row.map_err(loga::err).context("Reading row from foreign_key_list")?);
            }
            out
        };

        // Indexes (skip pk-origin).
        let mut indices: BTreeMap<String, Index> = BTreeMap::new();
        {
            struct IdxEntry {
                name: String,
                unique: bool,
            }

            let idx_entries: Vec<IdxEntry> = {
                let mut stmt =
                    conn
                        .prepare(&format!("PRAGMA index_list(\"{}\")", table_name))
                        .context("Preparing index_list")?;
                let mut out = vec![];
                let rows = stmt.query_map([], |r| {
                    Ok((r.get::<_, String>(1)?, r.get::<_, i64>(2)? != 0, r.get::<_, String>(3)?))
                }).context("Querying index_list")?;
                for row in rows {
                    let (name, unique, origin) = row.map_err(loga::err).context("Reading row from index_list")?;
                    if origin != "pk" {
                        out.push(IdxEntry {
                            name: name,
                            unique: unique,
                        });
                    }
                }
                out
            };
            for entry in idx_entries {
                let col_names: Vec<String> = {
                    let mut stmt =
                        conn
                            .prepare(&format!("PRAGMA index_info(\"{}\")", entry.name))
                            .context("Preparing index_info")?;
                    let mut out = vec![];
                    let rows = stmt.query_map([], |r| r.get::<_, String>(2)).context("Querying index_info")?;
                    for row in rows {
                        out.push(row.map_err(loga::err).context("Reading row from index_info")?);
                    }
                    out
                };

                // Convert sql column names to field_ids.
                let field_ids = col_names.iter().map(|col| {
                    sql_to_field_id
                        .get(col)
                        .cloned()
                        .ok_or_else(
                            || loga::err(
                                format!(
                                    "Index {:?} references unknown column {:?} in table {:?}",
                                    entry.name,
                                    col,
                                    table_name
                                ),
                            ),
                        )
                }).collect::<Result<Vec<String>, _>>()?;
                indices.insert(entry.name.clone(), Index {
                    id: entry.name,
                    renamed_from: None,
                    fields: field_ids,
                    unique: entry.unique,
                });
            }
        }
        table_raws.push((table_name.clone(), TableRaw {
            table: Table {
                id: table_name.clone(),
                renamed_from: None,
                fields: fields,
                indices: indices,
                constraints: constraints,
            },
            sql_to_field_id: sql_to_field_id,
            raw_fks: raw_fks,
        }));
    }

    // Pass 2: resolve FK remote column sql names to field_ids using the complete
    // table map.
    let sql_to_field_id_by_table: HashMap<String, HashMap<String, String>> =
        table_raws.iter().map(|(name, raw)| (name.clone(), raw.sql_to_field_id.clone())).collect();
    let mut tables: BTreeMap<String, Table> = BTreeMap::new();
    for (table_name, mut raw) in table_raws {
        // Group raw FKs by fk_id.
        let mut fk_map: HashMap<i64, (String, Vec<(i64, String, String)>)> = HashMap::new();
        for fk in raw.raw_fks {
            let entry = fk_map.entry(fk.fk_id).or_insert_with(|| (fk.remote_table.clone(), vec![]));
            entry.1.push((fk.seq, fk.local_col, fk.remote_col));
        }
        let mut fk_ids: Vec<i64> = fk_map.keys().copied().collect();
        fk_ids.sort();
        for fk_id in fk_ids {
            let (remote_table, mut pairs) = fk_map.remove(&fk_id).unwrap();
            pairs.sort_by_key(|(seq, _, _)| *seq);
            let remote_map = sql_to_field_id_by_table.get(&remote_table);
            let field_pairs =
                pairs.into_iter().map(|(_, local_col, remote_col)| -> Result<(String, String), loga::Error> {
                    let local_fid =
                        raw
                            .sql_to_field_id
                            .get(&local_col)
                            .cloned()
                            .ok_or_else(
                                || loga::err(
                                    format!(
                                        "FK {:?} in table {:?} references unknown local column {:?}",
                                        fk_id,
                                        table_name,
                                        local_col
                                    ),
                                ),
                            )?;
                    let remote_fid =
                        remote_map
                            .and_then(|m| m.get(&remote_col))
                            .cloned()
                            .ok_or_else(
                                || loga::err(
                                    format!(
                                        "FK {:?} in table {:?} references unknown column {:?} in remote table {:?}",
                                        fk_id,
                                        table_name,
                                        remote_col,
                                        remote_table
                                    ),
                                ),
                            )?;
                    return Ok((local_fid, remote_fid));
                }).collect::<Result<Vec<(String, String)>, _>>()?;
            let constraint_name = format!("{}_{}_fkey", table_name, fk_id);
            raw.table.constraints.insert(constraint_name.clone(), Constraint {
                id: constraint_name,
                renamed_from: None,
                type_: ConstraintType::ForeignKey(ForeignKeyDef {
                    remote_table: remote_table,
                    fields: field_pairs,
                }),
            });
        }
        tables.insert(table_name, raw.table);
    }
    return Ok(Version {
        tables: tables,
        custom_types: BTreeMap::new(),
    });
}
