const EXPECTED_ERROR: &str = "unnest has multiple columns named \"unnest\", give them column aliases";

fn main() {
    let _ = good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_safety",
        r#"select 1 as `x` from unnest(?) as `u`"#;
        &mut db,
        pairs: arr(i8, string) = & pairs
    );
}
