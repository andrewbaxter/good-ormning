use std::collections::HashMap;

fn sanitize_ident(s: &str) -> String {
    let out: String =
        s.chars().map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' }).collect();
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

fn emit_constraints_and_indices(
    lines: &mut Vec<String>,
    tvar: &str,
    table_key: &str,
    constraints: &std::collections::BTreeMap<String, good_ormning_core::pg::schema::constraint::Constraint>,
    indices: &std::collections::BTreeMap<String, good_ormning_core::pg::schema::index::Index>,
    field_var_map: &HashMap<(String, String), String>,
) {
    use good_ormning_core::pg::schema::constraint::ConstraintType;
    for constraint in constraints.values() {
        match &constraint.type_ {
            ConstraintType::PrimaryKey(pk) => {
                let refs = field_refs(field_var_map, table_key, &pk.fields);
                lines.push(format!("    {}.primary_key(\"{}\", &[{}]);", tvar, constraint.id, refs));
            },
            ConstraintType::ForeignKey(fk) => {
                let pairs: Vec<String> = fk
                    .fields
                    .iter()
                    .map(|(lf, rf)| {
                        let lvar = lookup_field(field_var_map, table_key, lf);
                        let rvar = lookup_field(field_var_map, &fk.remote_table, rf);
                        format!("(&{}, &{})", lvar, rvar)
                    })
                    .collect();
                lines.push(
                    format!("    {}.foreign_key(\"{}\", &[{}]);", tvar, constraint.id, pairs.join(", ")),
                );
            },
        }
    }
    for index in indices.values() {
        let refs = field_refs(field_var_map, table_key, &index.fields);
        if index.unique {
            lines.push(format!("    {}.unique_index(\"{}\", &[{}]);", tvar, index.id, refs));
        } else {
            lines.push(format!("    {}.index(\"{}\", &[{}]);", tvar, index.id, refs));
        }
    }
}

fn emit_constraints_and_indices_sqlite(
    lines: &mut Vec<String>,
    tvar: &str,
    table_key: &str,
    constraints: &std::collections::BTreeMap<String, good_ormning_core::sqlite::schema::constraint::Constraint>,
    indices: &std::collections::BTreeMap<String, good_ormning_core::sqlite::schema::index::Index>,
    field_var_map: &HashMap<(String, String), String>,
) {
    use good_ormning_core::sqlite::schema::constraint::ConstraintType;
    for constraint in constraints.values() {
        match &constraint.type_ {
            ConstraintType::PrimaryKey(pk) => {
                let refs = field_refs(field_var_map, table_key, &pk.fields);
                lines.push(format!("    {}.primary_key(\"{}\", &[{}]);", tvar, constraint.id, refs));
            },
            ConstraintType::ForeignKey(fk) => {
                let pairs: Vec<String> = fk
                    .fields
                    .iter()
                    .map(|(lf, rf)| {
                        let lvar = lookup_field(field_var_map, table_key, lf);
                        let rvar = lookup_field(field_var_map, &fk.remote_table, rf);
                        format!("(&{}, &{})", lvar, rvar)
                    })
                    .collect();
                lines.push(
                    format!("    {}.foreign_key(\"{}\", &[{}]);", tvar, constraint.id, pairs.join(", ")),
                );
            },
        }
    }
    for index in indices.values() {
        let refs = field_refs(field_var_map, table_key, &index.fields);
        if index.unique {
            lines.push(format!("    {}.unique_index(\"{}\", &[{}]);", tvar, index.id, refs));
        } else {
            lines.push(format!("    {}.index(\"{}\", &[{}]);", tvar, index.id, refs));
        }
    }
}

fn field_refs(field_var_map: &HashMap<(String, String), String>, table_key: &str, field_ids: &[String]) -> String {
    return field_ids
        .iter()
        .map(|fid| format!("&{}", lookup_field(field_var_map, table_key, fid)))
        .collect::<Vec<_>>()
        .join(", ");
}

fn lookup_field(field_var_map: &HashMap<(String, String), String>, table_key: &str, field_id: &str) -> String {
    return field_var_map
        .get(&(table_key.to_string(), field_id.to_string()))
        .cloned()
        .unwrap_or_else(|| format!("/* unknown: {}.{} */", table_key, field_id));
}

/// Generate a `build.rs` `main()` body for the given PostgreSQL `Version`.
pub fn generate_pg(version: &good_ormning_core::pg::Version, db_name: &str) -> String {
    let ns = "good_ormning::pg::schema::field";
    let mut lines: Vec<String> = vec![];
    lines.push("fn main() {".to_string());
    lines.push("    println!(\"cargo:rerun-if-changed=build.rs\");".to_string());
    lines.push("    let v = good_ormning::pg::Version::new();".to_string());

    // Map (table_key, field_id) → rust variable name for constraint/index refs.
    let mut field_var_map: HashMap<(String, String), String> = HashMap::new();

    for (table_key, table) in &version.tables {
        let tvar = format!("t_{}", sanitize_ident(table_key));
        lines.push(format!("    let {} = v.table(\"{}\");", tvar, table.id));

        for (field_id, field) in &table.fields {
            let fvar = format!("t_{}_{}", sanitize_ident(table_key), sanitize_ident(field_id));
            let fn_name = pg_field_fn(&field.type_.type_.type_.type_);
            let opt_chain = if field.type_.type_.opt { ".opt()" } else { "" };
            lines.push(
                format!(
                    "    let {} = {}.field(\"{}\", {}::{}(){}.build());",
                    fvar,
                    tvar,
                    field.id,
                    ns,
                    fn_name,
                    opt_chain
                ),
            );
            field_var_map.insert((table_key.clone(), field_id.clone()), fvar);
        }

        emit_constraints_and_indices(
            &mut lines,
            &tvar,
            table_key,
            &table.constraints,
            &table.indices,
            &field_var_map,
        );
    }

    lines.push(format!("    good_ormning::pg::generate(good_ormning::pg::GenerateArgs {{"));
    lines.push(format!("        db_name: Some(\"{}\".to_string()),", db_name));
    lines.push("        versions: vec![(1usize, v.build())],".to_string());
    lines.push("        ..Default::default()".to_string());
    lines.push("    }).unwrap();".to_string());
    lines.push("}".to_string());

    return lines.join("\n");
}

/// Generate a `build.rs` `main()` body for the given SQLite `Version`.
pub fn generate_sqlite(version: &good_ormning_core::sqlite::Version, db_name: &str) -> String {
    let ns = "good_ormning::sqlite::schema::field";
    let mut lines: Vec<String> = vec![];
    lines.push("fn main() {".to_string());
    lines.push("    println!(\"cargo:rerun-if-changed=build.rs\");".to_string());
    lines.push("    let v = good_ormning::sqlite::Version::new();".to_string());

    let mut field_var_map: HashMap<(String, String), String> = HashMap::new();

    for (table_key, table) in &version.tables {
        let tvar = format!("t_{}", sanitize_ident(table_key));
        lines.push(format!("    let {} = v.table(\"{}\");", tvar, table.id));

        for (field_id, field) in &table.fields {
            let fvar = format!("t_{}_{}", sanitize_ident(table_key), sanitize_ident(field_id));
            if field_id == "rowid" {
                // Rowid alias field: use rowid_field() instead of field().
                if field.id == "rowid" {
                    lines.push(format!("    let {} = {}.rowid_field(None);", fvar, tvar));
                } else {
                    lines.push(
                        format!("    let {} = {}.rowid_field(Some(\"{}\"));", fvar, tvar, field.id),
                    );
                }
            } else {
                let fn_name = sqlite_field_fn(&field.type_.type_.type_.type_);
                let opt_chain = if field.type_.type_.opt { ".opt()" } else { "" };
                lines.push(
                    format!(
                        "    let {} = {}.field(\"{}\", {}::{}(){}.build());",
                        fvar,
                        tvar,
                        field.id,
                        ns,
                        fn_name,
                        opt_chain
                    ),
                );
            }
            field_var_map.insert((table_key.clone(), field_id.clone()), fvar);
        }

        emit_constraints_and_indices_sqlite(
            &mut lines,
            &tvar,
            table_key,
            &table.constraints,
            &table.indices,
            &field_var_map,
        );
    }

    lines.push(format!(
        "    good_ormning::sqlite::generate(good_ormning::sqlite::GenerateArgs {{"
    ));
    lines.push(format!("        db_name: Some(\"{}\".to_string()),", db_name));
    lines.push("        versions: vec![(1usize, v.build())],".to_string());
    lines.push("        ..Default::default()".to_string());
    lines.push("    }).unwrap();".to_string());
    lines.push("}".to_string());

    return lines.join("\n");
}
