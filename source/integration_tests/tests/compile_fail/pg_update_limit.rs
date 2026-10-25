const EXPECTED_ERROR: &str = "Unsupported update syntax for PostgreSQL";

fn main() {
    let _ = good_ormning::pg::good_query!(
        dbm,
        "pg_gen_fixes",
        r#"update "t" set "n" = 1 limit 1"#;
        &mut db
    );
}
