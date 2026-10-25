const EXPECTED_ERROR: &str = "The right side of `in` must be a list, a subquery, or an array";

fn main() {
    let _ = good_ormning::pg::good_query_many!(
        dbm,
        "pg_gen_fixes",
        r#"select "t"."name" from "t" where "t"."n" = any("t"."n")"#;
        &mut db
    );
}
