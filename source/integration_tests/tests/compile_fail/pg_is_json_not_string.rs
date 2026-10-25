const EXPECTED_ERROR: &str = "Is json -- Operator arms types don't match: left has type I32 and right has String";

fn main() {
    let _ = good_ormning::pg::good_query_many!(
        dbm,
        "pg_gen_fixes",
        r#"select "t"."n" is json as "x" from "t""#;
        &mut db
    );
}
