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
    },
    tokio_postgres::Client,
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

fn map_type(typname: &str, col_default: Option<&str>) -> Result<SimpleSimpleType, loga::Error> {
    // int8 is bigint; bigserial has a nextval() default.
    if typname == "int8" {
        if col_default.map(|d| d.starts_with("nextval(")).unwrap_or(false) {
            return Ok(SimpleSimpleType::Auto);
        }
        return Ok(SimpleSimpleType::I64);
    }
    match typname {
        "int2" => return Ok(SimpleSimpleType::I16),
        "int4" => return Ok(SimpleSimpleType::I32),
        "float4" => return Ok(SimpleSimpleType::F32),
        "float8" => return Ok(SimpleSimpleType::F64),
        "bool" => return Ok(SimpleSimpleType::Bool),
        "bytea" => return Ok(SimpleSimpleType::Bytes),
        "text" | "varchar" | "bpchar" | "name" | "uuid" | "json" | "jsonb" => return Ok(SimpleSimpleType::String),
        _ => return Err(loga::err(format!("Unknown PostgreSQL type: {:?}", typname))),
    }
}

fn parse_index_columns(indexdef: &str) -> Result<Vec<String>, loga::Error> {
    let dialect = PostgreSqlDialect {};
    let mut stmts = Parser::parse_sql(&dialect, indexdef).map_err(loga::err).context("Failed to parse index DDL")?;
    let stmt = stmts.pop().ok_or_else(|| loga::err("Empty index DDL"))?;
    match stmt {
        Statement::CreateIndex(CreateIndex { columns, .. }) => {
            let mut names = vec![];
            for c in columns {
                match c.expr {
                    Expr::Identifier(id) => names.push(id.value),
                    other => {
                        return Err(loga::err(format!("Unexpected index column expression: {:?}", other)));
                    },
                }
            }
            return Ok(names);
        },
        other => return Err(loga::err(format!("Unexpected statement in index DDL: {:?}", other))),
    }
}

pub async fn read_schema(client: &Client) -> Result<Version, loga::Error> {
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
             AND c.relname NOT LIKE '__good_%'
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
                 pg_catalog.pg_get_expr (d.adbin, d.adrelid)
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
            let sst =
                map_type(
                    &typname,
                    col_default.as_deref(),
                ).context(format!("Mapping type for column {:?} of table {:?}", col_name, table_name))?;
            fields.insert(col_name.clone(), Field {
                id: col_name.clone(),
                renamed_from: None,
                type_: make_field_type(sst, is_nullable),
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

        // Non-constraint indexes: use pg_index, filtering out pk/fk/unique constraint
        // indexes.
        let idx_rows = client.query(
            //# genemichaels-external: sql-formatter-pg
            r#"SELECT
                 ic.relname,
                 pg_catalog.pg_get_indexdef (ix.indexrelid),
                 ix.indisunique
               FROM
                 pg_catalog.pg_index ix
                 JOIN pg_catalog.pg_class tc ON tc.oid = ix.indrelid
                 JOIN pg_catalog.pg_namespace tn ON tn.oid = tc.relnamespace
                 JOIN pg_catalog.pg_class ic ON ic.oid = ix.indexrelid
               WHERE
                 tn.nspname = 'public'
                 AND tc.relname = $1
                 AND NOT ix.indisprimary
                 AND NOT EXISTS (
                   SELECT
                     1
                   FROM
                     pg_catalog.pg_constraint con
                   WHERE
                     con.conindid = ix.indexrelid
                 )
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
            let cols =
                parse_index_columns(
                    &indexdef,
                ).context(format!("Parsing columns for index {:?} on table {:?}", index_name, table_name))?;
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
    return Ok(Version {
        tables: tables,
        custom_types: BTreeMap::new(),
    });
}
