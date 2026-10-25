const EXPECTED_ERROR: &str = "Locked table \"nope\" isn't a source in this select";

fn main() {
    let _ = good_ormning::pg::good_query_many!(
        dbm,
        "pg_gen_fixes",
        r#"select "t"."name" from "t" for update of "nope""#;
        &mut db
    );
}
