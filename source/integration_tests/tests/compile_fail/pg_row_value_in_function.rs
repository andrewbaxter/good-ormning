const EXPECTED_ERROR: &str = "Row values can only be used with `in` or for comparing rows";

fn main() {
    let _ = good_ormning::pg::good_query_many!(
        dbm,
        "pg_gen_fixes",
        r#"select "t"."name" from "t" where "t"."n" = coalesce(("t"."n", 1))"#;
        &mut db
    );
}
