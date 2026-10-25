const EXPECTED_ERROR: &str = "Insert columns must be unqualified column names";

fn main() {
    let _ = good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_native",
        r#"insert into t (t.name, n) values ('z', 1)"#;
        &mut db
    );
}
