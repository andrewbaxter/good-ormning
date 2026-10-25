const EXPECTED_ERROR: &str = "Unsupported table-valued function: generate_series";

fn main() {
    let _ = good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_safety",
        r#"select 1 as `x` from generate_series(?) as `g`"#;
        &mut db,
        id: i8 = 1
    );
}
