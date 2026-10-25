const EXPECTED_ERROR: &str = "Unsupported table function json_each_text, its output columns aren't known";

fn main() {
    let _ = good_ormning::pg::good_query_many!(
        dbm,
        "pg_gen_fixes",
        r#"select 1 as "x" from json_each_text('{}')"#;
        &mut db
    );
}
