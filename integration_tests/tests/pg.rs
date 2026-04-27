use {
    chrono::{
        TimeZone,
        Utc,
    },
    good_ormning::good_module,
    integration_tests::MyString,
    pglite_oxide::PgliteServer,
};

async fn db<'a>() -> Result<(tokio_postgres::Client, PgliteServer), loga::Error> {
    let server = PgliteServer::temporary_tcp().map_err(|e| loga::err(e))?;
    let (client, db_conn) =
        tokio_postgres::connect(&server.connection_uri(), tokio_postgres::NoTls).await.map_err(|e| loga::err(e))?;
    tokio::spawn(async move {
        if let Err(e) = db_conn.await {
            eprintln!("connection error: {}", e);
        }
    });
    Ok((client, server))
}

#[tokio::test]
async fn test_base_insert() -> Result<(), loga::Error> {
    good_module!("pg_gen_base_insert");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_base_insert",
        dbm::DbPgGenBaseInsert1(&mut db),
        r#"insert into "bannanana" ( "hizat" ) values ( $1 )"#;
        p1 = string;
        "soy"
    ).await?;
    assert_eq!(
        good_ormning::pg::good_query_one!(
            "pg_gen_base_insert",
            dbm::DbPgGenBaseInsert1(&mut db),
            r#"select "bannanana" . "hizat" as "hizat" from "bannanana""#
        ).await?,
        "soy"
    );
    Ok(())
}

#[tokio::test]
async fn test_param_i32() -> Result<(), loga::Error> {
    good_module!("pg_gen_param_i32");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_param_i32",
        dbm::DbPgGenParamI321(&mut db),
        r#"insert into "bananna" ( "hizat" ) values ( $1 )"#;
        p1 = i32;
        22
    ).await?;
    assert_eq!(
        good_ormning::pg::good_query_one!(
            "pg_gen_param_i32",
            dbm::DbPgGenParamI321(&mut db),
            r#"select "bananna" . "hizat" as "hizat" from "bananna""#
        ).await?,
        22
    );
    Ok(())
}

#[tokio::test]
async fn test_param_utctime_chrono() -> Result<(), loga::Error> {
    good_module!("pg_gen_param_utctime_chrono");
    let (mut db, _cont) = db().await?;
    let ref_date = chrono::TimeZone::with_ymd_and_hms(&chrono::Utc, 1937, 12, 1, 0, 0, 0).unwrap();
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_param_utctime_chrono",
        dbm::DbPgGenParamUtctimeChrono1(&mut db),
        r#"insert into "bananna" ( "hizat" ) values ( $1 )"#;
        p1 = utctime_s_chrono;
        ref_date
    ).await?;
    assert_eq!(
        good_ormning::pg::good_query_one!(
            "pg_gen_param_utctime_chrono",
            dbm::DbPgGenParamUtctimeChrono1(&mut db),
            r#"select "bananna" . "hizat" as "hizat" from "bananna""#
        ).await?,
        ref_date
    );
    Ok(())
}

#[tokio::test]
async fn test_param_utctime_jiff() -> Result<(), loga::Error> {
    good_module!("pg_gen_param_utctime_jiff");
    let (mut db, _cont) = db().await?;
    let ref_date =
        jiff::civil::DateTime::new(1937, 12, 1, 0, 0, 0, 0)
            .unwrap()
            .to_zoned(jiff::tz::TimeZone::UTC)
            .unwrap()
            .timestamp();
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_param_utctime_jiff",
        dbm::DbPgGenParamUtctimeJiff1(&mut db),
        r#"insert into "bananna" ( "hizat" ) values ( $1 )"#;
        p1 = utctime_s_jiff;
        ref_date
    ).await?;
    assert_eq!(
        good_ormning::pg::good_query_one!(
            "pg_gen_param_utctime_jiff",
            dbm::DbPgGenParamUtctimeJiff1(&mut db),
            r#"select "bananna" . "hizat" as "hizat" from "bananna""#
        ).await?,
        ref_date
    );
    Ok(())
}

#[tokio::test]
async fn test_param_opt_i32() -> Result<(), loga::Error> {
    good_module!("pg_gen_param_opt_i32");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_param_opt_i32",
        dbm::DbPgGenParamOptI321(&mut db),
        r#"insert into "bananna" ( "hizat" ) values ( $1 )"#;
        p1 = opt i32;
        Some(47)
    ).await?;
    assert_eq!(
        good_ormning::pg::good_query_one!(
            "pg_gen_param_opt_i32",
            dbm::DbPgGenParamOptI321(&mut db),
            r#"select "bananna" . "hizat" as "hizat" from "bananna""#
        ).await?,
        Some(47)
    );
    Ok(())
}

#[tokio::test]
async fn test_param_opt_i32_null() -> Result<(), loga::Error> {
    good_module!("pg_gen_param_opt_i32_null");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_param_opt_i32_null",
        dbm::DbPgGenParamOptI32Null1(&mut db),
        r#"insert into "bananna" ( "hizat" ) values ( null )"#
    ).await?;
    assert_eq!(
        good_ormning::pg::good_query_one!(
            "pg_gen_param_opt_i32_null",
            dbm::DbPgGenParamOptI32Null1(&mut db),
            r#"select "bananna" . "hizat" as "hizat" from "bananna""#
        ).await?,
        None
    );
    Ok(())
}

#[tokio::test]
async fn test_param_custom() -> Result<(), loga::Error> {
    good_module!("pg_gen_param_custom");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    let x_0 = integration_tests::MyBool(true);
    let x_1 = integration_tests::MyI32(13);
    let x_2 = integration_tests::MyI64(-22);
    let x_3 = integration_tests::MyU32(14);
    let x_4 = integration_tests::MyF32(12.);
    let x_5 = integration_tests::MyF64(99.);
    let x_6 = integration_tests::MyBytes("hi".as_bytes().to_vec());
    let x_7 = integration_tests::MyString("hogo".to_string());
    let x_8 = integration_tests::MyUtctimeChrono(Utc.with_ymd_and_hms(1999, 11, 14, 1, 2, 13).unwrap());
    let x_9 = integration_tests::MyUtctimeChrono(Utc.with_ymd_and_hms(1999, 6, 14, 10, 13, 57).unwrap());
    let x_10 =
        integration_tests::MyUtctimeJiff(
            jiff::civil::DateTime::new(1999, 11, 14, 1, 2, 13, 0)
                .unwrap()
                .to_zoned(jiff::tz::TimeZone::UTC)
                .unwrap()
                .timestamp(),
        );
    let x_11 =
        integration_tests::MyUtctimeJiff(
            jiff::civil::DateTime::new(1999, 6, 14, 10, 13, 57, 0)
                .unwrap()
                .to_zoned(jiff::tz::TimeZone::UTC)
                .unwrap()
                .timestamp(),
        );
    good_ormning::pg::good_query!(
        "pg_gen_param_custom",
        dbm::DbPgGenParamCustom1(&mut db),
        r#"insert into "bananna" ( "x_0" , "x_1" , "x_2" , "x_3" , "x_4" , "x_5" , "x_6" , "x_7" , "x_8" , "x_9" , "x_10" , "x_11" ) values ( $1 , $2 , $3 , $4 , $5 , $6 , $7 , $8 , $9 , $10 , $11 , $12 )"#;
        p1 = MyBool,
        p2 = MyI32,
        p3 = MyI64,
        p4 = MyU32,
        p5 = MyF32,
        p6 = MyF64,
        p7 = MyBytes,
        p8 = MyString,
        p9 = MyUtctimeChrono,
        p10 = MyUtctimeChrono,
        p11 = MyUtctimeJiff,
        p12 = MyUtctimeJiff;
        &x_0,
        &x_1,
        &x_2,
        &x_3,
        &x_4,
        &x_5,
        &x_6,
        &x_7,
        &x_8,
        &x_9,
        &x_10,
        &x_11,
    ).await?;
    let res =
        good_ormning::pg::good_query_one!(
            "pg_gen_param_custom",
            dbm::DbPgGenParamCustom1(&mut db),
            r#"select "bananna" . "x_0" as "x_0" , "bananna" . "x_1" as "x_1" , "bananna" . "x_2" as "x_2" , "bananna" . "x_3" as "x_3" , "bananna" . "x_4" as "x_4" , "bananna" . "x_5" as "x_5" , "bananna" . "x_6" as "x_6" , "bananna" . "x_7" as "x_7" , "bananna" . "x_8" as "x_8" , "bananna" . "x_9" as "x_9" , "bananna" . "x_10" as "x_10" , "bananna" . "x_11" as "x_11" from "bananna""#
        ).await?;
    assert_eq!(x_0, res.x_0);
    assert_eq!(x_1, res.x_1);
    assert_eq!(x_2, res.x_2);
    assert_eq!(x_3, res.x_3);
    assert_eq!(x_4, res.x_4);
    assert_eq!(x_5, res.x_5);
    assert_eq!(x_6, res.x_6);
    assert_eq!(x_7, res.x_7);
    assert_eq!(x_8, res.x_8);
    assert_eq!(x_9, res.x_9);
    assert_eq!(x_10, res.x_10);
    assert_eq!(x_11, res.x_11);
    Ok(())
}

#[tokio::test]
async fn test_param_opt_custom() -> Result<(), loga::Error> {
    good_module!("pg_gen_param_opt_custom");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_param_opt_custom",
        dbm::DbPgGenParamOptCustom1(&mut db),
        r#"insert into "bananna" ( "hizat" ) values ( $1 )"#;
        p1 = opt MyString;
        Some(&MyString("higgins".into()))
    ).await?;
    assert_eq!(
        good_ormning::pg::good_query_one!(
            "pg_gen_param_opt_custom",
            dbm::DbPgGenParamOptCustom1(&mut db),
            r#"select "bananna" . "hizat" as "hizat" from "bananna""#
        ).await?,
        Some(MyString("higgins".into()))
    );
    Ok(())
}

#[tokio::test]
async fn test_insert_on_conflict_do_nothing() -> Result<(), loga::Error> {
    good_module!("pg_gen_insert_on_conflict_do_nothing");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    assert!(good_ormning::pg::good_query_opt!(
        "pg_gen_insert_on_conflict_do_nothing",
        dbm::DbPgGenInsertOnConflictDoNothing1(&mut db),
        r#"insert into "bannanana" ( "hizat" ) values ( $1 ) on conflict do nothing returning 1 as "one""#;
        p1 = string;
        "soy"
    ).await?.is_some());
    assert!(good_ormning::pg::good_query_opt!(
        "pg_gen_insert_on_conflict_do_nothing",
        dbm::DbPgGenInsertOnConflictDoNothing1(&mut db),
        r#"insert into "bannanana" ( "hizat" ) values ( $1 ) on conflict do nothing returning 1 as "one""#;
        p1 = string;
        "soy"
    ).await?.is_none());
    Ok(())
}

#[tokio::test]
async fn test_insert_on_conflict_update() -> Result<(), loga::Error> {
    good_module!("pg_gen_insert_on_conflict_update");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    assert_eq!(good_ormning::pg::good_query_one!(
        "pg_gen_insert_on_conflict_update",
        dbm::DbPgGenInsertOnConflictUpdate1(&mut db),
        r#"insert into "bannanana" ( "hizat" , "two" ) values ( $1 , $2 ) on conflict ( "hizat" ) do update set "two" = "bannanana" . "two" + 1 returning "bannanana" . "two" as "two""#;
        p1 = string,
        p2 = i32;
        "soy",
        33
    ).await?, 33);
    assert_eq!(good_ormning::pg::good_query_one!(
        "pg_gen_insert_on_conflict_update",
        dbm::DbPgGenInsertOnConflictUpdate1(&mut db),
        r#"insert into "bannanana" ( "hizat" , "two" ) values ( $1 , $2 ) on conflict ( "hizat" ) do update set "two" = "bannanana" . "two" + 1 returning "bannanana" . "two" as "two""#;
        p1 = string,
        p2 = i32;
        "soy",
        7
    ).await?, 34);
    assert_eq!(good_ormning::pg::good_query_one!(
        "pg_gen_insert_on_conflict_update",
        dbm::DbPgGenInsertOnConflictUpdate1(&mut db),
        r#"insert into "bannanana" ( "hizat" , "two" ) values ( $1 , $2 ) on conflict ( "hizat" ) do update set "two" = "bannanana" . "two" + 1 returning "bannanana" . "two" as "two""#;
        p1 = string,
        p2 = i32;
        "yyyy",
        7
    ).await?, 7);
    Ok(())
}

#[tokio::test]
async fn test_update() -> Result<(), loga::Error> {
    good_module!("pg_gen_update");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_update",
        dbm::DbPgGenUpdate1(&mut db),
        r#"insert into "bananna" ( "hizat" ) values ( 'yog' )"#
    ).await?;
    assert_eq!(
        good_ormning::pg::good_query_one!(
            "pg_gen_update",
            dbm::DbPgGenUpdate1(&mut db),
            r#"select "bananna" . "hizat" as "hizat" from "bananna""#
        ).await?,
        "yog"
    );
    good_ormning::pg::good_query!(
        "pg_gen_update",
        dbm::DbPgGenUpdate1(&mut db),
        r#"update "bananna" set "hizat" = 'tep'"#
    ).await?;
    assert_eq!(
        good_ormning::pg::good_query_one!(
            "pg_gen_update",
            dbm::DbPgGenUpdate1(&mut db),
            r#"select "bananna" . "hizat" as "hizat" from "bananna""#
        ).await?,
        "tep"
    );
    Ok(())
}

#[tokio::test]
async fn test_update_where() -> Result<(), loga::Error> {
    good_module!("pg_gen_update_where");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_update_where",
        dbm::DbPgGenUpdateWhere1(&mut db),
        r#"insert into "ban" ( "hizat" ) values ( 'yog' )"#
    ).await?;
    assert_eq!(
        good_ormning::pg::good_query_one!(
            "pg_gen_update_where",
            dbm::DbPgGenUpdateWhere1(&mut db),
            r#"select "ban" . "hizat" as "hizat" from "ban""#
        ).await?,
        "yog"
    );
    good_ormning::pg::good_query!(
        "pg_gen_update_where",
        dbm::DbPgGenUpdateWhere1(&mut db),
        r#"update "ban" set "hizat" = $1 where "ban" . "hizat" = $2"#;
        p1 = string,
        p2 = string;
        "tep",
        "yog2"
    ).await?;
    assert_eq!(
        good_ormning::pg::good_query_one!(
            "pg_gen_update_where",
            dbm::DbPgGenUpdateWhere1(&mut db),
            r#"select "ban" . "hizat" as "hizat" from "ban""#
        ).await?,
        "yog"
    );
    good_ormning::pg::good_query!(
        "pg_gen_update_where",
        dbm::DbPgGenUpdateWhere1(&mut db),
        r#"update "ban" set "hizat" = $1 where "ban" . "hizat" = $2"#;
        p1 = string,
        p2 = string;
        "tep",
        "yog"
    ).await?;
    assert_eq!(
        good_ormning::pg::good_query_one!(
            "pg_gen_update_where",
            dbm::DbPgGenUpdateWhere1(&mut db),
            r#"select "ban" . "hizat" as "hizat" from "ban""#
        ).await?,
        "tep"
    );
    Ok(())
}

#[tokio::test]
async fn test_update_returning() -> Result<(), loga::Error> {
    good_module!("pg_gen_update_returning");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_update_returning",
        dbm::DbPgGenUpdateReturning1(&mut db),
        r#"insert into "b" ( "hizat" ) values ( 'yog' )"#
    ).await?;
    assert_eq!(
        good_ormning::pg::good_query_opt!(
            "pg_gen_update_returning",
            dbm::DbPgGenUpdateReturning1(&mut db),
            r#"update "b" set "hizat" = 'tep' returning "b" . "hizat" as "hizat""#
        ).await?,
        Some("tep".to_string())
    );
    Ok(())
}

#[tokio::test]
async fn test_delete() -> Result<(), loga::Error> {
    good_module!("pg_gen_delete");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_delete",
        dbm::DbPgGenDelete1(&mut db),
        r#"insert into "b" ( "hizat" ) values ( 'seeon' )"#
    ).await?;
    assert_eq!(
        good_ormning::pg::good_query_opt!(
            "pg_gen_delete",
            dbm::DbPgGenDelete1(&mut db),
            r#"select "b" . "hizat" as "hizat" from "b""#
        ).await?,
        Some("seeon".to_string())
    );
    good_ormning::pg::good_query!("pg_gen_delete", dbm::DbPgGenDelete1(&mut db), r#"delete from "b""#).await?;
    assert_eq!(
        good_ormning::pg::good_query_opt!(
            "pg_gen_delete",
            dbm::DbPgGenDelete1(&mut db),
            r#"select "b" . "hizat" as "hizat" from "b""#
        ).await?,
        None
    );
    Ok(())
}

#[tokio::test]
async fn test_delete_where() -> Result<(), loga::Error> {
    good_module!("pg_gen_delete_where");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_delete_where",
        dbm::DbPgGenDeleteWhere1(&mut db),
        r#"insert into "ba" ( "hizat" ) values ( 'seeon' )"#
    ).await?;
    good_ormning::pg::good_query!(
        "pg_gen_delete_where",
        dbm::DbPgGenDeleteWhere1(&mut db),
        r#"delete from "ba" where "ba" . "hizat" = $1"#;
        p1 = string;
        "nozo"
    ).await?;
    assert_eq!(
        good_ormning::pg::good_query_opt!(
            "pg_gen_delete_where",
            dbm::DbPgGenDeleteWhere1(&mut db),
            r#"select "ba" . "hizat" as "hizat" from "ba""#
        ).await?,
        Some("seeon".to_string())
    );
    good_ormning::pg::good_query!(
        "pg_gen_delete_where",
        dbm::DbPgGenDeleteWhere1(&mut db),
        r#"delete from "ba" where "ba" . "hizat" = $1"#;
        p1 = string;
        "seeon"
    ).await?;
    assert_eq!(
        good_ormning::pg::good_query_opt!(
            "pg_gen_delete_where",
            dbm::DbPgGenDeleteWhere1(&mut db),
            r#"select "ba" . "hizat" as "hizat" from "ba""#
        ).await?,
        None
    );
    Ok(())
}

#[tokio::test]
async fn test_delete_returning() -> Result<(), loga::Error> {
    good_module!("pg_gen_delete_returning");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_delete_returning",
        dbm::DbPgGenDeleteReturning1(&mut db),
        r#"insert into "b" ( "hizat" ) values ( 'seeon' )"#
    ).await?;
    assert!(
        good_ormning::pg::good_query_opt!(
            "pg_gen_delete_returning",
            dbm::DbPgGenDeleteReturning1(&mut db),
            r#"select "b" . "hizat" as "hizat" from "b""#
        )
            .await?
            .is_some()
    );
    good_ormning::pg::good_query!(
        "pg_gen_delete_returning",
        dbm::DbPgGenDeleteReturning1(&mut db),
        r#"delete from "b" where "b" . "hizat" = $1"#;
        p1 = string;
        "seeon"
    ).await?;
    assert!(
        good_ormning::pg::good_query_opt!(
            "pg_gen_delete_returning",
            dbm::DbPgGenDeleteReturning1(&mut db),
            r#"select "b" . "hizat" as "hizat" from "b""#
        )
            .await?
            .is_none()
    );
    Ok(())
}

#[tokio::test]
async fn test_select_join() -> Result<(), loga::Error> {
    good_module!("pg_gen_select_join");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, Some(&|v| Box::pin(async move {
        match v {
            dbm::DbPgGenSelectJoinVersions::V1(mut db) => {
                good_ormning::pg::good_query!(
                    "pg_gen_select_join",
                    dbm::DbPgGenSelectJoin1(&mut *db.0),
                    r#"insert into "b" ( "hizat" , "three" ) values ( 'key' , 33 )"#
                ).await?;
                good_ormning::pg::good_query!(
                    "pg_gen_select_join",
                    dbm::DbPgGenSelectJoin1(&mut *db.0),
                    r#"insert into "select_join_two" ( "hizat" , "two" ) values ( 'key' , 'no' )"#
                ).await?;
            },
            _ => { },
        }
        Ok(())
    }))).await?;
    let res =
        good_ormning::pg::good_query_one!(
            "pg_gen_select_join",
            dbm::DbPgGenSelectJoin1(&mut db),
            r#"select "b" . "three" as "three" , "select_join_two" . "two" as "two" from "b" left join "select_join_two" on ( "b" . "hizat" :: text ) = "select_join_two" . "hizat""#
        ).await?;
    assert_eq!(res.three, 33);
    assert_eq!(res.two, Some("no".into()));
    Ok(())
}

#[tokio::test]
async fn test_select_group_by() -> Result<(), loga::Error> {
    good_module!("pg_gen_select_group_by");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_select_group_by",
        dbm::DbPgGenSelectGroupBy1(&mut db),
        r#"insert into "bannanana" ( "hizat" , "hizat2" ) values ( $1 , $2 )"#;
        p1 = i32,
        p2 = i32;
        1,
        7
    ).await?;
    good_ormning::pg::good_query!(
        "pg_gen_select_group_by",
        dbm::DbPgGenSelectGroupBy1(&mut db),
        r#"insert into "bannanana" ( "hizat" , "hizat2" ) values ( $1 , $2 )"#;
        p1 = i32,
        p2 = i32;
        1,
        99
    ).await?;
    good_ormning::pg::good_query!(
        "pg_gen_select_group_by",
        dbm::DbPgGenSelectGroupBy1(&mut db),
        r#"insert into "bannanana" ( "hizat" , "hizat2" ) values ( $1 , $2 )"#;
        p1 = i32,
        p2 = i32;
        2,
        3
    ).await?;
    good_ormning::pg::good_query!(
        "pg_gen_select_group_by",
        dbm::DbPgGenSelectGroupBy1(&mut db),
        r#"insert into "bannanana" ( "hizat" , "hizat2" ) values ( $1 , $2 )"#;
        p1 = i32,
        p2 = i32;
        2,
        10
    ).await?;
    let mut res =
        good_ormning::pg::good_query_many!(
            "pg_gen_select_group_by",
            dbm::DbPgGenSelectGroupBy1(&mut db),
            r#"select sum ( "bannanana" . "hizat2" ) as "hizat2" from "bannanana" group by "bannanana" . "hizat""#
        ).await?;
    res.sort();
    assert_eq!(res, vec![13i64, 106i64]);
    Ok(())
}

#[tokio::test]
async fn test_select_limit() -> Result<(), loga::Error> {
    good_module!("pg_gen_select_limit");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_select_limit",
        dbm::DbPgGenSelectLimit1(&mut db),
        r#"insert into "bannanana" ( "hizat" ) values ( $1 )"#;
        p1 = string;
        "soy"
    ).await?;
    good_ormning::pg::good_query!(
        "pg_gen_select_limit",
        dbm::DbPgGenSelectLimit1(&mut db),
        r#"insert into "bannanana" ( "hizat" ) values ( $1 )"#;
        p1 = string;
        "soy"
    ).await?;
    good_ormning::pg::good_query!(
        "pg_gen_select_limit",
        dbm::DbPgGenSelectLimit1(&mut db),
        r#"insert into "bannanana" ( "hizat" ) values ( $1 )"#;
        p1 = string;
        "soy"
    ).await?;
    assert_eq!(
        good_ormning::pg::good_query_many!(
            "pg_gen_select_limit",
            dbm::DbPgGenSelectLimit1(&mut db),
            r#"select "bannanana" . "hizat" as "hizat" from "bannanana" limit 2"#
        )
            .await?
            .len(),
        2
    );
    Ok(())
}

#[tokio::test]
async fn test_select_order() -> Result<(), loga::Error> {
    good_module!("pg_gen_select_order");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_select_order",
        dbm::DbPgGenSelectOrder1(&mut db),
        r#"insert into "bannanana" ( "hizat" ) values ( $1 )"#;
        p1 = i32;
        0
    ).await?;
    good_ormning::pg::good_query!(
        "pg_gen_select_order",
        dbm::DbPgGenSelectOrder1(&mut db),
        r#"insert into "bannanana" ( "hizat" ) values ( $1 )"#;
        p1 = i32;
        12
    ).await?;
    good_ormning::pg::good_query!(
        "pg_gen_select_order",
        dbm::DbPgGenSelectOrder1(&mut db),
        r#"insert into "bannanana" ( "hizat" ) values ( $1 )"#;
        p1 = i32;
        9
    ).await?;
    assert_eq!(
        good_ormning::pg::good_query_many!(
            "pg_gen_select_order",
            dbm::DbPgGenSelectOrder1(&mut db),
            r#"select "bannanana" . "hizat" as "hizat" from "bannanana" order by "bannanana" . "hizat" asc"#
        ).await?,
        vec![0, 9, 12]
    );
    Ok(())
}

#[tokio::test]
async fn test_migrate_add_field() -> Result<(), loga::Error> {
    good_module!("pg_gen_migrate_add_field");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, Some(&|v| Box::pin(async move {
        match v {
            dbm::DbPgGenMigrateAddFieldVersions::V0(mut db) => {
                good_ormning::pg::good_query!(
                    "pg_gen_migrate_add_field",
                    0,
                    dbm::DbPgGenMigrateAddField0(&mut *db.0),
                    r#"insert into "bannna" ( "hizat" ) values ( 'nizoot' )"#
                ).await?;
            },
            _ => { },
        }
        Ok(())
    }))).await?;
    match good_ormning::pg::good_query_opt!(
        "pg_gen_migrate_add_field",
        dbm::DbPgGenMigrateAddField1(&mut db),
        r#"select "bannna" . "hizat" as "hizat" , "bannna" . "zomzom" as "zomzom" from "bannna""#
    ).await? {
        Some(x) => {
            assert_eq!(x.zomzom, true);
            assert_eq!(&x.hizat, "nizoot");
        },
        None => assert!(false),
    };
    Ok(())
}

#[tokio::test]
async fn test_migrate_rename_field() -> Result<(), loga::Error> {
    good_module!("pg_gen_migrate_rename_field");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_migrate_rename_field",
        dbm::DbPgGenMigrateRenameField1(&mut db),
        r#"insert into "bannna" ( "hizat" ) values ( 'nizoot' )"#
    ).await?;
    Ok(())
}

#[tokio::test]
async fn test_migrate_remove_field() -> Result<(), loga::Error> {
    good_module!("pg_gen_migrate_remove_field");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_migrate_remove_field",
        dbm::DbPgGenMigrateRemoveField1(&mut db),
        r#"insert into "bnanaa" ( "hizat" ) values ( $1 )"#;
        p1 = string;
        "yordol"
    ).await?;
    Ok(())
}

#[tokio::test]
async fn test_migrate_add_table() -> Result<(), loga::Error> {
    good_module!("pg_gen_migrate_add_table");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_migrate_add_table",
        dbm::DbPgGenMigrateAddTable1(&mut db),
        r#"insert into "migrate_add_table_two" ( "two" ) values ( $1 )"#;
        p1 = i32;
        23
    ).await?;
    Ok(())
}

#[tokio::test]
async fn test_migrate_rename_table() -> Result<(), loga::Error> {
    good_module!("pg_gen_migrate_rename_table");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_migrate_rename_table",
        dbm::DbPgGenMigrateRenameTable1(&mut db),
        r#"insert into "bana" ( "hizat" ) values ( $1 )"#;
        p1 = string;
        "inset"
    ).await?;
    Ok(())
}

#[tokio::test]
async fn test_migrate_remove_table() -> Result<(), loga::Error> {
    good_module!("pg_gen_migrate_remove_table");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    Ok(())
}

#[tokio::test]
async fn test_select_cte() -> Result<(), loga::Error> {
    good_module!("pg_gen_select_cte");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_select_cte",
        dbm::DbPgGenSelectCte1(&mut db),
        r#"insert into "select_cte_bannanana" ( "hizat" , "hizat2" ) values ( $1 , $2 )"#;
        p1 = i32,
        p2 = i32;
        1,
        7
    ).await?;
    good_ormning::pg::good_query!(
        "pg_gen_select_cte",
        dbm::DbPgGenSelectCte1(&mut db),
        r#"insert into "select_cte_bannanana" ( "hizat" , "hizat2" ) values ( $1 , $2 )"#;
        p1 = i32,
        p2 = i32;
        1,
        99
    ).await?;
    let mut res =
        good_ormning::pg::good_query_many!(
            "pg_gen_select_cte",
            dbm::DbPgGenSelectCte1(&mut db),
            r#"with "hibbo" ( "zathi" ) as ( select "select_cte_bannanana" . "hizat2" as "hizat2" from "select_cte_bannanana" ) select "hibbo" . "zathi" as "zathi" from "hibbo""#
        ).await?;
    res.sort();
    assert_eq!(res, vec![7, 99]);
    Ok(())
}

#[tokio::test]
async fn test_select_window() -> Result<(), loga::Error> {
    good_module!("pg_gen_select_window");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_select_window",
        dbm::DbPgGenSelectWindow1(&mut db),
        r#"insert into "select_window_bannanana" ( "hizat" , "hizat2" ) values ( $1 , $2 )"#;
        p1 = i32,
        p2 = i32;
        1,
        7
    ).await?;
    good_ormning::pg::good_query!(
        "pg_gen_select_window",
        dbm::DbPgGenSelectWindow1(&mut db),
        r#"insert into "select_window_bannanana" ( "hizat" , "hizat2" ) values ( $1 , $2 )"#;
        p1 = i32,
        p2 = i32;
        1,
        99
    ).await?;
    good_ormning::pg::good_query!(
        "pg_gen_select_window",
        dbm::DbPgGenSelectWindow1(&mut db),
        r#"insert into "select_window_bannanana" ( "hizat" , "hizat2" ) values ( $1 , $2 )"#;
        p1 = i32,
        p2 = i32;
        2,
        3
    ).await?;
    good_ormning::pg::good_query!(
        "pg_gen_select_window",
        dbm::DbPgGenSelectWindow1(&mut db),
        r#"insert into "select_window_bannanana" ( "hizat" , "hizat2" ) values ( $1 , $2 )"#;
        p1 = i32,
        p2 = i32;
        2,
        10
    ).await?;
    let mut res =
        good_ormning::pg::good_query_many!(
            "pg_gen_select_window",
            dbm::DbPgGenSelectWindow1(&mut db),
            r#"select sum ( "select_window_bannanana" . "hizat2" ) over ( partition by "select_window_bannanana" . "hizat" ) as "hizat2" from "select_window_bannanana""#
        )
            .await?
            .into_iter()
            .collect::<Vec<_>>();
    res.sort();
    assert_eq!(res, vec![13i64, 13i64, 106i64, 106i64]);
    Ok(())
}

#[tokio::test]
async fn test_migrate_make_field_optional() -> Result<(), loga::Error> {
    good_module!("pg_gen_migrate_make_field_optional");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    Ok(())
}

#[tokio::test]
async fn test_migrate_pre_migration() -> Result<(), loga::Error> {
    good_module!("pg_gen_migrate_pre_migration");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, Some(&|v| Box::pin(async move {
        match v {
            dbm::DbPgGenMigratePreMigrationVersions::V0(mut db) => {
                good_ormning::pg::good_query!(
                    "pg_gen_migrate_pre_migration",
                    0,
                    dbm::DbPgGenMigratePreMigration0(&mut *db.0),
                    r#"insert into "migrate_pre_migration_v0_two" ( "two" ) values ( 7 )"#
                ).await?;
            },
            _ => { },
        }
        Ok(())
    }))).await?;
    Ok(())
}
