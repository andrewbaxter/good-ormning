use good_ormning::runtime::ToGoodError;
use good_ormning_macros::*;
use {
    chrono::{
        Utc,
        TimeZone,
    },
    integration_tests::MyString,
};

pub mod sqlite_gen_base_insert {
    include!(concat!(env!("OUT_DIR"), "/good_ormning_sqlite_sqlite_gen_base_insert.rs"));
}

pub mod sqlite_gen_constraint {
    include!(concat!(env!("OUT_DIR"), "/good_ormning_sqlite_sqlite_gen_constraint.rs"));
}

pub mod sqlite_gen_param_i32 {
    include!(concat!(env!("OUT_DIR"), "/good_ormning_sqlite_sqlite_gen_param_i32.rs"));
}

pub mod sqlite_gen_param_utctime_s_chrono {
    include!(concat!(env!("OUT_DIR"), "/good_ormning_sqlite_sqlite_gen_param_utctime_s_chrono.rs"));
}

pub mod sqlite_gen_param_utctime_ms_chrono {
    include!(concat!(env!("OUT_DIR"), "/good_ormning_sqlite_sqlite_gen_param_utctime_ms_chrono.rs"));
}

pub mod sqlite_gen_param_utctime_s_jiff {
    include!(concat!(env!("OUT_DIR"), "/good_ormning_sqlite_sqlite_gen_param_utctime_s_jiff.rs"));
}

pub mod sqlite_gen_param_utctime_ms_jiff {
    include!(concat!(env!("OUT_DIR"), "/good_ormning_sqlite_sqlite_gen_param_utctime_ms_jiff.rs"));
}

pub mod sqlite_gen_param_opt_i32 {
    include!(concat!(env!("OUT_DIR"), "/good_ormning_sqlite_sqlite_gen_param_opt_i32.rs"));
}

pub mod sqlite_gen_param_opt_i32_null {
    include!(concat!(env!("OUT_DIR"), "/good_ormning_sqlite_sqlite_gen_param_opt_i32_null.rs"));
}

pub mod sqlite_gen_param_arr_i32 {
    include!(concat!(env!("OUT_DIR"), "/good_ormning_sqlite_sqlite_gen_param_arr_i32.rs"));
}

pub mod sqlite_gen_param_custom {
    include!(concat!(env!("OUT_DIR"), "/good_ormning_sqlite_sqlite_gen_param_custom.rs"));
}

pub mod sqlite_gen_param_opt_custom {
    include!(concat!(env!("OUT_DIR"), "/good_ormning_sqlite_sqlite_gen_param_opt_custom.rs"));
}

pub mod sqlite_gen_insert_on_conflict_do_nothing {
    include!(concat!(env!("OUT_DIR"), "/good_ormning_sqlite_sqlite_gen_insert_on_conflict_do_nothing.rs"));
}

pub mod sqlite_gen_insert_on_conflict_update {
    include!(concat!(env!("OUT_DIR"), "/good_ormning_sqlite_sqlite_gen_insert_on_conflict_update.rs"));
}

pub mod sqlite_gen_update {
    include!(concat!(env!("OUT_DIR"), "/good_ormning_sqlite_sqlite_gen_update.rs"));
}

pub mod sqlite_gen_update_where {
    include!(concat!(env!("OUT_DIR"), "/good_ormning_sqlite_sqlite_gen_update_where.rs"));
}

pub mod sqlite_gen_update_returning {
    include!(concat!(env!("OUT_DIR"), "/good_ormning_sqlite_sqlite_gen_update_returning.rs"));
}

pub mod sqlite_gen_delete {
    include!(concat!(env!("OUT_DIR"), "/good_ormning_sqlite_sqlite_gen_delete.rs"));
}

pub mod sqlite_gen_delete_where {
    include!(concat!(env!("OUT_DIR"), "/good_ormning_sqlite_sqlite_gen_delete_where.rs"));
}

pub mod sqlite_gen_delete_returning {
    include!(concat!(env!("OUT_DIR"), "/good_ormning_sqlite_sqlite_gen_delete_returning.rs"));
}

pub mod sqlite_gen_select_join {
    include!(concat!(env!("OUT_DIR"), "/good_ormning_sqlite_sqlite_gen_select_join.rs"));
}

pub mod sqlite_gen_select_group_by {
    include!(concat!(env!("OUT_DIR"), "/good_ormning_sqlite_sqlite_gen_select_group_by.rs"));
}

pub mod sqlite_gen_select_order {
    include!(concat!(env!("OUT_DIR"), "/good_ormning_sqlite_sqlite_gen_select_order.rs"));
}

pub mod sqlite_gen_select_limit {
    include!(concat!(env!("OUT_DIR"), "/good_ormning_sqlite_sqlite_gen_select_limit.rs"));
}

pub mod sqlite_gen_migrate_add_field {
    include!(concat!(env!("OUT_DIR"), "/good_ormning_sqlite_sqlite_gen_migrate_add_field.rs"));
}

pub mod sqlite_gen_migrate_rename_field {
    include!(concat!(env!("OUT_DIR"), "/good_ormning_sqlite_sqlite_gen_migrate_rename_field.rs"));
}

pub mod sqlite_gen_migrate_remove_field {
    include!(concat!(env!("OUT_DIR"), "/good_ormning_sqlite_sqlite_gen_migrate_remove_field.rs"));
}

pub mod sqlite_gen_migrate_add_table {
    include!(concat!(env!("OUT_DIR"), "/good_ormning_sqlite_sqlite_gen_migrate_add_table.rs"));
}

pub mod sqlite_gen_migrate_rename_table {
    include!(concat!(env!("OUT_DIR"), "/good_ormning_sqlite_sqlite_gen_migrate_rename_table.rs"));
}

pub mod sqlite_gen_migrate_remove_table {
    include!(concat!(env!("OUT_DIR"), "/good_ormning_sqlite_sqlite_gen_migrate_remove_table.rs"));
}

pub mod sqlite_gen_migrate_pre_migration {
    include!(concat!(env!("OUT_DIR"), "/good_ormning_sqlite_sqlite_gen_migrate_pre_migration.rs"));
}

pub mod sqlite_gen_select_cte {
    include!(concat!(env!("OUT_DIR"), "/good_ormning_sqlite_sqlite_gen_select_cte.rs"));
}

pub mod sqlite_gen_select_window {
    include!(concat!(env!("OUT_DIR"), "/good_ormning_sqlite_sqlite_gen_select_window.rs"));
}

pub mod sqlite_gen_select_junction {
    include!(concat!(env!("OUT_DIR"), "/good_ormning_sqlite_sqlite_gen_select_junction.rs"));
}

pub mod sqlite_gen_hello_world {
    include!(concat!(env!("OUT_DIR"), "/good_ormning_sqlite_sqlite_gen_hello_world.rs"));
}

#[test]
fn test_hello_world() -> Result<(), loga::Error> {
    use sqlite_gen_hello_world as queries;

    let mut db = rusqlite::Connection::open_in_memory()?;
    queries::migrate(&mut db, |_| Ok(()))?;
    good_query_sqlite!(
        "sqlite_gen_hello_world",
        r#"insert into "hello_world_users" ( "name" , "points" ) values ( ?1 , ?2 )"#;
        p0 = string,
        p1 = i64;
        &mut db,
        "rust human",
        0
    )?;
    for user_id in good_query_many_sqlite!(
        "sqlite_gen_hello_world",
        r#"select "hello_world_users" . "rowid" as "rowid" from "hello_world_users""#;
        ;
        &mut db
    )? {
        let user = good_query_one_sqlite!(
            "sqlite_gen_hello_world",
            r#"select "hello_world_users" . "name" as "name" , "hello_world_users" . "points" as "points" from "hello_world_users" where "hello_world_users" . "rowid" = ?1"#;
            p0 = i64;
            &mut db,
            user_id
        )?;
        println!("User {}: {}", user_id, user.name);
    }
    Ok(())
}

#[test]
fn test_base_insert() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_base_insert::migrate(&mut db, |_| Ok(()))?;
    good_query_sqlite!(
        "sqlite_gen_base_insert",
        r#"insert into "base_insert_bannanana" ( "hizat" ) values ( ?1 )"#;
        p0 = string;
        &mut db,
        "soy"
    )?;
    assert_eq!(good_query_one_sqlite!(
        "sqlite_gen_base_insert",
        r#"select "base_insert_bannanana" . "hizat" as "hizat" from "base_insert_bannanana""#;
        ;
        &mut db
    )?, "soy");
    Ok(())
}

#[test]
fn test_get_version_premigrate() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    assert_eq!(sqlite_gen_base_insert::get_schema_version(&mut db)?, None);
    Ok(())
}

#[test]
fn test_get_version_postmigrate() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_base_insert::migrate(&mut db, |_| Ok(()))?;
    assert_eq!(sqlite_gen_base_insert::get_schema_version(&mut db)?, Some(1));
    Ok(())
}

#[test]
fn test_constraint() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_base_insert::migrate(&mut db, |_| Ok(()))?;
    Ok(())
}

#[test]
fn test_param_i32() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_param_i32::migrate(&mut db, |_| Ok(()))?;
    good_query_sqlite!(
        "sqlite_gen_param_i32",
        r#"insert into "param_i32_bananna" ( "hizat" ) values ( ?1 )"#;
        p0 = i32;
        &mut db,
        22
    )?;
    assert_eq!(good_query_one_sqlite!(
        "sqlite_gen_param_i32",
        r#"select "param_i32_bananna" . "hizat" as "hizat" from "param_i32_bananna""#;
        ;
        &mut db
    )?, 22);
    Ok(())
}

#[test]
fn test_param_utctime_s_chrono() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_param_utctime_s_chrono::migrate(&mut db, |_| Ok(()))?;
    let ref_date = chrono::TimeZone::with_ymd_and_hms(&chrono::Utc, 1937, 12, 1, 0, 0, 0).unwrap();
    good_query_sqlite!(
        "sqlite_gen_param_utctime_s_chrono",
        r#"insert into "param_utctime_s_chrono_bananna" ( "hizat" ) values ( ?1 )"#;
        p0 = utctime_s_chrono;
        &mut db,
        ref_date
    )?;
    assert_eq!(good_query_one_sqlite!(
        "sqlite_gen_param_utctime_s_chrono",
        r#"select "param_utctime_s_chrono_bananna" . "hizat" as "hizat" from "param_utctime_s_chrono_bananna""#;
        ;
        &mut db
    )?, ref_date);
    Ok(())
}

#[test]
fn test_param_utctime_ms_chrono() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_param_utctime_ms_chrono::migrate(&mut db, |_| Ok(()))?;
    let ref_date = chrono::TimeZone::with_ymd_and_hms(&chrono::Utc, 1937, 12, 1, 0, 0, 0).unwrap();
    good_query_sqlite!(
        "sqlite_gen_param_utctime_ms_chrono",
        r#"insert into "param_utctime_ms_chrono_bananna" ( "hizat" ) values ( ?1 )"#;
        p0 = utctime_ms_chrono;
        &mut db,
        ref_date
    )?;
    assert_eq!(good_query_one_sqlite!(
        "sqlite_gen_param_utctime_ms_chrono",
        r#"select "param_utctime_ms_chrono_bananna" . "hizat" as "hizat" from "param_utctime_ms_chrono_bananna""#;
        ;
        &mut db
    )?, ref_date);
    Ok(())
}

#[test]
fn test_param_utctime_s_jiff() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_param_utctime_s_jiff::migrate(&mut db, |_| Ok(()))?;
    let ref_date =
        jiff::civil::DateTime::new(1937, 12, 1, 0, 0, 0, 0)
            .unwrap()
            .to_zoned(jiff::tz::TimeZone::UTC)
            .unwrap()
            .timestamp();
    good_query_sqlite!(
        "sqlite_gen_param_utctime_s_jiff",
        r#"insert into "param_utctime_s_jiff_bananna" ( "hizat" ) values ( ?1 )"#;
        p0 = utctime_s_jiff;
        &mut db,
        ref_date
    )?;
    assert_eq!(good_query_one_sqlite!(
        "sqlite_gen_param_utctime_s_jiff",
        r#"select "param_utctime_s_jiff_bananna" . "hizat" as "hizat" from "param_utctime_s_jiff_bananna""#;
        ;
        &mut db
    )?, ref_date);
    Ok(())
}

#[test]
fn test_param_utctime_ms_jiff() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_param_utctime_ms_jiff::migrate(&mut db, |_| Ok(()))?;
    let ref_date =
        jiff::civil::DateTime::new(1937, 12, 1, 0, 0, 0, 0)
            .unwrap()
            .to_zoned(jiff::tz::TimeZone::UTC)
            .unwrap()
            .timestamp();
    good_query_sqlite!(
        "sqlite_gen_param_utctime_ms_jiff",
        r#"insert into "param_utctime_ms_jiff_bananna" ( "hizat" ) values ( ?1 )"#;
        p0 = utctime_ms_jiff;
        &mut db,
        ref_date
    )?;
    assert_eq!(good_query_one_sqlite!(
        "sqlite_gen_param_utctime_ms_jiff",
        r#"select "param_utctime_ms_jiff_bananna" . "hizat" as "hizat" from "param_utctime_ms_jiff_bananna""#;
        ;
        &mut db
    )?, ref_date);
    Ok(())
}

#[test]
fn test_param_opt_i32() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_param_opt_i32::migrate(&mut db, |_| Ok(()))?;
    good_query_sqlite!(
        "sqlite_gen_param_opt_i32",
        r#"insert into "param_opt_i32_bananna" ( "hizat" ) values ( ?1 )"#;
        p0 = opt i32;
        &mut db,
        Some(47)
    )?;
    assert_eq!(good_query_one_sqlite!(
        "sqlite_gen_param_opt_i32",
        r#"select "param_opt_i32_bananna" . "hizat" as "hizat" from "param_opt_i32_bananna""#;
        ;
        &mut db
    )?, Some(47));
    Ok(())
}

#[test]
fn test_param_opt_i32_null() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_param_opt_i32_null::migrate(&mut db, |_| Ok(()))?;
    good_query_sqlite!(
        "sqlite_gen_param_opt_i32_null",
        r#"insert into "param_opt_i32_null_bananna" ( "hizat" ) values ( null )"#;
        ;
        &mut db
    )?;
    assert_eq!(good_query_one_sqlite!(
        "sqlite_gen_param_opt_i32_null",
        r#"select "param_opt_i32_null_bananna" . "hizat" as "hizat" from "param_opt_i32_null_bananna""#;
        ;
        &mut db
    )?, None);
    Ok(())
}

#[test]
fn test_param_arr_i32() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_param_arr_i32::migrate(&mut db, |_| Ok(()))?;
    good_query_sqlite!(
        "sqlite_gen_param_arr_i32",
        r#"insert into "param_arr_i32_bananna" ( "hizat" ) values ( ?1 )"#;
        p0 = i32;
        &mut db,
        7
    )?;
    assert_eq!(good_query_many_sqlite!(
        "sqlite_gen_param_arr_i32",
        r#"select "param_arr_i32_bananna" . "hizat" as "hizat" from "param_arr_i32_bananna" where "param_arr_i32_bananna" . "hizat" in ( select value from rarray ( ?1 ) )"#;
        p0 = arr i32;
        &mut db,
        vec![7]
    )?, vec![7]);
    Ok(())
}

#[test]
fn test_param_custom() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_param_custom::migrate(&mut db, |_| Ok(()))?;
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
    good_query_sqlite!(
        "sqlite_gen_param_custom",
        r#"insert into "param_custom_bananna" ( "x_0" , "x_1" , "x_2" , "x_3" , "x_4" , "x_5" , "x_6" , "x_7" , "x_8" , "x_9" , "x_10" , "x_11" ) values ( ?1 , ?2 , ?3 , ?4 , ?5 , ?6 , ?7 , ?8 , ?9 , ?10 , ?11 , ?12 )"#;
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
    )?;
    let res = good_query_one_sqlite!(
        "sqlite_gen_param_custom",
        r#"select "param_custom_bananna" . "x_0" as "x_0" , "param_custom_bananna" . "x_1" as "x_1" , "param_custom_bananna" . "x_2" as "x_2" , "param_custom_bananna" . "x_3" as "x_3" , "param_custom_bananna" . "x_4" as "x_4" , "param_custom_bananna" . "x_5" as "x_5" , "param_custom_bananna" . "x_6" as "x_6" , "param_custom_bananna" . "x_7" as "x_7" , "param_custom_bananna" . "x_8" as "x_8" , "param_custom_bananna" . "x_9" as "x_9" , "param_custom_bananna" . "x_10" as "x_10" , "param_custom_bananna" . "x_11" as "x_11" from "param_custom_bananna""#;
        ;
        &mut db
    )?;
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

#[test]
fn test_param_opt_custom() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_param_opt_custom::migrate(&mut db, |_| Ok(()))?;
    good_query_sqlite!(
        "sqlite_gen_param_opt_custom",
        r#"insert into "param_opt_custom_bananna" ( "hizat" ) values ( ?1 )"#;
        p0 = opt MyString;
        &mut db,
        Some(&MyString("higgins".into()))
    )?;
    assert_eq!(good_query_one_sqlite!(
        "sqlite_gen_param_opt_custom",
        r#"select "param_opt_custom_bananna" . "hizat" as "hizat" from "param_opt_custom_bananna""#;
        ;
        &mut db
    )?, Some(MyString("higgins".into())));
    Ok(())
}

#[test]
fn test_insert_on_conflict_do_nothing() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_insert_on_conflict_do_nothing::migrate(&mut db, |_| Ok(()))?;
    assert!(good_query_opt_sqlite!(
        "sqlite_gen_insert_on_conflict_do_nothing",
        r#"insert into "insert_on_conflict_do_nothing_bananna" ( "hizat" ) values ( ?1 ) on conflict do nothing returning 1 as "one""#;
        p0 = string;
        &mut db,
        "soy"
    )?.is_some());
    assert!(good_query_opt_sqlite!(
        "sqlite_gen_insert_on_conflict_do_nothing",
        r#"insert into "insert_on_conflict_do_nothing_bananna" ( "hizat" ) values ( ?1 ) on conflict do nothing returning 1 as "one""#;
        p0 = string;
        &mut db,
        "soy"
    )?.is_none());
    Ok(())
}

#[test]
fn test_insert_on_conflict_update() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_insert_on_conflict_update::migrate(&mut db, |_| Ok(()))?;
    assert_eq!(good_query_one_sqlite!(
        "sqlite_gen_insert_on_conflict_update",
        r#"insert into "insert_on_conflict_update_bananna" ( "hizat" , "two" ) values ( ?1 , ?2 ) on conflict ( "hizat" ) do update set "two" = "insert_on_conflict_update_bananna" . "two" + 1 returning "insert_on_conflict_update_bananna" . "two" as "two""#;
        p0 = string,
        p1 = i32;
        &mut db,
        "soy",
        33
    )?, 33);
    assert_eq!(good_query_one_sqlite!(
        "sqlite_gen_insert_on_conflict_update",
        r#"insert into "insert_on_conflict_update_bananna" ( "hizat" , "two" ) values ( ?1 , ?2 ) on conflict ( "hizat" ) do update set "two" = "insert_on_conflict_update_bananna" . "two" + 1 returning "insert_on_conflict_update_bananna" . "two" as "two""#;
        p0 = string,
        p1 = i32;
        &mut db,
        "soy",
        7
    )?, 34);
    assert_eq!(good_query_one_sqlite!(
        "sqlite_gen_insert_on_conflict_update",
        r#"insert into "insert_on_conflict_update_bananna" ( "hizat" , "two" ) values ( ?1 , ?2 ) on conflict ( "hizat" ) do update set "two" = "insert_on_conflict_update_bananna" . "two" + 1 returning "insert_on_conflict_update_bananna" . "two" as "two""#;
        p0 = string,
        p1 = i32;
        &mut db,
        "yyyy",
        7
    )?, 7);
    Ok(())
}

#[test]
fn test_update() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_update::migrate(&mut db, |_| Ok(()))?;
    good_query_sqlite!(
        "sqlite_gen_update",
        r#"insert into "update_bananna" ( "hizat" ) values ( 'yog' )"#;
        ;
        &mut db
    )?;
    assert_eq!(good_query_one_sqlite!(
        "sqlite_gen_update",
        r#"select "update_bananna" . "hizat" as "hizat" from "update_bananna""#;
        ;
        &mut db
    )?, "yog");
    good_query_sqlite!(
        "sqlite_gen_update",
        r#"update "update_bananna" set "hizat" = 'tep'"#;
        ;
        &mut db
    )?;
    assert_eq!(good_query_one_sqlite!(
        "sqlite_gen_update",
        r#"select "update_bananna" . "hizat" as "hizat" from "update_bananna""#;
        ;
        &mut db
    )?, "tep");
    Ok(())
}

#[test]
fn test_update_where() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_update_where::migrate(&mut db, |_| Ok(()))?;
    good_query_sqlite!(
        "sqlite_gen_update_where",
        r#"insert into "update_where_ban" ( "hizat" ) values ( 'yog' )"#;
        ;
        &mut db
    )?;
    assert_eq!(good_query_one_sqlite!(
        "sqlite_gen_update_where",
        r#"select "update_where_ban" . "hizat" as "hizat" from "update_where_ban""#;
        ;
        &mut db
    )?, "yog");
    good_query_sqlite!(
        "sqlite_gen_update_where",
        r#"update "update_where_ban" set "hizat" = ?1 where "update_where_ban" . "hizat" = ?2"#;
        p0 = string,
        p1 = string;
        &mut db,
        "tep",
        "yog2"
    )?;
    assert_eq!(good_query_one_sqlite!(
        "sqlite_gen_update_where",
        r#"select "update_where_ban" . "hizat" as "hizat" from "update_where_ban""#;
        ;
        &mut db
    )?, "yog");
    good_query_sqlite!(
        "sqlite_gen_update_where",
        r#"update "update_where_ban" set "hizat" = ?1 where "update_where_ban" . "hizat" = ?2"#;
        p0 = string,
        p1 = string;
        &mut db,
        "tep",
        "yog"
    )?;
    assert_eq!(good_query_one_sqlite!(
        "sqlite_gen_update_where",
        r#"select "update_where_ban" . "hizat" as "hizat" from "update_where_ban""#;
        ;
        &mut db
    )?, "tep");
    Ok(())
}

#[test]
fn test_update_returning() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_update_returning::migrate(&mut db, |_| Ok(()))?;
    good_query_sqlite!(
        "sqlite_gen_update_returning",
        r#"insert into "update_returning_b" ( "hizat" ) values ( 'yog' )"#;
        ;
        &mut db
    )?;
    assert_eq!(good_query_opt_sqlite!(
        "sqlite_gen_update_returning",
        r#"update "update_returning_b" set "hizat" = 'tep' returning "update_returning_b" . "hizat" as "hizat""#;
        ;
        &mut db
    )?, Some("tep".to_string()));
    Ok(())
}

#[test]
fn test_delete() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_delete::migrate(&mut db, |_| Ok(()))?;
    good_query_sqlite!(
        "sqlite_gen_delete",
        r#"insert into "delete_b" ( "hizat" ) values ( 'seeon' )"#;
        ;
        &mut db
    )?;
    assert_eq!(good_query_opt_sqlite!(
        "sqlite_gen_delete",
        r#"select "delete_b" . "hizat" as "hizat" from "delete_b""#;
        ;
        &mut db
    )?, Some("seeon".to_string()));
    good_query_sqlite!(
        "sqlite_gen_delete",
        r#"delete from "delete_b""#;
        ;
        &mut db
    )?;
    assert_eq!(good_query_opt_sqlite!(
        "sqlite_gen_delete",
        r#"select "delete_b" . "hizat" as "hizat" from "delete_b""#;
        ;
        &mut db
    )?, None);
    Ok(())
}

#[test]
fn test_delete_where() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_delete_where::migrate(&mut db, |_| Ok(()))?;
    good_query_sqlite!(
        "sqlite_gen_delete_where",
        r#"insert into "delete_where_ba" ( "hizat" ) values ( 'seeon' )"#;
        ;
        &mut db
    )?;
    good_query_sqlite!(
        "sqlite_gen_delete_where",
        r#"delete from "delete_where_ba" where "delete_where_ba" . "hizat" = ?1"#;
        p0 = string;
        &mut db,
        "nozo"
    )?;
    assert_eq!(good_query_opt_sqlite!(
        "sqlite_gen_delete_where",
        r#"select "delete_where_ba" . "hizat" as "hizat" from "delete_where_ba""#;
        ;
        &mut db
    )?, Some("seeon".to_string()));
    good_query_sqlite!(
        "sqlite_gen_delete_where",
        r#"delete from "delete_where_ba" where "delete_where_ba" . "hizat" = ?1"#;
        p0 = string;
        &mut db,
        "seeon"
    )?;
    assert_eq!(good_query_opt_sqlite!(
        "sqlite_gen_delete_where",
        r#"select "delete_where_ba" . "hizat" as "hizat" from "delete_where_ba""#;
        ;
        &mut db
    )?, None);
    Ok(())
}

#[test]
fn test_delete_returning() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_delete_where::migrate(&mut db, |_| Ok(()))?;
    good_query_sqlite!(
        "sqlite_gen_delete_where",
        r#"insert into "delete_where_ba" ( "hizat" ) values ( 'seeon' )"#;
        ;
        &mut db
    )?;
    assert!(good_query_opt_sqlite!(
        "sqlite_gen_delete_where",
        r#"select "delete_where_ba" . "hizat" as "hizat" from "delete_where_ba""#;
        ;
        &mut db
    )?.is_some());
    good_query_sqlite!(
        "sqlite_gen_delete_where",
        r#"delete from "delete_where_ba" where "delete_where_ba" . "hizat" = ?1"#;
        p0 = string;
        &mut db,
        "seeon"
    )?;
    assert!(good_query_opt_sqlite!(
        "sqlite_gen_delete_where",
        r#"select "delete_where_ba" . "hizat" as "hizat" from "delete_where_ba""#;
        ;
        &mut db
    )?.is_none());
    Ok(())
}

#[test]
fn test_select_join() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_select_join::migrate(&mut db, |v| {
        match v {
            sqlite_gen_select_join::DbSqliteGenSelectJoinVersions::V1(mut db) => {
                good_query_sqlite_ver!(
                    1,
                    "sqlite_gen_select_join",
                    r#"insert into "select_join_b" ( "hizat" , "three" ) values ( 'key' , 33 )"#;
                    ;
                    &mut *db.0
                )?;
                good_query_sqlite_ver!(
                    1,
                    "sqlite_gen_select_join",
                    r#"insert into "select_join_two" ( "hizat" , "two" ) values ( 'key' , 'no' )"#;
                    ;
                    &mut *db.0
                )?;
            },
            _ => { },
        }
        Ok(())
    })?;
    let res = good_query_one_sqlite!(
        "sqlite_gen_select_join",
        r#"select "select_join_b" . "three" as "three" , "select_join_two" . "two" as "two" from "select_join_b" left join "select_join_two" on ( "select_join_b" . "hizat" ) = "select_join_two" . "hizat""#;
        ;
        &mut db
    )?;
    assert_eq!(res.three, 33);
    assert_eq!(res.two, Some("no".into()));
    Ok(())
}

#[test]
fn test_select_group_by() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_select_group_by::migrate(&mut db, |_| Ok(()))?;
    good_query_sqlite!(
        "sqlite_gen_select_group_by",
        r#"insert into "select_group_by_bannanana" ( "hizat" , "hizat2" ) values ( ?1 , ?2 )"#;
        p0 = i32,
        p1 = i32;
        &mut db,
        1,
        7
    )?;
    good_query_sqlite!(
        "sqlite_gen_select_group_by",
        r#"insert into "select_group_by_bannanana" ( "hizat" , "hizat2" ) values ( ?1 , ?2 )"#;
        p0 = i32,
        p1 = i32;
        &mut db,
        1,
        99
    )?;
    good_query_sqlite!(
        "sqlite_gen_select_group_by",
        r#"insert into "select_group_by_bannanana" ( "hizat" , "hizat2" ) values ( ?1 , ?2 )"#;
        p0 = i32,
        p1 = i32;
        &mut db,
        2,
        3
    )?;
    good_query_sqlite!(
        "sqlite_gen_select_group_by",
        r#"insert into "select_group_by_bannanana" ( "hizat" , "hizat2" ) values ( ?1 , ?2 )"#;
        p0 = i32,
        p1 = i32;
        &mut db,
        2,
        10
    )?;
    let mut res = good_query_many_sqlite!(
        "sqlite_gen_select_group_by",
        r#"select sum ( "select_group_by_bannanana" . "hizat2" ) as "hizat2" from "select_group_by_bannanana" group by "select_group_by_bannanana" . "hizat""#;
        ;
        &mut db
    )?;
    res.sort();
    assert_eq!(res, vec![13, 106]);
    Ok(())
}

#[test]
fn test_select_limit() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_select_limit::migrate(&mut db, |_| Ok(()))?;
    good_query_sqlite!(
        "sqlite_gen_select_limit",
        r#"insert into "select_limit_bannanana" ( "hizat" ) values ( ?1 )"#;
        p0 = string;
        &mut db,
        "soy"
    )?;
    good_query_sqlite!(
        "sqlite_gen_select_limit",
        r#"insert into "select_limit_bannanana" ( "hizat" ) values ( ?1 )"#;
        p0 = string;
        &mut db,
        "soy"
    )?;
    good_query_sqlite!(
        "sqlite_gen_select_limit",
        r#"insert into "select_limit_bannanana" ( "hizat" ) values ( ?1 )"#;
        p0 = string;
        &mut db,
        "soy"
    )?;
    assert_eq!(good_query_many_sqlite!(
        "sqlite_gen_select_limit",
        r#"select "select_limit_bannanana" . "hizat" as "hizat" from "select_limit_bannanana" limit 2"#;
        ;
        &mut db
    )?.len(), 2);
    Ok(())
}

#[test]
fn test_select_order() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_select_order::migrate(&mut db, |_| Ok(()))?;
    good_query_sqlite!(
        "sqlite_gen_select_order",
        r#"insert into "select_order_bannanana" ( "hizat" ) values ( ?1 )"#;
        p0 = i32;
        &mut db,
        0
    )?;
    good_query_sqlite!(
        "sqlite_gen_select_order",
        r#"insert into "select_order_bannanana" ( "hizat" ) values ( ?1 )"#;
        p0 = i32;
        &mut db,
        12
    )?;
    good_query_sqlite!(
        "sqlite_gen_select_order",
        r#"insert into "select_order_bannanana" ( "hizat" ) values ( ?1 )"#;
        p0 = i32;
        &mut db,
        9
    )?;
    assert_eq!(good_query_many_sqlite!(
        "sqlite_gen_select_order",
        r#"select "select_order_bannanana" . "hizat" as "hizat" from "select_order_bannanana" order by "select_order_bannanana" . "hizat" asc"#;
        ;
        &mut db
    )?, vec![0, 9, 12]);
    Ok(())
}

#[test]
fn test_migrate_add_field() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_migrate_add_field::migrate(&mut db, |v| {
        match v {
            sqlite_gen_migrate_add_field::DbSqliteGenMigrateAddFieldVersions::V0(mut db) => {
                good_query_sqlite_ver!(
                    0,
                    "sqlite_gen_migrate_add_field",
                    r#"insert into "migrate_add_field_bannna" ( "hizat" ) values ( 'nizoot' )"#;
                    ;
                    &mut *db.0
                )?;
            },
            _ => { },
        }
        Ok(())
    })?;
    match good_query_opt_sqlite!(
        "sqlite_gen_migrate_add_field",
        r#"select "migrate_add_field_bannna" . "hizat" as "hizat" , "migrate_add_field_bannna" . "zomzom" as "zomzom" from "migrate_add_field_bannna""#;
        ;
        &mut db
    )? {
        Some(x) => {
            assert_eq!(x.zomzom, true);
            assert_eq!(&x.hizat, "nizoot");
        },
        None => assert!(false),
    };
    Ok(())
}

#[test]
fn test_migrate_rename_field() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_migrate_rename_field::migrate(&mut db, |_| Ok(()))?;
    good_query_sqlite!(
        "sqlite_gen_migrate_rename_field",
        r#"insert into "migrate_rename_field_bannna" ( "hizat" ) values ( 'nizoot' )"#;
        ;
        &mut db
    )?;
    Ok(())
}

#[test]
fn test_migrate_remove_field() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_migrate_remove_field::migrate(&mut db, |_| Ok(()))?;
    good_query_sqlite!(
        "sqlite_gen_migrate_remove_field",
        r#"insert into "migrate_remove_field_bnanaa" ( "hizat" ) values ( ?1 )"#;
        p0 = string;
        &mut db,
        "yordol"
    )?;
    Ok(())
}

#[test]
fn test_migrate_add_table() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_migrate_add_table::migrate(&mut db, |_| Ok(()))?;
    good_query_sqlite!(
        "sqlite_gen_migrate_add_table",
        r#"insert into "migrate_add_table_two" ( "two" ) values ( ?1 )"#;
        p0 = i32;
        &mut db,
        23
    )?;
    Ok(())
}

#[test]
fn test_migrate_rename_table() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_migrate_rename_table::migrate(&mut db, |_| Ok(()))?;
    good_query_sqlite!(
        "sqlite_gen_migrate_rename_table",
        r#"insert into "migrate_rename_table_bana" ( "hizat" ) values ( ?1 )"#;
        p0 = string;
        &mut db,
        "inset"
    )?;
    Ok(())
}

#[test]
fn test_migrate_remove_table() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_migrate_remove_table::migrate(&mut db, |_| Ok(()))?;
    Ok(())
}

#[test]
fn test_migrate_pre_migration() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_migrate_pre_migration::migrate(&mut db, |v| {
        match v {
            sqlite_gen_migrate_pre_migration::DbSqliteGenMigratePreMigrationVersions::V0(mut db) => {
                good_query_sqlite_ver!(
                    0,
                    "sqlite_gen_migrate_pre_migration",
                    r#"insert into "migrate_pre_migration_v0_two" ( "two" ) values ( 7 )"#;
                    ;
                    &mut *db.0
                )?;
            },
            _ => { },
        }
        Ok(())
    })?;
    Ok(())
}

#[test]
fn test_select_cte() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_select_cte::migrate(&mut db, |_| Ok(()))?;
    good_query_sqlite!(
        "sqlite_gen_select_cte",
        r#"insert into "select_cte_bannanana" ( "hizat" , "hizat2" ) values ( ?1 , ?2 )"#;
        p0 = i32,
        p1 = i32;
        &mut db,
        1,
        7
    )?;
    good_query_sqlite!(
        "sqlite_gen_select_cte",
        r#"insert into "select_cte_bannanana" ( "hizat" , "hizat2" ) values ( ?1 , ?2 )"#;
        p0 = i32,
        p1 = i32;
        &mut db,
        1,
        99
    )?;
    let mut res = good_query_many_sqlite!(
        "sqlite_gen_select_cte",
        r#"with "hibbo" ( "zathi" ) as ( select "select_cte_bannanana" . "hizat2" as "hizat2" from "select_cte_bannanana" ) select "hibbo" . "zathi" as "zathi" from "hibbo""#;
        ;
        &mut db
    )?;
    res.sort();
    assert_eq!(res, vec![7, 99]);
    Ok(())
}

#[test]
fn test_select_window() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_select_window::migrate(&mut db, |_| Ok(()))?;
    good_query_sqlite!(
        "sqlite_gen_select_window",
        r#"insert into "select_window_bannanana" ( "hizat" , "hizat2" ) values ( ?1 , ?2 )"#;
        p0 = i32,
        p1 = i32;
        &mut db,
        1,
        7
    )?;
    good_query_sqlite!(
        "sqlite_gen_select_window",
        r#"insert into "select_window_bannanana" ( "hizat" , "hizat2" ) values ( ?1 , ?2 )"#;
        p0 = i32,
        p1 = i32;
        &mut db,
        1,
        99
    )?;
    good_query_sqlite!(
        "sqlite_gen_select_window",
        r#"insert into "select_window_bannanana" ( "hizat" , "hizat2" ) values ( ?1 , ?2 )"#;
        p0 = i32,
        p1 = i32;
        &mut db,
        2,
        3
    )?;
    good_query_sqlite!(
        "sqlite_gen_select_window",
        r#"insert into "select_window_bannanana" ( "hizat" , "hizat2" ) values ( ?1 , ?2 )"#;
        p0 = i32,
        p1 = i32;
        &mut db,
        2,
        10
    )?;
    let mut res = good_query_many_sqlite!(
        "sqlite_gen_select_window",
        r#"select sum ( "select_window_bannanana" . "hizat2" ) over ( partition by "select_window_bannanana" . "hizat" ) as "hizat2" from "select_window_bannanana""#;
        ;
        &mut db
    )?.into_iter().collect::<Vec<_>>();
    res.sort();
    assert_eq!(res, vec![13, 13, 106, 106]);
    Ok(())
}

#[test]
fn test_select_junction() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_select_junction::migrate(&mut db, |_| Ok(()))?;
    good_query_sqlite!(
        "sqlite_gen_select_junction",
        r#"insert into "select_junction_bannanana" ( "hizat" , "hizat2" ) values ( ?1 , ?2 )"#;
        p0 = i32,
        p1 = i32;
        &mut db,
        1,
        7
    )?;
    good_query_sqlite!(
        "sqlite_gen_select_junction",
        r#"insert into "select_junction_bannanana" ( "hizat" , "hizat2" ) values ( ?1 , ?2 )"#;
        p0 = i32,
        p1 = i32;
        &mut db,
        2,
        3
    )?;
    let mut res = good_query_many_sqlite!(
        "sqlite_gen_select_junction",
        r#"select "select_junction_bannanana" . "hizat" as "hizat" from "select_junction_bannanana" union select "select_junction_bannanana" . "hizat2" as "hizat2" from "select_junction_bannanana""#;
        ;
        &mut db
    )?;
    res.sort();
    assert_eq!(res, vec![1, 2, 3, 7]);
    Ok(())
}
