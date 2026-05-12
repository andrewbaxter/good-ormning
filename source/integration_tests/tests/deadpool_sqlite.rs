use good_ormning::good_module;

#[tokio::test]
async fn test_deadpool_sqlite() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_base_insert");
    let dir = tempfile::tempdir().map_err(|e| loga::err(e))?;
    let db_path = dir.path().join("test.db");
    let cfg = deadpool_sqlite::Config::new(db_path);
    let pool =
        cfg
            .create_pool(deadpool_sqlite::Runtime::Tokio1)
            .map_err(|e| loga::err(e))?;

    // Get a connection and migrate + insert + query through interact()
    let conn = pool.get().await.map_err(|e| loga::err(e))?;
    conn.interact(|conn| -> Result<(), loga::Error> {
        let mut db = dbm::migrate(&mut *conn, None)?;
        good_ormning::sqlite::good_query!(
            dbm,
            "sqlite_gen_base_insert",
            //# genemichaels-external: sql-formatter-sqlite
            r#"insert into
                 "bannanana" ("hizat")
               values
                 (?1)
               "#;
            &mut db,
            p1: string = "deadpool_test"
        )?;
        assert_eq!(good_ormning::sqlite::good_query_one!(
            dbm,
            "sqlite_gen_base_insert",
            //# genemichaels-external: sql-formatter-sqlite
            r#"select
                 "bannanana"."hizat" as "hizat"
               from
                 "bannanana"
               "#;
            &mut db
        )?, "deadpool_test");
        return Ok(());
    }).await.map_err(|e| loga::err(e.to_string()))??;
    Ok(())
}
