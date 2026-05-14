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
    good_module!(dbm, "sqlite_gen_hello_world");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_hello_world",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "hello_world_users" ("name", "points")
           values
             (?1, ?2)
           "#;
        &mut db,
        p1: string = "rust human",
        p2: i64 = 0
    )?;
    for user_id in good_ormning::sqlite::good_query_many!(
        dbm,
        "sqlite_gen_hello_world",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             "hello_world_users"."rowid" as "rowid"
           from
             "hello_world_users"
           "#;
        &mut db
    )? {
        let user = good_ormning::sqlite::good_query_one!(
            dbm,
            "sqlite_gen_hello_world",
            //# genemichaels-external: sql-formatter-sqlite
            r#"select
                 "hello_world_users"."name" as "name",
                 "hello_world_users"."points" as "points"
               from
                 "hello_world_users"
               where
                 "hello_world_users"."rowid" = ?1
               "#;
            &mut db,
            p1: i64 = user_id
        )?;
        println!("User {}: {}", user_id, user.name);
    }
    Ok(())
}

#[test]
fn test_base_insert() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_base_insert");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
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
        p1: string = "soy"
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
    )?, "soy");
    Ok(())
}

#[test]
fn test_get_version_premigrate() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_base_insert");
    let mut db = rusqlite::Connection::open_in_memory()?;
    assert_eq!(dbm::get_schema_version(&mut db)?, None);
    Ok(())
}

#[test]
fn test_get_version_postmigrate() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_base_insert");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    assert_eq!(dbm::get_schema_version(&mut db.0)?, Some(1));
    Ok(())
}

#[test]
fn test_constraint() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_base_insert");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    Ok(())
}

#[test]
fn test_param_i32() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_param_i32");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_param_i32",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bananna" ("hizat")
           values
             (?1)
           "#;
        &mut db,
        p1: i32 = 22
    )?;
    assert_eq!(good_ormning::sqlite::good_query_one!(
        dbm,
        "sqlite_gen_param_i32",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             "bananna"."hizat" as "hizat"
           from
             "bananna"
           "#;
        &mut db
    )?, 22);
    Ok(())
}

#[test]
fn test_inline_param_i32() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_inline_param_i32");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_inline_param_i32",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bananna" ("hizat")
           values
             (${i32 = 22})
           "#;
        &mut db
    )?;
    assert_eq!(good_ormning::sqlite::good_query_one!(
        dbm,
        "sqlite_gen_inline_param_i32",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             "bananna"."hizat" as "hizat"
           from
             "bananna"
           where
             "bananna"."hizat" = ${i32 = 22}
           "#;
        &mut db
    )?, 22);
    Ok(())
}

#[test]
fn test_inline_param_complex() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_inline_param_i32");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    let val = 47;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_inline_param_i32",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bananna" ("hizat")
           values
             (${i32 = val})
           "#;
        &mut db
    )?;
    assert_eq!(good_ormning::sqlite::good_query_one!(
        dbm,
        "sqlite_gen_inline_param_i32",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             "bananna"."hizat" as "hizat"
           from
             "bananna"
           where
             "bananna"."hizat" = ${i32 = val}
             and ${bool = true}
           "#;
        &mut db
    )?, 47);
    Ok(())
}

#[test]
fn test_inline_param_with_path() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_inline_param_i32");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_inline_param_i32",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bananna" ("hizat")
           values
             (${i32 = 2147483647})
           "#;
        &mut db
    )?;
    assert_eq!(good_ormning::sqlite::good_query_one!(
        dbm,
        "sqlite_gen_inline_param_i32",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             "bananna"."hizat" as "hizat"
           from
             "bananna"
           "#;
        &mut db
    )?, 2147483647);
    Ok(())
}

#[test]
fn test_param_utctime_s_chrono() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_param_utctime_s_chrono");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    let ref_date = chrono::TimeZone::with_ymd_and_hms(&chrono::Utc, 1937, 12, 1, 0, 0, 0).unwrap();
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_param_utctime_s_chrono",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bananna" ("hizat")
           values
             (?1)
           "#;
        &mut db,
        p1: utctime_s_chrono = ref_date
    )?;
    assert_eq!(good_ormning::sqlite::good_query_one!(
        dbm,
        "sqlite_gen_param_utctime_s_chrono",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             "bananna"."hizat" as "hizat"
           from
             "bananna"
           "#;
        &mut db
    )?, ref_date);
    Ok(())
}

#[test]
fn test_param_utctime_ms_chrono() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_param_utctime_ms_chrono");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    let ref_date = chrono::TimeZone::with_ymd_and_hms(&chrono::Utc, 1937, 12, 1, 0, 0, 0).unwrap();
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_param_utctime_ms_chrono",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bananna" ("hizat")
           values
             (?1)
           "#;
        &mut db,
        p1: utctime_ms_chrono = ref_date
    )?;
    assert_eq!(good_ormning::sqlite::good_query_one!(
        dbm,
        "sqlite_gen_param_utctime_ms_chrono",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             "bananna"."hizat" as "hizat"
           from
             "bananna"
           "#;
        &mut db
    )?, ref_date);
    Ok(())
}

#[test]
fn test_param_utctime_s_jiff() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_param_utctime_s_jiff");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    let ref_date =
        jiff::civil::DateTime::new(1937, 12, 1, 0, 0, 0, 0)
            .unwrap()
            .to_zoned(jiff::tz::TimeZone::UTC)
            .unwrap()
            .timestamp();
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_param_utctime_s_jiff",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bananna" ("hizat")
           values
             (?1)
           "#;
        &mut db,
        p1: utctime_s_jiff = ref_date
    )?;
    assert_eq!(good_ormning::sqlite::good_query_one!(
        dbm,
        "sqlite_gen_param_utctime_s_jiff",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             "bananna"."hizat" as "hizat"
           from
             "bananna"
           "#;
        &mut db
    )?, ref_date);
    Ok(())
}

#[test]
fn test_param_utctime_ms_jiff() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_param_utctime_ms_jiff");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    let ref_date =
        jiff::civil::DateTime::new(1937, 12, 1, 0, 0, 0, 0)
            .unwrap()
            .to_zoned(jiff::tz::TimeZone::UTC)
            .unwrap()
            .timestamp();
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_param_utctime_ms_jiff",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bananna" ("hizat")
           values
             (?1)
           "#;
        &mut db,
        p1: utctime_ms_jiff = ref_date
    )?;
    assert_eq!(good_ormning::sqlite::good_query_one!(
        dbm,
        "sqlite_gen_param_utctime_ms_jiff",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             "bananna"."hizat" as "hizat"
           from
             "bananna"
           "#;
        &mut db
    )?, ref_date);
    Ok(())
}

#[test]
fn test_param_opt_i32() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_param_opt_i32");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_param_opt_i32",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bananna" ("hizat")
           values
             (?1)
           "#;
        &mut db,
        p1: opt i32 = Some(47)
    )?;
    assert_eq!(good_ormning::sqlite::good_query_one!(
        dbm,
        "sqlite_gen_param_opt_i32",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             "bananna"."hizat" as "hizat"
           from
             "bananna"
           "#;
        &mut db
    )?, Some(47));
    Ok(())
}

#[test]
fn test_param_opt_i32_null() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_param_opt_i32_null");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_param_opt_i32_null",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bananna" ("hizat")
           values
             (null)
           "#;
        &mut db
    )?;
    assert_eq!(good_ormning::sqlite::good_query_one!(
        dbm,
        "sqlite_gen_param_opt_i32_null",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             "bananna"."hizat" as "hizat"
           from
             "bananna"
           "#;
        &mut db
    )?, None);
    Ok(())
}

#[test]
fn test_param_arr_i32() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_param_arr_i32");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_param_arr_i32",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bananna" ("hizat")
           values
             (?1)
           "#;
        &mut db,
        p1: i32 = 7
    )?;
    assert_eq!(good_ormning::sqlite::good_query_many!(
        dbm,
        "sqlite_gen_param_arr_i32",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             "bananna"."hizat" as "hizat"
           from
             "bananna"
           where
             "bananna"."hizat" in (
               select
                 value
               from
                 rarray (?1)
             )
           "#;
        &mut db,
        p1: arr i32 = vec ![7]
    )?, vec![7]);
    Ok(())
}

#[test]
fn test_param_custom() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_param_custom");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
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
        dbm,
        "sqlite_gen_param_custom",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bananna" (
               "x_0",
               "x_1",
               "x_2",
               "x_3",
               "x_4",
               "x_5",
               "x_6",
               "x_7",
               "x_8",
               "x_9",
               "x_10",
               "x_11"
             )
           values
             (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
           "#;
        &mut db,
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
    )?;
    let res = good_ormning::sqlite::good_query_one!(
        dbm,
        "sqlite_gen_param_custom",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             "bananna"."x_0" as "x_0",
             "bananna"."x_1" as "x_1",
             "bananna"."x_2" as "x_2",
             "bananna"."x_3" as "x_3",
             "bananna"."x_4" as "x_4",
             "bananna"."x_5" as "x_5",
             "bananna"."x_6" as "x_6",
             "bananna"."x_7" as "x_7",
             "bananna"."x_8" as "x_8",
             "bananna"."x_9" as "x_9",
             "bananna"."x_10" as "x_10",
             "bananna"."x_11" as "x_11"
           from
             "bananna"
           "#;
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
    good_module!(dbm, "sqlite_gen_param_opt_custom");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_param_opt_custom",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bananna" ("hizat")
           values
             (?1)
           "#;
        &mut db,
        p1: opt MyString = Some(&MyString("higgins".into()))
    )?;
    assert_eq!(good_ormning::sqlite::good_query_one!(
        dbm,
        "sqlite_gen_param_opt_custom",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             "bananna"."hizat" as "hizat"
           from
             "bananna"
           "#;
        &mut db
    )?, Some(MyString("higgins".into())));
    Ok(())
}

#[test]
fn test_insert_on_conflict_do_nothing() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_insert_on_conflict_do_nothing");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    assert!(good_ormning::sqlite::good_query_opt!(
        dbm,
        "sqlite_gen_insert_on_conflict_do_nothing",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bannanana" ("hizat")
           values
             (?1)
           on conflict do nothing
           returning
             1 as "one"
           "#;
        &mut db,
        p1: string = "soy"
    )?.is_some());
    assert!(good_ormning::sqlite::good_query_opt!(
        dbm,
        "sqlite_gen_insert_on_conflict_do_nothing",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bannanana" ("hizat")
           values
             (?1)
           on conflict do nothing
           returning
             1 as "one"
           "#;
        &mut db,
        p1: string = "soy"
    )?.is_none());
    Ok(())
}

#[test]
fn test_insert_on_conflict_update() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_insert_on_conflict_update");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    assert_eq!(good_ormning::sqlite::good_query_one!(
        dbm,
        "sqlite_gen_insert_on_conflict_update",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bannanana" ("hizat", "two")
           values
             (?1, ?2)
           on conflict ("hizat") do update
           set
             "two" = "bannanana"."two" + 1
           returning
             "bannanana"."two" as "two"
           "#;
        &mut db,
        p1: string = "soy",
        p2: i32 = 33
    )?, 33);
    assert_eq!(good_ormning::sqlite::good_query_one!(
        dbm,
        "sqlite_gen_insert_on_conflict_update",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bannanana" ("hizat", "two")
           values
             (?1, ?2)
           on conflict ("hizat") do update
           set
             "two" = "bannanana"."two" + 1
           returning
             "bannanana"."two" as "two"
           "#;
        &mut db,
        p1: string = "soy",
        p2: i32 = 7
    )?, 34);
    assert_eq!(good_ormning::sqlite::good_query_one!(
        dbm,
        "sqlite_gen_insert_on_conflict_update",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bannanana" ("hizat", "two")
           values
             (?1, ?2)
           on conflict ("hizat") do update
           set
             "two" = "bannanana"."two" + 1
           returning
             "bannanana"."two" as "two"
           "#;
        &mut db,
        p1: string = "yyyy",
        p2: i32 = 7
    )?, 7);
    Ok(())
}

#[test]
fn test_update() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_update");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_update",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bananna" ("hizat")
           values
             ('yog')
           "#;
        &mut db
    )?;
    assert_eq!(good_ormning::sqlite::good_query_one!(
        dbm,
        "sqlite_gen_update",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             "bananna"."hizat" as "hizat"
           from
             "bananna"
           "#;
        &mut db
    )?, "yog");
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_update",
        //# genemichaels-external: sql-formatter-sqlite
        r#"update "bananna"
           set
             "hizat" = 'tep'
           "#;
        &mut db
    )?;
    assert_eq!(good_ormning::sqlite::good_query_one!(
        dbm,
        "sqlite_gen_update",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             "bananna"."hizat" as "hizat"
           from
             "bananna"
           "#;
        &mut db
    )?, "tep");
    Ok(())
}

#[test]
fn test_update_where() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_update_where");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_update_where",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "ban" ("hizat")
           values
             ('yog')
           "#;
        &mut db
    )?;
    assert_eq!(good_ormning::sqlite::good_query_one!(
        dbm,
        "sqlite_gen_update_where",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             "ban"."hizat" as "hizat"
           from
             "ban"
           "#;
        &mut db
    )?, "yog");
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_update_where",
        //# genemichaels-external: sql-formatter-sqlite
        r#"update "ban"
           set
             "hizat" = ?1
           where
             "ban"."hizat" = ?2
           "#;
        &mut db,
        p1: string = "tep",
        p2: string = "yog2"
    )?;
    assert_eq!(good_ormning::sqlite::good_query_one!(
        dbm,
        "sqlite_gen_update_where",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             "ban"."hizat" as "hizat"
           from
             "ban"
           "#;
        &mut db
    )?, "yog");
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_update_where",
        //# genemichaels-external: sql-formatter-sqlite
        r#"update "ban"
           set
             "hizat" = ?1
           where
             "ban"."hizat" = ?2
           "#;
        &mut db,
        p1: string = "tep",
        p2: string = "yog"
    )?;
    assert_eq!(good_ormning::sqlite::good_query_one!(
        dbm,
        "sqlite_gen_update_where",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             "ban"."hizat" as "hizat"
           from
             "ban"
           "#;
        &mut db
    )?, "tep");
    Ok(())
}

#[test]
fn test_update_returning() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_update_returning");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_update_returning",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "b" ("hizat")
           values
             ('yog')
           "#;
        &mut db
    )?;
    assert_eq!(good_ormning::sqlite::good_query_opt!(
        dbm,
        "sqlite_gen_update_returning",
        //# genemichaels-external: sql-formatter-sqlite
        r#"update "b"
           set
             "hizat" = 'tep'
           returning
             "b"."hizat" as "hizat"
           "#;
        &mut db
    )?, Some("tep".to_string()));
    Ok(())
}

#[test]
fn test_delete() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_delete");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_delete",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "b" ("hizat")
           values
             ('seeon')
           "#;
        &mut db
    )?;
    assert_eq!(good_ormning::sqlite::good_query_opt!(
        dbm,
        "sqlite_gen_delete",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             "b"."hizat" as "hizat"
           from
             "b"
           "#;
        &mut db
    )?, Some("seeon".to_string()));
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_delete",
        //# genemichaels-external: sql-formatter-sqlite
        r#"delete from "b"
           "#;
        &mut db
    )?;
    assert_eq!(good_ormning::sqlite::good_query_opt!(
        dbm,
        "sqlite_gen_delete",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             "b"."hizat" as "hizat"
           from
             "b"
           "#;
        &mut db
    )?, None);
    Ok(())
}

#[test]
fn test_delete_where() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_delete_where");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_delete_where",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "ba" ("hizat")
           values
             ('seeon')
           "#;
        &mut db
    )?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_delete_where",
        //# genemichaels-external: sql-formatter-sqlite
        r#"delete from "ba"
           where
             "ba"."hizat" = ?1
           "#;
        &mut db,
        p1: string = "nozo"
    )?;
    assert_eq!(good_ormning::sqlite::good_query_opt!(
        dbm,
        "sqlite_gen_delete_where",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             "ba"."hizat" as "hizat"
           from
             "ba"
           "#;
        &mut db
    )?, Some("seeon".to_string()));
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_delete_where",
        //# genemichaels-external: sql-formatter-sqlite
        r#"delete from "ba"
           where
             "ba"."hizat" = ?1
           "#;
        &mut db,
        p1: string = "seeon"
    )?;
    assert_eq!(good_ormning::sqlite::good_query_opt!(
        dbm,
        "sqlite_gen_delete_where",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             "ba"."hizat" as "hizat"
           from
             "ba"
           "#;
        &mut db
    )?, None);
    Ok(())
}

#[test]
fn test_delete_returning() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_delete_returning");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_delete_returning",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "b" ("hizat")
           values
             ('seeon')
           "#;
        &mut db
    )?;
    assert!(good_ormning::sqlite::good_query_opt!(
        dbm,
        "sqlite_gen_delete_returning",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             "b"."hizat" as "hizat"
           from
             "b"
           "#;
        &mut db
    )?.is_some());
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_delete_returning",
        //# genemichaels-external: sql-formatter-sqlite
        r#"delete from "b"
           where
             "b"."hizat" = ?1
           "#;
        &mut db,
        p1: string = "seeon"
    )?;
    assert!(good_ormning::sqlite::good_query_opt!(
        dbm,
        "sqlite_gen_delete_returning",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             "b"."hizat" as "hizat"
           from
             "b"
           "#;
        &mut db
    )?.is_none());
    Ok(())
}

#[test]
fn test_select_join() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_select_join");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, Some(&|v| {
        match v {
            dbm::DbSqliteGenSelectJoinVersions::V1(db) => {
                good_ormning::sqlite::good_query!(
                    dbm,
                    "sqlite_gen_select_join",
                    //# genemichaels-external: sql-formatter-sqlite
                    r#"insert into
                         "b" ("hizat", "three")
                       values
                         ('key', 33)
                       "#;
                    db
                )?;
                good_ormning::sqlite::good_query!(
                    dbm,
                    "sqlite_gen_select_join",
                    //# genemichaels-external: sql-formatter-sqlite
                    r#"insert into
                         "select_join_two" ("hizat", "two")
                       values
                         ('key', 'no')
                       "#;
                    db
                )?;
            },
        }
        Ok(())
    }))?;
    let res = good_ormning::sqlite::good_query_one!(
        dbm,
        "sqlite_gen_select_join",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             "b"."three" as "three",
             "select_join_two"."two" as "two"
           from
             "b"
             left join "select_join_two" on ("b"."hizat") = "select_join_two"."hizat"
           "#;
        &mut db
    )?;
    assert_eq!(res.three, 33);
    assert_eq!(res.two, Some("no".into()));
    Ok(())
}

#[test]
fn test_select_join_inner() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_select_join");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, Some(&|v| {
        match v {
            dbm::DbSqliteGenSelectJoinVersions::V1(db) => {
                good_ormning::sqlite::good_query!(
                    dbm,
                    "sqlite_gen_select_join",
                    //# genemichaels-external: sql-formatter-sqlite
                    r#"insert into
                         "b" ("hizat", "three")
                       values
                         ('key', 33)
                       "#;
                    db
                )?;
                good_ormning::sqlite::good_query!(
                    dbm,
                    "sqlite_gen_select_join",
                    //# genemichaels-external: sql-formatter-sqlite
                    r#"insert into
                         "b" ("hizat", "three")
                       values
                         ('nomatch', 44)
                       "#;
                    db
                )?;
                good_ormning::sqlite::good_query!(
                    dbm,
                    "sqlite_gen_select_join",
                    //# genemichaels-external: sql-formatter-sqlite
                    r#"insert into
                         "select_join_two" ("hizat", "two")
                       values
                         ('key', 'yes')
                       "#;
                    db
                )?;
            },
        }
        Ok(())
    }))?;
    let res = good_ormning::sqlite::good_query_many!(
        dbm,
        "sqlite_gen_select_join",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             "b"."three" as "three",
             "select_join_two"."two" as "two"
           from
             "b"
             inner join "select_join_two" on ("b"."hizat") = "select_join_two"."hizat"
           "#;
        &mut db
    )?;
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].three, 33);
    assert_eq!(res[0].two, "yes".to_string());
    Ok(())
}

#[test]
fn test_select_join_right() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_select_join");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, Some(&|v| {
        match v {
            dbm::DbSqliteGenSelectJoinVersions::V1(db) => {
                good_ormning::sqlite::good_query!(
                    dbm,
                    "sqlite_gen_select_join",
                    //# genemichaels-external: sql-formatter-sqlite
                    r#"insert into
                         "b" ("hizat", "three")
                       values
                         ('key', 33)
                       "#;
                    db
                )?;
                good_ormning::sqlite::good_query!(
                    dbm,
                    "sqlite_gen_select_join",
                    //# genemichaels-external: sql-formatter-sqlite
                    r#"insert into
                         "select_join_two" ("hizat", "two")
                       values
                         ('key', 'yes')
                       "#;
                    db
                )?;
                good_ormning::sqlite::good_query!(
                    dbm,
                    "sqlite_gen_select_join",
                    //# genemichaels-external: sql-formatter-sqlite
                    r#"insert into
                         "select_join_two" ("hizat", "two")
                       values
                         ('nomatch', 'orphan')
                       "#;
                    db
                )?;
            },
        }
        Ok(())
    }))?;
    let res = good_ormning::sqlite::good_query_many!(
        dbm,
        "sqlite_gen_select_join",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             "b"."three" as "three",
             "select_join_two"."two" as "two"
           from
             "b"
             right join "select_join_two" on ("b"."hizat") = "select_join_two"."hizat"
           "#;
        &mut db
    )?;
    assert_eq!(res.len(), 2);
    Ok(())
}

#[test]
fn test_select_join_full() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_select_join");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, Some(&|v| {
        match v {
            dbm::DbSqliteGenSelectJoinVersions::V1(db) => {
                good_ormning::sqlite::good_query!(
                    dbm,
                    "sqlite_gen_select_join",
                    //# genemichaels-external: sql-formatter-sqlite
                    r#"insert into
                         "b" ("hizat", "three")
                       values
                         ('key', 33)
                       "#;
                    db
                )?;
                good_ormning::sqlite::good_query!(
                    dbm,
                    "sqlite_gen_select_join",
                    //# genemichaels-external: sql-formatter-sqlite
                    r#"insert into
                         "b" ("hizat", "three")
                       values
                         ('lonely', 44)
                       "#;
                    db
                )?;
                good_ormning::sqlite::good_query!(
                    dbm,
                    "sqlite_gen_select_join",
                    //# genemichaels-external: sql-formatter-sqlite
                    r#"insert into
                         "select_join_two" ("hizat", "two")
                       values
                         ('key', 'yes')
                       "#;
                    db
                )?;
                good_ormning::sqlite::good_query!(
                    dbm,
                    "sqlite_gen_select_join",
                    //# genemichaels-external: sql-formatter-sqlite
                    r#"insert into
                         "select_join_two" ("hizat", "two")
                       values
                         ('orphan', 'no')
                       "#;
                    db
                )?;
            },
        }
        Ok(())
    }))?;
    let res = good_ormning::sqlite::good_query_many!(
        dbm,
        "sqlite_gen_select_join",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             "b"."three" as "three",
             "select_join_two"."two" as "two"
           from
             "b"
             full outer join "select_join_two" on ("b"."hizat") = "select_join_two"."hizat"
           "#;
        &mut db
    )?;
    assert_eq!(res.len(), 3);
    Ok(())
}

#[test]
fn test_select_join_cross() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_select_join");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, Some(&|v| {
        match v {
            dbm::DbSqliteGenSelectJoinVersions::V1(db) => {
                good_ormning::sqlite::good_query!(
                    dbm,
                    "sqlite_gen_select_join",
                    //# genemichaels-external: sql-formatter-sqlite
                    r#"insert into
                         "b" ("hizat", "three")
                       values
                         ('a', 1)
                       "#;
                    db
                )?;
                good_ormning::sqlite::good_query!(
                    dbm,
                    "sqlite_gen_select_join",
                    //# genemichaels-external: sql-formatter-sqlite
                    r#"insert into
                         "b" ("hizat", "three")
                       values
                         ('b', 2)
                       "#;
                    db
                )?;
                good_ormning::sqlite::good_query!(
                    dbm,
                    "sqlite_gen_select_join",
                    //# genemichaels-external: sql-formatter-sqlite
                    r#"insert into
                         "select_join_two" ("hizat", "two")
                       values
                         ('x', 'y')
                       "#;
                    db
                )?;
                good_ormning::sqlite::good_query!(
                    dbm,
                    "sqlite_gen_select_join",
                    //# genemichaels-external: sql-formatter-sqlite
                    r#"insert into
                         "select_join_two" ("hizat", "two")
                       values
                         ('z', 'w')
                       "#;
                    db
                )?;
            },
        }
        Ok(())
    }))?;
    let res = good_ormning::sqlite::good_query_many!(
        dbm,
        "sqlite_gen_select_join",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             "b"."three" as "three",
             "select_join_two"."two" as "two"
           from
             "b"
             cross join "select_join_two"
           "#;
        &mut db
    )?;
    assert_eq!(res.len(), 4);
    Ok(())
}

#[test]
fn test_select_group_by() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_select_group_by");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_select_group_by",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bannanana" ("hizat", "hizat2")
           values
             (?1, ?2)
           "#;
        &mut db,
        p1: i32 = 1,
        p2: i32 = 7
    )?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_select_group_by",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bannanana" ("hizat", "hizat2")
           values
             (?1, ?2)
           "#;
        &mut db,
        p1: i32 = 1,
        p2: i32 = 99
    )?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_select_group_by",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bannanana" ("hizat", "hizat2")
           values
             (?1, ?2)
           "#;
        &mut db,
        p1: i32 = 2,
        p2: i32 = 3
    )?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_select_group_by",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bannanana" ("hizat", "hizat2")
           values
             (?1, ?2)
           "#;
        &mut db,
        p1: i32 = 2,
        p2: i32 = 10
    )?;
    let mut res = good_ormning::sqlite::good_query_many!(
        dbm,
        "sqlite_gen_select_group_by",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             sum("bannanana"."hizat2") as "hizat2"
           from
             "bannanana"
           group by
             "bannanana"."hizat"
           "#;
        &mut db
    )?;
    res.sort();
    assert_eq!(res, vec![Some(13i32), Some(106i32)]);
    Ok(())
}

#[test]
fn test_select_limit() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_select_limit");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_select_limit",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bannanana" ("hizat")
           values
             (?1)
           "#;
        &mut db,
        p1: string = "soy"
    )?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_select_limit",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bannanana" ("hizat")
           values
             (?1)
           "#;
        &mut db,
        p1: string = "soy"
    )?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_select_limit",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bannanana" ("hizat")
           values
             (?1)
           "#;
        &mut db,
        p1: string = "soy"
    )?;
    assert_eq!(good_ormning::sqlite::good_query_many!(
        dbm,
        "sqlite_gen_select_limit",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             "bannanana"."hizat" as "hizat"
           from
             "bannanana"
           limit
             2
           "#;
        &mut db
    )?.len(), 2);
    Ok(())
}

#[test]
fn test_select_order() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_select_order");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_select_order",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bannanana" ("hizat")
           values
             (?1)
           "#;
        &mut db,
        p1: i32 = 0
    )?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_select_order",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bannanana" ("hizat")
           values
             (?1)
           "#;
        &mut db,
        p1: i32 = 12
    )?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_select_order",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bannanana" ("hizat")
           values
             (?1)
           "#;
        &mut db,
        p1: i32 = 9
    )?;
    assert_eq!(good_ormning::sqlite::good_query_many!(
        dbm,
        "sqlite_gen_select_order",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             "bannanana"."hizat" as "hizat"
           from
             "bannanana"
           order by
             "bannanana"."hizat" asc
           "#;
        &mut db
    )?, vec![0, 9, 12]);
    Ok(())
}

#[test]
fn test_migrate_add_field() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_migrate_add_field");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, Some(&|v| {
        match v {
            dbm::DbSqliteGenMigrateAddFieldVersions::V0(db) => {
                good_ormning::sqlite::good_query!(
                    dbm,
                    "sqlite_gen_migrate_add_field",
                    0,
                    //# genemichaels-external: sql-formatter-sqlite
                    r#"insert into
                         "bannna" ("hizat")
                       values
                         ('nizoot')
                       "#;
                    db
                )?;
            },
            _ => { },
        }
        Ok(())
    }))?;
    match good_ormning::sqlite::good_query_opt!(
        dbm,
        "sqlite_gen_migrate_add_field",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             "bannna"."hizat" as "hizat",
             "bannna"."zomzom" as "zomzom"
           from
             "bannna"
           "#;
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
    good_module!(dbm, "sqlite_gen_migrate_rename_field");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_migrate_rename_field",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bannna" ("hizat")
           values
             ('nizoot')
           "#;
        &mut db
    )?;
    Ok(())
}

#[test]
fn test_migrate_remove_field() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_migrate_remove_field");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_migrate_remove_field",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bnanaa" ("hizat")
           values
             (?1)
           "#;
        &mut db,
        p1: string = "yordol"
    )?;
    Ok(())
}

#[test]
fn test_migrate_add_table() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_migrate_add_table");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_migrate_add_table",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "migrate_add_table_two" ("two")
           values
             (?1)
           "#;
        &mut db,
        p1: i32 = 23
    )?;
    Ok(())
}

#[test]
fn test_migrate_rename_table() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_migrate_rename_table");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_migrate_rename_table",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bana" ("hizat")
           values
             (?1)
           "#;
        &mut db,
        p1: string = "inset"
    )?;
    Ok(())
}

#[test]
fn test_migrate_remove_table() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_migrate_remove_table");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    Ok(())
}

#[test]
fn test_migrate_pre_migration() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_migrate_pre_migration");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, Some(&|v| {
        match v {
            dbm::DbSqliteGenMigratePreMigrationVersions::V0(db) => {
                good_ormning::sqlite::good_query!(
                    dbm,
                    "sqlite_gen_migrate_pre_migration",
                    0,
                    //# genemichaels-external: sql-formatter-sqlite
                    r#"insert into
                         "migrate_pre_migration_v0_two" ("two")
                       values
                         (7)
                       "#;
                    db
                )?;
            },
            _ => { },
        }
        Ok(())
    }))?;
    Ok(())
}

#[test]
fn test_good_query_combinations() -> Result<(), loga::Error> {
    good_module!(dbm);
    good_module!(dbm_custom, "sqlite_gen_base_insert");
    let mut db_raw = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db_raw, None)?;

    // 1. No db/version
    good_ormning::sqlite::good_query!(
        dbm,
        "insert into default_table (id) values (1)";
        &mut db
    )?;

    // 2. Non-default db
    let mut db_custom_raw = rusqlite::Connection::open_in_memory()?;
    let mut db_custom = dbm_custom::migrate(db_custom_raw, None)?;
    good_ormning::sqlite::good_query!(
        dbm_custom,
        "sqlite_gen_base_insert",
        "insert into bannanana (hizat) values ('test')";
        &mut db_custom
    )?;

    // 3. Non-default version (of default db)
    let mut db_v0 = dbm::Db0(db.0);
    good_ormning::sqlite::good_query!(
        dbm,
        0,
        "insert into default_table (id) values (2)";
        &mut db_v0
    )?;
    db.0 = db_v0.0;

    // 4. Non-default db and version
    good_ormning::sqlite::good_query!(
        dbm_custom,
        "sqlite_gen_base_insert",
        1,
        "insert into bannanana (hizat) values ('test2')";
        &mut db_custom
    )?;
    Ok(())
}

#[test]
fn test_select_cte() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_select_cte");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_select_cte",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bannanana" ("hizat", "hizat2")
           values
             (?1, ?2)
           "#;
        &mut db,
        p1: i32 = 1,
        p2: i32 = 7
    )?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_select_cte",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bannanana" ("hizat", "hizat2")
           values
             (?1, ?2)
           "#;
        &mut db,
        p1: i32 = 1,
        p2: i32 = 99
    )?;
    let mut res = good_ormning::sqlite::good_query_many!(
        dbm,
        "sqlite_gen_select_cte",
        //# genemichaels-external: sql-formatter-sqlite
        r#"with
             "hibbo" ("zathi") as (
               select
                 "bannanana"."hizat2" as "hizat2"
               from
                 "bannanana"
             )
           select
             "hibbo"."zathi" as "zathi"
           from
             "hibbo"
           "#;
        &mut db
    )?;
    res.sort();
    assert_eq!(res, vec![7, 99]);
    Ok(())
}

#[test]
fn test_select_window() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_select_window");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_select_window",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bannanana" ("hizat", "hizat2")
           values
             (?1, ?2)
           "#;
        &mut db,
        p1: i32 = 1,
        p2: i32 = 7
    )?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_select_window",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bannanana" ("hizat", "hizat2")
           values
             (?1, ?2)
           "#;
        &mut db,
        p1: i32 = 1,
        p2: i32 = 99
    )?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_select_window",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bannanana" ("hizat", "hizat2")
           values
             (?1, ?2)
           "#;
        &mut db,
        p1: i32 = 2,
        p2: i32 = 3
    )?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_select_window",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bannanana" ("hizat", "hizat2")
           values
             (?1, ?2)
           "#;
        &mut db,
        p1: i32 = 2,
        p2: i32 = 10
    )?;
    let mut res = good_ormning::sqlite::good_query_many!(
        dbm,
        "sqlite_gen_select_window",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             sum("bannanana"."hizat2") over (
               partition by
                 "bannanana"."hizat"
             ) as "hizat2"
           from
             "bannanana"
           "#;
        &mut db
    )?.into_iter().collect::<Vec<_>>();
    res.sort();
    assert_eq!(res, vec![Some(13i32), Some(13i32), Some(106i32), Some(106i32)]);
    Ok(())
}

#[test]
fn test_query_filter() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_query_filter");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_query_filter",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bananna" ("hizat", "two")
           values
             (?1, ?2)
           "#;
        &mut db,
        p1: string = "a",
        p2: i32 = 10
    )?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_query_filter",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bananna" ("hizat", "two")
           values
             (?1, ?2)
           "#;
        &mut db,
        p1: string = "b",
        p2: i32 = 20
    )?;
    let res = good_ormning::sqlite::good_query_one!(
        dbm,
        "sqlite_gen_query_filter",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             sum("two") filter (
               where
                 "hizat" = 'a'
             ) as "x"
           from
             "bananna"
           "#;
        &mut db
    )?;
    assert_eq!(res, Some(10i32));
    return Ok(());
}

#[test]
fn test_query_window_frame() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_query_window_frame");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    for i in 1i32 ..= 3 {
        good_ormning::sqlite::good_query!(
            dbm,
            "sqlite_gen_query_window_frame",
            //# genemichaels-external: sql-formatter-sqlite
            r#"insert into
                 "bananna" ("hizat", "two")
               values
                 ('key', ?1)
               "#;
            &mut db,
            p1: i32 = i
        )?;
    }
    let res = good_ormning::sqlite::good_query_many!(
        dbm,
        "sqlite_gen_query_window_frame",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             sum("two") over (
               order by
                 "two" rows between unbounded preceding
                 and current row
             ) as "x"
           from
             "bananna"
           "#;
        &mut db
    )?;
    assert_eq!(res, vec![Some(1i32), Some(3i32), Some(6i32)]);
    return Ok(());
}

#[test]
fn test_query_collate() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_query_collate");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_query_collate",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bananna" ("hizat")
           values
             ('abc')
           "#;
        &mut db
    )?;
    let res = good_ormning::sqlite::good_query_one!(
        dbm,
        "sqlite_gen_query_collate",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             "hizat" as "x"
           from
             "bananna"
           where
             "hizat" collate nocase = 'ABC'
           "#;
        &mut db
    )?;
    assert_eq!(res, "abc");
    return Ok(());
}

#[test]
fn test_query_is_distinct_from() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_query_is_distinct_from");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_query_is_distinct_from",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bananna" ("hizat")
           values
             (null)
           "#;
        &mut db
    )?;
    let res = good_ormning::sqlite::good_query_one!(
        dbm,
        "sqlite_gen_query_is_distinct_from",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             count(*) as "x"
           from
             "bananna"
           where
             "hizat" is distinct
           from
             'abc'
           "#;
        &mut db
    )?;
    assert_eq!(res, 1i64);
    let res2 = good_ormning::sqlite::good_query_one!(
        dbm,
        "sqlite_gen_query_is_distinct_from",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             count(*) as "x"
           from
             "bananna"
           where
             "hizat" is not distinct
           from
             null
           "#;
        &mut db
    )?;
    assert_eq!(res2, 1i64);
    return Ok(());
}

#[test]
fn test_query_having() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_query_having");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_query_having",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bananna" ("hizat", "two")
           values
             ('a', 10)
           "#;
        &mut db
    )?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_query_having",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bananna" ("hizat", "two")
           values
             ('b', 20)
           "#;
        &mut db
    )?;
    let res = good_ormning::sqlite::good_query_many!(
        dbm,
        "sqlite_gen_query_having",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             "hizat" as "x"
           from
             "bananna"
           group by
             "hizat"
           having
             sum("two") > 15
           "#;
        &mut db
    )?;
    assert_eq!(res, vec!["b".to_string()]);
    return Ok(());
}

#[test]
fn test_query_glob() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_query_glob");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;

    //# genemichaels-external: sql-formatter-sqlite
    db.0.execute(r#"insert into "bananna" ( "hizat" ) values ( 'hello' )"#, [])?;

    //# genemichaels-external: sql-formatter-sqlite
    db.0.execute(r#"insert into "bananna" ( "hizat" ) values ( 'world' )"#, [])?;
    let count: i64 = 
        //# genemichaels-external: sql-formatter-sqlite
        db.0.query_row(r#"select count(*) from "bananna" where "hizat" glob 'hel*'"#, [], |row| row.get(0))?;
    assert_eq!(count, 1);
    return Ok(());
}

#[test]
fn test_query_indexed_by() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_query_indexed_by");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;

    //# genemichaels-external: sql-formatter-sqlite
    db.0.execute(r#"insert into "bananna" ( "hizat" ) values ( 'hello' )"#, [])?;
    let res: String = db.0.query_row(
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             "hizat"
           from
             "bananna" indexed by "bananna_hizat"
           where
             "hizat" = 'hello'
           "#,
        [],
        |row| row.get(0),
    )?;
    assert_eq!(res, "hello");
    return Ok(());
}

#[test]
fn test_query_cte_subquery() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_query_cte_subquery");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_query_cte_subquery",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bananna" ("hizat")
           values
             ('a')
           "#;
        &mut db
    )?;
    let res = good_ormning::sqlite::good_query_one!(
        dbm,
        "sqlite_gen_query_cte_subquery",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             (
               with
                 "t" as (
                   select
                     1 as "v"
                 )
               select
                 "v"
               from
                 "t"
             ) as "x"
           "#;
        &mut db
    )?;
    assert_eq!(res, 1i32);
    return Ok(());
}

#[test]
fn test_query_like_escape() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_query_like_escape");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_query_like_escape",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bananna" ("hizat")
           values
             ('a%b')
           "#;
        &mut db
    )?;
    let res = good_ormning::sqlite::good_query_one!(
        dbm,
        "sqlite_gen_query_like_escape",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             count(*) as "x"
           from
             "bananna"
           where
             "hizat" like 'a!%b' escape '!'
           "#;
        &mut db
    )?;
    assert_eq!(res, 1i64);
    return Ok(());
}

#[test]
fn test_select_junction() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_select_junction");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_select_junction",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bannanana" ("hizat", "hizat2")
           values
             (?1, ?2)
           "#;
        &mut db,
        p1: i32 = 1,
        p2: i32 = 7
    )?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_select_junction",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bannanana" ("hizat", "hizat2")
           values
             (?1, ?2)
           "#;
        &mut db,
        p1: i32 = 2,
        p2: i32 = 3
    )?;
    let mut res = good_ormning::sqlite::good_query_many!(
        dbm,
        "sqlite_gen_select_junction",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             "bannanana"."hizat" as "hizat"
           from
             "bannanana"
           union
           select
             "bannanana"."hizat2" as "hizat2"
           from
             "bannanana"
           "#;
        &mut db
    )?;
    res.sort();
    assert_eq!(res, vec![1, 2, 3, 7]);
    Ok(())
}

#[test]
fn test_returning_wildcard() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_base_insert");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    let res = good_ormning::sqlite::good_query_one!(
        dbm,
        "sqlite_gen_base_insert",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bannanana" ("hizat")
           values
             ('hi')
           returning
             *
           "#;
        &mut db
    )?;
    assert_eq!(res.hizat, "hi");
    Ok(())
}

#[test]
fn test_query_between() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_base_insert");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_base_insert",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bannanana" ("hizat", "hizat2")
           values
             ('a', 5)
           "#;
        &mut db
    )?;
    let res = good_ormning::sqlite::good_query_one!(
        dbm,
        "sqlite_gen_base_insert",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             count(*) as "x"
           from
             "bannanana"
           where
             "hizat2" between 1 and 10
           "#;
        &mut db
    )?;
    assert_eq!(res, 1i64);
    Ok(())
}

#[test]
fn test_query_case() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_base_insert");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_base_insert",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bannanana" ("hizat", "hizat2")
           values
             ('a', 5)
           "#;
        &mut db
    )?;
    let res = good_ormning::sqlite::good_query_one!(
        dbm,
        "sqlite_gen_base_insert",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             case
               when "hizat2" > 0 then 'positive'
               else 'non-positive'
             end as "res"
           from
             "bannanana"
           "#;
        &mut db
    )?;
    assert_eq!(res, "positive");
    Ok(())
}

#[test]
fn test_query_tuple_in() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_base_insert");
    let mut db = rusqlite::Connection::open_in_memory().map_err(loga::err)?;
    let mut db = dbm::migrate(db, None)?;
    db
        .0
        .execute("insert into bannanana (hizat, hizat2) values ('a', 1), ('b', 2), ('c', 3)", [])
        .map_err(loga::err)?;
    let res = good_ormning::sqlite::good_query_many!(
        dbm,
        "sqlite_gen_base_insert",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             hizat
           from
             bannanana
           where
             (hizat, hizat2) in (('a', 1), ('c', 3))
           order by
             hizat
           "#;
        &mut db
    )?;
    assert_eq!(res.len(), 2);
    assert_eq!(res[0], "a");
    assert_eq!(res[1], "c");
    Ok(())
}

#[test]
fn test_query_tuple_cmp() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_base_insert");
    let mut db = rusqlite::Connection::open_in_memory().map_err(loga::err)?;
    let mut db = dbm::migrate(db, None)?;
    db
        .0
        .execute("insert into bannanana (hizat, hizat2) values ('a', 1), ('b', 2), ('c', 3)", [])
        .map_err(loga::err)?;
    let res = good_ormning::sqlite::good_query_many!(
        dbm,
        "sqlite_gen_base_insert",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             hizat
           from
             bannanana
           where
             (hizat, hizat2) < ('b', 3)
           order by
             hizat
           "#;
        &mut db
    )?;
    assert_eq!(res.len(), 2);
    assert_eq!(res[0], "a");
    assert_eq!(res[1], "b");
    let res2 = good_ormning::sqlite::good_query_many!(
        dbm,
        "sqlite_gen_base_insert",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             hizat
           from
             bannanana
           where
             (hizat, hizat2) = ('b', 2)
           "#;
        &mut db
    )?;
    assert_eq!(res2.len(), 1);
    assert_eq!(res2[0], "b");
    Ok(())
}

#[test]
fn test_repeated_param() -> Result<(), loga::Error> {
    good_ormning::good_module!(dbm, "sqlite_gen_repeated_param");
    let mut db = rusqlite::Connection::open_in_memory().map_err(|e| loga::err(e))?;
    let mut db = dbm::migrate(db, None)?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_repeated_param",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "genrerank" (
               "date",
               "genre",
               "secondary",
               "sort",
               "rank",
               "track"
             )
           values
             (
               $repdate,
               $genre,
               $secondary,
               $sort,
               $rank,
               $track
             )
           on conflict ("genre", "secondary", "sort", "track") do update
           set
             "date" = $repdate,
             "rank" = $rank
           "#;
        &mut db,
        repdate: i32 = 20260501,
        genre: string = "rock",
        secondary: string = "classic",
        sort: i32 = 1,
        rank: i32 = 10,
        track: string = "song1"
    )?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_repeated_param",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "genrerank" (
               "date",
               "genre",
               "secondary",
               "sort",
               "rank",
               "track"
             )
           values
             (
               $repdate,
               $genre,
               $secondary,
               $sort,
               $rank,
               $track
             )
           on conflict ("genre", "secondary", "sort", "track") do update
           set
             "date" = $repdate,
             "rank" = $rank
           "#;
        &mut db,
        repdate: i32 = 20260502,
        genre: string = "rock",
        secondary: string = "classic",
        sort: i32 = 1,
        rank: i32 = 5,
        track: string = "song1"
    )?;
    assert_eq!(good_ormning::sqlite::good_query_one!(
        dbm,
        "sqlite_gen_repeated_param",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             "genrerank"."date" as "date"
           from
             "genrerank"
           "#;
        &mut db
    )?, 20260502);
    assert_eq!(good_ormning::sqlite::good_query_one!(
        dbm,
        "sqlite_gen_repeated_param",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             "genrerank"."rank" as "rank"
           from
             "genrerank"
           "#;
        &mut db
    )?, 5);
    Ok(())
}

#[test]
fn test_inline_param_i32_common_syntax() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_inline_param_i32_common");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_inline_param_i32_common",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bananna" ("hizat")
           values
             (${i32 = 22})
           "#;
        &mut db
    )?;
    assert_eq!(good_ormning::sqlite::good_query_one!(
        dbm,
        "sqlite_gen_inline_param_i32_common",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             "bananna"."hizat" as "hizat"
           from
             "bananna"
           where
             "bananna"."hizat" = ${i32 = 22}
           "#;
        &mut db
    )?, 22);
    Ok(())
}

#[test]
fn test_generated_query_functions_compile() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_query");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_query",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "bananna" ("hizat")
           values
             (?1)
           "#;
        &mut db,
        p1: string = "hello"
    )?;
    let results = dbm::hist_list_all(&mut db)?;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], "hello");
    Ok(())
}

#[test]
fn test_delete_cte_macro() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_delete_cte");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_delete_cte",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "b" ("hizat")
           values
             ('seeon')
           "#;
        &mut db
    )?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_delete_cte",
        //# genemichaels-external: sql-formatter-sqlite
        r#"with
             "hibbo" ("zathi") as (
               select
                 "b"."hizat" as "zathi"
               from
                 "b"
             )
           delete from "b"
           where
             exists (
               select
                 "hibbo"."zathi" as "zathi"
               from
                 "hibbo"
               where
                 "hibbo"."zathi" = "b"."hizat"
             )
           "#;
        &mut db
    )?;
    assert_eq!(good_ormning::sqlite::good_query_opt!(
        dbm,
        "sqlite_gen_delete_cte",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             "b"."hizat" as "hizat"
           from
             "b"
           "#;
        &mut db
    )?, None);
    Ok(())
}

#[test]
fn test_query_correlated_subquery() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_query_correlated_subquery");
    let mut db = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db, None)?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_query_correlated_subquery",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "b" ("hizat")
           values
             ('seeon')
           "#;
        &mut db
    )?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_query_correlated_subquery",
        //# genemichaels-external: sql-formatter-sqlite
        r#"insert into
             "snap" ("hizat")
           values
             ('seeon')
           "#;
        &mut db
    )?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_query_correlated_subquery",
        //# genemichaels-external: sql-formatter-sqlite
        r#"delete from "b"
           where
             exists (
               select
                 1
               from
                 "snap"
               where
                 "b"."hizat" = "snap"."hizat"
             )
           "#;
        &mut db
    )?;
    assert_eq!(good_ormning::sqlite::good_query_opt!(
        dbm,
        "sqlite_gen_query_correlated_subquery",
        //# genemichaels-external: sql-formatter-sqlite
        r#"select
             "b"."hizat" as "hizat"
           from
             "b"
           "#;
        &mut db
    )?, None);
    Ok(())
}

#[test]
fn test_insert_select() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_insert_select");
    let mut db_raw = rusqlite::Connection::open_in_memory()?;
    let mut db = dbm::migrate(db_raw, None)?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_insert_select",
        r#"insert into "triple" ("subject", "predicate", "object", "commit_", "exists")
                   values ('s1', 'p1', 'o1', 1, true),
                          ('s1', 'p1', 'o1', 2, false),
                          ('s2', 'p2', 'o2', 1, true)
                   "#;
        &mut db
    )?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_insert_select",
        r#"insert or ignore into "subjobj" ("value")
                   select
                     "subject"
                   from
                     "triple"
                   union
                   select
                     "object"
                   from
                     "triple"
                   "#;
        &mut db
    )?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_insert_select",
        r#"insert or ignore into "predicate" ("value")
                   select
                     "predicate"
                   from
                     "triple"
                   "#;
        &mut db
    )?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_insert_select",
        r#"insert into "triple2" ("subject", "predicate", "object", "commit_", "exists")
                   select
                     "subject",
                     "predicate",
                     "object",
                     "commit_",
                     "exists"
                   from
                     "triple"
                   "#;
        &mut db
    )?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_insert_select",
        r#"insert into "triple_snapshot" ("subject", "predicate", "object", "commit_")
                   select
                     "subject",
                     "predicate",
                     "object",
                     "commit_"
                   from
                     "triple" t1
                   where
                     (
                       "commit_" = (
                         select
                           max("commit_")
                         from
                           "triple" t2
                         where
                           (
                             "t1"."subject" = "t2"."subject"
                             and "t1"."predicate" = "t2"."predicate"
                             and "t1"."object" = "t2"."object"
                           )
                       )
                       and "exists" = true
                     )
                   "#;
        &mut db
    )?;
    Ok(())
}

#[test]
fn test_nested_paren() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_nested_paren");
    let mut db = rusqlite::Connection::open_in_memory().map_err(|e| loga::err(e))?;
    let mut db = dbm::migrate(db, None).map_err(|e| loga::err(e))?;
    // Insert: (val=1, a=false, b=true), (val=2, a=true, b=false), (val=3, a=false, b=false)
    // Query: WHERE val > 1 AND (a OR b)
    //   Correct parens: only val=2 matches (2>1 AND (true OR false))
    //   Without parens: (val > 1 AND a) OR b => val=1 also matches via b=true
    for (val, a, b) in [(1i32, false, true), (2i32, true, false), (3i32, false, false)] {
        good_ormning::sqlite::good_query!(
            dbm,
            "sqlite_gen_nested_paren",
            r#"insert into "t" ("val", "a", "b") values (?1, ?2, ?3)"#;
            &mut db,
            p1: i32 = val,
            p2: bool = a,
            p3: bool = b
        )?;
    }
    let rows = good_ormning::sqlite::good_query_many!(
        dbm,
        "sqlite_gen_nested_paren",
        r#"select
             "t"."val" as "val"
           from
             "t"
           where
             "t"."val" > ?1
             and ("t"."a" or "t"."b")
           "#;
        &mut db,
        p1: i32 = 1
    )?;
    assert_eq!(rows, vec![2i32]);
    Ok(())
}

#[test]
fn test_insert_on_conflict_update_excluded() -> Result<(), loga::Error> {
    good_module!(dbm, "sqlite_gen_insert_select");
    let mut db = rusqlite::Connection::open_in_memory().map_err(|e| loga::err(e))?;
    let mut db = dbm::migrate(db, None).map_err(|e| loga::err(e))?;
    good_ormning::sqlite::good_query!(
        dbm,
        "sqlite_gen_insert_select",
        r#"insert into "triple_snapshot" ("subject", "predicate", "object", "commit_")
           values (?1, ?2, ?3, ?4)
           on conflict ("subject", "predicate", "object") do update set "commit_" = excluded."commit_"
        "#;
        &mut db,
        p1: string = "a",
        p2: string = "b",
        p3: string = "c",
        p4: i64 = 1
    )?;
    Ok(())
}
