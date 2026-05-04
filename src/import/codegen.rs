use {
    genemichaels_lib::FormatConfig,
    loga::ResultContext,
    proc_macro2::{
        Ident,
        TokenStream,
    },
    quote::{
        format_ident,
        quote,
    },
    std::collections::HashMap,
};

fn sanitize_ident(s: &str) -> String {
    let out: String = s.chars().map(|c| if c.is_alphanumeric() || c == '_' {
        c
    } else {
        '_'
    }).collect();
    if out.starts_with(|c: char| c.is_ascii_digit()) {
        return format!("_{}", out);
    }
    return out;
}

fn pg_field_fn(sst: &good_ormning_core::pg::types::SimpleSimpleType) -> &'static str {
    use good_ormning_core::pg::types::SimpleSimpleType as S;

    match sst {
        S::Auto => "field_auto",
        S::I16 => "field_i16",
        S::I32 => "field_i32",
        S::I64 => "field_i64",
        S::U32 => "field_u32",
        S::F32 => "field_f32",
        S::F64 => "field_f64",
        S::Bool => "field_bool",
        S::String => "field_str",
        S::Bytes => "field_bytes",
        #[cfg(feature = "chrono")]
        S::UtcTimeSChrono => "field_utctime_s_chrono",
        #[cfg(feature = "chrono")]
        S::UtcTimeMsChrono => "field_utctime_ms_chrono",
        #[cfg(feature = "chrono")]
        S::FixedOffsetTimeChrono => "field_utctime_s_chrono",
        #[cfg(feature = "jiff")]
        S::UtcTimeSJiff => "field_utctime_s_jiff",
        #[cfg(feature = "jiff")]
        S::UtcTimeMsJiff => "field_utctime_ms_jiff",
    }
}

fn sqlite_field_fn(sst: &good_ormning_core::sqlite::types::SimpleSimpleType) -> &'static str {
    use good_ormning_core::sqlite::types::SimpleSimpleType as S;

    match sst {
        S::Auto => "field_auto",
        S::I16 => "field_i16",
        S::I32 => "field_i32",
        S::I64 => "field_i64",
        S::U32 => "field_u32",
        S::F32 => "field_f32",
        S::F64 => "field_f64",
        S::Bool => "field_bool",
        S::String => "field_str",
        S::Bytes => "field_bytes",
        #[cfg(feature = "chrono")]
        S::UtcTimeSChrono => "field_utctime_s_chrono",
        #[cfg(feature = "chrono")]
        S::UtcTimeMsChrono => "field_utctime_ms_chrono",
        #[cfg(feature = "chrono")]
        S::FixedOffsetTimeChrono => "field_utctime_s_chrono",
        #[cfg(feature = "jiff")]
        S::UtcTimeSJiff => "field_utctime_s_jiff",
        #[cfg(feature = "jiff")]
        S::UtcTimeMsJiff => "field_utctime_ms_jiff",
    }
}

fn lookup_field(
    field_var_map: &HashMap<(String, String), String>,
    table_key: &str,
    field_id: &str,
) -> Result<Ident, loga::Error> {
    let name =
        field_var_map
            .get(&(table_key.to_string(), field_id.to_string()))
            .cloned()
            .ok_or_else(|| loga::err(format!("Field {:?} not found in table {:?}", field_id, table_key)))?;
    return Ok(format_ident!("{}", name));
}

fn format_tokens(tokens: TokenStream) -> String {
    match genemichaels_lib::format_str(&tokens.to_string(), &FormatConfig::default()) {
        Ok(res) => return res.rendered,
        Err(_) => return tokens.to_string(),
    }
}

/// Generate a `build.rs` `main()` body for the given PostgreSQL `Version`.
pub fn generate_pg(version: &good_ormning_core::pg::Version, db_name: &str) -> Result<String, loga::Error> {
    use good_ormning_core::pg::schema::constraint::ConstraintType;

    let mut stmts: Vec<TokenStream> = vec![];
    stmts.push(quote!{
        println!("cargo:rerun-if-changed=build.rs");
    });
    stmts.push(quote!{
        let v = good_ormning::pg::Version::new();
    });
    let mut field_var_map: HashMap<(String, String), String> = HashMap::new();
    for (table_key, table) in &version.tables {
        let tvar = format_ident!("t_{}", sanitize_ident(table_key));
        let table_id = &table.id;
        stmts.push(quote!{
            let #tvar = v.table(#table_id);
        });
        for (field_id, field) in &table.fields {
            let fvar_name = format!("t_{}_{}", sanitize_ident(table_key), sanitize_ident(field_id));
            let fvar = format_ident!("{}", &fvar_name);
            let fn_ident = format_ident!("{}", pg_field_fn(&field.type_.type_.type_.type_));
            let field_id_str = &field.id;
            if field.type_.type_.opt {
                stmts.push(quote!{
                    let #fvar = #tvar.field(
                        #field_id_str,
                        good_ormning:: pg:: schema:: field::#fn_ident().opt().build()
                    );
                });
            } else {
                stmts.push(quote!{
                    let #fvar = #tvar.field(#field_id_str, good_ormning:: pg:: schema:: field::#fn_ident().build());
                });
            }
            field_var_map.insert((table_key.clone(), field_id.clone()), fvar_name);
        }
        for constraint in table.constraints.values() {
            let constraint_id = &constraint.id;
            match &constraint.type_ {
                ConstraintType::PrimaryKey(pk) => {
                    let refs =
                        pk
                            .fields
                            .iter()
                            .map(|fid| -> Result<TokenStream, loga::Error> {
                                let fvar = lookup_field(&field_var_map, table_key, fid)?;
                                return Ok(quote!{
                                    &#fvar
                                });
                            })
                            .collect::<Result<Vec<TokenStream>, _>>()
                            .context(format!("Generating primary key constraint {:?}", constraint_id))?;
                    stmts.push(quote!{
                        #tvar.primary_key(#constraint_id, &[#(#refs), *]);
                    });
                },
                ConstraintType::ForeignKey(fk) => {
                    let pairs =
                        fk
                            .fields
                            .iter()
                            .map(|(lf, rf)| -> Result<TokenStream, loga::Error> {
                                let lvar = lookup_field(&field_var_map, table_key, lf)?;
                                let rvar = lookup_field(&field_var_map, &fk.remote_table, rf)?;
                                return Ok(quote!{
                                    (&#lvar, &#rvar)
                                });
                            })
                            .collect::<Result<Vec<TokenStream>, _>>()
                            .context(format!("Generating foreign key constraint {:?}", constraint_id))?;
                    stmts.push(quote!{
                        #tvar.foreign_key(#constraint_id, &[#(#pairs), *]);
                    });
                },
            }
        }
        for index in table.indices.values() {
            let index_id = &index.id;
            let refs = index.fields.iter().map(|fid| -> Result<TokenStream, loga::Error> {
                let fvar = lookup_field(&field_var_map, table_key, fid)?;
                return Ok(quote!{
                    &#fvar
                });
            }).collect::<Result<Vec<TokenStream>, _>>().context(format!("Generating index {:?}", index_id))?;
            if index.unique {
                stmts.push(quote!{
                    #tvar.unique_index(#index_id, &[#(#refs), *]);
                });
            } else {
                stmts.push(quote!{
                    #tvar.index(#index_id, &[#(#refs), *]);
                });
            }
        }
    }
    stmts.push(quote!{
        good_ormning:: pg:: generate(good_ormning:: pg:: GenerateArgs {
            db_name: Some(#db_name.to_string()),
            versions: vec ![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    });
    return Ok(format_tokens(quote!{
        fn main() {
            #(#stmts) *
        }
    }));
}

/// Generate a `build.rs` `main()` body for the given SQLite `Version`.
pub fn generate_sqlite(version: &good_ormning_core::sqlite::Version, db_name: &str) -> Result<String, loga::Error> {
    use good_ormning_core::sqlite::schema::constraint::ConstraintType;

    let mut stmts: Vec<TokenStream> = vec![];
    stmts.push(quote!{
        println!("cargo:rerun-if-changed=build.rs");
    });
    stmts.push(quote!{
        let v = good_ormning::sqlite::Version::new();
    });
    let mut field_var_map: HashMap<(String, String), String> = HashMap::new();
    for (table_key, table) in &version.tables {
        let tvar = format_ident!("t_{}", sanitize_ident(table_key));
        let table_id = &table.id;
        stmts.push(quote!{
            let #tvar = v.table(#table_id);
        });
        for (field_id, field) in &table.fields {
            let fvar_name = format!("t_{}_{}", sanitize_ident(table_key), sanitize_ident(field_id));
            let fvar = format_ident!("{}", &fvar_name);
            if field_id == "rowid" {
                if field.id == "rowid" {
                    stmts.push(quote!{
                        let #fvar = #tvar.rowid_field(None);
                    });
                } else {
                    let sql_name = &field.id;
                    stmts.push(quote!{
                        let #fvar = #tvar.rowid_field(Some(#sql_name));
                    });
                }
            } else {
                let fn_ident = format_ident!("{}", sqlite_field_fn(&field.type_.type_.type_.type_));
                let field_id_str = &field.id;
                if field.type_.type_.opt {
                    stmts.push(quote!{
                        let #fvar = #tvar.field(
                            #field_id_str,
                            good_ormning:: sqlite:: schema:: field::#fn_ident().opt().build()
                        );
                    });
                } else {
                    stmts.push(quote!{
                        let #fvar = #tvar.field(
                            #field_id_str,
                            good_ormning:: sqlite:: schema:: field::#fn_ident().build()
                        );
                    });
                }
            }
            field_var_map.insert((table_key.clone(), field_id.clone()), fvar_name);
        }
        for constraint in table.constraints.values() {
            let constraint_id = &constraint.id;
            match &constraint.type_ {
                ConstraintType::PrimaryKey(pk) => {
                    let refs =
                        pk
                            .fields
                            .iter()
                            .map(|fid| -> Result<TokenStream, loga::Error> {
                                let fvar = lookup_field(&field_var_map, table_key, fid)?;
                                return Ok(quote!{
                                    &#fvar
                                });
                            })
                            .collect::<Result<Vec<TokenStream>, _>>()
                            .context(format!("Generating primary key constraint {:?}", constraint_id))?;
                    stmts.push(quote!{
                        #tvar.primary_key(#constraint_id, &[#(#refs), *]);
                    });
                },
                ConstraintType::ForeignKey(fk) => {
                    let pairs =
                        fk
                            .fields
                            .iter()
                            .map(|(lf, rf)| -> Result<TokenStream, loga::Error> {
                                let lvar = lookup_field(&field_var_map, table_key, lf)?;
                                let rvar = lookup_field(&field_var_map, &fk.remote_table, rf)?;
                                return Ok(quote!{
                                    (&#lvar, &#rvar)
                                });
                            })
                            .collect::<Result<Vec<TokenStream>, _>>()
                            .context(format!("Generating foreign key constraint {:?}", constraint_id))?;
                    stmts.push(quote!{
                        #tvar.foreign_key(#constraint_id, &[#(#pairs), *]);
                    });
                },
            }
        }
        for index in table.indices.values() {
            let index_id = &index.id;
            let refs = index.fields.iter().map(|fid| -> Result<TokenStream, loga::Error> {
                let fvar = lookup_field(&field_var_map, table_key, fid)?;
                return Ok(quote!{
                    &#fvar
                });
            }).collect::<Result<Vec<TokenStream>, _>>().context(format!("Generating index {:?}", index_id))?;
            if index.unique {
                stmts.push(quote!{
                    #tvar.unique_index(#index_id, &[#(#refs), *]);
                });
            } else {
                stmts.push(quote!{
                    #tvar.index(#index_id, &[#(#refs), *]);
                });
            }
        }
    }
    stmts.push(quote!{
        good_ormning:: sqlite:: generate(good_ormning:: sqlite:: GenerateArgs {
            db_name: Some(#db_name.to_string()),
            versions: vec ![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    });
    return Ok(format_tokens(quote!{
        fn main() {
            #(#stmts) *
        }
    }));
}
