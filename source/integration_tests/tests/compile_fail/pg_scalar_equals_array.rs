const EXPECTED_ERROR: &str = "Operator arms types don't match: left is not an array and right is an array";

fn main() {
    let _ = good_ormning::pg::good_query_many!(
        dbm,
        "pg_gen_fixes",
        r#"select "t"."name" from "t" where "t"."n" = array[7]"#;
        &mut db
    );
}
