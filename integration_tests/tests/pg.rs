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
    good_module!(dbm, "pg_gen_base_insert");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_base_insert",
        r#"insert into "bannanana" ( "hizat" ) values ( $1 )"#;
        dbm::DbPgGenBaseInsert1(&mut db),
        p1: string = "soy"
    ).await?;
    assert_eq!(good_ormning::pg::good_query_one!(
        "pg_gen_base_insert",
        r#"select "bannanana" . "hizat" as "hizat" from "bannanana""#;
        dbm::DbPgGenBaseInsert1(&mut db)
    ).await?, "soy");
    Ok(())
}

#[tokio::test]
async fn test_param_i32() -> Result<(), loga::Error> {
    good_module!(dbm, "pg_gen_param_i32");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_param_i32",
        r#"insert into "bananna" ( "hizat" ) values ( $1 )"#;
        dbm::DbPgGenParamI321(&mut db),
        p1: i32 = 22
    ).await?;
    assert_eq!(good_ormning::pg::good_query_one!(
        "pg_gen_param_i32",
        r#"select "bananna" . "hizat" as "hizat" from "bananna""#;
        dbm::DbPgGenParamI321(&mut db)
    ).await?, 22);
    Ok(())
}

#[tokio::test]
async fn test_inline_param_i32() -> Result<(), loga::Error> {
    good_module!(dbm, "pg_gen_inline_param_i32");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_inline_param_i32",
        r#"insert into "bananna" ( "hizat" ) values ( ${i32 = 22} )"#;
        dbm::DbPgGenInlineParamI321(&mut db)
    ).await?;
    assert_eq!(good_ormning::pg::good_query_one!(
        "pg_gen_inline_param_i32",
        r#"select "bananna" . "hizat" as "hizat" from "bananna" where "bananna" . "hizat" = ${i32 = 22}"#;
        dbm::DbPgGenInlineParamI321(&mut db)
    ).await?, 22);
    Ok(())
}

#[tokio::test]
async fn test_param_utctime_chrono() -> Result<(), loga::Error> {
    good_module!(dbm, "pg_gen_param_utctime_chrono");
    let (mut db, _cont) = db().await?;
    let ref_date = chrono::TimeZone::with_ymd_and_hms(&chrono::Utc, 1937, 12, 1, 0, 0, 0).unwrap();
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_param_utctime_chrono",
        r#"insert into "bananna" ( "hizat" ) values ( $1 )"#;
        dbm::DbPgGenParamUtctimeChrono1(&mut db),
        p1: utctime_s_chrono = ref_date
    ).await?;
    assert_eq!(good_ormning::pg::good_query_one!(
        "pg_gen_param_utctime_chrono",
        r#"select "bananna" . "hizat" as "hizat" from "bananna""#;
        dbm::DbPgGenParamUtctimeChrono1(&mut db)
    ).await?, ref_date);
    Ok(())
}

#[tokio::test]
async fn test_query_like() -> Result<(), loga::Error> {
    good_module!(dbm, "pg_gen_query_like");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_query_like",
        r#"insert into "bananna" ( "hizat" ) values ( $1 )"#;
        dbm::DbPgGenQueryLike1(&mut db),
        p1: string = "apple pie"
    ).await?;
    assert_eq!(good_ormning::pg::good_query_one!(
        "pg_gen_query_like",
        r#"select "bananna" . "hizat" as "hizat" from "bananna" where "bananna" . "hizat" like $1"#;
        dbm::DbPgGenQueryLike1(&mut db),
        p1: string = "apple%"
    ).await?, "apple pie");
    Ok(())
}

#[tokio::test]
async fn test_query_is_null() -> Result<(), loga::Error> {
    good_module!(dbm, "pg_gen_query_is_null");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_query_is_null",
        r#"insert into "bananna" ( "hizat" ) values ( $1 )"#;
        dbm::DbPgGenQueryIsNull1(&mut db),
        p1: string = "not null"
    ).await?;
    assert_eq!(good_ormning::pg::good_query_one!(
        "pg_gen_query_is_null",
        r#"select count(*) as "count" from "bananna" where "bananna" . "hizat" is not null"#;
        dbm::DbPgGenQueryIsNull1(&mut db)
    ).await?, 1i64);
    Ok(())
}

#[tokio::test]
async fn test_query_concat() -> Result<(), loga::Error> {
    good_module!(dbm, "pg_gen_query_concat");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_query_concat",
        r#"insert into "bananna" ( "hizat" ) values ( $1 )"#;
        dbm::DbPgGenQueryConcat1(&mut db),
        p1: string = "hello"
    ).await?;
    assert_eq!(good_ormning::pg::good_query_one!(
        "pg_gen_query_concat",
        r#"select "bananna" . "hizat" || ' world' as "out" from "bananna""#;
        dbm::DbPgGenQueryConcat1(&mut db)
    ).await?, "hello world");
    Ok(())
}

#[tokio::test]
async fn test_query_row_number() -> Result<(), loga::Error> {
    good_module!(dbm, "pg_gen_query_row_number");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_query_row_number",
        r#"insert into "bananna" ( "hizat" ) values ( $1 )"#;
        dbm::DbPgGenQueryRowNumber1(&mut db),
        p1: string = "a"
    ).await?;
    good_ormning::pg::good_query!(
        "pg_gen_query_row_number",
        r#"insert into "bananna" ( "hizat" ) values ( $1 )"#;
        dbm::DbPgGenQueryRowNumber1(&mut db),
        p1: string = "b"
    ).await?;
    let res = good_ormning::pg::good_query_many!(
        "pg_gen_query_row_number",
        r#"select "bananna" . "hizat" as "hizat", row_number() over (order by "bananna" . "hizat" asc) as "row_num" from "bananna""#;
        dbm::DbPgGenQueryRowNumber1(&mut db)
    ).await?;
    assert_eq!(res.len(), 2);
    assert_eq!(res[0].hizat, "a");
    assert_eq!(res[0].row_num, 1i64);
    assert_eq!(res[1].hizat, "b");
    assert_eq!(res[1].row_num, 2i64);
    Ok(())
}

#[tokio::test]
async fn test_param_opt_i32() -> Result<(), loga::Error> {
    good_module!(dbm, "pg_gen_param_opt_i32");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_param_opt_i32",
        r#"insert into "bananna" ( "hizat" ) values ( $1 )"#;
        dbm::DbPgGenParamOptI321(&mut db),
        p1: opt i32 = Some(47)
    ).await?;
    assert_eq!(good_ormning::pg::good_query_one!(
        "pg_gen_param_opt_i32",
        r#"select "bananna" . "hizat" as "hizat" from "bananna""#;
        dbm::DbPgGenParamOptI321(&mut db)
    ).await?, Some(47));
    Ok(())
}

#[tokio::test]
async fn test_param_opt_i32_null() -> Result<(), loga::Error> {
    good_module!(dbm, "pg_gen_param_opt_i32_null");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_param_opt_i32_null",
        r#"insert into "bananna" ( "hizat" ) values ( null )"#;
        dbm::DbPgGenParamOptI32Null1(&mut db)
    ).await?;
    assert_eq!(good_ormning::pg::good_query_one!(
        "pg_gen_param_opt_i32_null",
        r#"select "bananna" . "hizat" as "hizat" from "bananna""#;
        dbm::DbPgGenParamOptI32Null1(&mut db)
    ).await?, None);
    Ok(())
}

#[tokio::test]
async fn test_param_custom() -> Result<(), loga::Error> {
    good_module!(dbm, "pg_gen_param_custom");
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
        r#"insert into "bananna" ( "x_0" , "x_1" , "x_2" , "x_3" , "x_4" , "x_5" , "x_6" , "x_7" , "x_8" , "x_9" , "x_10" , "x_11" ) values ( $1 , $2 , $3 , $4 , $5 , $6 , $7 , $8 , $9 , $10 , $11 , $12 )"#;
        dbm::DbPgGenParamCustom1(&mut db),
        p1: MyBool = & x_0,
        p2: MyI32 = & x_1,
        p3: MyI64 = & x_2,
        p4: MyU32 = & x_3,
        p5: MyF32 = & x_4,
        p6: MyF64 = & x_5,
        p7: MyBytes = & x_6,
        p8: MyString = & x_7,
        p9: MyUtctimeChrono = & x_8,
        p10: MyUtctimeChrono = & x_9,
        p11: MyUtctimeJiff = & x_10,
        p12: MyUtctimeJiff = & x_11
    ).await?;
    let res = good_ormning::pg::good_query_one!(
        "pg_gen_param_custom",
        r#"select "bananna" . "x_0" as "x_0",
        "bananna" . "x_1" as "x_1",
        "bananna" . "x_2" as "x_2",
        "bananna" . "x_3" as "x_3",
        "bananna" . "x_4" as "x_4",
        "bananna" . "x_5" as "x_5",
        "bananna" . "x_6" as "x_6",
        "bananna" . "x_7" as "x_7",
        "bananna" . "x_8" as "x_8",
        "bananna" . "x_9" as "x_9",
        "bananna" . "x_10" as "x_10",
        "bananna" . "x_11" as "x_11" from "bananna""#;
        dbm::DbPgGenParamCustom1(&mut db)
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
    good_module!(dbm, "pg_gen_param_opt_custom");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_param_opt_custom",
        r#"insert into "bananna" ( "hizat" ) values ( $1 )"#;
        dbm::DbPgGenParamOptCustom1(&mut db),
        p1: opt MyString = Some(&MyString("higgins".into()))
    ).await?;
    assert_eq!(good_ormning::pg::good_query_one!(
        "pg_gen_param_opt_custom",
        r#"select "bananna" . "hizat" as "hizat" from "bananna""#;
        dbm::DbPgGenParamOptCustom1(&mut db)
    ).await?, Some(MyString("higgins".into())));
    Ok(())
}

#[tokio::test]
async fn test_insert_on_conflict_do_nothing() -> Result<(), loga::Error> {
    good_module!(dbm, "pg_gen_insert_on_conflict_do_nothing");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    assert!(good_ormning::pg::good_query_opt!(
        "pg_gen_insert_on_conflict_do_nothing",
        r#"insert into "bannanana" ( "hizat" ) values ( $1 ) on conflict do nothing returning 1 as "one""#;
        dbm::DbPgGenInsertOnConflictDoNothing1(&mut db),
        p1: string = "soy"
    ).await?.is_some());
    assert!(good_ormning::pg::good_query_opt!(
        "pg_gen_insert_on_conflict_do_nothing",
        r#"insert into "bannanana" ( "hizat" ) values ( $1 ) on conflict do nothing returning 1 as "one""#;
        dbm::DbPgGenInsertOnConflictDoNothing1(&mut db),
        p1: string = "soy"
    ).await?.is_none());
    Ok(())
}

#[tokio::test]
async fn test_insert_on_conflict_update() -> Result<(), loga::Error> {
    good_module!(dbm, "pg_gen_insert_on_conflict_update");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    assert_eq!(good_ormning::pg::good_query_one!(
        "pg_gen_insert_on_conflict_update",
        r#"insert into "bannanana" ( "hizat" , "two" ) values ( $1 , $2 ) on conflict ( "hizat" ) do update set "two" = "bannanana" . "two" + 1 returning "bannanana" . "two" as "two""#;
        dbm::DbPgGenInsertOnConflictUpdate1(&mut db),
        p1: string = "soy",
        p2: i32 = 33
    ).await?, 33);
    assert_eq!(good_ormning::pg::good_query_one!(
        "pg_gen_insert_on_conflict_update",
        r#"insert into "bannanana" ( "hizat" , "two" ) values ( $1 , $2 ) on conflict ( "hizat" ) do update set "two" = "bannanana" . "two" + 1 returning "bannanana" . "two" as "two""#;
        dbm::DbPgGenInsertOnConflictUpdate1(&mut db),
        p1: string = "soy",
        p2: i32 = 7
    ).await?, 34);
    assert_eq!(good_ormning::pg::good_query_one!(
        "pg_gen_insert_on_conflict_update",
        r#"insert into "bannanana" ( "hizat" , "two" ) values ( $1 , $2 ) on conflict ( "hizat" ) do update set "two" = "bannanana" . "two" + 1 returning "bannanana" . "two" as "two""#;
        dbm::DbPgGenInsertOnConflictUpdate1(&mut db),
        p1: string = "yyyy",
        p2: i32 = 7
    ).await?, 7);
    Ok(())
}

#[tokio::test]
async fn test_update() -> Result<(), loga::Error> {
    good_module!(dbm, "pg_gen_update");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_update",
        r#"insert into "bananna" ( "hizat" ) values ( 'yog' )"#;
        dbm::DbPgGenUpdate1(&mut db)
    ).await?;
    assert_eq!(good_ormning::pg::good_query_one!(
        "pg_gen_update",
        r#"select "bananna" . "hizat" as "hizat" from "bananna""#;
        dbm::DbPgGenUpdate1(&mut db)
    ).await?, "yog");
    good_ormning::pg::good_query!(
        "pg_gen_update",
        r#"update "bananna" set "hizat" = 'tep'"#;
        dbm::DbPgGenUpdate1(&mut db)
    ).await?;
    assert_eq!(good_ormning::pg::good_query_one!(
        "pg_gen_update",
        r#"select "bananna" . "hizat" as "hizat" from "bananna""#;
        dbm::DbPgGenUpdate1(&mut db)
    ).await?, "tep");
    Ok(())
}

#[tokio::test]
async fn test_update_where() -> Result<(), loga::Error> {
    good_module!(dbm, "pg_gen_update_where");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_update_where",
        r#"insert into "ban" ( "hizat" ) values ( 'yog' )"#;
        dbm::DbPgGenUpdateWhere1(&mut db)
    ).await?;
    assert_eq!(good_ormning::pg::good_query_one!(
        "pg_gen_update_where",
        r#"select "ban" . "hizat" as "hizat" from "ban""#;
        dbm::DbPgGenUpdateWhere1(&mut db)
    ).await?, "yog");
    good_ormning::pg::good_query!(
        "pg_gen_update_where",
        r#"update "ban" set "hizat" = $1 where "ban" . "hizat" = $2"#;
        dbm::DbPgGenUpdateWhere1(&mut db),
        p1: string = "tep",
        p2: string = "yog2"
    ).await?;
    assert_eq!(good_ormning::pg::good_query_one!(
        "pg_gen_update_where",
        r#"select "ban" . "hizat" as "hizat" from "ban""#;
        dbm::DbPgGenUpdateWhere1(&mut db)
    ).await?, "yog");
    good_ormning::pg::good_query!(
        "pg_gen_update_where",
        r#"update "ban" set "hizat" = $1 where "ban" . "hizat" = $2"#;
        dbm::DbPgGenUpdateWhere1(&mut db),
        p1: string = "tep",
        p2: string = "yog"
    ).await?;
    assert_eq!(good_ormning::pg::good_query_one!(
        "pg_gen_update_where",
        r#"select "ban" . "hizat" as "hizat" from "ban""#;
        dbm::DbPgGenUpdateWhere1(&mut db)
    ).await?, "tep");
    Ok(())
}

#[tokio::test]
async fn test_update_returning() -> Result<(), loga::Error> {
    good_module!(dbm, "pg_gen_update_returning");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_update_returning",
        r#"insert into "b" ( "hizat" ) values ( 'yog' )"#;
        dbm::DbPgGenUpdateReturning1(&mut db)
    ).await?;
    assert_eq!(good_ormning::pg::good_query_opt!(
        "pg_gen_update_returning",
        r#"update "b" set "hizat" = 'tep' returning "b" . "hizat" as "hizat""#;
        dbm::DbPgGenUpdateReturning1(&mut db)
    ).await?, Some("tep".to_string()));
    Ok(())
}

#[tokio::test]
async fn test_delete() -> Result<(), loga::Error> {
    good_module!(dbm, "pg_gen_delete");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_delete",
        r#"insert into "b" ( "hizat" ) values ( 'seeon' )"#;
        dbm::DbPgGenDelete1(&mut db)
    ).await?;
    assert_eq!(good_ormning::pg::good_query_opt!(
        "pg_gen_delete",
        r#"select "b" . "hizat" as "hizat" from "b""#;
        dbm::DbPgGenDelete1(&mut db)
    ).await?, Some("seeon".to_string()));
    good_ormning::pg::good_query!(
        "pg_gen_delete",
        r#"delete from "b""#;
        dbm::DbPgGenDelete1(&mut db)
    ).await?;
    assert_eq!(good_ormning::pg::good_query_opt!(
        "pg_gen_delete",
        r#"select "b" . "hizat" as "hizat" from "b""#;
        dbm::DbPgGenDelete1(&mut db)
    ).await?, None);
    Ok(())
}

#[tokio::test]
async fn test_delete_where() -> Result<(), loga::Error> {
    good_module!(dbm, "pg_gen_delete_where");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_delete_where",
        r#"insert into "ba" ( "hizat" ) values ( 'seeon' )"#;
        dbm::DbPgGenDeleteWhere1(&mut db)
    ).await?;
    good_ormning::pg::good_query!(
        "pg_gen_delete_where",
        r#"delete from "ba" where "ba" . "hizat" = $1"#;
        dbm::DbPgGenDeleteWhere1(&mut db),
        p1: string = "nozo"
    ).await?;
    assert_eq!(good_ormning::pg::good_query_opt!(
        "pg_gen_delete_where",
        r#"select "ba" . "hizat" as "hizat" from "ba""#;
        dbm::DbPgGenDeleteWhere1(&mut db)
    ).await?, Some("seeon".to_string()));
    good_ormning::pg::good_query!(
        "pg_gen_delete_where",
        r#"delete from "ba" where "ba" . "hizat" = $1"#;
        dbm::DbPgGenDeleteWhere1(&mut db),
        p1: string = "seeon"
    ).await?;
    assert_eq!(good_ormning::pg::good_query_opt!(
        "pg_gen_delete_where",
        r#"select "ba" . "hizat" as "hizat" from "ba""#;
        dbm::DbPgGenDeleteWhere1(&mut db)
    ).await?, None);
    Ok(())
}

#[tokio::test]
async fn test_delete_returning() -> Result<(), loga::Error> {
    good_module!(dbm, "pg_gen_delete_returning");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_delete_returning",
        r#"insert into "b" ( "hizat" ) values ( 'seeon' )"#;
        dbm::DbPgGenDeleteReturning1(&mut db)
    ).await?;
    assert!(good_ormning::pg::good_query_opt!(
        "pg_gen_delete_returning",
        r#"select "b" . "hizat" as "hizat" from "b""#;
        dbm::DbPgGenDeleteReturning1(&mut db)
    ).await?.is_some());
    good_ormning::pg::good_query!(
        "pg_gen_delete_returning",
        r#"delete from "b" where "b" . "hizat" = $1"#;
        dbm::DbPgGenDeleteReturning1(&mut db),
        p1: string = "seeon"
    ).await?;
    assert!(good_ormning::pg::good_query_opt!(
        "pg_gen_delete_returning",
        r#"select "b" . "hizat" as "hizat" from "b""#;
        dbm::DbPgGenDeleteReturning1(&mut db)
    ).await?.is_none());
    Ok(())
}

#[tokio::test]
async fn test_select_join() -> Result<(), loga::Error> {
    good_module!(dbm, "pg_gen_select_join");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, Some(&|v| Box::pin(async move {
        match v {
            dbm::DbPgGenSelectJoinVersions::V1(db) => {
                good_ormning::pg::good_query!(
                    "pg_gen_select_join",
                    r#"insert into "b" ( "hizat" , "three" ) values ( 'key' , 33 )"#;
                    dbm::DbPgGenSelectJoin1(&mut *db.0)
                ).await?;
                good_ormning::pg::good_query!(
                    "pg_gen_select_join",
                    r#"insert into "select_join_two" ( "hizat" , "two" ) values ( 'key' , 'no' )"#;
                    dbm::DbPgGenSelectJoin1(&mut *db.0)
                ).await?;
            },
        }
        Ok(())
    }))).await?;
    let res = good_ormning::pg::good_query_one!(
        "pg_gen_select_join",
        r#"select "b"."three" as "three", "select_join_two"."two" as "two" from "b" left join "select_join_two" on("b"."hizat":: text) = "select_join_two"."hizat" "#;
        dbm::DbPgGenSelectJoin1(&mut db)
    ).await?;
    assert_eq!(res.three, 33);
    assert_eq!(res.two, Some("no".into()));
    Ok(())
}

#[tokio::test]
async fn test_select_group_by() -> Result<(), loga::Error> {
    good_module!(dbm, "pg_gen_select_group_by");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_select_group_by",
        r#"insert into "bannanana" ( "hizat" , "hizat2" ) values ( $1 , $2 )"#;
        dbm::DbPgGenSelectGroupBy1(&mut db),
        p1: i32 = 1,
        p2: i32 = 7
    ).await?;
    good_ormning::pg::good_query!(
        "pg_gen_select_group_by",
        r#"insert into "bannanana" ( "hizat" , "hizat2" ) values ( $1 , $2 )"#;
        dbm::DbPgGenSelectGroupBy1(&mut db),
        p1: i32 = 1,
        p2: i32 = 99
    ).await?;
    good_ormning::pg::good_query!(
        "pg_gen_select_group_by",
        r#"insert into "bannanana" ( "hizat" , "hizat2" ) values ( $1 , $2 )"#;
        dbm::DbPgGenSelectGroupBy1(&mut db),
        p1: i32 = 2,
        p2: i32 = 3
    ).await?;
    good_ormning::pg::good_query!(
        "pg_gen_select_group_by",
        r#"insert into "bannanana" ( "hizat" , "hizat2" ) values ( $1 , $2 )"#;
        dbm::DbPgGenSelectGroupBy1(&mut db),
        p1: i32 = 2,
        p2: i32 = 10
    ).await?;
    let mut res = good_ormning::pg::good_query_many!(
        "pg_gen_select_group_by",
        r#"select sum ( "bannanana" . "hizat2" ) as "hizat2" from "bannanana" group by "bannanana" . "hizat""#;
        dbm::DbPgGenSelectGroupBy1(&mut db)
    ).await?;
    res.sort();
    assert_eq!(res, vec![Some(13i64), Some(106i64)]);
    Ok(())
}

#[tokio::test]
async fn test_select_limit() -> Result<(), loga::Error> {
    good_module!(dbm, "pg_gen_select_limit");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_select_limit",
        r#"insert into "bannanana" ( "hizat" ) values ( $1 )"#;
        dbm::DbPgGenSelectLimit1(&mut db),
        p1: string = "soy"
    ).await?;
    good_ormning::pg::good_query!(
        "pg_gen_select_limit",
        r#"insert into "bannanana" ( "hizat" ) values ( $1 )"#;
        dbm::DbPgGenSelectLimit1(&mut db),
        p1: string = "soy"
    ).await?;
    good_ormning::pg::good_query!(
        "pg_gen_select_limit",
        r#"insert into "bannanana" ( "hizat" ) values ( $1 )"#;
        dbm::DbPgGenSelectLimit1(&mut db),
        p1: string = "soy"
    ).await?;
    assert_eq!(good_ormning::pg::good_query_many!(
        "pg_gen_select_limit",
        r#"select "bannanana" . "hizat" as "hizat" from "bannanana" limit 2"#;
        dbm::DbPgGenSelectLimit1(&mut db)
    ).await?.len(), 2);
    Ok(())
}

#[tokio::test]
async fn test_select_order() -> Result<(), loga::Error> {
    good_module!(dbm, "pg_gen_select_order");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_select_order",
        r#"insert into "bannanana" ( "hizat" ) values ( $1 )"#;
        dbm::DbPgGenSelectOrder1(&mut db),
        p1: i32 = 0
    ).await?;
    good_ormning::pg::good_query!(
        "pg_gen_select_order",
        r#"insert into "bannanana" ( "hizat" ) values ( $1 )"#;
        dbm::DbPgGenSelectOrder1(&mut db),
        p1: i32 = 12
    ).await?;
    good_ormning::pg::good_query!(
        "pg_gen_select_order",
        r#"insert into "bannanana" ( "hizat" ) values ( $1 )"#;
        dbm::DbPgGenSelectOrder1(&mut db),
        p1: i32 = 9
    ).await?;
    assert_eq!(good_ormning::pg::good_query_many!(
        "pg_gen_select_order",
        r#"select "bannanana" . "hizat" as "hizat" from "bannanana" order by "bannanana" . "hizat" asc"#;
        dbm::DbPgGenSelectOrder1(&mut db)
    ).await?, vec![0, 9, 12]);
    Ok(())
}

#[tokio::test]
async fn test_migrate_add_field() -> Result<(), loga::Error> {
    good_module!(dbm, "pg_gen_migrate_add_field");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, Some(&|v| Box::pin(async move {
        match v {
            dbm::DbPgGenMigrateAddFieldVersions::V0(db) => {
                good_ormning::pg::good_query!(
                    "pg_gen_migrate_add_field",
                    "0",
                    r#"insert into "bannna" ( "hizat" ) values ( 'nizoot' )"#;
                    dbm::DbPgGenMigrateAddField0(&mut *db.0)
                ).await?;
            },
            _ => { },
        }
        Ok(())
    }))).await?;
    match good_ormning::pg::good_query_opt!(
        "pg_gen_migrate_add_field",
        r#"select "bannna"."hizat" as "hizat", "bannna"."zomzom" as "zomzom" from "bannna" "#;
        dbm::DbPgGenMigrateAddField1(&mut db)
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
    good_module!(dbm, "pg_gen_migrate_rename_field");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_migrate_rename_field",
        r#"insert into "bannna" ( "hizat" ) values ( 'nizoot' )"#;
        dbm::DbPgGenMigrateRenameField1(&mut db)
    ).await?;
    Ok(())
}

#[tokio::test]
async fn test_migrate_remove_field() -> Result<(), loga::Error> {
    good_module!(dbm, "pg_gen_migrate_remove_field");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_migrate_remove_field",
        r#"insert into "bnanaa" ( "hizat" ) values ( $1 )"#;
        dbm::DbPgGenMigrateRemoveField1(&mut db),
        p1: string = "yordol"
    ).await?;
    Ok(())
}

#[tokio::test]
async fn test_migrate_add_table() -> Result<(), loga::Error> {
    good_module!(dbm, "pg_gen_migrate_add_table");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_migrate_add_table",
        r#"insert into "migrate_add_table_two" ( "two" ) values ( $1 )"#;
        dbm::DbPgGenMigrateAddTable1(&mut db),
        p1: i32 = 23
    ).await?;
    Ok(())
}

#[tokio::test]
async fn test_migrate_rename_table() -> Result<(), loga::Error> {
    good_module!(dbm, "pg_gen_migrate_rename_table");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_migrate_rename_table",
        r#"insert into "bana" ( "hizat" ) values ( $1 )"#;
        dbm::DbPgGenMigrateRenameTable1(&mut db),
        p1: string = "inset"
    ).await?;
    Ok(())
}

#[tokio::test]
async fn test_migrate_remove_table() -> Result<(), loga::Error> {
    good_module!(dbm, "pg_gen_migrate_remove_table");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    Ok(())
}

#[tokio::test]
async fn test_select_cte() -> Result<(), loga::Error> {
    good_module!(dbm, "pg_gen_select_cte");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_select_cte",
        r#"insert into "select_cte_bannanana" ( "hizat" , "hizat2" ) values ( $1 , $2 )"#;
        dbm::DbPgGenSelectCte1(&mut db),
        p1: i32 = 1,
        p2: i32 = 7
    ).await?;
    good_ormning::pg::good_query!(
        "pg_gen_select_cte",
        r#"insert into "select_cte_bannanana" ( "hizat" , "hizat2" ) values ( $1 , $2 )"#;
        dbm::DbPgGenSelectCte1(&mut db),
        p1: i32 = 1,
        p2: i32 = 99
    ).await?;
    let mut res = good_ormning::pg::good_query_many!(
        "pg_gen_select_cte",
        r#"with "hibbo" ( "zathi" ) as ( select "select_cte_bannanana" . "hizat2" as "hizat2" from "select_cte_bannanana" ) select "hibbo" . "zathi" as "zathi" from "hibbo""#;
        dbm::DbPgGenSelectCte1(&mut db)
    ).await?;
    res.sort();
    assert_eq!(res, vec![7, 99]);
    Ok(())
}

#[tokio::test]
async fn test_select_window() -> Result<(), loga::Error> {
    good_module!(dbm, "pg_gen_select_window");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_select_window",
        r#"insert into "select_window_bannanana" ( "hizat" , "hizat2" ) values ( $1 , $2 )"#;
        dbm::DbPgGenSelectWindow1(&mut db),
        p1: i32 = 1,
        p2: i32 = 7
    ).await?;
    good_ormning::pg::good_query!(
        "pg_gen_select_window",
        r#"insert into "select_window_bannanana" ( "hizat" , "hizat2" ) values ( $1 , $2 )"#;
        dbm::DbPgGenSelectWindow1(&mut db),
        p1: i32 = 1,
        p2: i32 = 99
    ).await?;
    good_ormning::pg::good_query!(
        "pg_gen_select_window",
        r#"insert into "select_window_bannanana" ( "hizat" , "hizat2" ) values ( $1 , $2 )"#;
        dbm::DbPgGenSelectWindow1(&mut db),
        p1: i32 = 2,
        p2: i32 = 3
    ).await?;
    good_ormning::pg::good_query!(
        "pg_gen_select_window",
        r#"insert into "select_window_bannanana" ( "hizat" , "hizat2" ) values ( $1 , $2 )"#;
        dbm::DbPgGenSelectWindow1(&mut db),
        p1: i32 = 2,
        p2: i32 = 10
    ).await?;
    let mut res = good_ormning::pg::good_query_many!(
        "pg_gen_select_window",
        r#"select sum ( "select_window_bannanana" . "hizat2" ) over ( partition by "select_window_bannanana" . "hizat" ) as "hizat2" from "select_window_bannanana""#;
        dbm::DbPgGenSelectWindow1(&mut db)
    ).await?.into_iter().collect::<Vec<_>>();
    res.sort();
    assert_eq!(res, vec![Some(13i64), Some(13i64), Some(106i64), Some(106i64)]);
    Ok(())
}

#[tokio::test]
async fn test_migrate_make_field_optional() -> Result<(), loga::Error> {
    good_module!(dbm, "pg_gen_migrate_make_field_optional");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    Ok(())
}

#[tokio::test]
async fn test_query_filter() -> Result<(), loga::Error> {
    good_module!(dbm, "pg_gen_query_filter");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_query_filter",
        r#"insert into "bananna" ( "hizat" , "two" ) values ( $1 , $2 )"#;
        dbm::DbPgGenQueryFilter1(&mut db),
        p1: string = "a",
        p2: i32 = 10
    ).await?;
    good_ormning::pg::good_query!(
        "pg_gen_query_filter",
        r#"insert into "bananna" ( "hizat" , "two" ) values ( $1 , $2 )"#;
        dbm::DbPgGenQueryFilter1(&mut db),
        p1: string = "b",
        p2: i32 = 20
    ).await?;
    let res = good_ormning::pg::good_query_one!(
        "pg_gen_query_filter",
        r#"select sum("two") filter (where "hizat" = 'a') as "x" from "bananna""#;
        dbm::DbPgGenQueryFilter1(&mut db)
    ).await?;
    assert_eq!(res, Some(10i64));
    Ok(())
}

#[tokio::test]
async fn test_query_window_frame() -> Result<(), loga::Error> {
    good_module!(dbm, "pg_gen_query_window_frame");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    for i in 1 ..= 3 {
        good_ormning::pg::good_query!(
            "pg_gen_query_window_frame",
            r#"insert into "bananna" ( "hizat" , "two" ) values ( 'key' , $1 )"#;
            dbm::DbPgGenQueryWindowFrame1(&mut db),
            p1: i32 = i
        ).await?;
    }
    let res = good_ormning::pg::good_query_many!(
        "pg_gen_query_window_frame",
        r#"select sum("two") over (order by "two" rows between unbounded preceding and current row) as "x" from "bananna""#;
        dbm::DbPgGenQueryWindowFrame1(&mut db)
    ).await?;
    assert_eq!(res, vec![Some(1i64), Some(3i64), Some(6i64)]);
    Ok(())
}

#[tokio::test]
async fn test_query_collate() -> Result<(), loga::Error> {
    good_module!(dbm, "pg_gen_query_collate");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_query_collate",
        r#"insert into "bananna" ( "hizat" ) values ( 'abc' )"#;
        dbm::DbPgGenQueryCollate1(&mut db)
    ).await?;
    let res = good_ormning::pg::good_query_one!(
        "pg_gen_query_collate",
        r#"select "hizat" as "x" from "bananna" where "hizat" collate "C" = 'abc'"#;
        dbm::DbPgGenQueryCollate1(&mut db)
    ).await?;
    assert_eq!(res, "abc");
    Ok(())
}

#[tokio::test]
async fn test_query_is_distinct_from() -> Result<(), loga::Error> {
    good_module!(dbm, "pg_gen_query_is_distinct_from");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_query_is_distinct_from",
        r#"insert into "bananna" ( "hizat" ) values ( null )"#;
        dbm::DbPgGenQueryIsDistinctFrom1(&mut db)
    ).await?;
    let res = good_ormning::pg::good_query_one!(
        "pg_gen_query_is_distinct_from",
        r#"select count(*) as "x" from "bananna" where "hizat" is distinct from 'abc'"#;
        dbm::DbPgGenQueryIsDistinctFrom1(&mut db)
    ).await?;
    assert_eq!(res, 1i64);
    let res2 = good_ormning::pg::good_query_one!(
        "pg_gen_query_is_distinct_from",
        r#"select count(*) as "x" from "bananna" where "hizat" is not distinct from null"#;
        dbm::DbPgGenQueryIsDistinctFrom1(&mut db)
    ).await?;
    assert_eq!(res2, 1i64);
    Ok(())
}

#[tokio::test]
async fn test_query_having() -> Result<(), loga::Error> {
    good_module!(dbm, "pg_gen_query_having");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_query_having",
        r#"insert into "bananna" ( "hizat" , "two" ) values ( 'a' , 10 )"#;
        dbm::DbPgGenQueryHaving1(&mut db)
    ).await?;
    good_ormning::pg::good_query!(
        "pg_gen_query_having",
        r#"insert into "bananna" ( "hizat" , "two" ) values ( 'b' , 20 )"#;
        dbm::DbPgGenQueryHaving1(&mut db)
    ).await?;
    let res = good_ormning::pg::good_query_many!(
        "pg_gen_query_having",
        r#"select "hizat" as "x" from "bananna" group by "hizat" having sum("two") > 15"#;
        dbm::DbPgGenQueryHaving1(&mut db)
    ).await?;
    assert_eq!(res, vec!["b".to_string()]);
    Ok(())
}

#[tokio::test]
async fn test_query_cte_subquery() -> Result<(), loga::Error> {
    good_module!(dbm, "pg_gen_query_cte_subquery");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_query_cte_subquery",
        r#"insert into "bananna" ( "hizat" ) values ( 'a' )"#;
        dbm::DbPgGenQueryCteSubquery1(&mut db)
    ).await?;
    let res = good_ormning::pg::good_query_one!(
        "pg_gen_query_cte_subquery",
        r#"select (with "t" as (select 1 as "v") select "v" from "t") as "x""#;
        dbm::DbPgGenQueryCteSubquery1(&mut db)
    ).await?;
    assert_eq!(res, 1i32);
    Ok(())
}

#[tokio::test]
async fn test_query_like_escape() -> Result<(), loga::Error> {
    good_module!(dbm, "pg_gen_query_like_escape");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_query_like_escape",
        r#"insert into "bananna" ( "hizat" ) values ( 'a%b' )"#;
        dbm::DbPgGenQueryLikeEscape1(&mut db)
    ).await?;
    let res = good_ormning::pg::good_query_one!(
        "pg_gen_query_like_escape",
        r#"select count(*) as "x" from "bananna" where "hizat" like 'a!%b' escape '!'"#;
        dbm::DbPgGenQueryLikeEscape1(&mut db)
    ).await?;
    assert_eq!(res, 1i64);
    Ok(())
}

#[tokio::test]
async fn test_query_union() -> Result<(), loga::Error> {
    good_module!(dbm, "pg_gen_base_insert");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query_many!(
        "pg_gen_base_insert",
        r#"select "hizat" as "hizat" from "bannanana" union select "hizat" as "hizat" from "bannanana""#;
        dbm::DbPgGenBaseInsert1(&mut db)
    ).await?;
    Ok(())
}

#[tokio::test]
async fn test_returning_wildcard() -> Result<(), loga::Error> {
    good_module!(dbm, "pg_gen_base_insert");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    let res = good_ormning::pg::good_query_one!(
        "pg_gen_base_insert",
        r#"insert into "bannanana" ( "hizat" ) values ( 'hi' ) returning *"#;
        dbm::DbPgGenBaseInsert1(&mut db)
    ).await?;
    assert_eq!(res.hizat, "hi");
    Ok(())
}

#[tokio::test]
async fn test_query_between() -> Result<(), loga::Error> {
    good_module!(dbm, "pg_gen_base_insert");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_base_insert",
        r#"insert into "bannanana" ( "hizat" , "hizat2" ) values ( 'a' , 5 )"#;
        dbm::DbPgGenBaseInsert1(&mut db)
    ).await?;
    let res = good_ormning::pg::good_query_one!(
        "pg_gen_base_insert",
        r#"select count(*) as "x" from "bannanana" where "hizat2" between 1 and 10"#;
        dbm::DbPgGenBaseInsert1(&mut db)
    ).await?;
    assert_eq!(res, 1i64);
    Ok(())
}

#[tokio::test]
async fn test_query_case() -> Result<(), loga::Error> {
    good_module!(dbm, "pg_gen_base_insert");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_base_insert",
        r#"insert into "bannanana" ( "hizat" , "hizat2" ) values ( 'a' , 5 )"#;
        dbm::DbPgGenBaseInsert1(&mut db)
    ).await?;
    let res = good_ormning::pg::good_query_one!(
        "pg_gen_base_insert",
        r#"select case when "hizat2" > 0 then 'positive' else 'non-positive' end as "res" from "bannanana""#;
        dbm::DbPgGenBaseInsert1(&mut db)
    ).await?;
    assert_eq!(res, "positive");
    Ok(())
}

#[tokio::test]
async fn test_query_tuple_in() -> Result<(), loga::Error> {
    good_module!(dbm, "pg_gen_base_insert");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    db
        .execute("insert into bannanana (hizat, hizat2) values ('a', 1), ('b', 2), ('c', 3)", &[])
        .await
        .map_err(loga::err)?;
    let res = good_ormning::pg::good_query_many!(
        "pg_gen_base_insert",
        "select hizat from bannanana where (hizat, hizat2) in (('a', 1), ('c', 3)) order by hizat";
        dbm::DbPgGenBaseInsert1(&mut db)
    ).await?;
    assert_eq!(res.len(), 2);
    assert_eq!(res[0], "a");
    assert_eq!(res[1], "c");
    Ok(())
}

#[tokio::test]
async fn test_repeated_param() -> Result<(), loga::Error> {
    good_module!(dbm, "pg_gen_repeated_param");
    let (mut db, _cont) = db().await?;
    dbm::migrate(&mut db, None).await?;
    good_ormning::pg::good_query!(
        "pg_gen_repeated_param",
        r#"
                    insert into "genrerank"
                        (
                            "date", 
                            "genre", 
                            "secondary", 
                            "sort", 
                            "rank", 
                            "track"
                        )
                        values (
                            $date,
                            $genre,
                            $secondary,
                            $sort,
                            $rank,
                            $track
                        )
                    on conflict ("genre", "secondary", "sort", "track")
                        do update set "date" = $date, "rank" = $rank
    "#;
        dbm::DbPgGenRepeatedParam1(&mut db),
        date: i32 = 20260501,
        genre: string = "rock",
        secondary: string = "classic",
        sort: i32 = 1,
        rank: i32 = 10,
        track: string = "song1"
    ).await?;
    good_ormning::pg::good_query!(
        "pg_gen_repeated_param",
        r#"
                    insert into "genrerank"
                        (
                            "date", 
                            "genre", 
                            "secondary", 
                            "sort", 
                            "rank", 
                            "track"
                        )
                        values (
                            $date,
                            $genre,
                            $secondary,
                            $sort,
                            $rank,
                            $track
                        )
                    on conflict ("genre", "secondary", "sort", "track")
                        do update set "date" = $date, "rank" = $rank
    "#;
        dbm::DbPgGenRepeatedParam1(&mut db),
        date: i32 = 20260502,
        genre: string = "rock",
        secondary: string = "classic",
        sort: i32 = 1,
        rank: i32 = 5,
        track: string = "song1"
    ).await?;
    assert_eq!(good_ormning::pg::good_query_one!(
        "pg_gen_repeated_param",
        r#"select "genrerank" . "date" as "date" from "genrerank""#;
        dbm::DbPgGenRepeatedParam1(&mut db)
    ).await?, 20260502);
    assert_eq!(good_ormning::pg::good_query_one!(
        "pg_gen_repeated_param",
        r#"select "genrerank" . "rank" as "rank" from "genrerank""#;
        dbm::DbPgGenRepeatedParam1(&mut db)
    ).await?, 5);
    Ok(())
}
