const EXPECTED_ERROR: &str = "Table function unnest has multiple columns named \"unnest\", give them column aliases";

fn main() {
    let _ = good_ormning::pg::good_query_many!(
        dbm,
        "pg_gen_fixes",
        r#"select 1 as "x" from unnest(array[1], array[2])"#;
        &mut db
    );
}
