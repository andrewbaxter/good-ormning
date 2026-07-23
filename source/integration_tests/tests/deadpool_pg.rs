use {
    good_ormning::good_module,
    pglite_oxide::PgliteServer,
};

#[tokio::test]
async fn test_deadpool_pg_object() -> Result<(), loga::Error> {
    good_module!(dbm, "pg_gen_base_insert");
    let server = PgliteServer::temporary_tcp().map_err(|e| loga::err(e))?;

    // Migrate using a raw client first (pglite is single-connection)
    let (client, db_conn) =
        tokio_postgres::connect(&server.connection_uri(), tokio_postgres::NoTls).await.map_err(|e| loga::err(e))?;
    tokio::spawn(async move {
        if let Err(e) = db_conn.await {
            eprintln!("connection error: {}", e);
        }
    });
    let _db = dbm::migrate(client, None).await?;
    drop(_db);

    // Create a pool and get a connection from it
    let mut cfg = deadpool_postgres::Config::new();
    cfg.url = Some(server.connection_uri());
    let pool =
        cfg
            .create_pool(Some(deadpool_postgres::Runtime::Tokio1), tokio_postgres::NoTls)
            .map_err(|e| loga::err(e))?;
    let conn = pool.get().await.map_err(|e| loga::err(e))?;

    // Use the deadpool Object through the PgConnection trait via macros
    let mut db = dbm::DbPgGenBaseInsert1(conn);
    good_ormning::pg::good_query!(
        dbm,
        "pg_gen_base_insert",
        //# genemichaels-external: sql-formatter-pg
        r#"insert into
             "bannanana" ("hizat")
           values
             ($1)
           "#;
        &mut db,
        p1: string = "deadpool_test"
    ).await?;
    assert_eq!(good_ormning::pg::good_query_one!(
        dbm,
        "pg_gen_base_insert",
        //# genemichaels-external: sql-formatter-pg
        r#"select
             "bannanana"."hizat" as "hizat"
           from
             "bannanana"
           "#;
        &mut db
    ).await?, "deadpool_test");
    Ok(())
}
