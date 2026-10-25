const EXPECTED_ERROR: &str = "Unsupported update syntax for MariaDB";

fn main() {
    let _ = good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_native",
        r#"update /*+ NO_RANGE_OPTIMIZATION(t) */ t set n = 1"#;
        &mut db
    );
}
