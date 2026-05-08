use {
    convert_case::{
        Case,
        Casing,
    },
    good_ormning_core::{
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
        },
        parse_macro_input,
    },
};

mod convert;

struct ParamType {
    arr: bool,
    opt: bool,
    base: String,
}

struct GoodQueryInput {
    db_mod: Ident,
    version: Option<usize>,
    db_name: String,
    sql: String,
    param_types: Vec<(Ident, ParamType)>,
    conn: syn::Expr,
    params: Vec<syn::Expr>,
}

impl Parse for GoodQueryInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let db_mod: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let (db_name, version, sql) = {
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
        let mut last_end = 0;
        let bytes = sql.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'$' {
                if i + 1 < bytes.len() && bytes[i + 1] == b'{' {
                    final_sql.push_str(&sql[last_end .. i]);
                    i += 2;
                    let content_start = i;
                    while i < bytes.len() && bytes[i] != b'}' {
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
                        syn::Error::new(input.span(), "Invalid inline parameter format. Expected ${type = value}")
                    })?;
                    let (param_idx, name, pt, val) = parse_inline_param(input, type_str, val_str, params.len())?;
                    params.push(val);
                    param_types.push((name, pt));
                    final_sql.push_str(&format!("${}", param_idx));
                    last_end = i;
                    continue;
                }
            }
            i += 1;
        }
        final_sql.push_str(&sql[last_end..]);
        Ok(GoodQueryInput {
            db_mod: db_mod,
            version: version,
            db_name: db_name,
            sql: final_sql,
            param_types: param_types,
            conn: conn,
            params: params,
        })
    }
}

fn parse_inline_param(
    input: ParseStream,
    type_str: &str,
    val_str: &str,
    current_params_len: usize,
) -> syn::Result<(usize, Ident, ParamType, syn::Expr)> {
    let val: syn::Expr = syn::parse_str(val_str).map_err(|e| {
        syn::Error::new(input.span(), format!("Failed to parse inline parameter value: {}", e))
    })?;
    let type_tokens: proc_macro2::TokenStream = type_str.parse().map_err(|e| {
        syn::Error::new(input.span(), format!("Failed to parse inline parameter type tokens: {}", e))
    })?;

    use syn::parse::Parser;

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
        Ok((arr, opt, base))
    }).parse2(type_tokens).map_err(|e| {
        syn::Error::new(input.span(), format!("Failed to parse inline parameter type: {}", e))
    })?;
    if base_p.is_empty() {
        return Err(input.error("Expected base type in inline parameter"));
    }
    let param_idx = current_params_len + 1;
    let name = format_ident!("p{}", param_idx);
    Ok((param_idx, name, ParamType {
        arr: arr_p,
        opt: opt_p,
        base: base_p,
    }, val))
}

fn get_db_info(_engine: &str, provided_db_name: String) -> String {
    provided_db_name
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
    let statement = &ast[0];
    let mut errs = Errs::new();
    let out_dir = env::var("OUT_DIR").unwrap_or_else(|_| ".".to_string());
    let path =
        std::path::Path::new(&out_dir)
            .join("good_ormning")
            .join(good_ormning_core::utils::json_file_name(&db_name));
    if !path.exists() {
        let e = format!("Schema file not found at {:?}. Did you run the build script?", path.to_string_lossy());
        return quote!(compile_error!(#e));
    }
    let versions_map: HashMap<usize, PgVersion> = match serde_json::from_str(&fs::read_to_string(&path).unwrap()) {
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
    let mut query = crate::convert::pg::convert_query(&input, statement, &custom_types, &field_lookup);
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
        );
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
    let statement = &ast[0];
    let mut errs = Errs::new();
    let out_dir = env::var("OUT_DIR").unwrap_or_else(|_| ".".to_string());
    let path =
        std::path::Path::new(&out_dir)
            .join("good_ormning")
            .join(good_ormning_core::utils::json_file_name(&db_name));
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
    let mut query = crate::convert::sqlite::convert_query(&input, statement, &custom_types, &field_lookup);
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
        );
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
            #(#generated) * #query_name(#conn, #(#args,) *)
        }
    }
}

/// See the `good_query` macro help in the readme.
#[proc_macro]
pub fn good_query_pg(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as GoodQueryInput);
    parse_and_generate_pg(input, good_ormning_core::QueryResCount::None).into()
}

/// See the `good_query` macro help in the readme.
#[proc_macro]
pub fn good_query_one_pg(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as GoodQueryInput);
    parse_and_generate_pg(input, good_ormning_core::QueryResCount::One).into()
}

/// See the `good_query` macro help in the readme.
#[proc_macro]
pub fn good_query_opt_pg(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as GoodQueryInput);
    parse_and_generate_pg(input, good_ormning_core::QueryResCount::MaybeOne).into()
}

/// See the `good_query` macro help in the readme.
#[proc_macro]
pub fn good_query_many_pg(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as GoodQueryInput);
    parse_and_generate_pg(input, good_ormning_core::QueryResCount::Many).into()
}

/// See the `good_query` macro help in the readme.
#[proc_macro]
pub fn good_query_sqlite(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as GoodQueryInput);
    parse_and_generate_sqlite(input, good_ormning_core::QueryResCount::None).into()
}

/// See the `good_query` macro help in the readme.
#[proc_macro]
pub fn good_query_one_sqlite(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as GoodQueryInput);
    parse_and_generate_sqlite(input, good_ormning_core::QueryResCount::One).into()
}

/// See the `good_query` macro help in the readme.
#[proc_macro]
pub fn good_query_opt_sqlite(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as GoodQueryInput);
    parse_and_generate_sqlite(input, good_ormning_core::QueryResCount::MaybeOne).into()
}

/// See the `good_query` macro help in the readme.
#[proc_macro]
pub fn good_query_many_sqlite(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as GoodQueryInput);
    parse_and_generate_sqlite(input, good_ormning_core::QueryResCount::Many).into()
}
