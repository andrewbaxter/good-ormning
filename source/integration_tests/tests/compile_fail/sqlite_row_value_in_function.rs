const EXPECTED_ERROR: &str = "SQLite doesn't have arrays; lists can only be used with `in` or for comparing rows";

fn main() {
    let _ = good_ormning::sqlite::good_query_many!(
        dbm,
        "sqlite_gen_fixes",
        r#"select "t"."name" from "t" where "t"."n" = coalesce(("t"."n", 1))"#;
        &mut db
    );
}
