const EXPECTED_ERROR: &str = "Unsupported insert syntax for PostgreSQL";

fn main() {
    let _ = good_ormning::pg::good_query!(
        dbm,
        "pg_gen_fixes",
        r#"insert into "t" as "x" ("name", "n", "f", "u") values ('z', 1, 1.5, 1)"#;
        &mut db
    );
}
