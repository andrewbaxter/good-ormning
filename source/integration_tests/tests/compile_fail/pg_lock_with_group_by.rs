const EXPECTED_ERROR: &str = "Row locks can't be used with `distinct`, `group by`, `having`, or set operations";

fn main() {
    let _ = good_ormning::pg::good_query_many!(
        dbm,
        "pg_gen_fixes",
        r#"select "t"."n" from "t" group by "t"."n" for update"#;
        &mut db
    );
}
