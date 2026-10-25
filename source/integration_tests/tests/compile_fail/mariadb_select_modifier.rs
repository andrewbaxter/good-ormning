const EXPECTED_ERROR: &str = "Unsupported select syntax for MariaDB";

fn main() {
    let _ = good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_native",
        r#"select sql_calc_found_rows name from t"#;
        &mut db
    );
}
