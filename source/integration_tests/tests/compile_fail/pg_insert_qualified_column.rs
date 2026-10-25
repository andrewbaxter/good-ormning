const EXPECTED_ERROR: &str = "Insert columns must be unqualified column names";

fn main() {
    let _ = good_ormning::pg::good_query!(
        dbm,
        "pg_gen_fixes",
        r#"insert into "t" ("t"."name", "n", "f", "u") values ('z', 1, 1.5, 1)"#;
        &mut db
    );
}
