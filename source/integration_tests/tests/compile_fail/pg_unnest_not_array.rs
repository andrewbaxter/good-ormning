const EXPECTED_ERROR: &str = "`unnest` arguments must be arrays";

fn main() {
    let _ = good_ormning::pg::good_query_many!(
        dbm,
        "pg_gen_fixes",
        r#"select "u"."u" from unnest(1) as "u""#;
        &mut db
    );
}
