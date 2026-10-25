const EXPECTED_ERROR: &str = "Can't expand wildcard for an unaliased subquery or function";

fn main() {
    let _ = good_ormning::pg::good_query_many!(
        dbm,
        "pg_gen_fixes",
        r#"select * from "t", unnest(array[1])"#;
        &mut db
    );
}
