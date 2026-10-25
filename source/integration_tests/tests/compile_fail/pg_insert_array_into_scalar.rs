const EXPECTED_ERROR: &str = "Expression is an array but destination is not an array";

fn main() {
    let _ = good_ormning::pg::good_query!(
        dbm,
        "pg_gen_fixes",
        r#"insert into "t" ("name", "n", "f", "u") values ('z', array[1], 1.5, 1)"#;
        &mut db
    );
}
