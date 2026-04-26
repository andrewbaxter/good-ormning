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

pub mod pg_gen_base_insert { include!(concat!(env!("OUT_DIR"), "/good_ormning_pg_pg_gen_base_insert.rs")); }
pub mod pg_gen_param_i32 { include!(concat!(env!("OUT_DIR"), "/good_ormning_pg_pg_gen_param_i32.rs")); }
pub mod pg_gen_param_utctime_chrono { include!(concat!(env!("OUT_DIR"), "/good_ormning_pg_pg_gen_param_utctime_chrono.rs")); }
pub mod pg_gen_param_utctime_jiff { include!(concat!(env!("OUT_DIR"), "/good_ormning_pg_pg_gen_param_utctime_jiff.rs")); }
pub mod pg_gen_param_opt_i32 { include!(concat!(env!("OUT_DIR"), "/good_ormning_pg_pg_gen_param_opt_i32.rs")); }
pub mod pg_gen_param_opt_i32_null { include!(concat!(env!("OUT_DIR"), "/good_ormning_pg_pg_gen_param_opt_i32_null.rs")); }
pub mod pg_gen_param_custom { include!(concat!(env!("OUT_DIR"), "/good_ormning_pg_pg_gen_param_custom.rs")); }
pub mod pg_gen_param_opt_custom { include!(concat!(env!("OUT_DIR"), "/good_ormning_pg_pg_gen_param_opt_custom.rs")); }
pub mod pg_gen_insert_on_conflict_do_nothing { include!(concat!(env!("OUT_DIR"), "/good_ormning_pg_pg_gen_insert_on_conflict_do_nothing.rs")); }
pub mod pg_gen_insert_on_conflict_update { include!(concat!(env!("OUT_DIR"), "/good_ormning_pg_pg_gen_insert_on_conflict_update.rs")); }
pub mod pg_gen_update { include!(concat!(env!("OUT_DIR"), "/good_ormning_pg_pg_gen_update.rs")); }
pub mod pg_gen_update_where { include!(concat!(env!("OUT_DIR"), "/good_ormning_pg_pg_gen_update_where.rs")); }
pub mod pg_gen_update_returning { include!(concat!(env!("OUT_DIR"), "/good_ormning_pg_pg_gen_update_returning.rs")); }
pub mod pg_gen_delete { include!(concat!(env!("OUT_DIR"), "/good_ormning_pg_pg_gen_delete.rs")); }
pub mod pg_gen_delete_where { include!(concat!(env!("OUT_DIR"), "/good_ormning_pg_pg_gen_delete_where.rs")); }
pub mod pg_gen_delete_returning { include!(concat!(env!("OUT_DIR"), "/good_ormning_pg_pg_gen_delete_returning.rs")); }
pub mod pg_gen_select_join { include!(concat!(env!("OUT_DIR"), "/good_ormning_pg_pg_gen_select_join.rs")); }
pub mod pg_gen_select_group_by { include!(concat!(env!("OUT_DIR"), "/good_ormning_pg_pg_gen_select_group_by.rs")); }
pub mod pg_gen_select_order { include!(concat!(env!("OUT_DIR"), "/good_ormning_pg_pg_gen_select_order.rs")); }
pub mod pg_gen_select_limit { include!(concat!(env!("OUT_DIR"), "/good_ormning_pg_pg_gen_select_limit.rs")); }
pub mod pg_gen_migrate_add_field { include!(concat!(env!("OUT_DIR"), "/good_ormning_pg_pg_gen_migrate_add_field.rs")); }
pub mod pg_gen_migrate_rename_field { include!(concat!(env!("OUT_DIR"), "/good_ormning_pg_pg_gen_migrate_rename_field.rs")); }
pub mod pg_gen_migrate_remove_field { include!(concat!(env!("OUT_DIR"), "/good_ormning_pg_pg_gen_migrate_remove_field.rs")); }
pub mod pg_gen_migrate_add_table { include!(concat!(env!("OUT_DIR"), "/good_ormning_pg_pg_gen_migrate_add_table.rs")); }
pub mod pg_gen_migrate_rename_table { include!(concat!(env!("OUT_DIR"), "/good_ormning_pg_pg_gen_migrate_rename_table.rs")); }
pub mod pg_gen_migrate_remove_table { include!(concat!(env!("OUT_DIR"), "/good_ormning_pg_pg_gen_migrate_remove_table.rs")); }
pub mod pg_gen_select_cte { include!(concat!(env!("OUT_DIR"), "/good_ormning_pg_pg_gen_select_cte.rs")); }
pub mod pg_gen_select_window { include!(concat!(env!("OUT_DIR"), "/good_ormning_pg_pg_gen_select_window.rs")); }
pub mod pg_gen_migrate_make_field_optional { include!(concat!(env!("OUT_DIR"), "/good_ormning_pg_pg_gen_migrate_make_field_optional.rs")); }
pub mod pg_gen_migrate_pre_migration { include!(concat!(env!("OUT_DIR"), "/good_ormning_pg_pg_gen_migrate_pre_migration.rs")); }

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
    good_query_pg!("pg_gen_base_insert", 
        r#"insert into "base_insert_bannanana" ( "hizat" ) values ( $1 )"#;
        p0 = string;
        &mut db,
        "soy"
    ).await?;
    assert_eq!(good_query_one_pg!("pg_gen_base_insert", 
        r#"select "base_insert_bannanana" . "hizat" as "hizat" from "base_insert_bannanana""#;
        ;
        &mut db
    ).await?, "soy");
    Ok(())
}

#[tokio::test]
async fn test_param_i32() -> Result<(), loga::Error> {
    let (mut db, _cont) = db().await?;
    pg_gen_param_i32::migrate(&mut db).await?;
    good_query_pg!("pg_gen_param_i32", 
        r#"insert into "param_i32_bananna" ( "hizat" ) values ( $1 )"#;
        p0 = i32;
        &mut db,
        22
    ).await?;
    assert_eq!(good_query_one_pg!("pg_gen_param_i32", 
        r#"select "param_i32_bananna" . "hizat" as "hizat" from "param_i32_bananna""#;
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
    good_query_pg!("pg_gen_param_utctime_chrono", 
        r#"insert into "param_utctime_chrono_bananna" ( "hizat" ) values ( $1 )"#;
        p0 = utctime_s_chrono;
        &mut db,
        ref_date
    ).await?;
    assert_eq!(good_query_one_pg!("pg_gen_param_utctime_chrono", 
        r#"select "param_utctime_chrono_bananna" . "hizat" as "hizat" from "param_utctime_chrono_bananna""#;
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
    good_query_pg!("pg_gen_param_utctime_jiff", 
        r#"insert into "param_utctime_jiff_bananna" ( "hizat" ) values ( $1 )"#;
        p0 = utctime_s_jiff;
        &mut db,
        ref_date
    ).await?;
    assert_eq!(good_query_one_pg!("pg_gen_param_utctime_jiff", 
        r#"select "param_utctime_jiff_bananna" . "hizat" as "hizat" from "param_utctime_jiff_bananna""#;
        ;
        &mut db
    ).await?, ref_date);
    Ok(())
}

#[tokio::test]
async fn test_param_opt_i32() -> Result<(), loga::Error> {
    let (mut db, _cont) = db().await?;
    pg_gen_param_opt_i32::migrate(&mut db).await?;
    good_query_pg!("pg_gen_param_opt_i32", 
        r#"insert into "param_opt_i32_bananna" ( "hizat" ) values ( $1 )"#;
        p0 = opt i32;
        &mut db,
        Some(47)
    ).await?;
    assert_eq!(good_query_one_pg!("pg_gen_param_opt_i32", 
        r#"select "param_opt_i32_bananna" . "hizat" as "hizat" from "param_opt_i32_bananna""#;
        ;
        &mut db
    ).await?, Some(47));
    Ok(())
}

#[tokio::test]
async fn test_param_opt_i32_null() -> Result<(), loga::Error> {
    let (mut db, _cont) = db().await?;
    pg_gen_param_opt_i32_null::migrate(&mut db).await?;
    good_query_pg!("pg_gen_param_opt_i32_null", 
        r#"insert into "param_opt_i32_null_bananna" ( "hizat" ) values ( null )"#;
        ;
        &mut db
    ).await?;
    assert_eq!(good_query_one_pg!("pg_gen_param_opt_i32_null", 
        r#"select "param_opt_i32_null_bananna" . "hizat" as "hizat" from "param_opt_i32_null_bananna""#;
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
    good_query_pg!("pg_gen_param_custom", 
        r#"insert into "param_custom_bananna" ( "x_0" , "x_1" , "x_2" , "x_3" , "x_4" , "x_5" , "x_6" , "x_7" , "x_8" , "x_9" , "x_10" , "x_11" ) values ( $1 , $2 , $3 , $4 , $5 , $6 , $7 , $8 , $9 , $10 , $11 , $12 )"#;
        p0 = MyBool,
        p1 = MyI32,
        p2 = MyI64,
        p3 = MyU32,
        p4 = MyF32,
        p5 = MyF64,
        p6 = MyBytes,
        p7 = MyString,
        p8 = MyUtctimeChrono,
        p9 = MyUtctimeChrono,
        p10 = MyUtctimeJiff,
        p11 = MyUtctimeJiff;
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
    let res = good_query_one_pg!("pg_gen_param_custom", 
        r#"select "param_custom_bananna" . "x_0" as "x_0" , "param_custom_bananna" . "x_1" as "x_1" , "param_custom_bananna" . "x_2" as "x_2" , "param_custom_bananna" . "x_3" as "x_3" , "param_custom_bananna" . "x_4" as "x_4" , "param_custom_bananna" . "x_5" as "x_5" , "param_custom_bananna" . "x_6" as "x_6" , "param_custom_bananna" . "x_7" as "x_7" , "param_custom_bananna" . "x_8" as "x_8" , "param_custom_bananna" . "x_9" as "x_9" , "param_custom_bananna" . "x_10" as "x_10" , "param_custom_bananna" . "x_11" as "x_11" from "param_custom_bananna""#;
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
    good_query_pg!("pg_gen_param_opt_custom", 
        r#"insert into "param_opt_custom_bananna" ( "hizat" ) values ( $1 )"#;
        p0 = opt MyString;
        &mut db,
        Some(&MyString("higgins".into()))
    ).await?;
    assert_eq!(good_query_one_pg!("pg_gen_param_opt_custom", 
        r#"select "param_opt_custom_bananna" . "hizat" as "hizat" from "param_opt_custom_bananna""#;
        ;
        &mut db
    ).await?, Some(MyString("higgins".into())));
    Ok(())
}

#[tokio::test]
async fn test_insert_on_conflict_do_nothing() -> Result<(), loga::Error> {
    let (mut db, _cont) = db().await?;
    pg_gen_insert_on_conflict_do_nothing::migrate(&mut db).await?;
    assert!(good_query_opt_pg!("pg_gen_insert_on_conflict_do_nothing", 
        r#"insert into "insert_on_conflict_do_nothing_bananna" ( "hizat" ) values ( $1 ) on conflict do nothing returning 1 as "one""#;
        p0 = string;
        &mut db,
        "soy"
    ).await?.is_some());
    assert!(good_query_opt_pg!("pg_gen_insert_on_conflict_do_nothing", 
        r#"insert into "insert_on_conflict_do_nothing_bananna" ( "hizat" ) values ( $1 ) on conflict do nothing returning 1 as "one""#;
        p0 = string;
        &mut db,
        "soy"
    ).await?.is_none());
    Ok(())
}

#[tokio::test]
async fn test_insert_on_conflict_update() -> Result<(), loga::Error> {
    let (mut db, _cont) = db().await?;
    pg_gen_insert_on_conflict_update::migrate(&mut db).await?;
    assert_eq!(good_query_one_pg!("pg_gen_insert_on_conflict_update", 
        r#"insert into "insert_on_conflict_update_bananna" ( "hizat" , "two" ) values ( $1 , $2 ) on conflict ( "hizat" ) do update set "two" = "insert_on_conflict_update_bananna" . "two" + 1 returning "insert_on_conflict_update_bananna" . "two" as "two""#;
        p0 = string,
        p1 = i32;
        &mut db,
        "soy",
        33
    ).await?, 33);
    assert_eq!(good_query_one_pg!("pg_gen_insert_on_conflict_update", 
        r#"insert into "insert_on_conflict_update_bananna" ( "hizat" , "two" ) values ( $1 , $2 ) on conflict ( "hizat" ) do update set "two" = "insert_on_conflict_update_bananna" . "two" + 1 returning "insert_on_conflict_update_bananna" . "two" as "two""#;
        p0 = string,
        p1 = i32;
        &mut db,
        "soy",
        7
    ).await?, 34);
    assert_eq!(good_query_one_pg!("pg_gen_insert_on_conflict_update", 
        r#"insert into "insert_on_conflict_update_bananna" ( "hizat" , "two" ) values ( $1 , $2 ) on conflict ( "hizat" ) do update set "two" = "insert_on_conflict_update_bananna" . "two" + 1 returning "insert_on_conflict_update_bananna" . "two" as "two""#;
        p0 = string,
        p1 = i32;
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
    good_query_pg!("pg_gen_update", 
        r#"insert into "update_bananna" ( "hizat" ) values ( 'yog' )"#;
        ;
        &mut db
    ).await?;
    assert_eq!(good_query_one_pg!("pg_gen_update", 
        r#"select "update_bananna" . "hizat" as "hizat" from "update_bananna""#;
        ;
        &mut db
    ).await?, "yog");
    good_query_pg!("pg_gen_update", 
        r#"update "update_bananna" set "hizat" = 'tep'"#;
        ;
        &mut db
    ).await?;
    assert_eq!(good_query_one_pg!("pg_gen_update", 
        r#"select "update_bananna" . "hizat" as "hizat" from "update_bananna""#;
        ;
        &mut db
    ).await?, "tep");
    Ok(())
}

#[tokio::test]
async fn test_update_where() -> Result<(), loga::Error> {
    let (mut db, _cont) = db().await?;
    pg_gen_update_where::migrate(&mut db).await?;
    good_query_pg!("pg_gen_update_where", 
        r#"insert into "update_where_ban" ( "hizat" ) values ( 'yog' )"#;
        ;
        &mut db
    ).await?;
    assert_eq!(good_query_one_pg!("pg_gen_update_where", 
        r#"select "update_where_ban" . "hizat" as "hizat" from "update_where_ban""#;
        ;
        &mut db
    ).await?, "yog");
    good_query_pg!("pg_gen_update_where", 
        r#"update "update_where_ban" set "hizat" = $1 where "update_where_ban" . "hizat" = $2"#;
        p0 = string,
        p1 = string;
        &mut db,
        "tep",
        "yog2"
    ).await?;
    assert_eq!(good_query_one_pg!("pg_gen_update_where", 
        r#"select "update_where_ban" . "hizat" as "hizat" from "update_where_ban""#;
        ;
        &mut db
    ).await?, "yog");
    good_query_pg!("pg_gen_update_where", 
        r#"update "update_where_ban" set "hizat" = $1 where "update_where_ban" . "hizat" = $2"#;
        p0 = string,
        p1 = string;
        &mut db,
        "tep",
        "yog"
    ).await?;
    assert_eq!(good_query_one_pg!("pg_gen_update_where", 
        r#"select "update_where_ban" . "hizat" as "hizat" from "update_where_ban""#;
        ;
        &mut db
    ).await?, "tep");
    Ok(())
}

#[tokio::test]
async fn test_update_returning() -> Result<(), loga::Error> {
    let (mut db, _cont) = db().await?;
    pg_gen_update_returning::migrate(&mut db).await?;
    good_query_pg!("pg_gen_update_returning", 
        r#"insert into "update_returning_b" ( "hizat" ) values ( 'yog' )"#;
        ;
        &mut db
    ).await?;
    assert_eq!(good_query_opt_pg!("pg_gen_update_returning", 
        r#"update "update_returning_b" set "hizat" = 'tep' returning "update_returning_b" . "hizat" as "hizat""#;
        ;
        &mut db
    ).await?, Some("tep".to_string()));
    Ok(())
}

#[tokio::test]
async fn test_delete() -> Result<(), loga::Error> {
    let (mut db, _cont) = db().await?;
    pg_gen_delete::migrate(&mut db).await?;
    good_query_pg!("pg_gen_delete", 
        r#"insert into "delete_b" ( "hizat" ) values ( 'seeon' )"#;
        ;
        &mut db
    ).await?;
    assert_eq!(good_query_opt_pg!("pg_gen_delete", 
        r#"select "delete_b" . "hizat" as "hizat" from "delete_b""#;
        ;
        &mut db
    ).await?, Some("seeon".to_string()));
    good_query_pg!("pg_gen_delete", 
        r#"delete from "delete_b""#;
        ;
        &mut db
    ).await?;
    assert_eq!(good_query_opt_pg!("pg_gen_delete", 
        r#"select "delete_b" . "hizat" as "hizat" from "delete_b""#;
        ;
        &mut db
    ).await?, None);
    Ok(())
}

#[tokio::test]
async fn test_delete_where() -> Result<(), loga::Error> {
    let (mut db, _cont) = db().await?;
    pg_gen_delete_where::migrate(&mut db).await?;
    good_query_pg!("pg_gen_delete_where", 
        r#"insert into "delete_where_ba" ( "hizat" ) values ( 'seeon' )"#;
        ;
        &mut db
    ).await?;
    good_query_pg!("pg_gen_delete_where", 
        r#"delete from "delete_where_ba" where "delete_where_ba" . "hizat" = $1"#;
        p0 = string;
        &mut db,
        "nozo"
    ).await?;
    assert_eq!(good_query_opt_pg!("pg_gen_delete_where", 
        r#"select "delete_where_ba" . "hizat" as "hizat" from "delete_where_ba""#;
        ;
        &mut db
    ).await?, Some("seeon".to_string()));
    good_query_pg!("pg_gen_delete_where", 
        r#"delete from "delete_where_ba" where "delete_where_ba" . "hizat" = $1"#;
        p0 = string;
        &mut db,
        "seeon"
    ).await?;
    assert_eq!(good_query_opt_pg!("pg_gen_delete_where", 
        r#"select "delete_where_ba" . "hizat" as "hizat" from "delete_where_ba""#;
        ;
        &mut db
    ).await?, None);
    Ok(())
}

#[tokio::test]
async fn test_delete_returning() -> Result<(), loga::Error> {
    let (mut db, _cont) = db().await?;
    pg_gen_delete_where::migrate(&mut db).await?;
    good_query_pg!("pg_gen_delete_where", 
        r#"insert into "delete_where_ba" ( "hizat" ) values ( 'seeon' )"#;
        ;
        &mut db
    ).await?;
    assert!(good_query_opt_pg!("pg_gen_delete_where", 
        r#"select "delete_where_ba" . "hizat" as "hizat" from "delete_where_ba""#;
        ;
        &mut db
    ).await?.is_some());
    good_query_pg!("pg_gen_delete_where", 
        r#"delete from "delete_where_ba" where "delete_where_ba" . "hizat" = $1"#;
        p0 = string;
        &mut db,
        "seeon"
    ).await?;
    assert!(good_query_opt_pg!("pg_gen_delete_where", 
        r#"select "delete_where_ba" . "hizat" as "hizat" from "delete_where_ba""#;
        ;
        &mut db
    ).await?.is_none());
    Ok(())
}

#[tokio::test]
async fn test_select_join() -> Result<(), loga::Error> {
    let (mut db, _cont) = db().await?;
    pg_gen_select_join::migrate(&mut db).await?;
    let res = good_query_one_pg!("pg_gen_select_join", 
        r#"select "select_join_b" . "three" as "three" , "select_join_two" . "two" as "two" from "select_join_b" left join "select_join_two" on ( "select_join_b" . "hizat" :: text ) = "select_join_two" . "hizat""#;
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
    good_query_pg!("pg_gen_select_group_by", 
        r#"insert into "select_group_by_bannanana" ( "hizat" , "hizat2" ) values ( $1 , $2 )"#;
        p0 = i32,
        p1 = i32;
        &mut db,
        1,
        7
    ).await?;
    good_query_pg!("pg_gen_select_group_by", 
        r#"insert into "select_group_by_bannanana" ( "hizat" , "hizat2" ) values ( $1 , $2 )"#;
        p0 = i32,
        p1 = i32;
        &mut db,
        1,
        99
    ).await?;
    good_query_pg!("pg_gen_select_group_by", 
        r#"insert into "select_group_by_bannanana" ( "hizat" , "hizat2" ) values ( $1 , $2 )"#;
        p0 = i32,
        p1 = i32;
        &mut db,
        2,
        3
    ).await?;
    good_query_pg!("pg_gen_select_group_by", 
        r#"insert into "select_group_by_bannanana" ( "hizat" , "hizat2" ) values ( $1 , $2 )"#;
        p0 = i32,
        p1 = i32;
        &mut db,
        2,
        10
    ).await?;
    let mut res = good_query_many_pg!("pg_gen_select_group_by", 
        r#"select sum ( "select_group_by_bannanana" . "hizat2" ) as "hizat2" from "select_group_by_bannanana" group by "select_group_by_bannanana" . "hizat""#;
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
    good_query_pg!("pg_gen_select_limit", 
        r#"insert into "select_limit_bannanana" ( "hizat" ) values ( $1 )"#;
        p0 = string;
        &mut db,
        "soy"
    ).await?;
    good_query_pg!("pg_gen_select_limit", 
        r#"insert into "select_limit_bannanana" ( "hizat" ) values ( $1 )"#;
        p0 = string;
        &mut db,
        "soy"
    ).await?;
    good_query_pg!("pg_gen_select_limit", 
        r#"insert into "select_limit_bannanana" ( "hizat" ) values ( $1 )"#;
        p0 = string;
        &mut db,
        "soy"
    ).await?;
    assert_eq!(good_query_many_pg!("pg_gen_select_limit", 
        r#"select "select_limit_bannanana" . "hizat" as "hizat" from "select_limit_bannanana" limit 2"#;
        ;
        &mut db
    ).await?.len(), 2);
    Ok(())
}

#[tokio::test]
async fn test_select_order() -> Result<(), loga::Error> {
    let (mut db, _cont) = db().await?;
    pg_gen_select_order::migrate(&mut db).await?;
    good_query_pg!("pg_gen_select_order", 
        r#"insert into "select_order_bannanana" ( "hizat" ) values ( $1 )"#;
        p0 = i32;
        &mut db,
        0
    ).await?;
    good_query_pg!("pg_gen_select_order", 
        r#"insert into "select_order_bannanana" ( "hizat" ) values ( $1 )"#;
        p0 = i32;
        &mut db,
        12
    ).await?;
    good_query_pg!("pg_gen_select_order", 
        r#"insert into "select_order_bannanana" ( "hizat" ) values ( $1 )"#;
        p0 = i32;
        &mut db,
        9
    ).await?;
    assert_eq!(good_query_many_pg!("pg_gen_select_order", 
        r#"select "select_order_bannanana" . "hizat" as "hizat" from "select_order_bannanana" order by "select_order_bannanana" . "hizat" asc"#;
        ;
        &mut db
    ).await?, vec![0, 9, 12]);
    Ok(())
}

#[tokio::test]
async fn test_migrate_add_field() -> Result<(), loga::Error> {
    let (mut db, _cont) = db().await?;
    pg_gen_migrate_add_field::migrate(&mut db).await?;
    match good_query_opt_pg!("pg_gen_migrate_add_field", 
        r#"select "migrate_add_field_bannna" . "hizat" as "hizat" , "migrate_add_field_bannna" . "zomzom" as "zomzom" from "migrate_add_field_bannna""#;
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
    good_query_pg!("pg_gen_migrate_rename_field", 
        r#"insert into "migrate_rename_field_bannna" ( "hizat" ) values ( 'nizoot' )"#;
        ;
        &mut db
    ).await?;
    Ok(())
}

#[tokio::test]
async fn test_migrate_remove_field() -> Result<(), loga::Error> {
    let (mut db, _cont) = db().await?;
    pg_gen_migrate_remove_field::migrate(&mut db).await?;
    good_query_pg!("pg_gen_migrate_remove_field", 
        r#"insert into "migrate_remove_field_bnanaa" ( "hizat" ) values ( $1 )"#;
        p0 = string;
        &mut db,
        "yordol"
    ).await?;
    Ok(())
}

#[tokio::test]
async fn test_migrate_add_table() -> Result<(), loga::Error> {
    let (mut db, _cont) = db().await?;
    pg_gen_migrate_add_table::migrate(&mut db).await?;
    good_query_pg!("pg_gen_migrate_add_table", 
        r#"insert into "migrate_add_table_two" ( "two" ) values ( $1 )"#;
        p0 = i32;
        &mut db,
        23
    ).await?;
    Ok(())
}

#[tokio::test]
async fn test_migrate_rename_table() -> Result<(), loga::Error> {
    let (mut db, _cont) = db().await?;
    pg_gen_migrate_rename_table::migrate(&mut db).await?;
    good_query_pg!("pg_gen_migrate_rename_table", 
        r#"insert into "migrate_rename_table_bana" ( "hizat" ) values ( $1 )"#;
        p0 = string;
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
    good_query_pg!("pg_gen_select_cte", 
        r#"insert into "select_cte_bannanana" ( "hizat" , "hizat2" ) values ( $1 , $2 )"#;
        p0 = i32,
        p1 = i32;
        &mut db,
        1,
        7
    ).await?;
    good_query_pg!("pg_gen_select_cte", 
        r#"insert into "select_cte_bannanana" ( "hizat" , "hizat2" ) values ( $1 , $2 )"#;
        p0 = i32,
        p1 = i32;
        &mut db,
        1,
        99
    ).await?;
    let mut res = good_query_many_pg!("pg_gen_select_cte", 
        r#"with "hibbo" ( "zathi" ) as ( select "select_cte_bannanana" . "hizat2" as "hizat2" from "select_cte_bannanana" ) select "hibbo" . "zathi" as "zathi" from "hibbo""#;
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
    good_query_pg!("pg_gen_select_window", 
        r#"insert into "select_window_bannanana" ( "hizat" , "hizat2" ) values ( $1 , $2 )"#;
        p0 = i32,
        p1 = i32;
        &mut db,
        1,
        7
    ).await?;
    good_query_pg!("pg_gen_select_window", 
        r#"insert into "select_window_bannanana" ( "hizat" , "hizat2" ) values ( $1 , $2 )"#;
        p0 = i32,
        p1 = i32;
        &mut db,
        1,
        99
    ).await?;
    good_query_pg!("pg_gen_select_window", 
        r#"insert into "select_window_bannanana" ( "hizat" , "hizat2" ) values ( $1 , $2 )"#;
        p0 = i32,
        p1 = i32;
        &mut db,
        2,
        3
    ).await?;
    good_query_pg!("pg_gen_select_window", 
        r#"insert into "select_window_bannanana" ( "hizat" , "hizat2" ) values ( $1 , $2 )"#;
        p0 = i32,
        p1 = i32;
        &mut db,
        2,
        10
    ).await?;
    let mut res = good_query_many_pg!("pg_gen_select_window", 
        r#"select sum ( "select_window_bannanana" . "hizat2" ) over ( partition by "select_window_bannanana" . "hizat" ) as "hizat2" from "select_window_bannanana""#;
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
