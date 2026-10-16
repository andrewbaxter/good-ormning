mod convert;

use {
    convert_case::{
        Case,
        Casing,
    },
    good_ormning_core::{
        mariadb::{
            Version as MariadbVersion,
            query::utils::{
                MariadbFieldInfo,
                MariadbTableInfo,
            },
            schema::{
                field::FieldRef as MariadbFieldRef,
                table::TableRef as MariadbTableRef,
            },
        },
        pg::{
            Version as PgVersion,
            query::utils::{
                PgFieldInfo,
                PgTableInfo,
            },
            schema::{
                field::FieldRef as PgFieldRef,
                table::TableRef as PgTableRef,
            },
        },
        sqlite::{
            Version as SqliteVersion,
            query::utils::{
                SqliteFieldInfo,
                SqliteTableInfo,
            },
            schema::{
                field::FieldRef as SqliteFieldRef,
                table::TableRef as SqliteTableRef,
            },
        },
        utils::Errs,
    },
    proc_macro::TokenStream,
    quote::{
        format_ident,
        quote,
    },
    std::{
        collections::{
            HashMap,
            HashSet,
            hash_map::DefaultHasher,
        },
        env,
        fs,
        hash::{
            Hash,
            Hasher,
        },
    },
    syn::{
        Ident,
        LitInt,
        LitStr,
        Token,
        parse::{
            Parse,
            ParseStream,
            Parser,
        },
        parse_macro_input,
    },
};

fn get_db_info(_engine: &str, provided_db_name: String) -> String {
    return provided_db_name;
}

/// The converters report unsupported SQL by panicking; turn those into normal
/// compile errors rather than "proc macro panicked".
fn catch_errors(f: impl FnOnce() -> proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let old_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| { }));
    let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
    std::panic::set_hook(old_hook);
    return match res {
        Ok(t) => t,
        Err(e) => {
            let msg = if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "Unknown error".to_string()
            };
            let msg = format!("good-ormning: {}", msg);
            quote!(compile_error!(#msg))
        },
    };
}

#[proc_macro]
pub fn good_query_many_mariadb(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as GoodQueryInput);
    return catch_errors(|| parse_and_generate_mariadb(input, good_ormning_core::QueryResCount::Many)).into();
}

#[proc_macro]
pub fn good_query_one_mariadb(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as GoodQueryInput);
    return catch_errors(|| parse_and_generate_mariadb(input, good_ormning_core::QueryResCount::One)).into();
}

#[proc_macro]
pub fn good_query_opt_mariadb(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as GoodQueryInput);
    return catch_errors(|| parse_and_generate_mariadb(input, good_ormning_core::QueryResCount::MaybeOne)).into();
}

#[proc_macro]
pub fn good_query_mariadb(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as GoodQueryInput);
    return catch_errors(|| parse_and_generate_mariadb(input, good_ormning_core::QueryResCount::None)).into();
}

/// See the `good_query` macro help in the readme.
#[proc_macro]
pub fn good_query_many_pg(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as GoodQueryInput);
    return catch_errors(|| parse_and_generate_pg(input, good_ormning_core::QueryResCount::Many)).into();
}

/// See the `good_query` macro help in the readme.
#[proc_macro]
pub fn good_query_many_sqlite(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as GoodQueryInput);
    return catch_errors(|| parse_and_generate_sqlite(input, good_ormning_core::QueryResCount::Many)).into();
}

/// See the `good_query` macro help in the readme.
#[proc_macro]
pub fn good_query_one_pg(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as GoodQueryInput);
    return catch_errors(|| parse_and_generate_pg(input, good_ormning_core::QueryResCount::One)).into();
}

/// See the `good_query` macro help in the readme.
#[proc_macro]
pub fn good_query_one_sqlite(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as GoodQueryInput);
    return catch_errors(|| parse_and_generate_sqlite(input, good_ormning_core::QueryResCount::One)).into();
}

/// See the `good_query` macro help in the readme.
#[proc_macro]
pub fn good_query_opt_pg(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as GoodQueryInput);
    return catch_errors(|| parse_and_generate_pg(input, good_ormning_core::QueryResCount::MaybeOne)).into();
}

/// See the `good_query` macro help in the readme.
#[proc_macro]
pub fn good_query_opt_sqlite(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as GoodQueryInput);
    return catch_errors(|| parse_and_generate_sqlite(input, good_ormning_core::QueryResCount::MaybeOne)).into();
}

/// See the `good_query` macro help in the readme.
#[proc_macro]
pub fn good_query_pg(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as GoodQueryInput);
    return catch_errors(|| parse_and_generate_pg(input, good_ormning_core::QueryResCount::None)).into();
}

/// See the `good_query` macro help in the readme.
#[proc_macro]
pub fn good_query_sqlite(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as GoodQueryInput);
    return catch_errors(|| parse_and_generate_sqlite(input, good_ormning_core::QueryResCount::None)).into();
}

struct GoodQueryInput {
    conn: syn::Expr,
    db_mod: Ident,
    db_name: String,
    inline_params: Vec<(std::ops::Range<usize>, Ident)>,
    param_types: Vec<(Ident, ParamType)>,
    params: Vec<syn::Expr>,
    sql: String,
    version: Option<usize>,
}

impl Parse for GoodQueryInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let db_mod: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let (db_name, version, sql) = {
            if input.peek(LitInt) {
                let version_lit: LitInt = input.parse()?;
                let version = version_lit.base10_parse::<usize>()?;
                input.parse::<Token![,]>()?;
                let sql_lit: LitStr = input.parse()?;
                let sql = sql_lit.value();
                input.parse::<Token![;]>()?;
                ("".to_string(), Some(version), sql)
            } else {
                let first: LitStr = input.parse()?;
                if input.peek(Token![;]) {
                    input.parse::<Token![;]>()?;
                    ("".to_string(), None, first.value())
                } else {
                    input.parse::<Token![,]>()?;
                    let lookahead = input.lookahead1();
                    if lookahead.peek(LitInt) {
                        let version_lit: LitInt = input.parse()?;
                        let version = version_lit.base10_parse::<usize>()?;
                        input.parse::<Token![,]>()?;
                        let sql_lit: LitStr = input.parse()?;
                        let sql = sql_lit.value();
                        input.parse::<Token![;]>()?;
                        (first.value(), Some(version), sql)
                    } else if lookahead.peek(LitStr) {
                        let sql_lit: LitStr = input.parse()?;
                        let sql = sql_lit.value();
                        input.parse::<Token![;]>()?;
                        (first.value(), None, sql)
                    } else {
                        return Err(lookahead.error());
                    }
                }
            }
        };
        let conn: syn::Expr = input.parse()?;
        let mut param_types = Vec::new();
        let mut params = Vec::new();
        while input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }
            let name: Ident = input.parse()?;
            input.parse::<Token![:]>()?;
            let mut arr = false;
            let mut opt = false;
            let mut base = String::new();
            while input.peek(Ident) {
                let id: Ident = input.parse()?;
                if id == "arr" {
                    arr = true;
                } else if id == "opt" {
                    opt = true;
                } else {
                    base = id.to_string();
                    break;
                }
            }
            if base.is_empty() {
                return Err(input.error("Expected parameter type"));
            }
            input.parse::<Token![=]>()?;
            let val: syn::Expr = input.parse()?;
            param_types.push((name, ParamType {
                arr: arr,
                opt: opt,
                base: base,
            }));
            params.push(val);
        }
        let mut final_sql = String::new();
        let mut inline_params = vec![];
        let mut last_end = 0;
        let bytes = sql.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'$' {
                if i + 1 < bytes.len() && bytes[i + 1] == b'{' {
                    final_sql.push_str(&sql[last_end .. i]);
                    i += 2;
                    let content_start = i;
                    let mut depth = 1usize;
                    while i < bytes.len() {
                        match bytes[i] {
                            b'{' => {
                                depth += 1;
                            },
                            b'}' => {
                                depth -= 1;
                                if depth == 0 {
                                    break;
                                }
                            },
                            b'"' => {
                                // Skip over a rust string literal so braces inside it don't count
                                i += 1;
                                while i < bytes.len() && bytes[i] != b'"' {
                                    if bytes[i] == b'\\' {
                                        i += 1;
                                    }
                                    i += 1;
                                }
                            },
                            _ => { },
                        }
                        i += 1;
                    }
                    if i >= bytes.len() {
                        return Err(syn::Error::new(input.span(), "Unclosed inline parameter ${"));
                    }
                    let content = &sql[content_start .. i];
                    i += 1;
                    let mut split = None;
                    for (idx, b) in content.as_bytes().iter().enumerate() {
                        if *b == b'=' {
                            split = Some((&content[..idx], &content[idx + 1..]));
                            break;
                        }
                    }
                    let (type_str, val_str) = split.ok_or_else(|| {
                        return syn::Error::new(
                            input.span(),
                            "Invalid inline parameter format. Expected ${type = value}",
                        );
                    })?;
                    let val: syn::Expr = syn::parse_str(val_str).map_err(|e| {
                        return syn::Error::new(
                            input.span(),
                            format!("Failed to parse inline parameter value: {}", e),
                        );
                    })?;
                    let type_tokens: proc_macro2::TokenStream = type_str.parse().map_err(|e| {
                        return syn::Error::new(
                            input.span(),
                            format!("Failed to parse inline parameter type tokens: {}", e),
                        );
                    })?;
                    let (arr_p, opt_p, base_p) = (|type_input: ParseStream| {
                        let mut arr = false;
                        let mut opt = false;
                        let mut base = String::new();
                        while type_input.peek(Ident) {
                            let id: Ident = type_input.parse()?;
                            if id == "arr" {
                                arr = true;
                            } else if id == "opt" {
                                opt = true;
                            } else {
                                base = id.to_string();
                                break;
                            }
                        }
                        return Ok((arr, opt, base));
                    }).parse2(type_tokens).map_err(|e| {
                        return syn::Error::new(
                            input.span(),
                            format!("Failed to parse inline parameter type: {}", e),
                        );
                    })?;
                    if base_p.is_empty() {
                        return Err(input.error("Expected base type in inline parameter"));
                    }
                    let param_idx = params.len() + 1;
                    let name = format_ident!("p{}", param_idx);
                    let pt = ParamType {
                        arr: arr_p,
                        opt: opt_p,
                        base: base_p,
                    };
                    params.push(val);
                    param_types.push((name.clone(), pt));
                    let start = final_sql.len();
                    final_sql.push_str(&format!("${}", param_idx));
                    inline_params.push((start .. final_sql.len(), name));
                    last_end = i;
                    continue;
                }
            }
            i += 1;
        }
        final_sql.push_str(&sql[last_end..]);
        let mut seen_params = HashSet::new();
        for (name, _) in &param_types {
            let name = name.to_string();
            if !seen_params.insert(name.trim_start_matches("r#").to_string()) {
                return Err(
                    syn::Error::new(
                        input.span(),
                        format!(
                            "Parameter {} is defined more than once (inline parameters are named p1, p2, ...)",
                            name
                        ),
                    ),
                );
            }
        }
        return Ok(GoodQueryInput {
            db_mod: db_mod,
            version: version,
            db_name: db_name,
            inline_params: inline_params,
            sql: final_sql,
            param_types: param_types,
            conn: conn,
            params: params,
        });
    }
}

struct ParamType {
    arr: bool,
    base: String,
    opt: bool,
}

fn parse_and_generate_mariadb(
    input: GoodQueryInput,
    res_count: good_ormning_core::QueryResCount,
) -> proc_macro2::TokenStream {
    let db_name = get_db_info("mariadb", input.db_name.clone());
    let dialect = sqlparser::dialect::MySqlDialect {};
    let mut sql = String::new();
    let mut inline_locations = HashMap::new();
    let mut last_end = 0;
    for (range, ident) in &input.inline_params {
        sql.push_str(&input.sql[last_end .. range.start]);
        let line = sql.matches('\n').count() as u64 + 1;
        let column = sql.rsplit('\n').next().unwrap_or("").chars().count() as u64 + 1;
        inline_locations.insert((line, column), ident.to_string());
        sql.push('?');
        last_end = range.end;
    }
    sql.push_str(&input.sql[last_end..]);
    let tokens = match sqlparser::tokenizer::Tokenizer::new(&dialect, &sql).tokenize_with_location() {
        Ok(t) => t,
        Err(e) => {
            let e = e.to_string();
            return quote!(compile_error!(#e));
        },
    };
    let mut explicit_params =
        input
            .param_types
            .iter()
            .map(|(ident, _)| ident.to_string())
            .filter(|name| !inline_locations.values().any(|n| n == name));
    let mut param_locations = HashMap::new();
    for token in tokens {
        let sqlparser::tokenizer::Token::Placeholder(p) = &token.token else {
            continue;
        };
        let location = (token.span.start.line, token.span.start.column);
        let name = match inline_locations.get(&location) {
            Some(name) => name.clone(),
            None => {
                if p != "?" {
                    let e = format!("Unsupported parameter `{}`, MariaDB parameters are `?`", p);
                    return quote!(compile_error!(#e));
                }
                match explicit_params.next() {
                    Some(name) => name,
                    None => {
                        return quote!(compile_error!("The query has more `?` than parameters"));
                    },
                }
            },
        };
        param_locations.insert(location, name.trim_start_matches("r#").to_string());
    }
    let ast = match sqlparser::parser::Parser::parse_sql(&dialect, &sql) {
        Ok(ast) => ast,
        Err(e) => {
            let e = e.to_string();
            return quote!(compile_error!(#e));
        },
    };
    if ast.is_empty() {
        return quote!(compile_error!("Empty SQL statement"));
    }
    if ast.len() > 1 {
        return quote!(compile_error!("Queries must contain exactly one SQL statement"));
    }
    let statement = &ast[0];
    let mut errs = Errs::new();
    let out_dir = env::var("OUT_DIR").unwrap_or_else(|_| ".".to_string());
    let path =
        std::path::Path::new(&out_dir)
            .join("good_ormning")
            .join(good_ormning_core::utils::json_file_name("mariadb", &db_name));
    if !path.exists() {
        let e = format!("Schema file not found at {:?}. Did you run the build script?", path.to_string_lossy());
        return quote!(compile_error!(#e));
    }
    let schema_file: good_ormning_core::utils::SchemaFile<MariadbVersion> =
        match serde_json::from_str(&fs::read_to_string(&path).unwrap()) {
            Ok(m) => m,
            Err(e) => {
                let e = e.to_string();
                return quote!(compile_error!(#e));
            },
        };
    let versions_map = schema_file.versions;
    let mut field_lookup = HashMap::new();
    let version_i = input.version.unwrap_or_else(|| versions_map.keys().max().copied().unwrap_or(0));
    let version = match versions_map.get(&version_i) {
        Some(v) => v,
        None => {
            let e = format!("Version {} not found in schema for db {}", version_i, db_name);
            return quote!(compile_error!(#e));
        },
    };
    let custom_types = version.custom_types.clone();
    for (table_id, table) in &version.tables {
        let mut fields: HashMap<MariadbFieldRef, MariadbFieldInfo> = HashMap::new();
        for (field_id, field) in &table.fields {
            fields.insert(MariadbFieldRef {
                table_id: table_id.clone(),
                field_id: field_id.clone(),
            }, MariadbFieldInfo {
                sql_name: field.id.clone(),
                type_: field.type_.type_.clone(),
                has_default: field.type_.auto_increment || field.type_.default.is_some(),
            });
        }
        field_lookup.insert(MariadbTableRef(table_id.clone()), MariadbTableInfo {
            sql_name: table.id.clone(),
            fields: fields,
        });
    }
    let mut query = {
        let input = &input;
        let custom_types = &custom_types;
        let field_lookup = &field_lookup;
        let mut used_params = crate::convert::mariadb::ConvertState {
            param_locations: param_locations,
            used_params: HashSet::new(),
        };
        let body: Box<dyn good_ormning_core::mariadb::query::utils::QueryBody> = match statement {
            sqlparser::ast::Statement::Query(q) => {
                if let sqlparser::ast::SetExpr::Delete(_) = &*q.body {
                    panic!("MariaDB doesn't support `with` in deletes");
                }
                if let sqlparser::ast::SetExpr::Update(_) = &*q.body {
                    panic!("MariaDB doesn't support `with` in updates");
                }
                Box::new(
                    crate::convert::mariadb::convert_select_query(
                        input,
                        q,
                        &mut used_params,
                        custom_types,
                        field_lookup,
                    ),
                )
            },
            sqlparser::ast::Statement::Insert(insert) => {
                let used_params = &mut used_params;
                let table = crate::convert::mariadb::get_table_ref(match &insert.table {
                    sqlparser::ast::TableObject::TableName(n) => n,
                    _ => panic!("Unsupported insert table object: {:?}", insert.table),
                });
                if let Some(or) = &insert.or {
                    panic!("`insert or {}` is not supported in MariaDB", or);
                }
                if insert.priority.is_some() {
                    panic!("`low_priority`, `delayed` and `high_priority` aren't supported");
                }
                if insert.partitioned.is_some() || !insert.after_columns.is_empty() {
                    panic!("Partition selection isn't supported");
                }
                if insert.insert_alias.is_some() || insert.table_alias.is_some() {
                    panic!(
                        "MariaDB doesn't support insert aliases, use `value(column)` in `on duplicate key update`"
                    );
                }
                if insert.overwrite || insert.has_table_keyword || insert.settings.is_some() ||
                    insert.format_clause.is_some() {
                    panic!("Unsupported insert syntax for MariaDB: {}", statement);
                }
                if insert.columns.is_empty() && insert.assignments.is_empty() {
                    panic!(
                        "Inserts need a column list; without one the values depend on the column order in the database, which migrations don't control"
                    );
                }
                if let Some(sqlparser::ast::SetExpr::Values(v)) = insert.source.as_ref().map(|q| &*q.body) {
                    if v.explicit_row {
                        panic!("`values row(...)` isn't supported by MariaDB");
                    }
                }
                let columns: Vec<good_ormning_core::mariadb::schema::field::FieldRef> =
                    insert.columns.iter().map(|c| good_ormning_core::mariadb::schema::field::FieldRef {
                        table_id: table.0.clone(),
                        field_id: c.value.clone(),
                    }).collect();
                let assignment_target =
                    |a: &sqlparser::ast::Assignment| -> good_ormning_core::mariadb::schema::field::FieldRef {
                        let target_name = match &a.target {
                            sqlparser::ast::AssignmentTarget::ColumnName(name) => {
                                let parts = name.0.iter().map(|p| match p {
                                    sqlparser::ast::ObjectNamePart::Identifier(i) => i.value.clone(),
                                }).collect::<Vec<_>>();
                                match parts.as_slice() {
                                    [column] => column.clone(),
                                    [t, column] if *t == table.0 => column.clone(),
                                    _ => panic!(
                                        "Assignments in inserts must be to columns of the inserted table: {}",
                                        name
                                    ),
                                }
                            },
                            _ => panic!("Unsupported assignment target: {:?}", a.target),
                        };
                        return good_ormning_core::mariadb::schema::field::FieldRef {
                            table_id: table.0.clone(),
                            field_id: target_name,
                        };
                    };
                let source = if let Some(q) = &insert.source {
                    match &*q.body {
                        sqlparser::ast::SetExpr::Values(v) => {
                            let mut rows = vec![];
                            for row in &v.rows {
                                if row.len() != columns.len() {
                                    panic!(
                                        "Insert row has {} values but {} columns were specified",
                                        row.len(),
                                        columns.len()
                                    );
                                }
                                rows.push(
                                    row
                                        .iter()
                                        .map(
                                            |expr| crate::convert::mariadb::convert_expr(
                                                input,
                                                expr,
                                                used_params,
                                                custom_types,
                                                field_lookup,
                                            ),
                                        )
                                        .collect::<Vec<_>>(),
                                );
                            }
                            if rows.len() == 1 {
                                good_ormning_core::mariadb::query::insert::InsertSource::Values(
                                    columns.iter().cloned().zip(rows.pop().unwrap().into_iter()).collect(),
                                )
                            } else {
                                good_ormning_core::mariadb::query::insert::InsertSource::Rows(
                                    good_ormning_core::mariadb::query::insert::InsertRows {
                                        columns: columns,
                                        rows: rows,
                                    },
                                )
                            }
                        },
                        _ => {
                            good_ormning_core::mariadb::query::insert::InsertSource::Select {
                                columns: columns,
                                select: crate::convert::mariadb::convert_select_query(
                                    input,
                                    q,
                                    used_params,
                                    custom_types,
                                    field_lookup,
                                ),
                            }
                        },
                    }
                } else {
                    good_ormning_core::mariadb::query::insert::InsertSource::Values(
                        insert
                            .assignments
                            .iter()
                            .map(
                                |a| (
                                    assignment_target(a),
                                    crate::convert::mariadb::convert_expr(
                                        input,
                                        &a.value,
                                        used_params,
                                        custom_types,
                                        field_lookup,
                                    ),
                                ),
                            )
                            .collect(),
                    )
                };
                let mut on_conflict = None;
                if insert.replace_into {
                    if insert.ignore {
                        panic!("`replace` can't be used with `ignore`");
                    }
                    on_conflict = Some(good_ormning_core::mariadb::query::insert::InsertConflict::Replace);
                }
                if let Some(on) = &insert.on {
                    if on_conflict.is_some() {
                        panic!("`replace` can't be used with `on duplicate key update`");
                    }
                    match on {
                        sqlparser::ast::OnInsert::DuplicateKeyUpdate(assignments) => {
                            let mut updates = vec![];
                            for a in assignments {
                                updates.push(
                                    (
                                        assignment_target(a),
                                        crate::convert::mariadb::convert_expr(
                                            input,
                                            &a.value,
                                            used_params,
                                            custom_types,
                                            field_lookup,
                                        ),
                                    ),
                                );
                            }
                            on_conflict =
                                Some(
                                    good_ormning_core::mariadb::query::insert::InsertConflict::DuplicateKeyUpdate(
                                        updates,
                                    ),
                                );
                        },
                        sqlparser::ast::OnInsert::OnConflict(_) => panic!(
                            "MariaDB doesn't have `on conflict`, use `on duplicate key update` or `insert ignore` instead"
                        ),
                        _ => panic!("Unsupported `on` clause in insert: {:?}", on),
                    }
                }
                Box::new(good_ormning_core::mariadb::query::insert::Insert {
                    ignore: insert.ignore,
                    table: table.clone(),
                    source: source,
                    on_conflict: on_conflict,
                    returning: crate::convert::mariadb::convert_returning(
                        input,
                        &insert.returning,
                        used_params,
                        custom_types,
                        field_lookup,
                        &[(table.0.clone(), Some(table.clone()))],
                        &[],
                        &[],
                    ),
                })
            },
            sqlparser::ast::Statement::Update { table, assignments, from, selection, returning, or } => {
                if let Some(or) = or {
                    panic!("`update or {}` is not supported in MariaDB", or);
                }
                if from.is_some() {
                    panic!("MariaDB doesn't support `update ... from`, use `update a join b on ... set ...`");
                }
                if returning.is_some() {
                    panic!("MariaDB doesn't support `returning` in updates");
                }
                let used_params = &mut used_params;
                let (table_ref, alias) = match &table.relation {
                    sqlparser::ast::TableFactor::Table { name, alias, .. } => {
                        crate::convert::mariadb::check_plain_table(&table.relation);
                        (
                            crate::convert::mariadb::get_table_ref(name),
                            alias.as_ref().map(|a| a.name.value.clone()),
                        )
                    },
                    _ => panic!("Unsupported: Update table factor: {:?}", table.relation),
                };
                let join =
                    crate::convert::mariadb::convert_sql_joins(
                        input,
                        table,
                        used_params,
                        custom_types,
                        field_lookup,
                    );
                let mut sets = vec![];
                for a in assignments {
                    let target = match &a.target {
                        sqlparser::ast::AssignmentTarget::ColumnName(name) => name,
                        _ => panic!("Unsupported: AssignmentTarget: {:?}", a.target),
                    };
                    if target.0.len() > 2 {
                        panic!("Tables in other databases aren't supported: {}", target);
                    }
                    let qualifier = if target.0.len() >= 2 {
                        match &target.0[target.0.len() - 2] {
                            sqlparser::ast::ObjectNamePart::Identifier(i) => i.value.clone(),
                        }
                    } else {
                        alias.clone().unwrap_or_else(|| table_ref.0.clone())
                    };
                    let target_name = match target.0.last().unwrap() {
                        sqlparser::ast::ObjectNamePart::Identifier(i) => i.value.clone(),
                    };
                    let field = good_ormning_core::mariadb::schema::field::FieldRef {
                        table_id: qualifier,
                        field_id: target_name,
                    };
                    sets.push(
                        (
                            field,
                            crate::convert::mariadb::convert_expr(
                                input,
                                &a.value,
                                used_params,
                                custom_types,
                                field_lookup,
                            ),
                        ),
                    );
                }
                Box::new(good_ormning_core::mariadb::query::update::Update {
                    alias: alias,
                    join: join,
                    limit: None,
                    order: vec![],
                    table: table_ref,
                    values: sets,
                    where_: selection
                        .as_ref()
                        .map(
                            |e| crate::convert::mariadb::convert_expr(
                                input,
                                e,
                                used_params,
                                custom_types,
                                field_lookup,
                            ),
                        ),
                })
            },
            sqlparser::ast::Statement::Delete(delete) => {
                let used_params = &mut used_params;
                let from = match &delete.from {
                    sqlparser::ast::FromTable::WithFromKeyword(f) => f,
                    sqlparser::ast::FromTable::WithoutKeyword(f) => f,
                };
                if from.len() != 1 {
                    panic!("Delete must have exactly one table, use joins for other tables");
                }
                if delete.using.is_some() {
                    panic!(
                        "`delete ... using` isn't supported, use `delete alias from table as alias join ...` instead"
                    );
                }
                let (table_ref, alias) = match &from[0].relation {
                    sqlparser::ast::TableFactor::Table { name, alias, .. } => {
                        crate::convert::mariadb::check_plain_table(&from[0].relation);
                        (
                            crate::convert::mariadb::get_table_ref(name),
                            alias.as_ref().map(|a| a.name.value.clone()),
                        )
                    },
                    _ => panic!("Unsupported: Delete table factor: {:?}", from[0].relation),
                };
                let targets =
                    delete
                        .tables
                        .iter()
                        .map(|t| crate::convert::mariadb::get_table_ref(t).0)
                        .collect::<Vec<_>>();
                let join =
                    crate::convert::mariadb::convert_sql_joins(
                        input,
                        &from[0],
                        used_params,
                        custom_types,
                        field_lookup,
                    );
                let mut order = vec![];
                for o in &delete.order_by {
                    let e =
                        crate::convert::mariadb::convert_expr(
                            input,
                            &o.expr,
                            used_params,
                            custom_types,
                            field_lookup,
                        );
                    if o.options.nulls_first.is_some() {
                        panic!("MariaDB doesn't support `nulls first`/`nulls last`");
                    }
                    let dir = match o.options.asc {
                        Some(true) | None => good_ormning_core::mariadb::query::select::Order::Asc,
                        Some(false) => good_ormning_core::mariadb::query::select::Order::Desc,
                    };
                    order.push((e, dir));
                }
                Box::new(good_ormning_core::mariadb::query::delete::Delete {
                    targets: targets,
                    alias: alias.clone(),
                    join: join,
                    limit: delete
                        .limit
                        .as_ref()
                        .map(
                            |e| crate::convert::mariadb::convert_expr(
                                input,
                                e,
                                used_params,
                                custom_types,
                                field_lookup,
                            ),
                        ),
                    order: order,
                    table: table_ref.clone(),
                    where_: delete
                        .selection
                        .as_ref()
                        .map(
                            |e| crate::convert::mariadb::convert_expr(
                                input,
                                e,
                                used_params,
                                custom_types,
                                field_lookup,
                            ),
                        ),
                    returning: crate::convert::mariadb::convert_returning(
                        input,
                        &delete.returning,
                        used_params,
                        custom_types,
                        field_lookup,
                        &[(alias.unwrap_or_else(|| table_ref.0.clone()), Some(table_ref.clone()))],
                        &[],
                        &[],
                    ),
                })
            },
            _ => panic!("Unsupported: Statement type: {:?}", statement),
        };
        for (ident, _) in &input.param_types {
            let s = ident.to_string();
            let s = s.trim_start_matches("r#");
            if !used_params.used_params.contains(s) {
                panic!("Parameter {} not used in query", ident);
            }
        }
        good_ormning_core::mariadb::Query {
            name: "unnamed".to_string(),
            body: body,
            res_count: good_ormning_core::QueryResCount::Many,
            res_name: None,
        }
    };
    query.res_count = res_count;
    let mut hasher = DefaultHasher::new();
    input.sql.hash(&mut hasher);
    let query_hash = hasher.finish();
    let query_name = format_ident!("good_query_{}", query_hash);
    query.name = query_name.to_string();
    let pascal_db_name: String = db_name.to_case(Case::Pascal);
    let db_mod = &input.db_mod;
    let db_type = if let Some(v) = input.version {
        let name = format_ident!("Db{}{}", pascal_db_name, v);
        quote!(#db_mod::#name < impl:: good_ormning:: runtime:: mariadb:: MariadbConnection >)
    } else {
        let name = format_ident!("Db{}{}", pascal_db_name, version_i);
        quote!(#db_mod::#name < impl:: good_ormning:: runtime:: mariadb:: MariadbConnection >)
    };
    let generated =
        good_ormning_core::mariadb::query::generate::generate_query_functions(
            &mut errs,
            field_lookup,
            vec![query],
            "inline",
            db_type,
            Some(
                &input
                    .param_types
                    .iter()
                    .map(|(ident, _)| ident.to_string().trim_start_matches("r#").to_string())
                    .collect::<Vec<_>>(),
            ),
            schema_file.default_time_library,
        );
    if let Err(e) = errs.raise() {
        let e = e.join("\n");
        return quote!(compile_error!(#e));
    }
    let conn = &input.conn;
    let args = &input.params;
    let db_name_lit = LitStr::new(&db_name, input.db_mod.span());
    let db_mod_str = input.db_mod.to_string();
    let _db_mod_lit = LitStr::new(&db_mod_str, input.db_mod.span());
    quote!{
        {
            const _:() = {
                if !:: good_ormning:: runtime:: utils:: str_eq(#db_mod:: DB_NAME, #db_name_lit) {
                    #[allow(unconditional_panic)]
                    let _ = ["Database name mismatch"][1];
                }
            };
            use ::good_ormning::runtime::GoodError;
            use ::good_ormning::runtime::ToGoodError;
            use ::good_ormning::runtime::mariadb::MariadbConnection;
            #(#generated) * #query_name(#conn, #(#args,) *)
        }
    }
}

fn parse_and_generate_pg(
    input: GoodQueryInput,
    res_count: good_ormning_core::QueryResCount,
) -> proc_macro2::TokenStream {
    let db_name = get_db_info("pg", input.db_name.clone());
    let dialect = sqlparser::dialect::PostgreSqlDialect {};
    let ast = match sqlparser::parser::Parser::parse_sql(&dialect, &input.sql) {
        Ok(ast) => ast,
        Err(e) => {
            let e = e.to_string();
            return quote!(compile_error!(#e));
        },
    };
    if ast.is_empty() {
        return quote!(compile_error!("Empty SQL statement"));
    }
    if ast.len() > 1 {
        return quote!(compile_error!("Queries must contain exactly one SQL statement"));
    }
    let statement = &ast[0];
    let mut errs = Errs::new();
    let out_dir = env::var("OUT_DIR").unwrap_or_else(|_| ".".to_string());
    let path =
        std::path::Path::new(&out_dir)
            .join("good_ormning")
            .join(good_ormning_core::utils::json_file_name("pg", &db_name));
    if !path.exists() {
        let e = format!("Schema file not found at {:?}. Did you run the build script?", path.to_string_lossy());
        return quote!(compile_error!(#e));
    }
    let schema_file: good_ormning_core::utils::SchemaFile<PgVersion> =
        match serde_json::from_str(&fs::read_to_string(&path).unwrap()) {
            Ok(m) => m,
            Err(e) => {
                let e = e.to_string();
                return quote!(compile_error!(#e));
            },
        };
    let versions_map = schema_file.versions;
    let mut field_lookup = HashMap::new();
    let version_i = input.version.unwrap_or_else(|| versions_map.keys().max().copied().unwrap_or(0));
    let version = match versions_map.get(&version_i) {
        Some(v) => v,
        None => {
            let e = format!("Version {} not found in schema for db {}", version_i, db_name);
            return quote!(compile_error!(#e));
        },
    };
    let custom_types = version.custom_types.clone();
    for (table_id, table) in &version.tables {
        let mut fields: HashMap<PgFieldRef, PgFieldInfo> = HashMap::new();
        for (field_id, field) in &table.fields {
            fields.insert(PgFieldRef {
                table_id: table_id.clone(),
                field_id: field_id.clone(),
            }, PgFieldInfo {
                sql_name: field.id.clone(),
                type_: field.type_.type_.clone(),
            });
        }
        field_lookup.insert(PgTableRef(table_id.clone()), PgTableInfo {
            sql_name: table.id.clone(),
            fields: fields,
        });
    }
    let mut query = {
        let input = &input;
        let custom_types = &custom_types;
        let field_lookup = &field_lookup;
        let mut used_params = HashSet::new();
        let body: Box<dyn good_ormning_core::pg::query::utils::QueryBody> = match statement {
            sqlparser::ast::Statement::Query(q) => {
                if let sqlparser::ast::SetExpr::Delete(sqlparser::ast::Statement::Delete(d)) = &*q.body {
                    let mut del =
                        crate::convert::pg::convert_delete(input, d, &mut used_params, custom_types, field_lookup);
                    if let Some(with) = &q.with {
                        del.with =
                            Some(
                                crate::convert::pg::convert_with(
                                    input,
                                    with,
                                    &mut used_params,
                                    custom_types,
                                    field_lookup,
                                ),
                            );
                    }
                    Box::new(del)
                } else {
                    Box::new(
                        crate::convert::pg::convert_select_query(
                            input,
                            q,
                            &mut used_params,
                            custom_types,
                            field_lookup,
                        ),
                    )
                }
            },
            sqlparser::ast::Statement::Insert(insert) => {
                let used_params = &mut used_params;
                let table = crate::convert::pg::get_table_ref(match &insert.table {
                    sqlparser::ast::TableObject::TableName(n) => n,
                    _ => panic!("Unsupported insert table object: {:?}", insert.table),
                });
                let columns: Vec<good_ormning_core::pg::schema::field::FieldRef> =
                    insert.columns.iter().map(|c| good_ormning_core::pg::schema::field::FieldRef {
                        table_id: table.0.clone(),
                        field_id: c.value.clone(),
                    }).collect();
                let source = if let Some(q) = &insert.source {
                    match &*q.body {
                        sqlparser::ast::SetExpr::Values(v) => {
                            let mut rows = vec![];
                            for row in &v.rows {
                                if row.len() != columns.len() {
                                    panic!(
                                        "Insert row has {} values but {} columns were specified",
                                        row.len(),
                                        columns.len()
                                    );
                                }
                                rows.push(
                                    row
                                        .iter()
                                        .map(
                                            |expr| crate::convert::pg::convert_expr(
                                                input,
                                                expr,
                                                used_params,
                                                custom_types,
                                                field_lookup,
                                            ),
                                        )
                                        .collect::<Vec<_>>(),
                                );
                            }
                            if rows.len() == 1 {
                                good_ormning_core::pg::query::insert::InsertSource::Values(
                                    columns.iter().cloned().zip(rows.pop().unwrap().into_iter()).collect(),
                                )
                            } else {
                                good_ormning_core::pg::query::insert::InsertSource::Rows(
                                    good_ormning_core::pg::query::insert::InsertRows {
                                        columns: columns,
                                        rows: rows,
                                    },
                                )
                            }
                        },
                        _ => {
                            good_ormning_core::pg::query::insert::InsertSource::Select {
                                columns: columns,
                                select: crate::convert::pg::convert_select_query(
                                    input,
                                    q,
                                    used_params,
                                    custom_types,
                                    field_lookup,
                                ),
                            }
                        },
                    }
                } else {
                    good_ormning_core::pg::query::insert::InsertSource::Values(vec![])
                };
                let on_conflict = if let Some(on) = &insert.on {
                    match on {
                        sqlparser::ast::OnInsert::OnConflict(oc) => {
                            let conflict = match &oc.conflict_target {
                                Some(sqlparser::ast::ConflictTarget::Columns(idents)) => idents
                                    .iter()
                                    .map(|id| good_ormning_core::pg::schema::field::FieldRef {
                                        table_id: table.0.clone(),
                                        field_id: id.value.clone(),
                                    })
                                    .collect(),
                                Some(sqlparser::ast::ConflictTarget::OnConstraint(c)) => panic!(
                                    "`on conflict on constraint` is not supported, use a list of columns instead: {}",
                                    c
                                ),
                                None => vec![],
                            };
                            match &oc.action {
                                sqlparser::ast::OnConflictAction::DoNothing => {
                                    if conflict.is_empty() {
                                        Some(good_ormning_core::pg::query::insert::InsertConflict::DoNothing)
                                    } else {
                                        Some(
                                            good_ormning_core::pg::query::insert::InsertConflict::DoNothingOn {
                                                conflict: conflict,
                                            },
                                        )
                                    }
                                },
                                sqlparser::ast::OnConflictAction::DoUpdate(du) => {
                                    if let Some(sel) = &du.selection {
                                        panic!("`on conflict do update ... where` is not supported: {}", sel);
                                    }
                                    let mut updates = vec![];
                                    for a in &du.assignments {
                                        let target_name = match &a.target {
                                            sqlparser::ast::AssignmentTarget::ColumnName(name) => match name
                                                .0
                                                .last()
                                                .unwrap() {
                                                sqlparser::ast::ObjectNamePart::Identifier(i) => i.value.clone(),
                                            },
                                            _ => panic!("Unsupported assignment target: {:?}", a.target),
                                        };
                                        let field = good_ormning_core::pg::schema::field::FieldRef {
                                            table_id: table.0.clone(),
                                            field_id: target_name,
                                        };
                                        updates.push(
                                            (
                                                field,
                                                crate::convert::pg::convert_expr(
                                                    input,
                                                    &a.value,
                                                    used_params,
                                                    custom_types,
                                                    field_lookup,
                                                ),
                                            ),
                                        );
                                    }
                                    Some(good_ormning_core::pg::query::insert::InsertConflict::DoUpdate {
                                        conflict: conflict,
                                        set: updates,
                                    })
                                },
                            }
                        },
                        sqlparser::ast::OnInsert::DuplicateKeyUpdate(_) => panic!(
                            "`on duplicate key update` is not supported"
                        ),
                        _ => panic!("Unsupported `on` clause in insert: {:?}", on),
                    }
                } else {
                    if let Some(or) = &insert.or {
                        panic!("`insert or {}` is not supported in PostgreSQL", or);
                    }
                    None
                };
                Box::new(good_ormning_core::pg::query::insert::Insert {
                    table: table.clone(),
                    source: source,
                    on_conflict: on_conflict,
                    returning: crate::convert::pg::convert_returning(
                        input,
                        &insert.returning,
                        used_params,
                        custom_types,
                        field_lookup,
                        &[(table.0.clone(), Some(table.clone()))],
                    ),
                })
            },
            sqlparser::ast::Statement::Update { table, assignments, from, selection, returning, or } => {
                if let Some(or) = or {
                    panic!("`update or {}` is not supported in PostgreSQL", or);
                }
                let used_params = &mut used_params;
                if !table.joins.is_empty() {
                    panic!("Joins on the updated table aren't supported, use `update ... from`");
                }
                let (table_ref, alias) = match &table.relation {
                    sqlparser::ast::TableFactor::Table { name, alias, .. } => {
                        (crate::convert::pg::get_table_ref(name), alias.as_ref().map(|a| a.name.value.clone()))
                    },
                    _ => panic!("Unsupported: Update table factor: {:?}", table.relation),
                };
                let mut from_sources = vec![];
                if let Some(from) = from {
                    let tables = match from {
                        sqlparser::ast::UpdateTableFromKind::BeforeSet(t) |
                        sqlparser::ast::UpdateTableFromKind::AfterSet(t) => t,
                    };
                    for t in tables {
                        if !t.joins.is_empty() {
                            panic!(
                                "Joins in `update ... from` aren't supported, list the tables separated by commas instead"
                            );
                        }
                        from_sources.push(
                            crate::convert::pg::convert_table_factor_pg(
                                input,
                                &t.relation,
                                used_params,
                                custom_types,
                                field_lookup,
                            ),
                        );
                    }
                }
                let mut sets = vec![];
                for a in assignments {
                    let target_name = match &a.target {
                        sqlparser::ast::AssignmentTarget::ColumnName(name) => match name.0.last().unwrap() {
                            sqlparser::ast::ObjectNamePart::Identifier(i) => i.value.clone(),
                        },
                        _ => panic!("Unsupported: AssignmentTarget: {:?}", a.target),
                    };
                    let field = good_ormning_core::pg::schema::field::FieldRef {
                        table_id: table_ref.0.clone(),
                        field_id: target_name,
                    };
                    sets.push(
                        (
                            field,
                            crate::convert::pg::convert_expr(
                                input,
                                &a.value,
                                used_params,
                                custom_types,
                                field_lookup,
                            ),
                        ),
                    );
                }
                Box::new(good_ormning_core::pg::query::update::Update {
                    alias: alias.clone(),
                    from: from_sources,
                    table: table_ref.clone(),
                    values: sets,
                    where_: selection
                        .as_ref()
                        .map(
                            |e| crate::convert::pg::convert_expr(input, e, used_params, custom_types, field_lookup),
                        ),
                    returning: crate::convert::pg::convert_returning(
                        input,
                        returning,
                        used_params,
                        custom_types,
                        field_lookup,
                        &[(alias.unwrap_or_else(|| table_ref.0.clone()), Some(table_ref.clone()))],
                    ),
                })
            },
            sqlparser::ast::Statement::Delete(delete) => Box::new(
                crate::convert::pg::convert_delete(input, delete, &mut used_params, custom_types, field_lookup),
            ),
            _ => panic!("Unsupported: Statement type: {:?}", statement),
        };
        for (ident, _) in &input.param_types {
            let s = ident.to_string();
            let s = s.trim_start_matches("r#");
            if !used_params.contains(s) {
                panic!("Parameter {} not used in query", ident);
            }
        }
        good_ormning_core::pg::Query {
            name: "unnamed".to_string(),
            body: body,
            res_count: good_ormning_core::QueryResCount::Many,
            res_name: None,
        }
    };
    query.res_count = res_count;
    let mut hasher = DefaultHasher::new();
    input.sql.hash(&mut hasher);
    let query_hash = hasher.finish();
    let query_name = format_ident!("good_query_{}", query_hash);
    query.name = query_name.to_string();
    let pascal_db_name: String = db_name.to_case(Case::Pascal);
    let db_mod = &input.db_mod;
    let db_type = if let Some(v) = input.version {
        let name = format_ident!("Db{}{}", pascal_db_name, v);
        quote!(#db_mod::#name < impl:: good_ormning:: runtime:: pg:: PgConnection >)
    } else {
        let name = format_ident!("Db{}{}", pascal_db_name, version_i);
        quote!(#db_mod::#name < impl:: good_ormning:: runtime:: pg:: PgConnection >)
    };
    let generated =
        good_ormning_core::pg::query::generate::generate_query_functions(
            &mut errs,
            field_lookup,
            vec![query],
            "inline",
            db_type,
            Some(
                &input
                    .param_types
                    .iter()
                    .map(|(ident, _)| ident.to_string().trim_start_matches("r#").to_string())
                    .collect::<Vec<_>>(),
            ),
            schema_file.default_time_library,
        );
    if let Err(e) = errs.raise() {
        let e = e.join("\n");
        return quote!(compile_error!(#e));
    }
    let conn = &input.conn;
    let args = &input.params;
    let db_name_lit = LitStr::new(&db_name, input.db_mod.span());
    let db_mod_str = input.db_mod.to_string();
    let _db_mod_lit = LitStr::new(&db_mod_str, input.db_mod.span());
    quote!{
        {
            const _:() = {
                if !:: good_ormning:: runtime:: utils:: str_eq(#db_mod:: DB_NAME, #db_name_lit) {
                    #[allow(unconditional_panic)]
                    let _ = ["Database name mismatch"][1];
                }
            };
            use ::good_ormning::runtime::GoodError;
            use ::good_ormning::runtime::ToGoodError;
            use ::good_ormning::runtime::pg::PgConnection;
            use ::good_ormning::runtime::pg::tokio_postgres;
            #(#generated) * #query_name(#conn, #(#args,) *)
        }
    }
}

fn parse_and_generate_sqlite(
    input: GoodQueryInput,
    res_count: good_ormning_core::QueryResCount,
) -> proc_macro2::TokenStream {
    let db_name = get_db_info("sqlite", input.db_name.clone());
    let dialect = sqlparser::dialect::SQLiteDialect {};
    let ast = match sqlparser::parser::Parser::parse_sql(&dialect, &input.sql) {
        Ok(ast) => ast,
        Err(e) => {
            let e = e.to_string();
            return quote!(compile_error!(#e));
        },
    };
    if ast.is_empty() {
        return quote!(compile_error!("Empty SQL statement"));
    }
    if ast.len() > 1 {
        return quote!(compile_error!("Queries must contain exactly one SQL statement"));
    }
    let statement = &ast[0];
    let mut errs = Errs::new();
    let out_dir = env::var("OUT_DIR").unwrap_or_else(|_| ".".to_string());
    let path =
        std::path::Path::new(&out_dir)
            .join("good_ormning")
            .join(good_ormning_core::utils::json_file_name("sqlite", &db_name));
    if !path.exists() {
        let e = format!("Schema file not found at {:?}. Did you run the build script?", path.to_string_lossy());
        return quote!(compile_error!(#e));
    }
    let versions_map: HashMap<usize, SqliteVersion> =
        match serde_json::from_str(&fs::read_to_string(&path).unwrap()) {
            Ok(m) => m,
            Err(e) => {
                let e = e.to_string();
                return quote!(compile_error!(#e));
            },
        };
    let mut field_lookup = HashMap::new();
    let version_i = input.version.unwrap_or_else(|| versions_map.keys().max().copied().unwrap_or(0));
    let version = match versions_map.get(&version_i) {
        Some(v) => v,
        None => {
            let e = format!("Version {} not found in schema for db {}", version_i, db_name);
            return quote!(compile_error!(#e));
        },
    };
    let custom_types = version.custom_types.clone();
    for (table_id, table) in &version.tables {
        let mut fields: HashMap<SqliteFieldRef, SqliteFieldInfo> = HashMap::new();
        for (field_id, field) in &table.fields {
            fields.insert(SqliteFieldRef {
                table_id: table_id.clone(),
                field_id: field_id.clone(),
            }, SqliteFieldInfo {
                sql_name: field.id.clone(),
                type_: field.type_.type_.clone(),
            });
        }
        field_lookup.insert(SqliteTableRef(table_id.clone()), SqliteTableInfo {
            sql_name: table.id.clone(),
            fields: fields,
        });
    }
    let mut query = {
        let input = &input;
        let custom_types = &custom_types;
        let field_lookup = &field_lookup;
        let mut used_params = HashSet::new();
        let body: Box<dyn good_ormning_core::sqlite::query::utils::QueryBody> = match statement {
            sqlparser::ast::Statement::Query(q) => {
                if let sqlparser::ast::SetExpr::Delete(sqlparser::ast::Statement::Delete(d)) = &*q.body {
                    let mut del =
                        crate::convert::sqlite::convert_delete(
                            input,
                            d,
                            &mut used_params,
                            custom_types,
                            field_lookup,
                        );
                    if let Some(with) = &q.with {
                        del.with =
                            Some(
                                crate::convert::sqlite::convert_with(
                                    input,
                                    with,
                                    &mut used_params,
                                    custom_types,
                                    field_lookup,
                                ),
                            );
                    }
                    Box::new(del)
                } else {
                    Box::new(
                        crate::convert::sqlite::convert_select_query(
                            input,
                            q,
                            &mut used_params,
                            custom_types,
                            field_lookup,
                        ),
                    )
                }
            },
            sqlparser::ast::Statement::Insert(insert) => {
                let used_params = &mut used_params;
                let table = crate::convert::sqlite::get_table_ref(match &insert.table {
                    sqlparser::ast::TableObject::TableName(n) => n,
                    _ => panic!("Unsupported insert table object: {:?}", insert.table),
                });
                let columns: Vec<good_ormning_core::sqlite::schema::field::FieldRef> =
                    insert.columns.iter().map(|c| good_ormning_core::sqlite::schema::field::FieldRef {
                        table_id: table.0.clone(),
                        field_id: c.value.clone(),
                    }).collect();
                let source = if let Some(q) = &insert.source {
                    match &*q.body {
                        sqlparser::ast::SetExpr::Values(v) => {
                            let mut rows = vec![];
                            for row in &v.rows {
                                if row.len() != columns.len() {
                                    panic!(
                                        "Insert row has {} values but {} columns were specified",
                                        row.len(),
                                        columns.len()
                                    );
                                }
                                rows.push(
                                    row
                                        .iter()
                                        .map(
                                            |expr| crate::convert::sqlite::convert_expr(
                                                input,
                                                expr,
                                                used_params,
                                                custom_types,
                                                field_lookup,
                                            ),
                                        )
                                        .collect::<Vec<_>>(),
                                );
                            }
                            if rows.len() == 1 {
                                good_ormning_core::sqlite::query::insert::InsertSource::Values(
                                    columns.iter().cloned().zip(rows.pop().unwrap().into_iter()).collect(),
                                )
                            } else {
                                good_ormning_core::sqlite::query::insert::InsertSource::Rows(
                                    good_ormning_core::sqlite::query::insert::InsertRows {
                                        columns: columns,
                                        rows: rows,
                                    },
                                )
                            }
                        },
                        _ => {
                            good_ormning_core::sqlite::query::insert::InsertSource::Select {
                                columns: columns,
                                select: crate::convert::sqlite::convert_select_query(
                                    input,
                                    q,
                                    used_params,
                                    custom_types,
                                    field_lookup,
                                ),
                            }
                        },
                    }
                } else {
                    good_ormning_core::sqlite::query::insert::InsertSource::Values(vec![])
                };
                let on_conflict = if let Some(on) = &insert.on {
                    match on {
                        sqlparser::ast::OnInsert::OnConflict(oc) => {
                            let conflict = match &oc.conflict_target {
                                Some(sqlparser::ast::ConflictTarget::Columns(idents)) => idents
                                    .iter()
                                    .map(|id| good_ormning_core::sqlite::schema::field::FieldRef {
                                        table_id: table.0.clone(),
                                        field_id: id.value.clone(),
                                    })
                                    .collect(),
                                Some(sqlparser::ast::ConflictTarget::OnConstraint(c)) => panic!(
                                    "`on conflict on constraint` is not supported, use a list of columns instead: {}",
                                    c
                                ),
                                None => vec![],
                            };
                            match &oc.action {
                                sqlparser::ast::OnConflictAction::DoNothing => {
                                    if conflict.is_empty() {
                                        Some(good_ormning_core::sqlite::query::insert::InsertConflict::DoNothing)
                                    } else {
                                        Some(
                                            good_ormning_core::sqlite::query::insert::InsertConflict::DoNothingOn {
                                                conflict: conflict,
                                            },
                                        )
                                    }
                                },
                                sqlparser::ast::OnConflictAction::DoUpdate(du) => {
                                    if let Some(sel) = &du.selection {
                                        panic!("`on conflict do update ... where` is not supported: {}", sel);
                                    }
                                    let mut updates = vec![];
                                    for a in &du.assignments {
                                        let target_name = match &a.target {
                                            sqlparser::ast::AssignmentTarget::ColumnName(name) => match name
                                                .0
                                                .last()
                                                .unwrap() {
                                                sqlparser::ast::ObjectNamePart::Identifier(i) => i.value.clone(),
                                            },
                                            _ => panic!("Unsupported assignment target: {:?}", a.target),
                                        };
                                        let field = good_ormning_core::sqlite::schema::field::FieldRef {
                                            table_id: table.0.clone(),
                                            field_id: target_name,
                                        };
                                        updates.push(
                                            (
                                                field,
                                                crate::convert::sqlite::convert_expr(
                                                    input,
                                                    &a.value,
                                                    used_params,
                                                    custom_types,
                                                    field_lookup,
                                                ),
                                            ),
                                        );
                                    }
                                    Some(good_ormning_core::sqlite::query::insert::InsertConflict::DoUpdate {
                                        conflict: conflict,
                                        set: updates,
                                    })
                                },
                            }
                        },
                        sqlparser::ast::OnInsert::DuplicateKeyUpdate(_) => panic!(
                            "`on duplicate key update` is not supported"
                        ),
                        _ => panic!("Unsupported `on` clause in insert: {:?}", on),
                    }
                } else {
                    match insert.or {
                        None => None,
                        Some(sqlparser::ast::SqliteOnConflict::Ignore) => Some(
                            good_ormning_core::sqlite::query::insert::InsertConflict::OrIgnore,
                        ),
                        Some(sqlparser::ast::SqliteOnConflict::Replace) => Some(
                            good_ormning_core::sqlite::query::insert::InsertConflict::OrReplace,
                        ),
                        Some(sqlparser::ast::SqliteOnConflict::Abort) => Some(
                            good_ormning_core::sqlite::query::insert::InsertConflict::OrAbort,
                        ),
                        Some(sqlparser::ast::SqliteOnConflict::Fail) => Some(
                            good_ormning_core::sqlite::query::insert::InsertConflict::OrFail,
                        ),
                        Some(sqlparser::ast::SqliteOnConflict::Rollback) => Some(
                            good_ormning_core::sqlite::query::insert::InsertConflict::OrRollback,
                        ),
                    }
                };
                Box::new(good_ormning_core::sqlite::query::insert::Insert {
                    table: table.clone(),
                    source: source,
                    on_conflict: on_conflict,
                    returning: crate::convert::sqlite::convert_returning(
                        input,
                        &insert.returning,
                        used_params,
                        custom_types,
                        field_lookup,
                        &[(table.0.clone(), Some(table.clone()))],
                    ),
                })
            },
            sqlparser::ast::Statement::Update { table, assignments, from, selection, returning, or } => {
                if let Some(or) = or {
                    panic!("`update or {}` is not supported", or);
                }
                let used_params = &mut used_params;
                if !table.joins.is_empty() {
                    panic!("Joins on the updated table aren't supported, use `update ... from`");
                }
                let (table_ref, alias) = match &table.relation {
                    sqlparser::ast::TableFactor::Table { name, alias, .. } => {
                        (
                            crate::convert::sqlite::get_table_ref(name),
                            alias.as_ref().map(|a| a.name.value.clone()),
                        )
                    },
                    _ => panic!("Unsupported: Update table factor: {:?}", table.relation),
                };
                let mut from_sources = vec![];
                if let Some(from) = from {
                    let tables = match from {
                        sqlparser::ast::UpdateTableFromKind::BeforeSet(t) |
                        sqlparser::ast::UpdateTableFromKind::AfterSet(t) => t,
                    };
                    for t in tables {
                        if !t.joins.is_empty() {
                            panic!(
                                "Joins in `update ... from` aren't supported, list the tables separated by commas instead"
                            );
                        }
                        from_sources.push(
                            crate::convert::sqlite::convert_table_factor_sqlite(
                                input,
                                &t.relation,
                                used_params,
                                custom_types,
                                field_lookup,
                            ),
                        );
                    }
                }
                let mut values = vec![];
                for a in assignments {
                    let target_name = match &a.target {
                        sqlparser::ast::AssignmentTarget::ColumnName(name) => match name.0.last().unwrap() {
                            sqlparser::ast::ObjectNamePart::Identifier(i) => i.value.clone(),
                        },
                        _ => panic!("Unsupported: AssignmentTarget: {:?}", a.target),
                    };
                    let field = good_ormning_core::sqlite::schema::field::FieldRef {
                        table_id: table_ref.0.clone(),
                        field_id: target_name,
                    };
                    values.push(
                        (
                            field,
                            crate::convert::sqlite::convert_expr(
                                input,
                                &a.value,
                                used_params,
                                custom_types,
                                field_lookup,
                            ),
                        ),
                    );
                }
                Box::new(good_ormning_core::sqlite::query::update::Update {
                    alias: alias.clone(),
                    from: from_sources,
                    table: table_ref.clone(),
                    values: values,
                    where_: selection
                        .as_ref()
                        .map(
                            |e| crate::convert::sqlite::convert_expr(
                                input,
                                e,
                                used_params,
                                custom_types,
                                field_lookup,
                            ),
                        ),
                    returning: crate::convert::sqlite::convert_returning(
                        input,
                        returning,
                        used_params,
                        custom_types,
                        field_lookup,
                        &[(alias.unwrap_or_else(|| table_ref.0.clone()), Some(table_ref.clone()))],
                    ),
                    index_hint: None,
                })
            },
            sqlparser::ast::Statement::Delete(delete) => Box::new(
                crate::convert::sqlite::convert_delete(input, delete, &mut used_params, custom_types, field_lookup),
            ),
            _ => panic!("Unsupported: Statement type: {:?}", statement),
        };
        for (ident, _) in &input.param_types {
            let s = ident.to_string();
            let s = s.trim_start_matches("r#");
            if !used_params.contains(s) {
                panic!("Parameter {} not used in query", ident);
            }
        }
        good_ormning_core::sqlite::Query {
            name: "unnamed".to_string(),
            body: body,
            res_count: good_ormning_core::QueryResCount::Many,
            res_name: None,
        }
    };
    query.res_count = res_count;
    let mut hasher = DefaultHasher::new();
    input.sql.hash(&mut hasher);
    let query_hash = hasher.finish();
    let query_name = format_ident!("good_query_{}", query_hash);
    query.name = query_name.to_string();
    let pascal_db_name: String = db_name.to_case(Case::Pascal);
    let db_mod = &input.db_mod;
    let db_type = if let Some(v) = input.version {
        let name = format_ident!("Db{}{}", pascal_db_name, v);
        quote!(#db_mod::#name < impl:: good_ormning:: runtime:: sqlite:: SqliteConnection >)
    } else {
        let name = format_ident!("Db{}{}", pascal_db_name, version_i);
        quote!(#db_mod::#name < impl:: good_ormning:: runtime:: sqlite:: SqliteConnection >)
    };
    let generated =
        good_ormning_core::sqlite::query::generate::generate_query_functions(
            &mut errs,
            field_lookup,
            vec![query],
            "inline",
            db_type,
            Some(
                &input
                    .param_types
                    .iter()
                    .map(|(ident, _)| ident.to_string().trim_start_matches("r#").to_string())
                    .collect::<Vec<_>>(),
            ),
        );
    if let Err(e) = errs.raise() {
        let e = e.join("\n");
        return quote!(compile_error!(#e));
    }
    let conn = &input.conn;
    let args = &input.params;
    let db_name_lit = LitStr::new(&db_name, input.db_mod.span());
    let db_mod_str = input.db_mod.to_string();
    let _db_mod_lit = LitStr::new(&db_mod_str, input.db_mod.span());
    quote!{
        {
            const _:() = {
                if !:: good_ormning:: runtime:: utils:: str_eq(#db_mod:: DB_NAME, #db_name_lit) {
                    #[allow(unconditional_panic)]
                    let _ = ["Database name mismatch"][1];
                }
            };
            use ::good_ormning::runtime::GoodError;
            use ::good_ormning::runtime::ToGoodError;
            use ::good_ormning::runtime::sqlite::SqliteConnection;
            use ::good_ormning::runtime::sqlite::rusqlite;
            #(#generated) * #query_name(#conn, #(#args,) *)
        }
    }
}
