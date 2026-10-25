const EXPECTED_ERROR: &str = "this field isn't available here (available fields: [])";

fn main() {
    let _ = good_ormning::pg::good_query_many!(
        dbm,
        "pg_gen_fixes",
        r#"select "s"."c" from "t" as "x", (select "x"."n" as "c") as "s""#;
        &mut db
    );
}
