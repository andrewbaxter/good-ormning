const EXPECTED_ERROR: &str = "Unsupported table-valued function: generate_series";

fn main() {
    let _ = good_ormning::sqlite::good_query_many!(
        dbm,
        "sqlite_gen_fixes",
        r#"select 1 as "x" from generate_series(1, 2)"#;
        &mut db
    );
}
