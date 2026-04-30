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
    version: Option<usize>,
    db_name: String,
    sql: String,
    param_types: Vec<(Ident, ParamType)>,
    conn: syn::Expr,
    params: Vec<syn::Expr>,
}

impl Parse for GoodQueryInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut literals = Vec::new();
        while !input.peek(Token![;]) && !input.is_empty() {
            literals.push(input.parse::<LitStr>()?);
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            } else if !input.peek(Token![;]) {
                return Err(input.error("Expected , or ; after literal"));
            }
        }
        input.parse::<Token![;]>()?;
        let (db_name, version, sql) = match literals.len() {
            1 => ("".to_string(), None, literals[0].value()),
            2 => (literals[0].value(), None, literals[1].value()),
            3 => {
                let v_str = literals[1].value();
                let v = v_str.parse::<usize>().map_err(|_| {
                    syn::Error::new(literals[1].span(), "Version must be a positive integer")
                })?;
                (literals[0].value(), Some(v), literals[2].value())
            },
            _ => {
                return Err(
                    input.error(
                        "Expected 1, 2, or 3 literals before semicolon (sql, [db_name, sql], or [db_name, version, sql])",
                    ),
                );
            },
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
        Ok(GoodQueryInput {
            version: version,
            db_name: db_name,
            sql: sql,
            param_types: param_types,
            conn: conn,
            params: params,
        })
    }
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
    let db_type = if let Some(v) = input.version {
        let name = format_ident!("Db{}{}", pascal_db_name, v);
        quote!(dbm::#name <'_ >)
    } else {
        let name = format_ident!("Db{}{}", pascal_db_name, version_i);
        quote!(dbm::#name <'_ >)
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
    quote!{
        {
            use ::good_ormning::runtime::GoodError;
            use ::good_ormning::runtime::ToGoodError;
            use ::good_ormning::runtime::pg::PgConnection;
            #(#generated) * #query_name(& mut #conn, #(#args,) *)
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
    let db_type = if let Some(v) = input.version {
        let name = format_ident!("Db{}{}", pascal_db_name, v);
        quote!(dbm::#name <'_, impl:: good_ormning:: runtime:: sqlite:: SqliteConnection >)
    } else {
        let name = format_ident!("Db{}{}", pascal_db_name, version_i);
        quote!(dbm::#name <'_, impl:: good_ormning:: runtime:: sqlite:: SqliteConnection >)
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
    quote!{
        {
            use ::good_ormning::runtime::GoodError;
            use ::good_ormning::runtime::ToGoodError;
            use ::good_ormning::runtime::sqlite::SqliteConnection;
            #(#generated) * #query_name(& mut #conn, #(#args,) *)
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
