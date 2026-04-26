use good_ormning::runtime::ToGoodError;
use good_ormning_macros::*;
use {
    chrono::{
        Utc,
        TimeZone,
    },
    integration_tests::MyString,
};

pub mod sqlite_gen_base_insert;
pub mod sqlite_gen_constraint;
pub mod sqlite_gen_param_i32;
pub mod sqlite_gen_param_utctime_s_chrono;
pub mod sqlite_gen_param_utctime_ms_chrono;
pub mod sqlite_gen_param_utctime_s_jiff;
pub mod sqlite_gen_param_utctime_ms_jiff;
pub mod sqlite_gen_param_opt_i32;
pub mod sqlite_gen_param_opt_i32_null;
pub mod sqlite_gen_param_arr_i32;
pub mod sqlite_gen_param_custom;
pub mod sqlite_gen_param_opt_custom;
pub mod sqlite_gen_insert_on_conflict_do_nothing;
pub mod sqlite_gen_insert_on_conflict_update;
pub mod sqlite_gen_update;
pub mod sqlite_gen_update_where;
pub mod sqlite_gen_update_returning;
pub mod sqlite_gen_delete;
pub mod sqlite_gen_delete_where;
pub mod sqlite_gen_delete_returning;
pub mod sqlite_gen_select_join;
pub mod sqlite_gen_select_group_by;
pub mod sqlite_gen_select_order;
pub mod sqlite_gen_select_limit;
pub mod sqlite_gen_migrate_add_field;
pub mod sqlite_gen_migrate_rename_field;
pub mod sqlite_gen_migrate_remove_field;
pub mod sqlite_gen_migrate_add_table;
pub mod sqlite_gen_migrate_rename_table;
pub mod sqlite_gen_migrate_remove_table;
pub mod sqlite_gen_migrate_pre_migration;
pub mod sqlite_gen_select_cte;
pub mod sqlite_gen_select_window;
pub mod sqlite_gen_select_junction;
pub mod sqlite_gen_hello_world;

#[test]
fn test_hello_world() -> Result<(), loga::Error> {
    use sqlite_gen_hello_world as queries;

    let mut db = rusqlite::Connection::open_in_memory()?;
    queries::migrate(&mut db)?;
    good_query_sqlite!(
        r#"insert into "users" ( "name" , "points" ) values ( ?1 , ?2 )"#;
        name = &str,
        points = i64;
        &mut db,
        "rust human",
        0
    )?;
    for user_id in good_query_many_sqlite!(
        r#"select "users" . "rowid" as "rowid" from "users""#;
        ;
        &mut db
    )? {
        let user = good_query_one_sqlite!(
            r#"select "users" . "name" as "name" , "users" . "points" as "points" from "users" where "users" . "rowid" = ?1"#;
            id = i64;
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
    sqlite_gen_base_insert::migrate(&mut db)?;
    good_query_sqlite!(
        r#"insert into "bannanana" ( "hizat" ) values ( ?1 )"#;
        text = &str;
        &mut db,
        "soy"
    )?;
    assert_eq!(good_query_one_sqlite!(
        r#"select "bannanana" . "hizat" as "hizat" from "bannanana""#;
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
    sqlite_gen_base_insert::migrate(&mut db)?;
    assert_eq!(sqlite_gen_base_insert::get_schema_version(&mut db)?, Some(0));
    Ok(())
}

#[test]
fn test_constraint() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_base_insert::migrate(&mut db)?;
    Ok(())
}

#[test]
fn test_param_i32() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_param_i32::migrate(&mut db)?;
    good_query_sqlite!(
        r#"insert into "bananna_sqlite_gen_constraint" ( "hizat" ) values ( ?1 )"#;
        val = i32;
        &mut db,
        22
    )?;
    assert_eq!(good_query_one_sqlite!(
        r#"select "bananna_sqlite_gen_constraint" . "hizat" as "hizat" from "bananna_sqlite_gen_constraint""#;
        ;
        &mut db
    )?, 22);
    Ok(())
}

#[test]
fn test_param_utctime_s_chrono() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_param_utctime_s_chrono::migrate(&mut db)?;
    let ref_date = chrono::TimeZone::with_ymd_and_hms(&chrono::Utc, 1937, 12, 1, 0, 0, 0).unwrap();
    good_query_sqlite!(
        r#"insert into "bananna_sqlite_gen_param_i32" ( "hizat" ) values ( ?1 )"#;
        val = chrono:: DateTime < chrono:: Utc >;
        &mut db,
        ref_date
    )?;
    assert_eq!(good_query_one_sqlite!(
        r#"select "bananna_sqlite_gen_param_i32" . "hizat" as "hizat" from "bananna_sqlite_gen_param_i32""#;
        ;
        &mut db
    )?, ref_date);
    Ok(())
}

#[test]
fn test_param_utctime_ms_chrono() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_param_utctime_ms_chrono::migrate(&mut db)?;
    let ref_date = chrono::TimeZone::with_ymd_and_hms(&chrono::Utc, 1937, 12, 1, 0, 0, 0).unwrap();
    good_query_sqlite!(
        r#"insert into "bananna_sqlite_gen_param_utctime_s_chrono" ( "hizat" ) values ( ?1 )"#;
        val = chrono:: DateTime < chrono:: Utc >;
        &mut db,
        ref_date
    )?;
    assert_eq!(good_query_one_sqlite!(
        r#"select "bananna_sqlite_gen_param_utctime_s_chrono" . "hizat" as "hizat" from "bananna_sqlite_gen_param_utctime_s_chrono""#;
        ;
        &mut db
    )?, ref_date);
    Ok(())
}

#[test]
fn test_param_utctime_s_jiff() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_param_utctime_s_jiff::migrate(&mut db)?;
    let ref_date =
        jiff::civil::DateTime::new(1937, 12, 1, 0, 0, 0, 0)
            .unwrap()
            .to_zoned(jiff::tz::TimeZone::UTC)
            .unwrap()
            .timestamp();
    good_query_sqlite!(
        r#"insert into "bananna_sqlite_gen_param_utctime_ms_chrono" ( "hizat" ) values ( ?1 )"#;
        val = jiff::Timestamp;
        &mut db,
        ref_date
    )?;
    assert_eq!(good_query_one_sqlite!(
        r#"select "bananna_sqlite_gen_param_utctime_ms_chrono" . "hizat" as "hizat" from "bananna_sqlite_gen_param_utctime_ms_chrono""#;
        ;
        &mut db
    )?, ref_date);
    Ok(())
}

#[test]
fn test_param_utctime_ms_jiff() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_param_utctime_ms_jiff::migrate(&mut db)?;
    let ref_date =
        jiff::civil::DateTime::new(1937, 12, 1, 0, 0, 0, 0)
            .unwrap()
            .to_zoned(jiff::tz::TimeZone::UTC)
            .unwrap()
            .timestamp();
    good_query_sqlite!(
        r#"insert into "bananna_sqlite_gen_param_utctime_s_jiff" ( "hizat" ) values ( ?1 )"#;
        val = jiff::Timestamp;
        &mut db,
        ref_date
    )?;
    assert_eq!(good_query_one_sqlite!(
        r#"select "bananna_sqlite_gen_param_utctime_s_jiff" . "hizat" as "hizat" from "bananna_sqlite_gen_param_utctime_s_jiff""#;
        ;
        &mut db
    )?, ref_date);
    Ok(())
}

#[test]
fn test_param_opt_i32() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_param_opt_i32::migrate(&mut db)?;
    good_query_sqlite!(
        r#"insert into "bananna_sqlite_gen_param_utctime_ms_jiff" ( "hizat" ) values ( ?1 )"#;
        val = Option < i32 >;
        &mut db,
        Some(47)
    )?;
    assert_eq!(good_query_one_sqlite!(
        r#"select "bananna_sqlite_gen_param_utctime_ms_jiff" . "hizat" as "hizat" from "bananna_sqlite_gen_param_utctime_ms_jiff""#;
        ;
        &mut db
    )?, Some(47));
    Ok(())
}

#[test]
fn test_param_opt_i32_null() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_param_opt_i32_null::migrate(&mut db)?;
    good_query_sqlite!(
        r#"insert into "bananna_sqlite_gen_param_opt_i32" ( "hizat" ) values ( null )"#;
        ;
        &mut db
    )?;
    assert_eq!(good_query_one_sqlite!(
        r#"select "bananna_sqlite_gen_param_opt_i32" . "hizat" as "hizat" from "bananna_sqlite_gen_param_opt_i32""#;
        ;
        &mut db
    )?, None);
    Ok(())
}

#[test]
fn test_param_arr_i32() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_param_arr_i32::migrate(&mut db)?;
    good_query_sqlite!(
        r#"insert into "bananna_sqlite_gen_migrate_pre_migration" ( "hizat" ) values ( ?1 )"#;
        val = i32;
        &mut db,
        7
    )?;
    assert_eq!(good_query_many_sqlite!(
        r#"select "bananna_sqlite_gen_migrate_pre_migration" . "hizat" as "hizat" from "bananna_sqlite_gen_migrate_pre_migration" where "bananna_sqlite_gen_migrate_pre_migration" . "hizat" in rarray(?1)"#;
        vals = Vec < i32 >;
        &mut db,
        vec![7]
    )?, vec![7]);
    Ok(())
}

#[test]
fn test_param_custom() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_param_custom::migrate(&mut db)?;
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
        r#"insert into "bananna_sqlite_gen_param_opt_i32_null" ( "x_0" , "x_1" , "x_2" , "x_3" , "x_4" , "x_5" , "x_6" , "x_7" , "x_8" , "x_9" , "x_10" , "x_11" ) values ( ?1 , ?2 , ?3 , ?4 , ?5 , ?6 , ?7 , ?8 , ?9 , ?10 , ?11 , ?12 )"#;
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
    )?;
    let res = good_query_one_sqlite!(
        r#"select "bananna_sqlite_gen_param_opt_i32_null" . "x_0" as "x_0" , "bananna_sqlite_gen_param_opt_i32_null" . "x_1" as "x_1" , "bananna_sqlite_gen_param_opt_i32_null" . "x_2" as "x_2" , "bananna_sqlite_gen_param_opt_i32_null" . "x_3" as "x_3" , "bananna_sqlite_gen_param_opt_i32_null" . "x_4" as "x_4" , "bananna_sqlite_gen_param_opt_i32_null" . "x_5" as "x_5" , "bananna_sqlite_gen_param_opt_i32_null" . "x_6" as "x_6" , "bananna_sqlite_gen_param_opt_i32_null" . "x_7" as "x_7" , "bananna_sqlite_gen_param_opt_i32_null" . "x_8" as "x_8" , "bananna_sqlite_gen_param_opt_i32_null" . "x_9" as "x_9" , "bananna_sqlite_gen_param_opt_i32_null" . "x_10" as "x_10" , "bananna_sqlite_gen_param_opt_i32_null" . "x_11" as "x_11" from "bananna_sqlite_gen_param_opt_i32_null""#;
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
    sqlite_gen_param_opt_custom::migrate(&mut db)?;
    good_query_sqlite!(
        r#"insert into "bananna_sqlite_gen_param_custom" ( "hizat" ) values ( ?1 )"#;
        text = Option <& integration_tests:: MyString >;
        &mut db,
        Some(&MyString("higgins".into()))
    )?;
    assert_eq!(good_query_one_sqlite!(
        r#"select "bananna_sqlite_gen_param_custom" . "hizat" as "hizat" from "bananna_sqlite_gen_param_custom""#;
        ;
        &mut db
    )?, Some(MyString("higgins".into())));
    Ok(())
}

#[test]
fn test_insert_on_conflict_do_nothing() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_insert_on_conflict_do_nothing::migrate(&mut db)?;
    assert!(good_query_opt_sqlite!(
        r#"insert into "bannanana" ( "hizat" ) values ( ?1 ) on conflict do nothing returning 1 as "one""#;
        text = &str;
        &mut db,
        "soy"
    )?.is_some());
    assert!(good_query_opt_sqlite!(
        r#"insert into "bannanana" ( "hizat" ) values ( ?1 ) on conflict do nothing returning 1 as "one""#;
        text = &str;
        &mut db,
        "soy"
    )?.is_none());
    Ok(())
}

#[test]
fn test_insert_on_conflict_update() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_insert_on_conflict_update::migrate(&mut db)?;
    assert_eq!(good_query_one_sqlite!(
        r#"insert into "bannanana" ( "hizat" , "two" ) values ( ?1 , ?2 ) on conflict ( "hizat" ) do update set "two" = "bannanana" . "two" + 1 returning "bannanana" . "two" as "two""#;
        text = &str,
        two = i32;
        &mut db,
        "soy",
        33
    )?, 33);
    assert_eq!(good_query_one_sqlite!(
        r#"insert into "bannanana" ( "hizat" , "two" ) values ( ?1 , ?2 ) on conflict ( "hizat" ) do update set "two" = "bannanana" . "two" + 1 returning "bannanana" . "two" as "two""#;
        text = &str,
        two = i32;
        &mut db,
        "soy",
        7
    )?, 34);
    assert_eq!(good_query_one_sqlite!(
        r#"insert into "bannanana" ( "hizat" , "two" ) values ( ?1 , ?2 ) on conflict ( "hizat" ) do update set "two" = "bannanana" . "two" + 1 returning "bannanana" . "two" as "two""#;
        text = &str,
        two = i32;
        &mut db,
        "yyyy",
        7
    )?, 7);
    Ok(())
}

#[test]
fn test_update() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_update::migrate(&mut db)?;
    good_query_sqlite!(
        r#"insert into "bananna_sqlite_gen_param_opt_custom" ( "hizat" ) values ( 'yog' )"#;
        ;
        &mut db
    )?;
    assert_eq!(good_query_one_sqlite!(
        r#"select "bananna_sqlite_gen_param_opt_custom" . "hizat" as "hizat" from "bananna_sqlite_gen_param_opt_custom""#;
        ;
        &mut db
    )?, "yog");
    good_query_sqlite!(
        r#"update "bananna_sqlite_gen_param_opt_custom" set "hizat" = 'tep'"#;
        ;
        &mut db
    )?;
    assert_eq!(good_query_one_sqlite!(
        r#"select "bananna_sqlite_gen_param_opt_custom" . "hizat" as "hizat" from "bananna_sqlite_gen_param_opt_custom""#;
        ;
        &mut db
    )?, "tep");
    Ok(())
}

#[test]
fn test_update_where() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_update_where::migrate(&mut db)?;
    good_query_sqlite!(
        r#"insert into "ban" ( "hizat" ) values ( 'yog' )"#;
        ;
        &mut db
    )?;
    assert_eq!(good_query_one_sqlite!(
        r#"select "ban" . "hizat" as "hizat" from "ban""#;
        ;
        &mut db
    )?, "yog");
    good_query_sqlite!(
        r#"update "ban" set "hizat" = ?1 where "ban" . "hizat" = ?2"#;
        val = &str,
        cond = &str;
        &mut db,
        "tep",
        "yog2"
    )?;
    assert_eq!(good_query_one_sqlite!(
        r#"select "ban" . "hizat" as "hizat" from "ban""#;
        ;
        &mut db
    )?, "yog");
    good_query_sqlite!(
        r#"update "ban" set "hizat" = ?1 where "ban" . "hizat" = ?2"#;
        val = &str,
        cond = &str;
        &mut db,
        "tep",
        "yog"
    )?;
    assert_eq!(good_query_one_sqlite!(
        r#"select "ban" . "hizat" as "hizat" from "ban""#;
        ;
        &mut db
    )?, "tep");
    Ok(())
}

#[test]
fn test_update_returning() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_update_returning::migrate(&mut db)?;
    good_query_sqlite!(
        r#"insert into "b" ( "hizat" ) values ( 'yog' )"#;
        ;
        &mut db
    )?;
    assert_eq!(good_query_opt_sqlite!(
        r#"update "b" set "hizat" = 'tep' returning "b" . "hizat" as "hizat""#;
        ;
        &mut db
    )?, Some("tep".to_string()));
    Ok(())
}

#[test]
fn test_delete() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_delete::migrate(&mut db)?;
    good_query_sqlite!(
        r#"insert into "b" ( "hizat" ) values ( 'seeon' )"#;
        ;
        &mut db
    )?;
    assert_eq!(good_query_opt_sqlite!(
        r#"select "b" . "hizat" as "hizat" from "b""#;
        ;
        &mut db
    )?, Some("seeon".to_string()));
    good_query_sqlite!(
        r#"delete from "b""#;
        ;
        &mut db
    )?;
    assert_eq!(good_query_opt_sqlite!(
        r#"select "b" . "hizat" as "hizat" from "b""#;
        ;
        &mut db
    )?, None);
    Ok(())
}

#[test]
fn test_delete_where() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_delete_where::migrate(&mut db)?;
    good_query_sqlite!(
        r#"insert into "ba" ( "hizat" ) values ( 'seeon' )"#;
        ;
        &mut db
    )?;
    good_query_sqlite!(
        r#"delete from "ba" where "ba" . "hizat" = ?1"#;
        hiz = &str;
        &mut db,
        "nozo"
    )?;
    assert_eq!(good_query_opt_sqlite!(
        r#"select "ba" . "hizat" as "hizat" from "ba""#;
        ;
        &mut db
    )?, Some("seeon".to_string()));
    good_query_sqlite!(
        r#"delete from "ba" where "ba" . "hizat" = ?1"#;
        hiz = &str;
        &mut db,
        "seeon"
    )?;
    assert_eq!(good_query_opt_sqlite!(
        r#"select "ba" . "hizat" as "hizat" from "ba""#;
        ;
        &mut db
    )?, None);
    Ok(())
}

#[test]
fn test_delete_returning() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_delete_where::migrate(&mut db)?;
    good_query_sqlite!(
        r#"insert into "ba" ( "hizat" ) values ( 'seeon' )"#;
        ;
        &mut db
    )?;
    assert!(good_query_opt_sqlite!(
        r#"select "ba" . "hizat" as "hizat" from "ba""#;
        ;
        &mut db
    )?.is_some());
    good_query_sqlite!(
        r#"delete from "ba" where "ba" . "hizat" = ?1"#;
        hiz = &str;
        &mut db,
        "seeon"
    )?;
    assert!(good_query_opt_sqlite!(
        r#"select "ba" . "hizat" as "hizat" from "ba""#;
        ;
        &mut db
    )?.is_none());
    Ok(())
}

#[test]
fn test_select_join() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_select_join::migrate(&mut db)?;
    let res = good_query_one_sqlite!(
        r#"select "b" . "three" as "three" , "two_sqlite_gen_delete_returning" . "two" as "two" from "b" left join "two_sqlite_gen_delete_returning" on cast ( "b" . "hizat" as text ) = "two_sqlite_gen_delete_returning" . "hizat""#;
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
    sqlite_gen_select_group_by::migrate(&mut db)?;
    good_query_sqlite!(
        r#"insert into "bannanana" ( "hizat" , "hizat2" ) values ( ?1 , ?2 )"#;
        v = i32,
        v2 = i32;
        &mut db,
        1,
        7
    )?;
    good_query_sqlite!(
        r#"insert into "bannanana" ( "hizat" , "hizat2" ) values ( ?1 , ?2 )"#;
        v = i32,
        v2 = i32;
        &mut db,
        1,
        99
    )?;
    good_query_sqlite!(
        r#"insert into "bannanana" ( "hizat" , "hizat2" ) values ( ?1 , ?2 )"#;
        v = i32,
        v2 = i32;
        &mut db,
        2,
        3
    )?;
    good_query_sqlite!(
        r#"insert into "bannanana" ( "hizat" , "hizat2" ) values ( ?1 , ?2 )"#;
        v = i32,
        v2 = i32;
        &mut db,
        2,
        10
    )?;
    let mut res = good_query_many_sqlite!(
        r#"select sum ( "bannanana" . "hizat2" ) as "hizat2" from "bannanana" group by "bannanana" . "hizat""#;
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
    sqlite_gen_select_limit::migrate(&mut db)?;
    good_query_sqlite!(
        r#"insert into "bannanana" ( "hizat" ) values ( ?1 )"#;
        text = &str;
        &mut db,
        "soy"
    )?;
    good_query_sqlite!(
        r#"insert into "bannanana" ( "hizat" ) values ( ?1 )"#;
        text = &str;
        &mut db,
        "soy"
    )?;
    good_query_sqlite!(
        r#"insert into "bannanana" ( "hizat" ) values ( ?1 )"#;
        text = &str;
        &mut db,
        "soy"
    )?;
    assert_eq!(good_query_many_sqlite!(
        r#"select "bannanana" . "hizat" as "hizat" from "bannanana" limit 2"#;
        ;
        &mut db
    )?.len(), 2);
    Ok(())
}

#[test]
fn test_select_order() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_select_order::migrate(&mut db)?;
    good_query_sqlite!(
        r#"insert into "bannanana" ( "hizat" ) values ( ?1 )"#;
        v = i32;
        &mut db,
        0
    )?;
    good_query_sqlite!(
        r#"insert into "bannanana" ( "hizat" ) values ( ?1 )"#;
        v = i32;
        &mut db,
        12
    )?;
    good_query_sqlite!(
        r#"insert into "bannanana" ( "hizat" ) values ( ?1 )"#;
        v = i32;
        &mut db,
        9
    )?;
    assert_eq!(good_query_many_sqlite!(
        r#"select "bannanana" . "hizat" as "hizat" from "bannanana" order by "bannanana" . "hizat" asc"#;
        ;
        &mut db
    )?, vec![0, 9, 12]);
    Ok(())
}

#[test]
fn test_migrate_add_field() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_migrate_add_field::migrate(&mut db)?;
    match good_query_opt_sqlite!(
        r#"select "bannna" . "hizat" as "hizat" , "bannna" . "zomzom" as "zomzom" from "bannna""#;
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
    sqlite_gen_migrate_rename_field::migrate(&mut db)?;
    good_query_sqlite!(
        r#"insert into "bannna" ( "hizat" ) values ( 'nizoot' )"#;
        ;
        &mut db
    )?;
    Ok(())
}

#[test]
fn test_migrate_remove_field() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_migrate_remove_field::migrate(&mut db)?;
    good_query_sqlite!(
        r#"insert into "bnanaa" ( "hizat" ) values ( ?1 )"#;
        okolor = &str;
        &mut db,
        "yordol"
    )?;
    Ok(())
}

#[test]
fn test_migrate_add_table() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_migrate_add_table::migrate(&mut db)?;
    good_query_sqlite!(
        r#"insert into "two_sqlite_gen_migrate_remove_field" ( "two" ) values ( ?1 )"#;
        two = i32;
        &mut db,
        23
    )?;
    Ok(())
}

#[test]
fn test_migrate_rename_table() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_migrate_rename_table::migrate(&mut db)?;
    good_query_sqlite!(
        r#"insert into "bana" ( "hizat" ) values ( ?1 )"#;
        two = &str;
        &mut db,
        "inset"
    )?;
    Ok(())
}

#[test]
fn test_migrate_remove_table() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_migrate_remove_table::migrate(&mut db)?;
    Ok(())
}

#[test]
fn test_migrate_pre_migration() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_migrate_pre_migration::migrate(&mut db)?;
    Ok(())
}

#[test]
fn test_select_cte() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_select_cte::migrate(&mut db)?;
    good_query_sqlite!(
        r#"insert into "bannanana" ( "hizat" , "hizat2" ) values ( ?1 , ?2 )"#;
        v = i32,
        v2 = i32;
        &mut db,
        1,
        7
    )?;
    good_query_sqlite!(
        r#"insert into "bannanana" ( "hizat" , "hizat2" ) values ( ?1 , ?2 )"#;
        v = i32,
        v2 = i32;
        &mut db,
        1,
        99
    )?;
    let mut res = good_query_many_sqlite!(
        r#"with "hibbo" ( "zathi" ) as ( select "bannanana" . "hizat2" as "hizat2" from "bannanana" ) select "hibbo" . "zathi" as "zathi" from "hibbo""#;
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
    sqlite_gen_select_window::migrate(&mut db)?;
    good_query_sqlite!(
        r#"insert into "bannanana" ( "hizat" , "hizat2" ) values ( ?1 , ?2 )"#;
        v = i32,
        v2 = i32;
        &mut db,
        1,
        7
    )?;
    good_query_sqlite!(
        r#"insert into "bannanana" ( "hizat" , "hizat2" ) values ( ?1 , ?2 )"#;
        v = i32,
        v2 = i32;
        &mut db,
        1,
        99
    )?;
    good_query_sqlite!(
        r#"insert into "bannanana" ( "hizat" , "hizat2" ) values ( ?1 , ?2 )"#;
        v = i32,
        v2 = i32;
        &mut db,
        2,
        3
    )?;
    good_query_sqlite!(
        r#"insert into "bannanana" ( "hizat" , "hizat2" ) values ( ?1 , ?2 )"#;
        v = i32,
        v2 = i32;
        &mut db,
        2,
        10
    )?;
    let mut res = good_query_many_sqlite!(
        r#"select sum ( "bannanana" . "hizat2" ) over ( partition by "bannanana" . "hizat" ) as "hizat2" from "bannanana""#;
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
    sqlite_gen_select_junction::migrate(&mut db)?;
    good_query_sqlite!(
        r#"insert into "bannanana" ( "hizat" , "hizat2" ) values ( ?1 , ?2 )"#;
        v = i32,
        v2 = i32;
        &mut db,
        1,
        7
    )?;
    good_query_sqlite!(
        r#"insert into "bannanana" ( "hizat" , "hizat2" ) values ( ?1 , ?2 )"#;
        v = i32,
        v2 = i32;
        &mut db,
        2,
        3
    )?;
    let mut res = good_query_many_sqlite!(
        r#"select "bannanana" . "hizat" as "hizat" from "bannanana" union select "bannanana" . "hizat2" as "hizat2" from "bannanana""#;
        ;
        &mut db
    )?;
    res.sort();
    assert_eq!(res, vec![1, 2, 3, 7]);
    Ok(())
}
