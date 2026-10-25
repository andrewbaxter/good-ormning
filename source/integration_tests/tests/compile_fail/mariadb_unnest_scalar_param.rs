const EXPECTED_ERROR: &str = "`unnest` takes a list (`arr`) parameter";

fn main() {
    let _ = good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_safety",
        r#"select `u`.`u` from unnest(?) as `u`"#;
        &mut db,
        id: i8 = 1
    );
}
