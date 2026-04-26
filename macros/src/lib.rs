use std::env;
use std::fs;
use std::collections::HashMap;
use good_ormning::pg::Version as PgVersion;
use good_ormning::sqlite::Version as SqliteVersion;
use proc_macro::TokenStream;
use quote::{
    quote,
    format_ident,
};
use syn::{
    parse_macro_input,
    parse::{
        Parse,
        ParseStream,
    },
    Ident,
    Token,
    LitStr,
};
use good_ormning::utils::Errs;

mod convert;

struct ParamType {
    arr: bool,
    opt: bool,
    base: String,
}

struct GoodQueryInput {
    db_name: Option<String>,
    sql: String,
    params: Vec<(Ident, ParamType)>,
    conn: syn::Expr,
    args: Vec<syn::Expr>,
}

impl Parse for GoodQueryInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let first: syn::Lit = input.parse()?;
        let (db_name, sql) = match first {
            syn::Lit::Str(s) => {
                if input.peek(Token![,]) {
                    input.parse::<Token![,]>()?;
                    let sql_lit: LitStr = input.parse()?;
                    (Some(s.value()), sql_lit.value())
                } else {
                    (None, s.value())
                }
            },
            _ => return Err(input.error("Expected database name or SQL string")),
        };
        let mut params = Vec::new();
        let mut conn: Option<syn::Expr> = None;
        let mut args = Vec::new();
        if input.peek(Token![;]) {
            input.parse::<Token![;]>()?;
            while !input.is_empty() && !input.peek(Token![;]) {
                let name: Ident = input.parse()?;
                input.parse::<Token![=]>()?;
                let mut arr = false;
                let mut opt = false;
                while input.peek(Ident) {
                    let id: Ident = input.parse()?;
                    if id == "arr" {
                        arr = true;
                    } else if id == "opt" {
                        opt = true;
                    } else {
                        params.push((name, ParamType {
                            arr,
                            opt,
                            base: id.to_string(),
                        }));
                        break;
                    }
                }
                if input.peek(Token![,]) {
                    input.parse::<Token![,]>()?;
                }
            }
            if input.peek(Token![;]) {
                input.parse::<Token![;]>()?;
                if !input.is_empty() {
                    conn = Some(input.parse()?);
                    if input.peek(Token![,]) {
                        input.parse::<Token![,]>()?;
                    }
                }
                while !input.is_empty() {
                    let arg: syn::Expr = input.parse()?;
                    args.push(arg);
                    if input.peek(Token![,] ) {
                        input.parse::<Token![,]>()?;
                    }
                }
            }
        }
        Ok(GoodQueryInput {
            db_name,
            sql,
            params,
            conn: conn.unwrap_or_else(|| syn::parse_str("db").unwrap()),
            args,
        })
    }
}

fn get_db_info(engine: &str, provided_db_name: Option<String>) -> Result<String, proc_macro2::TokenStream> {
    let out_dir = env::var("OUT_DIR").unwrap_or_else(|_| ".".to_string());
    let json_dir = std::path::Path::new(&out_dir).join("good_ormning");
    if let Some(name) = provided_db_name {
        return Ok(name);
    }
    let mut dbs = Vec::new();
    if let Ok(entries) = fs::read_dir(&json_dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.starts_with(&format!("{}_", engine)) && name.ends_with(".json") {
                    let db = &name[engine.len() + 1..name.len() - 5];
                    dbs.push(db.to_string());
                }
            }
        }
    }
    if dbs.is_empty() {
        return Ok("default".to_string());
    }
    if dbs.len() == 1 {
        return Ok(dbs.pop().unwrap());
    }
    Err(
        quote!(
            compile_error!(
                "Multiple databases found. Please specify the database name as the first argument to the macro."
            )
        )
    )
}

fn parse_and_generate_pg(
    input: GoodQueryInput,
    res_count: good_ormning::QueryResCount,
) -> proc_macro2::TokenStream {
    let db_name = match get_db_info("pg", input.db_name.clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let dialect = sqlparser::dialect::PostgreSqlDialect {};
    let ast = sqlparser::parser::Parser::parse_sql(&dialect, &input.sql).unwrap();
    let statement = &ast[0];
    let mut errs = Errs::new();
    let out_dir = env::var("OUT_DIR").unwrap_or_else(|_| ".".to_string());
    let path = std::path::Path::new(&out_dir).join("good_ormning").join(format!("pg_{}.json", db_name));
    let versions_map: HashMap<usize, PgVersion> = if path.exists() {
        serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap_or_default()
    } else {
        HashMap::new()
    };
    let mut field_lookup = HashMap::new();
    let mut custom_types = std::collections::BTreeMap::new();
    let latest_version = versions_map.keys().max().and_then(|k| versions_map.get(k));
    if let Some(v) = latest_version {
        custom_types = v.custom_types.clone();
        for (table_id, table) in &v.tables {
            let mut fields = HashMap::new();
            for (field_id, field) in &table.fields {
                fields.insert(good_ormning::pg::schema::field::FieldRef {
                    table_id: table_id.clone(),
                    field_id: field_id.clone(),
                }, good_ormning::pg::query::utils::PgFieldInfo {
                    sql_name: field.id.clone(),
                    type_: field.type_.type_.clone(),
                });
            }
            field_lookup.insert(good_ormning::pg::schema::table::TableRef(table_id.clone()), good_ormning::pg::query::utils::PgTableInfo {
                sql_name: table.id.clone(),
                fields: fields,
            });
        }
    }
    let mut query = crate::convert::pg::convert_query(&input, statement, &custom_types);
    query.res_count = res_count;

    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    input.sql.hash(&mut hasher);
    let query_hash = hasher.finish();
    let query_name = format_ident!("good_query_{}", query_hash);
    query.name = query_name.to_string();

    let generated =
        good_ormning::pg::query::generate::generate_query_functions(&mut errs, field_lookup, vec![query], "inline");
    let conn = &input.conn;
    let args = &input.args;
    quote!{
        {
            use ::good_ormning::runtime::GoodError;
            #(#generated) *
            #query_name(#conn, #(#args), *)
        }
    }
}

fn parse_and_generate_sqlite(
    input: GoodQueryInput,
    res_count: good_ormning::QueryResCount,
) -> proc_macro2::TokenStream {
    let db_name = match get_db_info("sqlite", input.db_name.clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let dialect = sqlparser::dialect::SQLiteDialect {};
    let ast = sqlparser::parser::Parser::parse_sql(&dialect, &input.sql).unwrap();
    let statement = &ast[0];
    let mut errs = Errs::new();
    let out_dir = env::var("OUT_DIR").unwrap_or_else(|_| ".".to_string());
    let path = std::path::Path::new(&out_dir).join("good_ormning").join(format!("sqlite_{}.json", db_name));
    let versions_map: HashMap<usize, SqliteVersion> = if path.exists() {
        serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap_or_default()
    } else {
        HashMap::new()
    };
    let mut field_lookup = HashMap::new();
    let mut custom_types = std::collections::BTreeMap::new();
    let latest_version = versions_map.keys().max().and_then(|k| versions_map.get(k));
    if let Some(v) = latest_version {
        custom_types = v.custom_types.clone();
        for (table_id, table) in &v.tables {
            let mut fields = HashMap::new();
            for (field_id, field) in &table.fields {
                fields.insert(good_ormning::sqlite::schema::field::FieldRef {
                    table_id: table_id.clone(),
                    field_id: field_id.clone(),
                }, good_ormning::sqlite::query::utils::SqliteFieldInfo {
                    sql_name: field.id.clone(),
                    type_: field.type_.type_.clone(),
                });
            }
            field_lookup.insert(good_ormning::sqlite::schema::table::TableRef(table_id.clone()), good_ormning::sqlite::query::utils::SqliteTableInfo {
                sql_name: table.id.clone(),
                fields: fields,
            });
        }
    }
    let mut query = crate::convert::sqlite::convert_query(&input, statement, &custom_types);
    query.res_count = res_count;

    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    input.sql.hash(&mut hasher);
    let query_hash = hasher.finish();
    let query_name = format_ident!("good_query_{}", query_hash);
    query.name = query_name.to_string();

    let generated =
        good_ormning::sqlite::query::generate::generate_query_functions(
            &mut errs,
            field_lookup,
            vec![query],
            "inline",
        );
    let conn = &input.conn;
    let args = &input.args;
    quote!{
        {
            use ::good_ormning::runtime::GoodError;
            #(#generated) *
            #query_name(#conn, #(#args), *)
        }
    }
}

#[proc_macro]
pub fn good_query_pg(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as GoodQueryInput);
    parse_and_generate_pg(input, good_ormning::QueryResCount::None).into()
}

#[proc_macro]
pub fn good_query_one_pg(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as GoodQueryInput);
    parse_and_generate_pg(input, good_ormning::QueryResCount::One).into()
}

#[proc_macro]
pub fn good_query_opt_pg(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as GoodQueryInput);
    parse_and_generate_pg(input, good_ormning::QueryResCount::MaybeOne).into()
}

#[proc_macro]
pub fn good_query_many_pg(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as GoodQueryInput);
    parse_and_generate_pg(input, good_ormning::QueryResCount::Many).into()
}

#[proc_macro]
pub fn good_query_sqlite(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as GoodQueryInput);
    parse_and_generate_sqlite(input, good_ormning::QueryResCount::None).into()
}

#[proc_macro]
pub fn good_query_one_sqlite(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as GoodQueryInput);
    parse_and_generate_sqlite(input, good_ormning::QueryResCount::One).into()
}

#[proc_macro]
pub fn good_query_opt_sqlite(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as GoodQueryInput);
    parse_and_generate_sqlite(input, good_ormning::QueryResCount::MaybeOne).into()
}

#[proc_macro]
pub fn good_query_many_sqlite(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as GoodQueryInput);
    parse_and_generate_sqlite(input, good_ormning::QueryResCount::Many).into()
}
