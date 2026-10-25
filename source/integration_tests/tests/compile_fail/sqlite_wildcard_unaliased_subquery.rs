const EXPECTED_ERROR: &str = "Can't expand wildcard for an unaliased subquery or function";

fn main() {
    let _ = good_ormning::sqlite::good_query_many!(
        dbm,
        "sqlite_gen_fixes",
        r#"select * from "t", (select 1 as "x")"#;
        &mut db
    );
}
