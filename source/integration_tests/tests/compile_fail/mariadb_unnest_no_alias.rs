const EXPECTED_ERROR: &str = "MariaDB requires derived tables to have an alias";

fn main() {
    let _ = good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_safety",
        r#"select 1 as `x` from unnest(?)"#;
        &mut db,
        ids: arr i8 = & ids
    );
}
