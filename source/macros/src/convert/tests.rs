use {
    crate::GoodQueryInput,
    sqlparser::{
        ast as sql,
        dialect::{
            Dialect,
            MySqlDialect,
            PostgreSqlDialect,
            SQLiteDialect,
        },
        parser::Parser,
    },
    std::collections::{
        BTreeMap,
        HashMap,
        HashSet,
    },
};

fn input(sql: &str) -> GoodQueryInput {
    return GoodQueryInput {
        conn: syn::parse_quote!(conn),
        db_mod: syn::parse_quote!(db),
        db_name: "db".to_string(),
        inline_params: vec![],
        param_types: vec![],
        params: vec![],
        sql: sql.to_string(),
        version: None,
    };
}

fn parse(dialect: &dyn Dialect, sql: &str) -> sql::Statement {
    let mut ast = Parser::parse_sql(dialect, sql).unwrap_or_else(|e| panic!("Failed to parse {}: {}", sql, e));
    assert_eq!(ast.len(), 1);
    return ast.pop().unwrap();
}

fn panic_message(f: impl FnOnce()) -> Option<String> {
    let err = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).err()?;
    return Some(
        err
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| err.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default(),
    );
}

fn assert_rejected(sql: &str, expected: &str, f: impl FnOnce()) {
    match panic_message(f) {
        Some(m) => assert!(m.contains(expected), "{}: expected panic containing {:?}, got {:?}", sql, expected, m),
        None => panic!("{}: was accepted, expected panic containing {:?}", sql, expected),
    }
}

fn pg(sql: &str, expected: &str) {
    let stmt = parse(&PostgreSqlDialect {}, sql);
    let input = input(sql);
    assert_rejected(sql, expected, || match &stmt {
        sql::Statement::Query(q) => {
            crate::convert::pg::convert_select_query(
                &input,
                q,
                &mut HashSet::new(),
                &BTreeMap::new(),
                &HashMap::new(),
            );
        },
        sql::Statement::Delete(d) => {
            crate::convert::pg::convert_delete(&input, d, &mut HashSet::new(), &BTreeMap::new(), &HashMap::new());
        },
        _ => panic!("Unexpected statement in test: {}", stmt),
    });
}

fn sqlite(sql: &str, expected: &str) {
    let stmt = parse(&SQLiteDialect {}, sql);
    let input = input(sql);
    assert_rejected(sql, expected, || match &stmt {
        sql::Statement::Query(q) => {
            crate::convert::sqlite::convert_select_query(
                &input,
                q,
                &mut HashSet::new(),
                &BTreeMap::new(),
                &HashMap::new(),
            );
        },
        sql::Statement::Delete(d) => {
            crate::convert::sqlite::convert_delete(
                &input,
                d,
                &mut HashSet::new(),
                &BTreeMap::new(),
                &HashMap::new(),
            );
        },
        _ => panic!("Unexpected statement in test: {}", stmt),
    });
}

fn mariadb(sql: &str, expected: &str) {
    let stmt = parse(&MySqlDialect {}, sql);
    let input = input(sql);
    assert_rejected(sql, expected, || match &stmt {
        sql::Statement::Query(q) => {
            crate::convert::mariadb::convert_select_query(&input, q, &mut crate::convert::mariadb::ConvertState {
                param_locations: HashMap::new(),
                used_params: HashSet::new(),
            }, &BTreeMap::new(), &HashMap::new());
        },
        _ => panic!("Unexpected statement in test: {}", stmt),
    });
}

#[test]
fn test_pg_rejects_unsupported() {
    pg("select x from t fetch first 10 percent rows only", "doesn't support `fetch ... percent`");
    pg("select x from t for update of a.b", "Locked tables must be unqualified");
    pg("select x into y from t", "Unsupported select syntax for PostgreSQL");
    pg("select sum(x order by x) over () from t", "`order by` inside the arguments of window function");
    pg("select lower(x order by x) from t", "only supported in aggregate functions");
    pg("select percentile_cont(0.5) within group (order by x) from t", "Unsupported function syntax");
    pg("select try_cast(x as int) from t", "Only `cast(x as type)`");
    pg("select x from t as a(b)", "Column aliases are only supported on table functions");
    pg("select x from (select 1) as s(a)", "Column aliases are only supported on table functions");
    pg("select x from public.generate_series(1, 2)", "Table functions must be unqualified names");
    pg("select x from t with ordinality", "Unsupported table syntax");
    pg("select x from t tablesample bernoulli (10)", "Unsupported table syntax");
    pg("delete from t using u where t.x = u.x limit 1", "Unsupported delete syntax for PostgreSQL");
}

#[test]
fn test_sqlite_rejects_unsupported() {
    sqlite("select lower(x order by x) from t", "only supported in aggregate functions");
    sqlite("delete from t where x = 1 limit 1", "Unsupported delete syntax for SQLite");
    sqlite("delete from t where x = 1 order by x", "Unsupported delete syntax for SQLite");
}

#[test]
fn test_mariadb_rejects_unsupported() {
    mariadb("select high_priority x from t", "Unsupported select syntax for MariaDB");
    mariadb("select straight_join x from t", "Unsupported select syntax for MariaDB");
    mariadb("select x from t cross join u on 1", "Join operator is not supported");
    mariadb("select x from t where x like 'a' escape '!' and x like any ('a')", "`like any` is not supported");
}
