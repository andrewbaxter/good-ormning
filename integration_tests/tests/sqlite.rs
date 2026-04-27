use {
    chrono::{
        TimeZone,
        Utc,
    },
    good_ormning::good_module,
    integration_tests::MyString,
};

#[test]
fn test_hello_world() -> Result<(), loga::Error> {
    good_module!("sqlite_gen_hello_world");
    let mut db = rusqlite::Connection::open_in_memory()?;
    dbm::migrate(&mut db, None)?;
    good_ormning::sqlite::good_query!(
        "sqlite_gen_hello_world",
        dbm::DbSqliteGenHelloWorld1(&mut db),
        r#"insert into "hello_world_users" ( "name" , "points" ) values ( ?1 , ?2 )"#;
        p1 = string,
        p2 = i64;
        "rust human",
        0
    )?;
    for user_id in good_ormning::sqlite::good_query_many!(
        "sqlite_gen_hello_world",
        dbm::DbSqliteGenHelloWorld1(&mut db),
        r#"select "hello_world_users" . "rowid" as "rowid" from "hello_world_users""#
    )? {
        let user = good_ormning::sqlite::good_query_one!(
            "sqlite_gen_hello_world",
            dbm::DbSqliteGenHelloWorld1(&mut db),
            r#"select "hello_world_users" . "name" as "name" , "hello_world_users" . "points" as "points" from "hello_world_users" where "hello_world_users" . "rowid" = ?1"#;
            p1 = i64;
            user_id
        )?;
        println!("User {}: {}", user_id, user.name);
    }
    Ok(())
}

#[test]
fn test_base_insert() -> Result<(), loga::Error> {
    good_module!("sqlite_gen_base_insert");
    let mut db = rusqlite::Connection::open_in_memory()?;
    dbm::migrate(&mut db, None)?;
    good_ormning::sqlite::good_query!(
        "sqlite_gen_base_insert",
        dbm::DbSqliteGenBaseInsert1(&mut db),
        r#"insert into "bannanana" ( "hizat" ) values ( ?1 )"#;
        p1 = string;
        "soy"
    )?;
    assert_eq!(
        good_ormning::sqlite::good_query_one!(
            "sqlite_gen_base_insert",
            dbm::DbSqliteGenBaseInsert1(&mut db),
            r#"select "bannanana" . "hizat" as "hizat" from "bannanana""#
        )?,
        "soy"
    );
    Ok(())
}

#[test]
fn test_get_version_premigrate() -> Result<(), loga::Error> {
    good_module!("sqlite_gen_base_insert");
    let mut db = rusqlite::Connection::open_in_memory()?;
    assert_eq!(dbm::get_schema_version(&mut db)?, None);
    Ok(())
}

#[test]
fn test_get_version_postmigrate() -> Result<(), loga::Error> {
    good_module!("sqlite_gen_base_insert");
    let mut db = rusqlite::Connection::open_in_memory()?;
    dbm::migrate(&mut db, None)?;
    assert_eq!(dbm::get_schema_version(&mut db)?, Some(1));
    Ok(())
}

#[test]
fn test_constraint() -> Result<(), loga::Error> {
    good_module!("sqlite_gen_base_insert");
    let mut db = rusqlite::Connection::open_in_memory()?;
    dbm::migrate(&mut db, None)?;
    Ok(())
}

#[test]
fn test_param_i32() -> Result<(), loga::Error> {
    good_module!("sqlite_gen_param_i32");
    let mut db = rusqlite::Connection::open_in_memory()?;
    dbm::migrate(&mut db, None)?;
    good_ormning::sqlite::good_query!(
        "sqlite_gen_param_i32",
        dbm::DbSqliteGenParamI321(&mut db),
        r#"insert into "bananna" ( "hizat" ) values ( ?1 )"#;
        p1 = i32;
        22
    )?;
    assert_eq!(
        good_ormning::sqlite::good_query_one!(
            "sqlite_gen_param_i32",
            dbm::DbSqliteGenParamI321(&mut db),
            r#"select "bananna" . "hizat" as "hizat" from "bananna""#
        )?,
        22
    );
    Ok(())
}

#[test]
fn test_param_utctime_s_chrono() -> Result<(), loga::Error> {
    good_module!("sqlite_gen_param_utctime_s_chrono");
    let mut db = rusqlite::Connection::open_in_memory()?;
    dbm::migrate(&mut db, None)?;
    let ref_date = chrono::TimeZone::with_ymd_and_hms(&chrono::Utc, 1937, 12, 1, 0, 0, 0).unwrap();
    good_ormning::sqlite::good_query!(
        "sqlite_gen_param_utctime_s_chrono",
        dbm::DbSqliteGenParamUtctimeSChrono1(&mut db),
        r#"insert into "bananna" ( "hizat" ) values ( ?1 )"#;
        p1 = utctime_s_chrono;
        ref_date
    )?;
    assert_eq!(
        good_ormning::sqlite::good_query_one!(
            "sqlite_gen_param_utctime_s_chrono",
            dbm::DbSqliteGenParamUtctimeSChrono1(&mut db),
            r#"select "bananna" . "hizat" as "hizat" from "bananna""#
        )?,
        ref_date
    );
    Ok(())
}

#[test]
fn test_param_utctime_ms_chrono() -> Result<(), loga::Error> {
    good_module!("sqlite_gen_param_utctime_ms_chrono");
    let mut db = rusqlite::Connection::open_in_memory()?;
    dbm::migrate(&mut db, None)?;
    let ref_date = chrono::TimeZone::with_ymd_and_hms(&chrono::Utc, 1937, 12, 1, 0, 0, 0).unwrap();
    good_ormning::sqlite::good_query!(
        "sqlite_gen_param_utctime_ms_chrono",
        dbm::DbSqliteGenParamUtctimeMsChrono1(&mut db),
        r#"insert into "bananna" ( "hizat" ) values ( ?1 )"#;
        p1 = utctime_ms_chrono;
        ref_date
    )?;
    assert_eq!(
        good_ormning::sqlite::good_query_one!(
            "sqlite_gen_param_utctime_ms_chrono",
            dbm::DbSqliteGenParamUtctimeMsChrono1(&mut db),
            r#"select "bananna" . "hizat" as "hizat" from "bananna""#
        )?,
        ref_date
    );
    Ok(())
}

#[test]
fn test_param_utctime_s_jiff() -> Result<(), loga::Error> {
    good_module!("sqlite_gen_param_utctime_s_jiff");
    let mut db = rusqlite::Connection::open_in_memory()?;
    dbm::migrate(&mut db, None)?;
    let ref_date =
        jiff::civil::DateTime::new(1937, 12, 1, 0, 0, 0, 0)
            .unwrap()
            .to_zoned(jiff::tz::TimeZone::UTC)
            .unwrap()
            .timestamp();
    good_ormning::sqlite::good_query!(
        "sqlite_gen_param_utctime_s_jiff",
        dbm::DbSqliteGenParamUtctimeSJiff1(&mut db),
        r#"insert into "bananna" ( "hizat" ) values ( ?1 )"#;
        p1 = utctime_s_jiff;
        ref_date
    )?;
    assert_eq!(
        good_ormning::sqlite::good_query_one!(
            "sqlite_gen_param_utctime_s_jiff",
            dbm::DbSqliteGenParamUtctimeSJiff1(&mut db),
            r#"select "bananna" . "hizat" as "hizat" from "bananna""#
        )?,
        ref_date
    );
    Ok(())
}

#[test]
fn test_param_utctime_ms_jiff() -> Result<(), loga::Error> {
    good_module!("sqlite_gen_param_utctime_ms_jiff");
    let mut db = rusqlite::Connection::open_in_memory()?;
    dbm::migrate(&mut db, None)?;
    let ref_date =
        jiff::civil::DateTime::new(1937, 12, 1, 0, 0, 0, 0)
            .unwrap()
            .to_zoned(jiff::tz::TimeZone::UTC)
            .unwrap()
            .timestamp();
    good_ormning::sqlite::good_query!(
        "sqlite_gen_param_utctime_ms_jiff",
        dbm::DbSqliteGenParamUtctimeMsJiff1(&mut db),
        r#"insert into "bananna" ( "hizat" ) values ( ?1 )"#;
        p1 = utctime_ms_jiff;
        ref_date
    )?;
    assert_eq!(
        good_ormning::sqlite::good_query_one!(
            "sqlite_gen_param_utctime_ms_jiff",
            dbm::DbSqliteGenParamUtctimeMsJiff1(&mut db),
            r#"select "bananna" . "hizat" as "hizat" from "bananna""#
        )?,
        ref_date
    );
    Ok(())
}

#[test]
fn test_param_opt_i32() -> Result<(), loga::Error> {
    good_module!("sqlite_gen_param_opt_i32");
    let mut db = rusqlite::Connection::open_in_memory()?;
    dbm::migrate(&mut db, None)?;
    good_ormning::sqlite::good_query!(
        "sqlite_gen_param_opt_i32",
        dbm::DbSqliteGenParamOptI321(&mut db),
        r#"insert into "bananna" ( "hizat" ) values ( ?1 )"#;
        p1 = opt i32;
        Some(47)
    )?;
    assert_eq!(
        good_ormning::sqlite::good_query_one!(
            "sqlite_gen_param_opt_i32",
            dbm::DbSqliteGenParamOptI321(&mut db),
            r#"select "bananna" . "hizat" as "hizat" from "bananna""#
        )?,
        Some(47)
    );
    Ok(())
}

#[test]
fn test_param_opt_i32_null() -> Result<(), loga::Error> {
    good_module!("sqlite_gen_param_opt_i32_null");
    let mut db = rusqlite::Connection::open_in_memory()?;
    dbm::migrate(&mut db, None)?;
    good_ormning::sqlite::good_query!(
        "sqlite_gen_param_opt_i32_null",
        dbm::DbSqliteGenParamOptI32Null1(&mut db),
        r#"insert into "bananna" ( "hizat" ) values ( null )"#
    )?;
    assert_eq!(
        good_ormning::sqlite::good_query_one!(
            "sqlite_gen_param_opt_i32_null",
            dbm::DbSqliteGenParamOptI32Null1(&mut db),
            r#"select "bananna" . "hizat" as "hizat" from "bananna""#
        )?,
        None
    );
    Ok(())
}

#[test]
fn test_param_arr_i32() -> Result<(), loga::Error> {
    good_module!("sqlite_gen_param_arr_i32");
    let mut db = rusqlite::Connection::open_in_memory()?;
    dbm::migrate(&mut db, None)?;
    good_ormning::sqlite::good_query!(
        "sqlite_gen_param_arr_i32",
        dbm::DbSqliteGenParamArrI321(&mut db),
        r#"insert into "bananna" ( "hizat" ) values ( ?1 )"#;
        p1 = i32;
        7
    )?;
    assert_eq!(good_ormning::sqlite::good_query_many!(
        "sqlite_gen_param_arr_i32",
        dbm::DbSqliteGenParamArrI321(&mut db),
        r#"select "bananna" . "hizat" as "hizat" from "bananna" where "bananna" . "hizat" in ( select value from rarray ( ?1 ) )"#;
        p1 = arr i32;
        vec![7]
    )?, vec![7]);
    Ok(())
}

#[test]
fn test_param_custom() -> Result<(), loga::Error> {
    good_module!("sqlite_gen_param_custom");
    let mut db = rusqlite::Connection::open_in_memory()?;
    dbm::migrate(&mut db, None)?;
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
    good_ormning::sqlite::good_query!(
        "sqlite_gen_param_custom",
        dbm::DbSqliteGenParamCustom1(&mut db),
        r#"insert into "bananna" ( "x_0" , "x_1" , "x_2" , "x_3" , "x_4" , "x_5" , "x_6" , "x_7" , "x_8" , "x_9" , "x_10" , "x_11" ) values ( ?1 , ?2 , ?3 , ?4 , ?5 , ?6 , ?7 , ?8 , ?9 , ?10 , ?11 , ?12 )"#;
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
    )?;
    let res =
        good_ormning::sqlite::good_query_one!(
            "sqlite_gen_param_custom",
            dbm::DbSqliteGenParamCustom1(&mut db),
            r#"select "bananna" . "x_0" as "x_0" , "bananna" . "x_1" as "x_1" , "bananna" . "x_2" as "x_2" , "bananna" . "x_3" as "x_3" , "bananna" . "x_4" as "x_4" , "bananna" . "x_5" as "x_5" , "bananna" . "x_6" as "x_6" , "bananna" . "x_7" as "x_7" , "bananna" . "x_8" as "x_8" , "bananna" . "x_9" as "x_9" , "bananna" . "x_10" as "x_10" , "bananna" . "x_11" as "x_11" from "bananna""#
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
    good_module!("sqlite_gen_param_opt_custom");
    let mut db = rusqlite::Connection::open_in_memory()?;
    dbm::migrate(&mut db, None)?;
    good_ormning::sqlite::good_query!(
        "sqlite_gen_param_opt_custom",
        dbm::DbSqliteGenParamOptCustom1(&mut db),
        r#"insert into "bananna" ( "hizat" ) values ( ?1 )"#;
        p1 = opt MyString;
        Some(&MyString("higgins".into()))
    )?;
    assert_eq!(
        good_ormning::sqlite::good_query_one!(
            "sqlite_gen_param_opt_custom",
            dbm::DbSqliteGenParamOptCustom1(&mut db),
            r#"select "bananna" . "hizat" as "hizat" from "bananna""#
        )?,
        Some(MyString("higgins".into()))
    );
    Ok(())
}

#[test]
fn test_insert_on_conflict_do_nothing() -> Result<(), loga::Error> {
    good_module!("sqlite_gen_insert_on_conflict_do_nothing");
    let mut db = rusqlite::Connection::open_in_memory()?;
    dbm::migrate(&mut db, None)?;
    assert!(good_ormning::sqlite::good_query_opt!(
        "sqlite_gen_insert_on_conflict_do_nothing",
        dbm::DbSqliteGenInsertOnConflictDoNothing1(&mut db),
        r#"insert into "bannanana" ( "hizat" ) values ( ?1 ) on conflict do nothing returning 1 as "one""#;
        p1 = string;
        "soy"
    )?.is_some());
    assert!(good_ormning::sqlite::good_query_opt!(
        "sqlite_gen_insert_on_conflict_do_nothing",
        dbm::DbSqliteGenInsertOnConflictDoNothing1(&mut db),
        r#"insert into "bannanana" ( "hizat" ) values ( ?1 ) on conflict do nothing returning 1 as "one""#;
        p1 = string;
        "soy"
    )?.is_none());
    Ok(())
}

#[test]
fn test_insert_on_conflict_update() -> Result<(), loga::Error> {
    good_module!("sqlite_gen_insert_on_conflict_update");
    let mut db = rusqlite::Connection::open_in_memory()?;
    dbm::migrate(&mut db, None)?;
    assert_eq!(good_ormning::sqlite::good_query_one!(
        "sqlite_gen_insert_on_conflict_update",
        dbm::DbSqliteGenInsertOnConflictUpdate1(&mut db),
        r#"insert into "bannanana" ( "hizat" , "two" ) values ( ?1 , ?2 ) on conflict ( "hizat" ) do update set "two" = "bannanana" . "two" + 1 returning "bannanana" . "two" as "two""#;
        p1 = string,
        p2 = i32;
        "soy",
        33
    )?, 33);
    assert_eq!(good_ormning::sqlite::good_query_one!(
        "sqlite_gen_insert_on_conflict_update",
        dbm::DbSqliteGenInsertOnConflictUpdate1(&mut db),
        r#"insert into "bannanana" ( "hizat" , "two" ) values ( ?1 , ?2 ) on conflict ( "hizat" ) do update set "two" = "bannanana" . "two" + 1 returning "bannanana" . "two" as "two""#;
        p1 = string,
        p2 = i32;
        "soy",
        7
    )?, 34);
    assert_eq!(good_ormning::sqlite::good_query_one!(
        "sqlite_gen_insert_on_conflict_update",
        dbm::DbSqliteGenInsertOnConflictUpdate1(&mut db),
        r#"insert into "bannanana" ( "hizat" , "two" ) values ( ?1 , ?2 ) on conflict ( "hizat" ) do update set "two" = "bannanana" . "two" + 1 returning "bannanana" . "two" as "two""#;
        p1 = string,
        p2 = i32;
        "yyyy",
        7
    )?, 7);
    Ok(())
}

#[test]
fn test_update() -> Result<(), loga::Error> {
    good_module!("sqlite_gen_update");
    let mut db = rusqlite::Connection::open_in_memory()?;
    dbm::migrate(&mut db, None)?;
    good_ormning::sqlite::good_query!(
        "sqlite_gen_update",
        dbm::DbSqliteGenUpdate1(&mut db),
        r#"insert into "bananna" ( "hizat" ) values ( 'yog' )"#
    )?;
    assert_eq!(
        good_ormning::sqlite::good_query_one!(
            "sqlite_gen_update",
            dbm::DbSqliteGenUpdate1(&mut db),
            r#"select "bananna" . "hizat" as "hizat" from "bananna""#
        )?,
        "yog"
    );
    good_ormning::sqlite::good_query!(
        "sqlite_gen_update",
        dbm::DbSqliteGenUpdate1(&mut db),
        r#"update "bananna" set "hizat" = 'tep'"#
    )?;
    assert_eq!(
        good_ormning::sqlite::good_query_one!(
            "sqlite_gen_update",
            dbm::DbSqliteGenUpdate1(&mut db),
            r#"select "bananna" . "hizat" as "hizat" from "bananna""#
        )?,
        "tep"
    );
    Ok(())
}

#[test]
fn test_update_where() -> Result<(), loga::Error> {
    good_module!("sqlite_gen_update_where");
    let mut db = rusqlite::Connection::open_in_memory()?;
    dbm::migrate(&mut db, None)?;
    good_ormning::sqlite::good_query!(
        "sqlite_gen_update_where",
        dbm::DbSqliteGenUpdateWhere1(&mut db),
        r#"insert into "ban" ( "hizat" ) values ( 'yog' )"#
    )?;
    assert_eq!(
        good_ormning::sqlite::good_query_one!(
            "sqlite_gen_update_where",
            dbm::DbSqliteGenUpdateWhere1(&mut db),
            r#"select "ban" . "hizat" as "hizat" from "ban""#
        )?,
        "yog"
    );
    good_ormning::sqlite::good_query!(
        "sqlite_gen_update_where",
        dbm::DbSqliteGenUpdateWhere1(&mut db),
        r#"update "ban" set "hizat" = ?1 where "ban" . "hizat" = ?2"#;
        p1 = string,
        p2 = string;
        "tep",
        "yog2"
    )?;
    assert_eq!(
        good_ormning::sqlite::good_query_one!(
            "sqlite_gen_update_where",
            dbm::DbSqliteGenUpdateWhere1(&mut db),
            r#"select "ban" . "hizat" as "hizat" from "ban""#
        )?,
        "yog"
    );
    good_ormning::sqlite::good_query!(
        "sqlite_gen_update_where",
        dbm::DbSqliteGenUpdateWhere1(&mut db),
        r#"update "ban" set "hizat" = ?1 where "ban" . "hizat" = ?2"#;
        p1 = string,
        p2 = string;
        "tep",
        "yog"
    )?;
    assert_eq!(
        good_ormning::sqlite::good_query_one!(
            "sqlite_gen_update_where",
            dbm::DbSqliteGenUpdateWhere1(&mut db),
            r#"select "ban" . "hizat" as "hizat" from "ban""#
        )?,
        "tep"
    );
    Ok(())
}

#[test]
fn test_update_returning() -> Result<(), loga::Error> {
    good_module!("sqlite_gen_update_returning");
    let mut db = rusqlite::Connection::open_in_memory()?;
    dbm::migrate(&mut db, None)?;
    good_ormning::sqlite::good_query!(
        "sqlite_gen_update_returning",
        dbm::DbSqliteGenUpdateReturning1(&mut db),
        r#"insert into "b" ( "hizat" ) values ( 'yog' )"#
    )?;
    assert_eq!(
        good_ormning::sqlite::good_query_opt!(
            "sqlite_gen_update_returning",
            dbm::DbSqliteGenUpdateReturning1(&mut db),
            r#"update "b" set "hizat" = 'tep' returning "b" . "hizat" as "hizat""#
        )?,
        Some("tep".to_string())
    );
    Ok(())
}

#[test]
fn test_delete() -> Result<(), loga::Error> {
    good_module!("sqlite_gen_delete");
    let mut db = rusqlite::Connection::open_in_memory()?;
    dbm::migrate(&mut db, None)?;
    good_ormning::sqlite::good_query!(
        "sqlite_gen_delete",
        dbm::DbSqliteGenDelete1(&mut db),
        r#"insert into "b" ( "hizat" ) values ( 'seeon' )"#
    )?;
    assert_eq!(
        good_ormning::sqlite::good_query_opt!(
            "sqlite_gen_delete",
            dbm::DbSqliteGenDelete1(&mut db),
            r#"select "b" . "hizat" as "hizat" from "b""#
        )?,
        Some("seeon".to_string())
    );
    good_ormning::sqlite::good_query!("sqlite_gen_delete", dbm::DbSqliteGenDelete1(&mut db), r#"delete from "b""#)?;
    assert_eq!(
        good_ormning::sqlite::good_query_opt!(
            "sqlite_gen_delete",
            dbm::DbSqliteGenDelete1(&mut db),
            r#"select "b" . "hizat" as "hizat" from "b""#
        )?,
        None
    );
    Ok(())
}

#[test]
fn test_delete_where() -> Result<(), loga::Error> {
    good_module!("sqlite_gen_delete_where");
    let mut db = rusqlite::Connection::open_in_memory()?;
    dbm::migrate(&mut db, None)?;
    good_ormning::sqlite::good_query!(
        "sqlite_gen_delete_where",
        dbm::DbSqliteGenDeleteWhere1(&mut db),
        r#"insert into "ba" ( "hizat" ) values ( 'seeon' )"#
    )?;
    good_ormning::sqlite::good_query!(
        "sqlite_gen_delete_where",
        dbm::DbSqliteGenDeleteWhere1(&mut db),
        r#"delete from "ba" where "ba" . "hizat" = ?1"#;
        p1 = string;
        "nozo"
    )?;
    assert_eq!(
        good_ormning::sqlite::good_query_opt!(
            "sqlite_gen_delete_where",
            dbm::DbSqliteGenDeleteWhere1(&mut db),
            r#"select "ba" . "hizat" as "hizat" from "ba""#
        )?,
        Some("seeon".to_string())
    );
    good_ormning::sqlite::good_query!(
        "sqlite_gen_delete_where",
        dbm::DbSqliteGenDeleteWhere1(&mut db),
        r#"delete from "ba" where "ba" . "hizat" = ?1"#;
        p1 = string;
        "seeon"
    )?;
    assert_eq!(
        good_ormning::sqlite::good_query_opt!(
            "sqlite_gen_delete_where",
            dbm::DbSqliteGenDeleteWhere1(&mut db),
            r#"select "ba" . "hizat" as "hizat" from "ba""#
        )?,
        None
    );
    Ok(())
}

#[test]
fn test_delete_returning() -> Result<(), loga::Error> {
    good_module!("sqlite_gen_delete_returning");
    let mut db = rusqlite::Connection::open_in_memory()?;
    dbm::migrate(&mut db, None)?;
    good_ormning::sqlite::good_query!(
        "sqlite_gen_delete_returning",
        dbm::DbSqliteGenDeleteReturning1(&mut db),
        r#"insert into "b" ( "hizat" ) values ( 'seeon' )"#
    )?;
    assert!(
        good_ormning::sqlite::good_query_opt!(
            "sqlite_gen_delete_returning",
            dbm::DbSqliteGenDeleteReturning1(&mut db),
            r#"select "b" . "hizat" as "hizat" from "b""#
        )?.is_some()
    );
    good_ormning::sqlite::good_query!(
        "sqlite_gen_delete_returning",
        dbm::DbSqliteGenDeleteReturning1(&mut db),
        r#"delete from "b" where "b" . "hizat" = ?1"#;
        p1 = string;
        "seeon"
    )?;
    assert!(
        good_ormning::sqlite::good_query_opt!(
            "sqlite_gen_delete_returning",
            dbm::DbSqliteGenDeleteReturning1(&mut db),
            r#"select "b" . "hizat" as "hizat" from "b""#
        )?.is_none()
    );
    Ok(())
}

#[test]
fn test_select_join() -> Result<(), loga::Error> {
    good_module!("sqlite_gen_select_join");
    let mut db = rusqlite::Connection::open_in_memory()?;
    dbm::migrate(&mut db, Some(&|v| {
        match v {
            dbm::DbSqliteGenSelectJoinVersions::V1(mut db) => {
                good_ormning::sqlite::good_query!(
                    "sqlite_gen_select_join",
                    dbm::DbSqliteGenSelectJoin1(&mut *db.0),
                    r#"insert into "b" ( "hizat" , "three" ) values ( 'key' , 33 )"#
                )?;
                good_ormning::sqlite::good_query!(
                    "sqlite_gen_select_join",
                    dbm::DbSqliteGenSelectJoin1(&mut *db.0),
                    r#"insert into "select_join_two" ( "hizat" , "two" ) values ( 'key' , 'no' )"#
                )?;
            },
            _ => { },
        }
        Ok(())
    }))?;
    let res =
        good_ormning::sqlite::good_query_one!(
            "sqlite_gen_select_join",
            dbm::DbSqliteGenSelectJoin1(&mut db),
            r#"select "b" . "three" as "three" , "select_join_two" . "two" as "two" from "b" left join "select_join_two" on ( "b" . "hizat" ) = "select_join_two" . "hizat""#
        )?;
    assert_eq!(res.three, 33);
    assert_eq!(res.two, Some("no".into()));
    Ok(())
}

#[test]
fn test_select_group_by() -> Result<(), loga::Error> {
    good_module!("sqlite_gen_select_group_by");
    let mut db = rusqlite::Connection::open_in_memory()?;
    dbm::migrate(&mut db, None)?;
    good_ormning::sqlite::good_query!(
        "sqlite_gen_select_group_by",
        dbm::DbSqliteGenSelectGroupBy1(&mut db),
        r#"insert into "bannanana" ( "hizat" , "hizat2" ) values ( ?1 , ?2 )"#;
        p1 = i32,
        p2 = i32;
        1,
        7
    )?;
    good_ormning::sqlite::good_query!(
        "sqlite_gen_select_group_by",
        dbm::DbSqliteGenSelectGroupBy1(&mut db),
        r#"insert into "bannanana" ( "hizat" , "hizat2" ) values ( ?1 , ?2 )"#;
        p1 = i32,
        p2 = i32;
        1,
        99
    )?;
    good_ormning::sqlite::good_query!(
        "sqlite_gen_select_group_by",
        dbm::DbSqliteGenSelectGroupBy1(&mut db),
        r#"insert into "bannanana" ( "hizat" , "hizat2" ) values ( ?1 , ?2 )"#;
        p1 = i32,
        p2 = i32;
        2,
        3
    )?;
    good_ormning::sqlite::good_query!(
        "sqlite_gen_select_group_by",
        dbm::DbSqliteGenSelectGroupBy1(&mut db),
        r#"insert into "bannanana" ( "hizat" , "hizat2" ) values ( ?1 , ?2 )"#;
        p1 = i32,
        p2 = i32;
        2,
        10
    )?;
    let mut res =
        good_ormning::sqlite::good_query_many!(
            "sqlite_gen_select_group_by",
            dbm::DbSqliteGenSelectGroupBy1(&mut db),
            r#"select sum ( "bannanana" . "hizat2" ) as "hizat2" from "bannanana" group by "bannanana" . "hizat""#
        )?;
    res.sort();
    assert_eq!(res, vec![13, 106]);
    Ok(())
}

#[test]
fn test_select_limit() -> Result<(), loga::Error> {
    good_module!("sqlite_gen_select_limit");
    let mut db = rusqlite::Connection::open_in_memory()?;
    dbm::migrate(&mut db, None)?;
    good_ormning::sqlite::good_query!(
        "sqlite_gen_select_limit",
        dbm::DbSqliteGenSelectLimit1(&mut db),
        r#"insert into "bannanana" ( "hizat" ) values ( ?1 )"#;
        p1 = string;
        "soy"
    )?;
    good_ormning::sqlite::good_query!(
        "sqlite_gen_select_limit",
        dbm::DbSqliteGenSelectLimit1(&mut db),
        r#"insert into "bannanana" ( "hizat" ) values ( ?1 )"#;
        p1 = string;
        "soy"
    )?;
    good_ormning::sqlite::good_query!(
        "sqlite_gen_select_limit",
        dbm::DbSqliteGenSelectLimit1(&mut db),
        r#"insert into "bannanana" ( "hizat" ) values ( ?1 )"#;
        p1 = string;
        "soy"
    )?;
    assert_eq!(
        good_ormning::sqlite::good_query_many!(
            "sqlite_gen_select_limit",
            dbm::DbSqliteGenSelectLimit1(&mut db),
            r#"select "bannanana" . "hizat" as "hizat" from "bannanana" limit 2"#
        )?.len(),
        2
    );
    Ok(())
}

#[test]
fn test_select_order() -> Result<(), loga::Error> {
    good_module!("sqlite_gen_select_order");
    let mut db = rusqlite::Connection::open_in_memory()?;
    dbm::migrate(&mut db, None)?;
    good_ormning::sqlite::good_query!(
        "sqlite_gen_select_order",
        dbm::DbSqliteGenSelectOrder1(&mut db),
        r#"insert into "bannanana" ( "hizat" ) values ( ?1 )"#;
        p1 = i32;
        0
    )?;
    good_ormning::sqlite::good_query!(
        "sqlite_gen_select_order",
        dbm::DbSqliteGenSelectOrder1(&mut db),
        r#"insert into "bannanana" ( "hizat" ) values ( ?1 )"#;
        p1 = i32;
        12
    )?;
    good_ormning::sqlite::good_query!(
        "sqlite_gen_select_order",
        dbm::DbSqliteGenSelectOrder1(&mut db),
        r#"insert into "bannanana" ( "hizat" ) values ( ?1 )"#;
        p1 = i32;
        9
    )?;
    assert_eq!(
        good_ormning::sqlite::good_query_many!(
            "sqlite_gen_select_order",
            dbm::DbSqliteGenSelectOrder1(&mut db),
            r#"select "bannanana" . "hizat" as "hizat" from "bannanana" order by "bannanana" . "hizat" asc"#
        )?,
        vec![0, 9, 12]
    );
    Ok(())
}

#[test]
fn test_migrate_add_field() -> Result<(), loga::Error> {
    good_module!("sqlite_gen_migrate_add_field");
    let mut db = rusqlite::Connection::open_in_memory()?;
    dbm::migrate(&mut db, Some(&|v| {
        match v {
            dbm::DbSqliteGenMigrateAddFieldVersions::V0(mut db) => {
                good_ormning::sqlite::good_query!(
                    "sqlite_gen_migrate_add_field",
                    0,
                    dbm::DbSqliteGenMigrateAddField0(&mut *db.0),
                    r#"insert into "bannna" ( "hizat" ) values ( 'nizoot' )"#
                )?;
            },
            _ => { },
        }
        Ok(())
    }))?;
    match good_ormning::sqlite::good_query_opt!(
        "sqlite_gen_migrate_add_field",
        dbm::DbSqliteGenMigrateAddField1(&mut db),
        r#"select "bannna" . "hizat" as "hizat" , "bannna" . "zomzom" as "zomzom" from "bannna""#
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
    good_module!("sqlite_gen_migrate_rename_field");
    let mut db = rusqlite::Connection::open_in_memory()?;
    dbm::migrate(&mut db, None)?;
    good_ormning::sqlite::good_query!(
        "sqlite_gen_migrate_rename_field",
        dbm::DbSqliteGenMigrateRenameField1(&mut db),
        r#"insert into "bannna" ( "hizat" ) values ( 'nizoot' )"#
    )?;
    Ok(())
}

#[test]
fn test_migrate_remove_field() -> Result<(), loga::Error> {
    good_module!("sqlite_gen_migrate_remove_field");
    let mut db = rusqlite::Connection::open_in_memory()?;
    dbm::migrate(&mut db, None)?;
    good_ormning::sqlite::good_query!(
        "sqlite_gen_migrate_remove_field",
        dbm::DbSqliteGenMigrateRemoveField1(&mut db),
        r#"insert into "bnanaa" ( "hizat" ) values ( ?1 )"#;
        p1 = string;
        "yordol"
    )?;
    Ok(())
}

#[test]
fn test_migrate_add_table() -> Result<(), loga::Error> {
    good_module!("sqlite_gen_migrate_add_table");
    let mut db = rusqlite::Connection::open_in_memory()?;
    dbm::migrate(&mut db, None)?;
    good_ormning::sqlite::good_query!(
        "sqlite_gen_migrate_add_table",
        dbm::DbSqliteGenMigrateAddTable1(&mut db),
        r#"insert into "migrate_add_table_two" ( "two" ) values ( ?1 )"#;
        p1 = i32;
        23
    )?;
    Ok(())
}

#[test]
fn test_migrate_rename_table() -> Result<(), loga::Error> {
    good_module!("sqlite_gen_migrate_rename_table");
    let mut db = rusqlite::Connection::open_in_memory()?;
    dbm::migrate(&mut db, None)?;
    good_ormning::sqlite::good_query!(
        "sqlite_gen_migrate_rename_table",
        dbm::DbSqliteGenMigrateRenameTable1(&mut db),
        r#"insert into "bana" ( "hizat" ) values ( ?1 )"#;
        p1 = string;
        "inset"
    )?;
    Ok(())
}

#[test]
fn test_migrate_remove_table() -> Result<(), loga::Error> {
    good_module!("sqlite_gen_migrate_remove_table");
    let mut db = rusqlite::Connection::open_in_memory()?;
    dbm::migrate(&mut db, None)?;
    Ok(())
}

#[test]
fn test_migrate_pre_migration() -> Result<(), loga::Error> {
    good_module!("sqlite_gen_migrate_pre_migration");
    let mut db = rusqlite::Connection::open_in_memory()?;
    dbm::migrate(&mut db, Some(&|v| {
        match v {
            dbm::DbSqliteGenMigratePreMigrationVersions::V0(mut db) => {
                good_ormning::sqlite::good_query!(
                    "sqlite_gen_migrate_pre_migration",
                    0,
                    dbm::DbSqliteGenMigratePreMigration0(&mut *db.0),
                    r#"insert into "migrate_pre_migration_v0_two" ( "two" ) values ( 7 )"#
                )?;
            },
            _ => { },
        }
        Ok(())
    }))?;
    Ok(())
}

#[test]
fn test_select_cte() -> Result<(), loga::Error> {
    good_module!("sqlite_gen_select_cte");
    let mut db = rusqlite::Connection::open_in_memory()?;
    dbm::migrate(&mut db, None)?;
    good_ormning::sqlite::good_query!(
        "sqlite_gen_select_cte",
        dbm::DbSqliteGenSelectCte1(&mut db),
        r#"insert into "bannanana" ( "hizat" , "hizat2" ) values ( ?1 , ?2 )"#;
        p1 = i32,
        p2 = i32;
        1,
        7
    )?;
    good_ormning::sqlite::good_query!(
        "sqlite_gen_select_cte",
        dbm::DbSqliteGenSelectCte1(&mut db),
        r#"insert into "bannanana" ( "hizat" , "hizat2" ) values ( ?1 , ?2 )"#;
        p1 = i32,
        p2 = i32;
        1,
        99
    )?;
    let mut res =
        good_ormning::sqlite::good_query_many!(
            "sqlite_gen_select_cte",
            dbm::DbSqliteGenSelectCte1(&mut db),
            r#"with "hibbo" ( "zathi" ) as ( select "bannanana" . "hizat2" as "hizat2" from "bannanana" ) select "hibbo" . "zathi" as "zathi" from "hibbo""#
        )?;
    res.sort();
    assert_eq!(res, vec![7, 99]);
    Ok(())
}

#[test]
fn test_select_window() -> Result<(), loga::Error> {
    good_module!("sqlite_gen_select_window");
    let mut db = rusqlite::Connection::open_in_memory()?;
    dbm::migrate(&mut db, None)?;
    good_ormning::sqlite::good_query!(
        "sqlite_gen_select_window",
        dbm::DbSqliteGenSelectWindow1(&mut db),
        r#"insert into "bannanana" ( "hizat" , "hizat2" ) values ( ?1 , ?2 )"#;
        p1 = i32,
        p2 = i32;
        1,
        7
    )?;
    good_ormning::sqlite::good_query!(
        "sqlite_gen_select_window",
        dbm::DbSqliteGenSelectWindow1(&mut db),
        r#"insert into "bannanana" ( "hizat" , "hizat2" ) values ( ?1 , ?2 )"#;
        p1 = i32,
        p2 = i32;
        1,
        99
    )?;
    good_ormning::sqlite::good_query!(
        "sqlite_gen_select_window",
        dbm::DbSqliteGenSelectWindow1(&mut db),
        r#"insert into "bannanana" ( "hizat" , "hizat2" ) values ( ?1 , ?2 )"#;
        p1 = i32,
        p2 = i32;
        2,
        3
    )?;
    good_ormning::sqlite::good_query!(
        "sqlite_gen_select_window",
        dbm::DbSqliteGenSelectWindow1(&mut db),
        r#"insert into "bannanana" ( "hizat" , "hizat2" ) values ( ?1 , ?2 )"#;
        p1 = i32,
        p2 = i32;
        2,
        10
    )?;
    let mut res =
        good_ormning::sqlite::good_query_many!(
            "sqlite_gen_select_window",
            dbm::DbSqliteGenSelectWindow1(&mut db),
            r#"select sum ( "bannanana" . "hizat2" ) over ( partition by "bannanana" . "hizat" ) as "hizat2" from "bannanana""#
        )?
            .into_iter()
            .collect::<Vec<_>>();
    res.sort();
    assert_eq!(res, vec![13, 13, 106, 106]);
    Ok(())
}

#[test]
fn test_select_junction() -> Result<(), loga::Error> {
    good_module!("sqlite_gen_select_junction");
    let mut db = rusqlite::Connection::open_in_memory()?;
    dbm::migrate(&mut db, None)?;
    good_ormning::sqlite::good_query!(
        "sqlite_gen_select_junction",
        dbm::DbSqliteGenSelectJunction1(&mut db),
        r#"insert into "bannanana" ( "hizat" , "hizat2" ) values ( ?1 , ?2 )"#;
        p1 = i32,
        p2 = i32;
        1,
        7
    )?;
    good_ormning::sqlite::good_query!(
        "sqlite_gen_select_junction",
        dbm::DbSqliteGenSelectJunction1(&mut db),
        r#"insert into "bannanana" ( "hizat" , "hizat2" ) values ( ?1 , ?2 )"#;
        p1 = i32,
        p2 = i32;
        2,
        3
    )?;
    let mut res =
        good_ormning::sqlite::good_query_many!(
            "sqlite_gen_select_junction",
            dbm::DbSqliteGenSelectJunction1(&mut db),
            r#"select "bannanana" . "hizat" as "hizat" from "bannanana" union select "bannanana" . "hizat2" as "hizat2" from "bannanana""#
        )?;
    res.sort();
    assert_eq!(res, vec![1, 2, 3, 7]);
    Ok(())
}
