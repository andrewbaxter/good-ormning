const EXPECTED_ERROR: &str = "Table function generate_series has 1 columns but 2 aliases were given";

fn main() {
    let _ = good_ormning::pg::good_query_many!(
        dbm,
        "pg_gen_fixes",
        r#"select "g"."a" from generate_series(1, 2) as "g"("a", "b")"#;
        &mut db
    );
}
