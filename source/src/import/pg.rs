use {
    good_ormning_core::pg::{
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
    sqlparser::{
        ast::{
            CreateIndex,
            Expr,
            Statement,
        },
        dialect::PostgreSqlDialect,
        parser::Parser,
    },
    std::collections::{
        BTreeMap,
        HashMap,
        HashSet,
    },
    tokio_postgres::Client,
};

pub async fn read_schema(client: &Client, prefer_chrono: bool) -> Result<(Version, Vec<String>), loga::Error> {
    let mut failures = vec![];
    let table_rows = client.query(
        //# genemichaels-external: sql-formatter-pg
        r#"SELECT
             c.relname
           FROM
             pg_catalog.pg_class c
             JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
           WHERE
             n.nspname = 'public'
             AND c.relkind = 'r'
             AND c.relname NOT LIKE '\_\_good\_%' ESCAPE '\'
           ORDER BY
             c.relname
           "#,
        &[],
    ).await.context("Querying table list")?;
    let table_names: Vec<String> = table_rows.iter().map(|r| r.get(0)).collect();
    let mut tables: BTreeMap<String, Table> = BTreeMap::new();
    for table_name in &table_names {
        // Columns via pg_attribute + pg_type.
        let col_rows = client.query(
            //# genemichaels-external: sql-formatter-pg
            r#"SELECT
                 a.attname,
                 t.typname,
                 NOT a.attnotnull,
                 pg_catalog.pg_get_expr (d.adbin, d.adrelid),
                 a.attidentity::text
               FROM
                 pg_catalog.pg_attribute a
                 JOIN pg_catalog.pg_class c ON c.oid = a.attrelid
                 JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
                 JOIN pg_catalog.pg_type t ON t.oid = a.atttypid
                 LEFT JOIN pg_catalog.pg_attrdef d ON d.adrelid = a.attrelid
                 AND d.adnum = a.attnum
               WHERE
                 n.nspname = 'public'
                 AND c.relname = $1
                 AND a.attnum > 0
                 AND NOT a.attisdropped
               ORDER BY
                 a.attnum
               "#,
            &[table_name],
        ).await.context("Querying columns")?;

        // In PG, field_id == sql column name (no rowid concept).
        let mut fields: BTreeMap<String, Field> = BTreeMap::new();
        for row in &col_rows {
            let col_name: String = row.get(0);
            let typname: String = row.get(1);
            let is_nullable: bool = row.get(2);
            let col_default: Option<String> = row.get(3);
            let attidentity: String = row.get(4);
            let is_serial = col_default.as_deref().map(|d| d.starts_with("nextval(")).unwrap_or(false);
            let is_identity = !attidentity.is_empty();
            let sst = if is_serial || is_identity {
                if typname != "int8" {
                    eprintln!(
                        "Warning: column {:?} of table {:?} is a {} {} column; good-ormning only supports 64-bit auto-increment columns, widening to bigserial",
                        col_name,
                        table_name,
                        typname,
                        if is_identity {
                            "identity"
                        } else {
                            "serial"
                        }
                    );
                }
                SimpleSimpleType::Auto
            } else {
                match typname.as_str() {
                    "int2" => SimpleSimpleType::I16,
                    "int4" => SimpleSimpleType::I32,
                    "int8" => SimpleSimpleType::I64,
                    "float4" => SimpleSimpleType::F32,
                    "float8" => SimpleSimpleType::F64,
                    "bool" => SimpleSimpleType::Bool,
                    "bytea" => SimpleSimpleType::Bytes,
                    "text" | "varchar" | "bpchar" | "name" | "uuid" | "json" | "jsonb" => SimpleSimpleType::String,
                    "timestamptz" => {
                        if prefer_chrono {
                            SimpleSimpleType::UtcTimeMsChrono
                        } else {
                            SimpleSimpleType::UtcTimeMsJiff
                        }
                    },
                    _ => {
                        failures.push(
                            format!(
                                "Column {:?} of table {:?} has unsupported SQL type {:?}",
                                col_name,
                                table_name,
                                typname
                            ),
                        );
                        continue;
                    },
                }
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
                        opt: is_nullable,
                        arr: false,
                    },
                    migration_default: None,
                },
            });
        }

        // Primary key constraints via pg_constraint.
        let pk_rows = client.query(
            //# genemichaels-external: sql-formatter-pg
            r#"SELECT
                 con.conname,
                 a.attname
               FROM
                 pg_catalog.pg_constraint con
                 JOIN pg_catalog.pg_class c ON c.oid = con.conrelid
                 JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
                 JOIN pg_catalog.pg_attribute a ON a.attrelid = c.oid
                 AND a.attnum = ANY (con.conkey)
               WHERE
                 n.nspname = 'public'
                 AND c.relname = $1
                 AND con.contype = 'p'
               ORDER BY
                 con.conname,
                 array_position(con.conkey, a.attnum)
               "#,
            &[table_name],
        ).await.context("Querying primary key constraints")?;
        let mut constraints: BTreeMap<String, Constraint> = BTreeMap::new();
        if !pk_rows.is_empty() {
            let pk_name: String = pk_rows[0].get(0);
            let pk_fields: Vec<String> = pk_rows.iter().map(|r| r.get(1)).collect();
            constraints.insert(pk_name.clone(), Constraint {
                id: pk_name,
                renamed_from: None,
                type_: ConstraintType::PrimaryKey(PrimaryKeyDef { fields: pk_fields }),
            });
        }

        // Foreign key constraints via pg_constraint.
        let fk_rows = client.query(
            //# genemichaels-external: sql-formatter-pg
            r#"SELECT
                 con.conname,
                 la.attname,
                 fc.relname,
                 fa.attname
               FROM
                 pg_catalog.pg_constraint con
                 JOIN pg_catalog.pg_class c ON c.oid = con.conrelid
                 JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
                 JOIN pg_catalog.pg_class fc ON fc.oid = con.confrelid
                 CROSS JOIN LATERAL unnest(con.conkey, con.confkey) AS u (local_att, foreign_att)
                 JOIN pg_catalog.pg_attribute la ON la.attrelid = c.oid
                 AND la.attnum = u.local_att
                 JOIN pg_catalog.pg_attribute fa ON fa.attrelid = fc.oid
                 AND fa.attnum = u.foreign_att
               WHERE
                 n.nspname = 'public'
                 AND c.relname = $1
                 AND con.contype = 'f'
               ORDER BY
                 con.conname,
                 u.local_att
               "#,
            &[table_name],
        ).await.context("Querying foreign key constraints")?;
        let mut fk_map: HashMap<String, (String, Vec<(String, String)>)> = HashMap::new();
        for row in &fk_rows {
            let constraint_name: String = row.get(0);
            let local_col: String = row.get(1);
            let remote_table: String = row.get(2);
            let remote_col: String = row.get(3);
            let entry = fk_map.entry(constraint_name.clone()).or_insert_with(|| (remote_table, vec![]));
            entry.1.push((local_col, remote_col));
        }
        let mut fk_names: Vec<String> = fk_map.keys().cloned().collect();
        fk_names.sort();
        for fk_name in fk_names {
            let (remote_table, col_pairs) = fk_map.remove(&fk_name).unwrap();
            constraints.insert(fk_name.clone(), Constraint {
                id: fk_name,
                renamed_from: None,
                type_: ConstraintType::ForeignKey(ForeignKeyDef {
                    remote_table: remote_table,
                    fields: col_pairs,
                }),
            });
        }

        // Indexes, excluding the primary key index (covered by the PK constraint) and
        // exclusion constraint indexes (not representable). Unique constraints are kept
        // as unique indexes.
        let idx_rows = client.query(
            //# genemichaels-external: sql-formatter-pg
            r#"SELECT
                 ic.relname,
                 pg_catalog.pg_get_indexdef (ix.indexrelid),
                 ix.indisunique,
                 ix.indpred IS NOT NULL,
                 ix.indexprs IS NOT NULL,
                 EXISTS (
                   SELECT
                     1
                   FROM
                     pg_catalog.pg_constraint con
                   WHERE
                     con.conindid = ix.indexrelid
                     AND con.contype = 'x'
                 )
               FROM
                 pg_catalog.pg_index ix
                 JOIN pg_catalog.pg_class tc ON tc.oid = ix.indrelid
                 JOIN pg_catalog.pg_namespace tn ON tn.oid = tc.relnamespace
                 JOIN pg_catalog.pg_class ic ON ic.oid = ix.indexrelid
               WHERE
                 tn.nspname = 'public'
                 AND tc.relname = $1
                 AND NOT ix.indisprimary
               ORDER BY
                 ic.relname
               "#,
            &[table_name],
        ).await.context("Querying indexes")?;
        let mut indices: BTreeMap<String, Index> = BTreeMap::new();
        for row in &idx_rows {
            let index_name: String = row.get(0);
            let indexdef: String = row.get(1);
            let is_unique: bool = row.get(2);
            let is_partial: bool = row.get(3);
            let is_expression: bool = row.get(4);
            let is_exclusion: bool = row.get(5);
            if is_partial {
                failures.push(
                    format!(
                        "Index {:?} on table {:?}: partial indexes (WHERE) are not supported",
                        index_name,
                        table_name
                    ),
                );
                continue;
            }
            if is_expression {
                failures.push(
                    format!("Index {:?} on table {:?}: expression indexes are not supported", index_name, table_name),
                );
                continue;
            }
            if is_exclusion {
                failures.push(
                    format!(
                        "Index {:?} on table {:?}: exclusion constraints are not supported",
                        index_name,
                        table_name
                    ),
                );
                continue;
            }
            let cols = (|| -> Result<Vec<String>, loga::Error> {
                let dialect = PostgreSqlDialect {};
                let mut stmts =
                    Parser::parse_sql(&dialect, &indexdef)
                        .map_err(loga::err)
                        .context("Failed to parse index DDL")?;
                let stmt = stmts.pop().ok_or_else(|| loga::err("Empty index DDL"))?;
                let columns = match stmt {
                    Statement::CreateIndex(CreateIndex { columns, .. }) => columns,
                    other => return Err(loga::err(format!("Unexpected statement in index DDL: {:?}", other))),
                };
                let mut names = vec![];
                for c in columns {
                    match c.column.expr {
                        Expr::Identifier(id) => names.push(id.value),
                        other => {
                            return Err(loga::err(format!("Unexpected index column expression: {:?}", other)));
                        },
                    }
                }
                return Ok(names);
            })().context(format!("Parsing columns for index {:?} on table {:?}", index_name, table_name))?;
            indices.insert(index_name.clone(), Index {
                id: index_name,
                renamed_from: None,
                fields: cols,
                unique: is_unique,
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
