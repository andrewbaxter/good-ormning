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
    Token,
    Ident,
    LitStr,
};
use sqlparser::{
    dialect::GenericDialect,
    parser::Parser,
};
use good_ormning::utils::Errs;

mod convert;

struct GoodQueryInput {
    sql: String,
    params: Vec<(Ident, syn::Type)>,
    conn: syn::Expr,
    args: Vec<syn::Expr>,
}

impl Parse for GoodQueryInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let sql_lit: LitStr = input.parse()?;
        let sql = sql_lit.value();
        let mut params = Vec::new();
        let mut conn: Option<syn::Expr> = None;
        let mut args = Vec::new();
        if input.peek(Token![;]) {
            input.parse::<Token![;]>()?;
            while !input.is_empty() && !input.peek(Token![;]) {
                let name: Ident = input.parse()?;
                input.parse::<Token![=]>()?;
                let ty: syn::Type = input.parse()?;
                params.push((name, ty));
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
                    if input.peek(Token![,]) {
                        input.parse::<Token![,]>()?;
                    }
                }
            }
        }
        Ok(GoodQueryInput {
            sql,
            params,
            conn: conn.unwrap_or_else(|| syn::parse_str("db").unwrap()),
            args,
        })
    }
}

fn parse_and_generate_pg(input: GoodQueryInput, res_count: good_ormning::QueryResCount) -> proc_macro2::TokenStream {
    let dialect = sqlparser::dialect::PostgreSqlDialect {};
    let ast = Parser::parse_sql(&dialect, &input.sql).unwrap();
    let statement = &ast[0];
    let mut errs = Errs::new();
    let mut query = crate::convert::pg::convert_query(&input, statement);
    query.res_count = res_count;
    query.name = "good_query_inline".to_string();
    let out_dir = env::var("OUT_DIR").unwrap_or_else(|_| ".".to_string());
    let path = std::path::Path::new(&out_dir).join("good_ormning_pg_versions.json");
    let versions_map: HashMap<usize, PgVersion> = if path.exists() {
        serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap_or_default()
    } else {
        HashMap::new()
    };
    let mut field_lookup = HashMap::new();
    if let Some(v) = versions_map.get(&0) {
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
            field_lookup.insert(
                good_ormning::pg::schema::table::TableRef(table_id.clone()),
                good_ormning::pg::query::utils::PgTableInfo {
                    sql_name: table.id.clone(),
                    fields,
                },
            );
        }
    }
    let generated =
        good_ormning::pg::query::generate::generate_query_functions(&mut errs, field_lookup, vec![query], "inline");
    let conn = &input.conn;
    let args = &input.args;
    quote!{
        async {
            #(#generated) * let res: Result < _,
            good_ormning:: runtime:: GoodError > = good_query_inline(#conn, #(#args), *).await;
            res
        }
    }
}

fn parse_and_generate_sqlite(
    input: GoodQueryInput,
    res_count: good_ormning::QueryResCount,
) -> proc_macro2::TokenStream {
    let dialect = sqlparser::dialect::SQLiteDialect {};
    let ast = Parser::parse_sql(&dialect, &input.sql).unwrap();
    let statement = &ast[0];
    let mut errs = Errs::new();
    let mut query = crate::convert::sqlite::convert_query(&input, statement);
    query.res_count = res_count;
    query.name = "good_query_inline".to_string();
    let out_dir = env::var("OUT_DIR").unwrap_or_else(|_| ".".to_string());
    let path = std::path::Path::new(&out_dir).join("good_ormning_sqlite_versions.json");
    let versions_map: HashMap<usize, SqliteVersion> = if path.exists() {
        serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap_or_default()
    } else {
        HashMap::new()
    };
    let mut field_lookup = HashMap::new();
    if let Some(v) = versions_map.get(&0) {
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
            field_lookup.insert(
                good_ormning::sqlite::schema::table::TableRef(table_id.clone()),
                good_ormning::sqlite::query::utils::SqliteTableInfo {
                    sql_name: table.id.clone(),
                    fields,
                },
            );
        }
    }
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
            #(#generated) * let res: Result < _,
            good_ormning:: runtime:: GoodError > = good_query_inline(#conn, #(#args), *);
            res
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

#[proc_macro]
pub fn good_query(input: TokenStream) -> TokenStream {
    good_query_pg(input)
}

#[proc_macro]
pub fn good_query_one(input: TokenStream) -> TokenStream {
    good_query_one_pg(input)
}

#[proc_macro]
pub fn good_query_opt(input: TokenStream) -> TokenStream {
    good_query_opt_pg(input)
}

#[proc_macro]
pub fn good_query_many(input: TokenStream) -> TokenStream {
    good_query_many_pg(input)
}
