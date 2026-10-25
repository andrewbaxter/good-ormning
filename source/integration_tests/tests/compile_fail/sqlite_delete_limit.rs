const EXPECTED_ERROR: &str = "Unsupported delete syntax for SQLite";

fn main() {
    let _ = good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_fixes",
        r#"delete from "t" where "n" = 1 limit 1"#;
        &mut db
    );
}
