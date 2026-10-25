const EXPECTED_ERROR: &str = "Insert columns must be unqualified column names";

fn main() {
    let _ = good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_fixes",
        r#"insert into "t" ("t"."name", "n", "f") values ('z', 1, 1.5)"#;
        &mut db
    );
}
