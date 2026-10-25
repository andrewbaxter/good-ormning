const EXPECTED_ERROR: &str = "A select can't have both `limit` and `fetch`";

fn main() {
    let _ = good_ormning::pg::good_query_many!(
        dbm,
        "pg_gen_fixes",
        r#"select "t"."name" from "t" limit 1 fetch first 1 rows only"#;
        &mut db
    );
}
