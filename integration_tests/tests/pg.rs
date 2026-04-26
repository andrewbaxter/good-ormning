use good_ormning::runtime::ToGoodError;
use good_ormning_macros::*;
use {
    chrono::{
        TimeZone,
        Utc,
    },
    integration_tests::MyString,
    pglite_oxide::PgliteServer,
};

pub mod pg_gen_base_insert;
pub mod pg_gen_param_i32;
pub mod pg_gen_param_utctime_chrono;
pub mod pg_gen_param_utctime_jiff;
pub mod pg_gen_param_opt_i32;
pub mod pg_gen_param_opt_i32_null;
pub mod pg_gen_param_custom;
pub mod pg_gen_param_opt_custom;
pub mod pg_gen_insert_on_conflict_do_nothing;
pub mod pg_gen_insert_on_conflict_update;
pub mod pg_gen_update;
pub mod pg_gen_update_where;
pub mod pg_gen_update_returning;
pub mod pg_gen_delete;
pub mod pg_gen_delete_where;
pub mod pg_gen_delete_returning;
pub mod pg_gen_select_join;
pub mod pg_gen_select_group_by;
pub mod pg_gen_select_order;
pub mod pg_gen_select_limit;
pub mod pg_gen_migrate_add_field;
pub mod pg_gen_migrate_rename_field;
pub mod pg_gen_migrate_remove_field;
pub mod pg_gen_migrate_add_table;
pub mod pg_gen_migrate_rename_table;
pub mod pg_gen_migrate_remove_table;
pub mod pg_gen_select_cte;
pub mod pg_gen_select_window;
pub mod pg_gen_migrate_make_field_optional;
pub mod pg_gen_migrate_pre_migration;

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
    let (mut db, _cont) = db().await?;
    pg_gen_base_insert::migrate(&mut db).await?;
    good_query_pg!(
        r#"insert into "bannanana" ( "hizat" ) values ( $1 )"#;
        text = &str;
        &mut db,
        "soy"
    ).await?;
    assert_eq!(good_query_one_pg!(
        r#"select "bannanana" . "hizat" as "hizat" from "bannanana""#;
        ;
        &mut db
    ).await?, "soy");
    Ok(())
}

#[tokio::test]
async fn test_param_i32() -> Result<(), loga::Error> {
    let (mut db, _cont) = db().await?;
    pg_gen_param_i32::migrate(&mut db).await?;
    good_query_pg!(
        r#"insert into "bananna_pg_gen_base_insert" ( "hizat" ) values ( $1 )"#;
        val = i32;
        &mut db,
        22
    ).await?;
    assert_eq!(good_query_one_pg!(
        r#"select "bananna_pg_gen_base_insert" . "hizat" as "hizat" from "bananna_pg_gen_base_insert""#;
        ;
        &mut db
    ).await?, 22);
    Ok(())
}

#[tokio::test]
async fn test_param_utctime_chrono() -> Result<(), loga::Error> {
    let (mut db, _cont) = db().await?;
    let ref_date = chrono::TimeZone::with_ymd_and_hms(&chrono::Utc, 1937, 12, 1, 0, 0, 0).unwrap();
    pg_gen_param_utctime_chrono::migrate(&mut db).await?;
    good_query_pg!(
        r#"insert into "bananna_pg_gen_param_i32" ( "hizat" ) values ( $1 )"#;
        val = chrono:: DateTime < chrono:: Utc >;
        &mut db,
        ref_date
    ).await?;
    assert_eq!(good_query_one_pg!(
        r#"select "bananna_pg_gen_param_i32" . "hizat" as "hizat" from "bananna_pg_gen_param_i32""#;
        ;
        &mut db
    ).await?, ref_date);
    Ok(())
}

#[tokio::test]
async fn test_param_utctime_jiff() -> Result<(), loga::Error> {
    let (mut db, _cont) = db().await?;
    let ref_date =
        jiff::civil::DateTime::new(1937, 12, 1, 0, 0, 0, 0)
            .unwrap()
            .to_zoned(jiff::tz::TimeZone::UTC)
            .unwrap()
            .timestamp();
    pg_gen_param_utctime_jiff::migrate(&mut db).await?;
    good_query_pg!(
        r#"insert into "bananna_pg_gen_param_utctime_chrono" ( "hizat" ) values ( $1 )"#;
        val = jiff::Timestamp;
        &mut db,
        ref_date
    ).await?;
    assert_eq!(good_query_one_pg!(
        r#"select "bananna_pg_gen_param_utctime_chrono" . "hizat" as "hizat" from "bananna_pg_gen_param_utctime_chrono""#;
        ;
        &mut db
    ).await?, ref_date);
    Ok(())
}

#[tokio::test]
async fn test_param_opt_i32() -> Result<(), loga::Error> {
    let (mut db, _cont) = db().await?;
    pg_gen_param_opt_i32::migrate(&mut db).await?;
    good_query_pg!(
        r#"insert into "bananna_pg_gen_param_utctime_jiff" ( "hizat" ) values ( $1 )"#;
        val = Option < i32 >;
        &mut db,
        Some(47)
    ).await?;
    assert_eq!(good_query_one_pg!(
        r#"select "bananna_pg_gen_param_utctime_jiff" . "hizat" as "hizat" from "bananna_pg_gen_param_utctime_jiff""#;
        ;
        &mut db
    ).await?, Some(47));
    Ok(())
}

#[tokio::test]
async fn test_param_opt_i32_null() -> Result<(), loga::Error> {
    let (mut db, _cont) = db().await?;
    pg_gen_param_opt_i32_null::migrate(&mut db).await?;
    good_query_pg!(
        r#"insert into "bananna_pg_gen_param_opt_i32" ( "hizat" ) values ( null )"#;
        ;
        &mut db
    ).await?;
    assert_eq!(good_query_one_pg!(
        r#"select "bananna_pg_gen_param_opt_i32" . "hizat" as "hizat" from "bananna_pg_gen_param_opt_i32""#;
        ;
        &mut db
    ).await?, None);
    Ok(())
}

#[tokio::test]
async fn test_param_custom() -> Result<(), loga::Error> {
    let (mut db, _cont) = db().await?;
    pg_gen_param_custom::migrate(&mut db).await?;
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
    good_query_pg!(
        r#"insert into "bananna_pg_gen_param_opt_i32_null" ( "x_0" , "x_1" , "x_2" , "x_3" , "x_4" , "x_5" , "x_6" , "x_7" , "x_8" , "x_9" , "x_10" , "x_11" ) values ( $1 , $2 , $3 , $4 , $5 , $6 , $7 , $8 , $9 , $10 , $11 , $12 )"#;
        x_0 = &integration_tests::MyBool,
        x_1 = &integration_tests::MyI32,
        x_2 = &integration_tests::MyI64,
        x_3 = &integration_tests::MyU32,
        x_4 = &integration_tests::MyF32,
        x_5 = &integration_tests::MyF64,
        x_6 = &integration_tests::MyBytes,
        x_7 = &integration_tests::MyString,
        x_8 = &integration_tests::MyUtctimeChrono,
        x_9 = &integration_tests::MyUtctimeChrono,
        x_10 = &integration_tests::MyUtctimeJiff,
        x_11 = &integration_tests::MyUtctimeJiff;
        &mut db,
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
    let res = good_query_one_pg!(
        r#"select "bananna_pg_gen_param_opt_i32_null" . "x_0" as "x_0" , "bananna_pg_gen_param_opt_i32_null" . "x_1" as "x_1" , "bananna_pg_gen_param_opt_i32_null" . "x_2" as "x_2" , "bananna_pg_gen_param_opt_i32_null" . "x_3" as "x_3" , "bananna_pg_gen_param_opt_i32_null" . "x_4" as "x_4" , "bananna_pg_gen_param_opt_i32_null" . "x_5" as "x_5" , "bananna_pg_gen_param_opt_i32_null" . "x_6" as "x_6" , "bananna_pg_gen_param_opt_i32_null" . "x_7" as "x_7" , "bananna_pg_gen_param_opt_i32_null" . "x_8" as "x_8" , "bananna_pg_gen_param_opt_i32_null" . "x_9" as "x_9" , "bananna_pg_gen_param_opt_i32_null" . "x_10" as "x_10" , "bananna_pg_gen_param_opt_i32_null" . "x_11" as "x_11" from "bananna_pg_gen_param_opt_i32_null""#;
        ;
        &mut db
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
    let (mut db, _cont) = db().await?;
    pg_gen_param_opt_custom::migrate(&mut db).await?;
    good_query_pg!(
        r#"insert into "bananna_pg_gen_param_custom" ( "hizat" ) values ( $1 )"#;
        text = Option <& integration_tests:: MyString >;
        &mut db,
        Some(&MyString("higgins".into()))
    ).await?;
    assert_eq!(good_query_one_pg!(
        r#"select "bananna_pg_gen_param_custom" . "hizat" as "hizat" from "bananna_pg_gen_param_custom""#;
        ;
        &mut db
    ).await?, Some(MyString("higgins".into())));
    Ok(())
}

#[tokio::test]
async fn test_insert_on_conflict_do_nothing() -> Result<(), loga::Error> {
    let (mut db, _cont) = db().await?;
    pg_gen_insert_on_conflict_do_nothing::migrate(&mut db).await?;
    assert!(good_query_opt_pg!(
        r#"insert into "bannanana" ( "hizat" ) values ( $1 ) on conflict do nothing returning 1 as "one""#;
        text = &str;
        &mut db,
        "soy"
    ).await?.is_some());
    assert!(good_query_opt_pg!(
        r#"insert into "bannanana" ( "hizat" ) values ( $1 ) on conflict do nothing returning 1 as "one""#;
        text = &str;
        &mut db,
        "soy"
    ).await?.is_none());
    Ok(())
}

#[tokio::test]
async fn test_insert_on_conflict_update() -> Result<(), loga::Error> {
    let (mut db, _cont) = db().await?;
    pg_gen_insert_on_conflict_update::migrate(&mut db).await?;
    assert_eq!(good_query_one_pg!(
        r#"insert into "bannanana" ( "hizat" , "two" ) values ( $1 , $2 ) on conflict ( "hizat" ) do update set "two" = "bannanana" . "two" + 1 returning "bannanana" . "two" as "two""#;
        text = &str,
        two = i32;
        &mut db,
        "soy",
        33
    ).await?, 33);
    assert_eq!(good_query_one_pg!(
        r#"insert into "bannanana" ( "hizat" , "two" ) values ( $1 , $2 ) on conflict ( "hizat" ) do update set "two" = "bannanana" . "two" + 1 returning "bannanana" . "two" as "two""#;
        text = &str,
        two = i32;
        &mut db,
        "soy",
        7
    ).await?, 34);
    assert_eq!(good_query_one_pg!(
        r#"insert into "bannanana" ( "hizat" , "two" ) values ( $1 , $2 ) on conflict ( "hizat" ) do update set "two" = "bannanana" . "two" + 1 returning "bannanana" . "two" as "two""#;
        text = &str,
        two = i32;
        &mut db,
        "yyyy",
        7
    ).await?, 7);
    Ok(())
}

#[tokio::test]
async fn test_update() -> Result<(), loga::Error> {
    let (mut db, _cont) = db().await?;
    pg_gen_update::migrate(&mut db).await?;
    good_query_pg!(
        r#"insert into "bananna_pg_gen_param_opt_custom" ( "hizat" ) values ( 'yog' )"#;
        ;
        &mut db
    ).await?;
    assert_eq!(good_query_one_pg!(
        r#"select "bananna_pg_gen_param_opt_custom" . "hizat" as "hizat" from "bananna_pg_gen_param_opt_custom""#;
        ;
        &mut db
    ).await?, "yog");
    good_query_pg!(
        r#"update "bananna_pg_gen_param_opt_custom" set "hizat" = 'tep'"#;
        ;
        &mut db
    ).await?;
    assert_eq!(good_query_one_pg!(
        r#"select "bananna_pg_gen_param_opt_custom" . "hizat" as "hizat" from "bananna_pg_gen_param_opt_custom""#;
        ;
        &mut db
    ).await?, "tep");
    Ok(())
}

#[tokio::test]
async fn test_update_where() -> Result<(), loga::Error> {
    let (mut db, _cont) = db().await?;
    pg_gen_update_where::migrate(&mut db).await?;
    good_query_pg!(
        r#"insert into "ban" ( "hizat" ) values ( 'yog' )"#;
        ;
        &mut db
    ).await?;
    assert_eq!(good_query_one_pg!(
        r#"select "ban" . "hizat" as "hizat" from "ban""#;
        ;
        &mut db
    ).await?, "yog");
    good_query_pg!(
        r#"update "ban" set "hizat" = $1 where "ban" . "hizat" = $2"#;
        val = &str,
        cond = &str;
        &mut db,
        "tep",
        "yog2"
    ).await?;
    assert_eq!(good_query_one_pg!(
        r#"select "ban" . "hizat" as "hizat" from "ban""#;
        ;
        &mut db
    ).await?, "yog");
    good_query_pg!(
        r#"update "ban" set "hizat" = $1 where "ban" . "hizat" = $2"#;
        val = &str,
        cond = &str;
        &mut db,
        "tep",
        "yog"
    ).await?;
    assert_eq!(good_query_one_pg!(
        r#"select "ban" . "hizat" as "hizat" from "ban""#;
        ;
        &mut db
    ).await?, "tep");
    Ok(())
}

#[tokio::test]
async fn test_update_returning() -> Result<(), loga::Error> {
    let (mut db, _cont) = db().await?;
    pg_gen_update_returning::migrate(&mut db).await?;
    good_query_pg!(
        r#"insert into "b" ( "hizat" ) values ( 'yog' )"#;
        ;
        &mut db
    ).await?;
    assert_eq!(good_query_opt_pg!(
        r#"update "b" set "hizat" = 'tep' returning "b" . "hizat" as "hizat""#;
        ;
        &mut db
    ).await?, Some("tep".to_string()));
    Ok(())
}

#[tokio::test]
async fn test_delete() -> Result<(), loga::Error> {
    let (mut db, _cont) = db().await?;
    pg_gen_delete::migrate(&mut db).await?;
    good_query_pg!(
        r#"insert into "b" ( "hizat" ) values ( 'seeon' )"#;
        ;
        &mut db
    ).await?;
    assert_eq!(good_query_opt_pg!(
        r#"select "b" . "hizat" as "hizat" from "b""#;
        ;
        &mut db
    ).await?, Some("seeon".to_string()));
    good_query_pg!(
        r#"delete from "b""#;
        ;
        &mut db
    ).await?;
    assert_eq!(good_query_opt_pg!(
        r#"select "b" . "hizat" as "hizat" from "b""#;
        ;
        &mut db
    ).await?, None);
    Ok(())
}

#[tokio::test]
async fn test_delete_where() -> Result<(), loga::Error> {
    let (mut db, _cont) = db().await?;
    pg_gen_delete_where::migrate(&mut db).await?;
    good_query_pg!(
        r#"insert into "ba" ( "hizat" ) values ( 'seeon' )"#;
        ;
        &mut db
    ).await?;
    good_query_pg!(
        r#"delete from "ba" where "ba" . "hizat" = $1"#;
        hiz = &str;
        &mut db,
        "nozo"
    ).await?;
    assert_eq!(good_query_opt_pg!(
        r#"select "ba" . "hizat" as "hizat" from "ba""#;
        ;
        &mut db
    ).await?, Some("seeon".to_string()));
    good_query_pg!(
        r#"delete from "ba" where "ba" . "hizat" = $1"#;
        hiz = &str;
        &mut db,
        "seeon"
    ).await?;
    assert_eq!(good_query_opt_pg!(
        r#"select "ba" . "hizat" as "hizat" from "ba""#;
        ;
        &mut db
    ).await?, None);
    Ok(())
}

#[tokio::test]
async fn test_delete_returning() -> Result<(), loga::Error> {
    let (mut db, _cont) = db().await?;
    pg_gen_delete_where::migrate(&mut db).await?;
    good_query_pg!(
        r#"insert into "ba" ( "hizat" ) values ( 'seeon' )"#;
        ;
        &mut db
    ).await?;
    assert!(good_query_opt_pg!(
        r#"select "ba" . "hizat" as "hizat" from "ba""#;
        ;
        &mut db
    ).await?.is_some());
    good_query_pg!(
        r#"delete from "ba" where "ba" . "hizat" = $1"#;
        hiz = &str;
        &mut db,
        "seeon"
    ).await?;
    assert!(good_query_opt_pg!(
        r#"select "ba" . "hizat" as "hizat" from "ba""#;
        ;
        &mut db
    ).await?.is_none());
    Ok(())
}

#[tokio::test]
async fn test_select_join() -> Result<(), loga::Error> {
    let (mut db, _cont) = db().await?;
    pg_gen_select_join::migrate(&mut db).await?;
    let res = good_query_one_pg!(
        r#"select "b" . "three" as "three" , "two_pg_gen_delete_returning" . "two" as "two" from "b" left join "two_pg_gen_delete_returning" on ( "b" . "hizat" :: text ) = "two_pg_gen_delete_returning" . "hizat""#;
        ;
        &mut db
    ).await?;
    assert_eq!(res.three, 33);
    assert_eq!(res.two, Some("no".into()));
    Ok(())
}

#[tokio::test]
async fn test_select_group_by() -> Result<(), loga::Error> {
    let (mut db, _cont) = db().await?;
    pg_gen_select_group_by::migrate(&mut db).await?;
    good_query_pg!(
        r#"insert into "bannanana" ( "hizat" , "hizat2" ) values ( $1 , $2 )"#;
        v = i32,
        v2 = i32;
        &mut db,
        1,
        7
    ).await?;
    good_query_pg!(
        r#"insert into "bannanana" ( "hizat" , "hizat2" ) values ( $1 , $2 )"#;
        v = i32,
        v2 = i32;
        &mut db,
        1,
        99
    ).await?;
    good_query_pg!(
        r#"insert into "bannanana" ( "hizat" , "hizat2" ) values ( $1 , $2 )"#;
        v = i32,
        v2 = i32;
        &mut db,
        2,
        3
    ).await?;
    good_query_pg!(
        r#"insert into "bannanana" ( "hizat" , "hizat2" ) values ( $1 , $2 )"#;
        v = i32,
        v2 = i32;
        &mut db,
        2,
        10
    ).await?;
    let mut res = good_query_many_pg!(
        r#"select sum ( "bannanana" . "hizat2" ) as "hizat2" from "bannanana" group by "bannanana" . "hizat""#;
        ;
        &mut db
    ).await?;
    res.sort();
    assert_eq!(res, vec![13i64, 106i64]);
    Ok(())
}

#[tokio::test]
async fn test_select_limit() -> Result<(), loga::Error> {
    let (mut db, _cont) = db().await?;
    pg_gen_select_limit::migrate(&mut db).await?;
    good_query_pg!(
        r#"insert into "bannanana" ( "hizat" ) values ( $1 )"#;
        text = &str;
        &mut db,
        "soy"
    ).await?;
    good_query_pg!(
        r#"insert into "bannanana" ( "hizat" ) values ( $1 )"#;
        text = &str;
        &mut db,
        "soy"
    ).await?;
    good_query_pg!(
        r#"insert into "bannanana" ( "hizat" ) values ( $1 )"#;
        text = &str;
        &mut db,
        "soy"
    ).await?;
    assert_eq!(good_query_many_pg!(
        r#"select "bannanana" . "hizat" as "hizat" from "bannanana" limit 2"#;
        ;
        &mut db
    ).await?.len(), 2);
    Ok(())
}

#[tokio::test]
async fn test_select_order() -> Result<(), loga::Error> {
    let (mut db, _cont) = db().await?;
    pg_gen_select_order::migrate(&mut db).await?;
    good_query_pg!(
        r#"insert into "bannanana" ( "hizat" ) values ( $1 )"#;
        v = i32;
        &mut db,
        0
    ).await?;
    good_query_pg!(
        r#"insert into "bannanana" ( "hizat" ) values ( $1 )"#;
        v = i32;
        &mut db,
        12
    ).await?;
    good_query_pg!(
        r#"insert into "bannanana" ( "hizat" ) values ( $1 )"#;
        v = i32;
        &mut db,
        9
    ).await?;
    assert_eq!(good_query_many_pg!(
        r#"select "bannanana" . "hizat" as "hizat" from "bannanana" order by "bannanana" . "hizat" asc"#;
        ;
        &mut db
    ).await?, vec![0, 9, 12]);
    Ok(())
}

#[tokio::test]
async fn test_migrate_add_field() -> Result<(), loga::Error> {
    let (mut db, _cont) = db().await?;
    pg_gen_migrate_add_field::migrate(&mut db).await?;
    match good_query_opt_pg!(
        r#"select "bannna" . "hizat" as "hizat" , "bannna" . "zomzom" as "zomzom" from "bannna""#;
        ;
        &mut db
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
    let (mut db, _cont) = db().await?;
    pg_gen_migrate_rename_field::migrate(&mut db).await?;
    good_query_pg!(
        r#"insert into "bannna" ( "hizat" ) values ( 'nizoot' )"#;
        ;
        &mut db
    ).await?;
    Ok(())
}

#[tokio::test]
async fn test_migrate_remove_field() -> Result<(), loga::Error> {
    let (mut db, _cont) = db().await?;
    pg_gen_migrate_remove_field::migrate(&mut db).await?;
    good_query_pg!(
        r#"insert into "bnanaa" ( "hizat" ) values ( $1 )"#;
        okolor = &str;
        &mut db,
        "yordol"
    ).await?;
    Ok(())
}

#[tokio::test]
async fn test_migrate_add_table() -> Result<(), loga::Error> {
    let (mut db, _cont) = db().await?;
    pg_gen_migrate_add_table::migrate(&mut db).await?;
    good_query_pg!(
        r#"insert into "two_pg_gen_migrate_remove_field" ( "two" ) values ( $1 )"#;
        two = i32;
        &mut db,
        23
    ).await?;
    Ok(())
}

#[tokio::test]
async fn test_migrate_rename_table() -> Result<(), loga::Error> {
    let (mut db, _cont) = db().await?;
    pg_gen_migrate_rename_table::migrate(&mut db).await?;
    good_query_pg!(
        r#"insert into "bana" ( "hizat" ) values ( $1 )"#;
        two = &str;
        &mut db,
        "inset"
    ).await?;
    Ok(())
}

#[tokio::test]
async fn test_migrate_remove_table() -> Result<(), loga::Error> {
    let (mut db, _cont) = db().await?;
    pg_gen_migrate_remove_table::migrate(&mut db).await?;
    Ok(())
}

#[tokio::test]
async fn test_select_cte() -> Result<(), loga::Error> {
    let (mut db, _cont) = db().await?;
    pg_gen_select_cte::migrate(&mut db).await?;
    good_query_pg!(
        r#"insert into "bannanana" ( "hizat" , "hizat2" ) values ( $1 , $2 )"#;
        v = i32,
        v2 = i32;
        &mut db,
        1,
        7
    ).await?;
    good_query_pg!(
        r#"insert into "bannanana" ( "hizat" , "hizat2" ) values ( $1 , $2 )"#;
        v = i32,
        v2 = i32;
        &mut db,
        1,
        99
    ).await?;
    let mut res = good_query_many_pg!(
        r#"with "hibbo" ( "zathi" ) as ( select "bannanana" . "hizat2" as "hizat2" from "bannanana" ) select "hibbo" . "zathi" as "zathi" from "hibbo""#;
        ;
        &mut db
    ).await?;
    res.sort();
    assert_eq!(res, vec![7, 99]);
    Ok(())
}

#[tokio::test]
async fn test_select_window() -> Result<(), loga::Error> {
    let (mut db, _cont) = db().await?;
    pg_gen_select_window::migrate(&mut db).await?;
    good_query_pg!(
        r#"insert into "bannanana" ( "hizat" , "hizat2" ) values ( $1 , $2 )"#;
        v = i32,
        v2 = i32;
        &mut db,
        1,
        7
    ).await?;
    good_query_pg!(
        r#"insert into "bannanana" ( "hizat" , "hizat2" ) values ( $1 , $2 )"#;
        v = i32,
        v2 = i32;
        &mut db,
        1,
        99
    ).await?;
    good_query_pg!(
        r#"insert into "bannanana" ( "hizat" , "hizat2" ) values ( $1 , $2 )"#;
        v = i32,
        v2 = i32;
        &mut db,
        2,
        3
    ).await?;
    good_query_pg!(
        r#"insert into "bannanana" ( "hizat" , "hizat2" ) values ( $1 , $2 )"#;
        v = i32,
        v2 = i32;
        &mut db,
        2,
        10
    ).await?;
    let mut res = good_query_many_pg!(
        r#"select sum ( "bannanana" . "hizat2" ) over ( partition by "bannanana" . "hizat" ) as "hizat2" from "bannanana""#;
        ;
        &mut db
    ).await?.into_iter().collect::<Vec<_>>();
    res.sort();
    assert_eq!(res, vec![13i64, 13i64, 106i64, 106i64]);
    Ok(())
}

#[tokio::test]
async fn test_migrate_make_field_optional() -> Result<(), loga::Error> {
    let (mut db, _cont) = db().await?;
    pg_gen_migrate_make_field_optional::migrate(&mut db).await?;
    Ok(())
}

#[tokio::test]
async fn test_migrate_pre_migration() -> Result<(), loga::Error> {
    let (mut db, _cont) = db().await?;
    pg_gen_migrate_pre_migration::migrate(&mut db).await?;
    Ok(())
}
