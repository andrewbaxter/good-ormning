use {
    good_ormning_core::sqlite::{
        Version,
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
    },
    loga::ResultContext,
    rusqlite::Connection,
    std::collections::{
        BTreeMap,
        HashMap,
    },
};

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

fn map_type(type_str: &str) -> Option<SimpleSimpleType> {
    let t = type_str.to_lowercase();
    if t.contains("real") || t.contains("floa") || t.contains("doub") {
        return Some(SimpleSimpleType::F64);
    } else if t.contains("bool") {
        return Some(SimpleSimpleType::Bool);
    } else if t.contains("blob") || t.is_empty() {
        return Some(SimpleSimpleType::Bytes);
    } else if t.contains("char") || t.contains("clob") || t.contains("text") {
        return Some(SimpleSimpleType::String);
    } else if t.contains("int") {
        return Some(SimpleSimpleType::I64);
    } else {
        return None;
    }
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
                 AND name NOT LIKE '\_\_good\_%' ESCAPE '\'
                 AND name NOT LIKE 'sqlite\_%' ESCAPE '\'
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
    // per-table map of lowercased sql_name → field_id for use when resolving FK
    // targets (SQLite identifiers are case-insensitive, and `foreign_key_list`
    // reports names as typed in the DDL).
    struct RawFk {
        fk_id: i64,
        local_col: String,
        remote_col: Option<String>,
        remote_table: String,
        seq: i64,
    }

    struct TableRaw {
        pk_field_ids: Vec<String>,
        raw_fks: Vec<RawFk>,
        sql_to_field_id: HashMap<String, String>,
        table: Table,
    }

    let mut table_raws: Vec<(String, TableRaw)> = vec![];
    for table_name in &table_names {
        struct ColInfo {
            name: String,
            notnull: bool,
            pk_pos: i64,
            type_str: String,
        }

        let cols: Vec<ColInfo> = {
            let mut stmt = conn.prepare(
                //# genemichaels-external: sql-formatter-sqlite
                r#"SELECT
                     name,
                     type,
                     "notnull",
                     pk
                   FROM
                     pragma_table_info (?)
                   "#,
            ).context("Preparing table_info")?;
            let mut out = vec![];
            let rows = stmt.query_map([table_name], |r| {
                Ok(ColInfo {
                    name: r.get(0)?,
                    type_str: r.get::<_, Option<String>>(1)?.unwrap_or_default(),
                    notnull: r.get::<_, i64>(2)? != 0,
                    pk_pos: r.get::<_, i64>(3)?,
                })
            }).context("Querying table_info")?;
            for row in rows {
                out.push(row.map_err(loga::err).context("Reading row from table_info")?);
            }
            out
        };

        // Detect rowid alias: exactly one column with pk_pos > 0 and type "INTEGER".
        let mut pk_cols: Vec<&ColInfo> = cols.iter().filter(|c| c.pk_pos > 0).collect();
        pk_cols.sort_by_key(|c| c.pk_pos);
        let rowid_alias: Option<&ColInfo> = if pk_cols.len() == 1 && pk_cols[0].type_str.to_lowercase() == "integer" {
            Some(pk_cols[0])
        } else {
            None
        };
        let mut fields: BTreeMap<String, Field> = BTreeMap::new();

        // lowercased sql column name → the field_id used as BTreeMap key
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
                sql_to_field_id.insert(col.name.to_lowercase(), "rowid".to_string());
            } else {
                if col.name.eq_ignore_ascii_case("rowid") {
                    return Err(
                        loga::err(
                            format!(
                                "Table {:?} has a regular column named {:?}; good-ormning reserves the field id \"rowid\" for the implicit rowid or an INTEGER PRIMARY KEY alias, so this table can't be imported",
                                table_name,
                                col.name
                            ),
                        ),
                    );
                }
                let field_id = col.name.clone();
                let sst = match map_type(&col.type_str) {
                    Some(sst) => sst,
                    None => {
                        return Err(
                            loga::err(
                                format!(
                                    "Column {:?} of table {:?} has unsupported SQL type {:?}",
                                    col.name,
                                    table_name,
                                    col.type_str
                                ),
                            ),
                        );
                    },
                };
                fields.insert(field_id.clone(), Field {
                    id: col.name.clone(),
                    renamed_from: None,
                    type_: make_field_type(sst, !col.notnull),
                });
                sql_to_field_id.insert(col.name.to_lowercase(), field_id);
            }
        }

        // Primary key constraint. For a rowid alias this is needed so good-ormning
        // generates INTEGER PRIMARY KEY; otherwise it covers single- and multi-column
        // explicit PKs.
        let pk_field_ids: Vec<String> = pk_cols.iter().map(|c| sql_to_field_id[&c.name.to_lowercase()].clone()).collect();
        if !pk_field_ids.is_empty() {
            constraints.insert(format!("{}_pkey", table_name), Constraint {
                id: format!("{}_pkey", table_name),
                renamed_from: None,
                type_: ConstraintType::PrimaryKey(PrimaryKeyDef { fields: pk_field_ids.clone() }),
            });
        }

        // Raw FK rows (resolved in pass 2 once all tables are known).
        let raw_fks: Vec<RawFk> = {
            let mut stmt = conn.prepare(
                //# genemichaels-external: sql-formatter-sqlite
                r#"SELECT
                     id,
                     seq,
                     "table",
                     "from",
                     "to"
                   FROM
                     pragma_foreign_key_list (?)
                   "#,
            ).context("Preparing foreign_key_list")?;
            let mut out = vec![];
            let rows = stmt.query_map([table_name], |r| {
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
                origin: String,
                partial: bool,
                unique: bool,
            }

            let idx_entries: Vec<IdxEntry> = {
                let mut stmt = conn.prepare(
                    //# genemichaels-external: sql-formatter-sqlite
                    r#"SELECT
                         name,
                         "unique",
                         origin,
                         partial
                       FROM
                         pragma_index_list (?)
                       ORDER BY
                         name
                       "#,
                ).context("Preparing index_list")?;
                let mut out = vec![];
                let rows = stmt.query_map([table_name], |r| {
                    Ok(IdxEntry {
                        name: r.get(0)?,
                        unique: r.get::<_, i64>(1)? != 0,
                        origin: r.get(2)?,
                        partial: r.get::<_, i64>(3)? != 0,
                    })
                }).context("Querying index_list")?;
                for row in rows {
                    out.push(row.map_err(loga::err).context("Reading row from index_list")?);
                }
                out
            };
            for entry in idx_entries {
                if entry.origin == "pk" {
                    continue;
                }
                if entry.partial {
                    eprintln!(
                        "Warning: skipping index {:?} on table {:?}: partial indexes (WHERE) are not supported",
                        entry.name,
                        table_name
                    );
                    continue;
                }

                // Key columns only (`key = 1`); a NULL name on a key column means an expression.
                let col_names: Vec<Option<String>> = {
                    let mut stmt = conn.prepare(
                        //# genemichaels-external: sql-formatter-sqlite
                        r#"SELECT
                             name
                           FROM
                             pragma_index_xinfo (?)
                           WHERE
                             key = 1
                           ORDER BY
                             seqno
                           "#,
                    ).context("Preparing index_xinfo")?;
                    let mut out = vec![];
                    let rows =
                        stmt
                            .query_map([&entry.name], |r| r.get::<_, Option<String>>(0))
                            .context("Querying index_xinfo")?;
                    for row in rows {
                        out.push(row.map_err(loga::err).context("Reading row from index_xinfo")?);
                    }
                    out
                };
                if col_names.iter().any(|c| c.is_none()) {
                    eprintln!(
                        "Warning: skipping index {:?} on table {:?}: expression indexes are not supported",
                        entry.name,
                        table_name
                    );
                    continue;
                }
                let col_names: Vec<String> = col_names.into_iter().map(|c| c.unwrap()).collect();

                // Convert sql column names to field_ids.
                let field_ids = col_names.iter().map(|col| {
                    sql_to_field_id
                        .get(&col.to_lowercase())
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

                // UNIQUE constraints get automatic `sqlite_autoindex_*` names which can't be used
                // in CREATE INDEX, so give them a synthetic name.
                let index_name = if entry.origin == "u" {
                    format!("{}_{}_unique", table_name, col_names.join("_"))
                } else {
                    entry.name
                };
                indices.insert(index_name.clone(), Index {
                    id: index_name,
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
            pk_field_ids: pk_field_ids,
            sql_to_field_id: sql_to_field_id,
            raw_fks: raw_fks,
        }));
    }

    // Pass 2: resolve FK remote column sql names to field_ids using the complete
    // table map. Keyed by lowercased table name since FK DDL may differ in case.
    struct RemoteInfo {
        name: String,
        pk_field_ids: Vec<String>,
        sql_to_field_id: HashMap<String, String>,
    }

    let remote_by_table: HashMap<String, RemoteInfo> =
        table_raws.iter().map(|(name, raw)| (name.to_lowercase(), RemoteInfo {
            name: name.clone(),
            pk_field_ids: raw.pk_field_ids.clone(),
            sql_to_field_id: raw.sql_to_field_id.clone(),
        })).collect();
    let mut tables: BTreeMap<String, Table> = BTreeMap::new();
    for (table_name, mut raw) in table_raws {
        // Group raw FKs by fk_id.
        let mut fk_map: HashMap<i64, (String, Vec<(i64, String, Option<String>)>)> = HashMap::new();
        for fk in raw.raw_fks {
            let entry = fk_map.entry(fk.fk_id).or_insert_with(|| (fk.remote_table.clone(), vec![]));
            entry.1.push((fk.seq, fk.local_col, fk.remote_col));
        }
        let mut fk_ids: Vec<i64> = fk_map.keys().copied().collect();
        fk_ids.sort();
        for fk_id in fk_ids {
            let (remote_table, mut pairs) = fk_map.remove(&fk_id).unwrap();
            pairs.sort_by_key(|(seq, _, _)| *seq);
            let remote =
                remote_by_table
                    .get(&remote_table.to_lowercase())
                    .ok_or_else(
                        || loga::err(
                            format!(
                                "FK {:?} in table {:?} references unknown remote table {:?}",
                                fk_id,
                                table_name,
                                remote_table
                            ),
                        ),
                    )?;
            let field_pairs =
                pairs.into_iter().map(|(seq, local_col, remote_col)| -> Result<(String, String), loga::Error> {
                    let local_fid =
                        raw
                            .sql_to_field_id
                            .get(&local_col.to_lowercase())
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
                    let remote_fid = match remote_col {
                        Some(remote_col) => remote
                            .sql_to_field_id
                            .get(&remote_col.to_lowercase())
                            .cloned()
                            .ok_or_else(
                                || loga::err(
                                    format!(
                                        "FK {:?} in table {:?} references unknown column {:?} in remote table {:?}",
                                        fk_id,
                                        table_name,
                                        remote_col,
                                        remote.name
                                    ),
                                ),
                            )?,
                        None => {
                            // `REFERENCES parent` with no column list means the parent's primary key,
                            // positionally.
                            remote
                                .pk_field_ids
                                .get(seq as usize)
                                .cloned()
                                .ok_or_else(
                                    || loga::err(
                                        format!(
                                            "FK {:?} in table {:?} references the primary key of remote table {:?} (no column list), but that table has no primary key column at position {}",
                                            fk_id,
                                            table_name,
                                            remote.name,
                                            seq
                                        ),
                                    ),
                                )?
                        },
                    };
                    return Ok((local_fid, remote_fid));
                }).collect::<Result<Vec<(String, String)>, _>>()?;
            let constraint_name = format!("{}_{}_fkey", table_name, fk_id);
            raw.table.constraints.insert(constraint_name.clone(), Constraint {
                id: constraint_name,
                renamed_from: None,
                type_: ConstraintType::ForeignKey(ForeignKeyDef {
                    remote_table: remote.name.clone(),
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
