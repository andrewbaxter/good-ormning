use {
    good_ormning_core::{
        pg::{
            graph::utils::PgMigrateCtx,
            query::utils::{
                PgFieldInfo,
                PgTableInfo,
            },
            schema::{
                field::FieldRef,
                table::TableRef,
            },
        },
        utils::Errs,
    },
    proc_macro2::TokenStream,
    convert_case::{
        Casing,
        Case,
    },
    quote::{
        format_ident,
        quote,
    },
    std::{
        collections::HashMap,
        env,
        fs,
        path::Path,
    },
};
pub use {
    good_ormning_core::pg::*,
    good_ormning_macros::{
        good_query_many_pg as good_query_many,
        good_query_one_pg as good_query_one,
        good_query_opt_pg as good_query_opt,
        good_query_pg as good_query,
    },
};

/// Generate Rust code for migrations and queries.
///
/// # Arguments
///
/// * `output` - the path to a single rust source file where the output will be written
///
/// * `versions` - a list of database version ids and schema versions. The ids must be
///   consecutive but can start from any number. Once a version has been applied to a
///   production database it shouldn't be modified again (modifications should be done
///   in a new version).
///
///   These will be turned into migrations as part of the `migrate` function.
///
/// * `queries` - a list of queries against the schema in the latest version. These
///   will be turned into functions.
///
/// # Returns
///
/// * Error - a list of validation or generation errors that occurred
pub fn generate(db_name: Option<&str>, versions: Vec<(usize, Version)>) -> Result<(), Vec<String>> {
    let db_name = db_name.unwrap_or(good_ormning_core::utils::DEFAULT_DB_NAME);
    let out_dir = env::var("OUT_DIR").map_err(|e| vec![format!("OUT_DIR not set: {:?}", e)])?;
    let out_dir = Path::new(&out_dir);
    let output = out_dir.join(good_ormning_core::utils::rs_file_name(db_name));
    let json_dir = out_dir.join("good_ormning");
    if let Err(e) = fs::create_dir_all(&json_dir) {
        return Err(vec![format!("Error creating directory {:?}: {:?}", json_dir, e)]);
    }
    let json_path = json_dir.join(good_ormning_core::utils::json_file_name(db_name));

    // Serialize versions for proc macro
    {
        eprintln!("DEBUG: Writing versions to {:?}", json_path);
        let mut versions_map: HashMap<usize, Version> = if json_path.exists() {
            serde_json::from_str(&fs::read_to_string(&json_path).unwrap()).unwrap_or_default()
        } else {
            HashMap::new()
        };
        for (version_i, version) in versions.iter() {
            let entry = versions_map.entry(*version_i).or_insert_with(|| Version::default());
            for (k, v) in &version.tables {
                entry.tables.insert(k.clone(), v.clone());
            }
            for (k, v) in &version.custom_types {
                entry.custom_types.insert(k.clone(), v.clone());
            }
        }
        let _ = fs::write(json_path, serde_json::to_string(&versions_map).unwrap());
    }
    let mut errs = Errs::new();
    let mut migrations = vec![];
    let mut prev_version: Option<Version> = None;
    let mut prev_version_i: Option<i64> = None;
    let mut field_lookup: HashMap<TableRef, PgTableInfo> = HashMap::new();
    for (version_i, version) in &versions {
        let path = rpds::vector![format!("Migration to {}", version_i)];
        let mut migration = vec![];

        // Prep for current version
        field_lookup.clear();
        for (table_id, table) in &version.tables {
            let mut fields: HashMap<FieldRef, PgFieldInfo> = HashMap::new();
            for (field_id, field) in &table.fields {
                fields.insert(FieldRef {
                    table_id: table_id.clone(),
                    field_id: field_id.clone(),
                }, PgFieldInfo {
                    sql_name: field.id.clone(),
                    type_: field.type_.type_.clone(),
                });
            }
            field_lookup.insert(TableRef(table_id.clone()), PgTableInfo {
                sql_name: table.id.clone(),
                fields: fields,
            });
        }
        let version_i = *version_i as i64;
        if let Some(i) = prev_version_i {
            if version_i != i as i64 + 1 {
                errs.err(
                    &path,
                    format!(
                        "Version numbers are not consecutive ({} to {}) - was an intermediate version deleted?",
                        i,
                        version_i
                    ),
                );
            }
        }

        // Main migrations
        {
            let mut table_sql_names = HashMap::new();
            for (table_id, table) in &version.tables {
                table_sql_names.insert(table_id.clone(), table.id.clone());
            }
            let mut state = PgMigrateCtx::new(errs.clone(), table_sql_names, version.clone());
            let current_nodes = version.to_migrate_nodes();
            let prev_nodes = prev_version.take().map(|s| s.to_migrate_nodes());
            good_ormning_core::graphmigrate::migrate(&mut state, prev_nodes, &current_nodes);
            for statement in &state.statements {
                migration.push(quote!{
                    {
                        let query = #statement;
                        txn.execute(query, &[]).await.to_good_error_query(query)?;
                    };
                });
            }
            errs = state.errs.clone();
        }

        // Build migration
        let pascal_db_name: String = db_name.to_case(Case::Pascal);
        let enum_name = format_ident!("Db{}Versions", pascal_db_name);
        let newtype_name = format_ident!("Db{}{}", pascal_db_name, version_i as usize);
        let enum_variant = format_ident!("V{}", version_i as usize);
        migrations.push(quote!{
            if version < #version_i {
                #(#migration) * {
                    let query = "update __good_version set version = $1";
                    good_ormning:: runtime:: pg:: PgConnection:: execute(
                        &mut txn,
                        query,
                        &[& #version_i]
                    ).await.to_good_error_query(query) ?;
                }
                if let Some(callback) = & callback {
                    callback(#enum_name::#enum_variant(#newtype_name(&mut txn))).await ?;
                }
            }
        });

        // Next iter prep
        prev_version = Some(version.clone());
        prev_version_i = Some(version_i);
    }

    // Compile, output
    let last_version_i = prev_version_i.unwrap() as i64;
    let pascal_db_name: String = db_name.to_case(Case::Pascal);
    let enum_name = format_ident!("Db{}Versions", pascal_db_name);
    let mut enum_variants = vec![];
    let mut db_types = vec![];
    for (version_i, _) in &versions {
        let newtype_name = format_ident!("Db{}{}", pascal_db_name, version_i);
        let enum_variant = format_ident!("V{}", version_i);
        enum_variants.push(quote!(#enum_variant(#newtype_name <'a >)));
        db_types.push(quote!{
            pub struct #newtype_name <'a >(pub &'a mut dyn good_ormning:: runtime:: pg:: PgConnection);
        });
    }
    let latest_newtype_name = format_ident!("Db{}{}", pascal_db_name, last_version_i as usize);
    let db_alias_name = format_ident!("Db{}", pascal_db_name);
    let db_others: Vec<TokenStream> = vec![];
    let tokens = quote!{
        #![allow(unused)] use good_ormning:: runtime:: GoodError;
        use good_ormning::runtime::ToGoodError;
        #(#db_types) * pub enum #enum_name <'a > {
            #(#enum_variants,) *
        }
        pub use #latest_newtype_name as #db_alias_name;
        async fn init_db(db: & mut impl good_ormning:: runtime:: pg:: PgConnection) -> Result <(),
        GoodError > {
            {
                let query =
                    "create table if not exists __good_version (rid int primary key, version bigint not null, lock int not null);";
                good_ormning::runtime::pg::PgConnection::execute(db, query, &[]).await.to_good_error_query(query)?;
            }
            {
                let query =
                    "insert into __good_version (rid, version, lock) values (0, -1, 0) on conflict do nothing;";
                good_ormning::runtime::pg::PgConnection::execute(db, query, &[]).await.to_good_error_query(query)?;
            }
            Ok(())
        }
        pub async fn migrate(
            db: & mut tokio_postgres:: Client,
            callback: Option <&(
                dyn for <'b > Fn(
                    #enum_name <'b >
                ) -> std:: pin:: Pin < Box < dyn std:: future:: Future < Output = Result <(),
                GoodError >> + Send + 'b >> + Send + Sync
            ) >
        ) -> Result <(),
        GoodError > {
            init_db(db).await?;
            loop {
                let mut txn = db.transaction().await.to_good_error(|| "Failed to start transaction".to_string())?;
                let migrated = {
                    let query = "update __good_version set lock = 1 where rid = 0 and lock = 0 returning version";
                    let res =
                        good_ormning::runtime::pg::PgConnection::query(&mut txn, query, &[])
                            .await
                            .to_good_error_query(query)?;
                    let version = match res.first() {
                        Some(r) => {
                            let ver: i64 = r.get(0usize);
                            Some(ver)
                        },
                        None => {
                            None
                        },
                    };
                    if let Some(version) = version {
                        if version > #last_version_i {
                            return Err(
                                GoodError(
                                    format!(
                                        "The latest known version is {}, but the schema is at unknown version {}",
                                        #last_version_i,
                                        version
                                    ),
                                ),
                            );
                        }
                        #(#migrations) * {
                            let query = "update __good_version set lock = 0";
                            good_ormning::runtime::pg::PgConnection::execute(&mut txn, query, &[])
                                .await
                                .to_good_error_query(query)?;
                        }
                        true
                    }
                    else {
                        false
                    }
                };
                if migrated {
                    txn.commit().await.to_good_error(|| "Failed to commit transaction".to_string())?;
                    return Ok(());
                }
                else {
                    txn.rollback().await.to_good_error(|| "Failed to rollback transaction".to_string())?;
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            }
        }
        pub async fn get_schema_version(
            db: & mut impl good_ormning:: runtime:: pg:: PgConnection
        ) -> Result < Option < i64 >,
        GoodError > {
            init_db(db).await?;
            let query = "select version from __good_version where rid = 0";
            let res = db.query(query, &[]).await.to_good_error_query(query)?;
            if let Some(r) = res.first() {
                let x: i64 = r.get(0usize);
                if x == -1 {
                    return Ok(None);
                } else {
                    return Ok(Some(x));
                }
            }
            Ok(None)
        }
        #(#db_others) *
    };
    match genemichaels_lib::format_str(&tokens.to_string(), &genemichaels_lib::FormatConfig::default()) {
        Ok(src) => {
            match fs::write(&output, src.rendered.as_bytes()) {
                Ok(_) => { },
                Err(e) => errs.err(
                    &rpds::vector![],
                    format!("Failed to write generated code to {:?}: {:?}", output, e),
                ),
            };
        },
        Err(e) => {
            errs.err(&rpds::vector![], format!("Error formatting generated code: {:?}\n{}", e, tokens));
        },
    };
    errs.raise()?;
    Ok(())
}

#[cfg(test)]
mod test {
    use {
        super::{
            generate,
            query::expr::SerialExpr,
            schema::field::{
                field_auto,
                field_i32,
                field_str,
            },
            Version,
        },
    };

    #[test]
    fn test_add_field_serial_bad() {
        assert!(generate(None, vec![
            // Versions (previous)
            (0usize, {
                let v = Version::new();
                v.table("zMOY9YMCK").field("z437INV6D", field_str().build());
                v.build()
            }),
            (1usize, {
                let v = Version::new();
                let bananna = v.table("zMOY9YMCK");
                bananna.field("z437INV6D", field_str().build());
                bananna.field("zPREUVAOD", field_auto().migrate_fill(SerialExpr::LitAuto(0)).build());
                v.build()
            })
        ]).is_err());
    }

    #[test]
    #[should_panic]
    fn test_add_field_dup_bad() {
        generate(None, vec![
            // Versions (previous)
            (0usize, {
                let v = Version::new();
                v.table("zPAO2PJU4").field("z437INV6D", field_str().build());
                v.build()
            }),
            (1usize, {
                let v = Version::new();
                let bananna = v.table("zQZQ8E2WD");
                bananna.field("z437INV6D", field_str().build());
                bananna.field("z437INV6D", field_i32().build());
                v.build()
            })
        ]).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_add_table_dup_bad() {
        generate(None, vec![
            // Versions (previous)
            (0usize, {
                let v = Version::new();
                v.table("zSNS34DYI").field("z437INV6D", field_str().build());
                v.build()
            }),
            (1usize, {
                let v = Version::new();
                v.table("zSNS34DYI").field("z437INV6D", field_str().build());
                v.table("zSNS34DYI").field("z437INV6D", field_str().build());
                v.build()
            })
        ]).unwrap();
    }

    #[test]
    fn test_res_count_none_bad() {
        let v = Version::new();
        let bananna = v.table("z5S18LWQE");
        bananna.field("z437INV6D", field_str().build());
        assert!(generate(None, vec![(0usize, v.build())],).is_err());
    }

    #[test]
    fn test_select_nothing_bad() {
        let v = Version::new();
        v.table("zOOR88EQ9").field("z437INV6D", field_str().build());
        assert!(generate(None, vec![(0usize, v.build())],).is_err());
    }

    #[test]
    fn test_returning_none_bad() {
        let v = Version::new();
        let bananna = v.table("zZPD1I2EF");
        bananna.field("z437INV6D", field_str().build());
        assert!(generate(None, vec![(0usize, v.build())],).is_err());
    }
}
