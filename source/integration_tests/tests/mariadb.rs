use {
    chrono::NaiveDate,
    good_ormning::good_module,
    integration_tests::{
        MyString,
        mariadb_server::db,
    },
};

#[tokio::test]
async fn test_base_insert() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_base_insert");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_base_insert",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `bannanana` (`hizat`)
           values
             (?)
           "#;
        &mut db,
        p1: string = "soy"
    ).await?;
    assert_eq!(good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_base_insert",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             `bannanana`.`hizat` as `hizat`
           from
             `bannanana`
           "#;
        &mut db
    ).await?, "soy");
    Ok(())
}

#[tokio::test]
async fn test_delete() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_delete");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_delete",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `b` (`hizat`)
           values
             ('seeon')
           "#;
        &mut db
    ).await?;
    assert_eq!(good_ormning::mariadb::good_query_opt!(
        dbm,
        "mariadb_gen_delete",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             `b`.`hizat` as `hizat`
           from
             `b`
           "#;
        &mut db
    ).await?, Some("seeon".to_string()));
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_delete",
        //# genemichaels-external: sql-formatter-mariadb
        r#"delete from `b`
           "#;
        &mut db
    ).await?;
    assert_eq!(good_ormning::mariadb::good_query_opt!(
        dbm,
        "mariadb_gen_delete",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             `b`.`hizat` as `hizat`
           from
             `b`
           "#;
        &mut db
    ).await?, None);
    Ok(())
}

#[tokio::test]
async fn test_delete_alias() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_delete_cte");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_delete_cte",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `b` (`hizat`)
           values
             ('seeon')
           "#;
        &mut db
    ).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_delete_cte",
        //# genemichaels-external: sql-formatter-mariadb
        r#"delete `bb`
           from
             `b` as `bb`
           where
             `bb`.`hizat` = 'seeon'
           "#;
        &mut db
    ).await?;
    assert_eq!(good_ormning::mariadb::good_query_opt!(
        dbm,
        "mariadb_gen_delete_cte",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             `b`.`hizat` as `hizat`
           from
             `b`
           "#;
        &mut db
    ).await?, None);
    Ok(())
}

#[tokio::test]
async fn test_delete_returning() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_delete_returning");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_delete_returning",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `b` (`hizat`)
           values
             ('seeon')
           "#;
        &mut db
    ).await?;
    assert!(good_ormning::mariadb::good_query_opt!(
        dbm,
        "mariadb_gen_delete_returning",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             `b`.`hizat` as `hizat`
           from
             `b`
           "#;
        &mut db
    ).await?.is_some());
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_delete_returning",
        //# genemichaels-external: sql-formatter-mariadb
        r#"delete from `b`
           where
             `b`.`hizat` = ?
           "#;
        &mut db,
        p1: string = "seeon"
    ).await?;
    assert!(good_ormning::mariadb::good_query_opt!(
        dbm,
        "mariadb_gen_delete_returning",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             `b`.`hizat` as `hizat`
           from
             `b`
           "#;
        &mut db
    ).await?.is_none());
    Ok(())
}

#[tokio::test]
async fn test_delete_where() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_delete_where");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_delete_where",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `ba` (`hizat`)
           values
             ('seeon')
           "#;
        &mut db
    ).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_delete_where",
        //# genemichaels-external: sql-formatter-mariadb
        r#"delete from `ba`
           where
             `ba`.`hizat` = ?
           "#;
        &mut db,
        p1: string = "nozo"
    ).await?;
    assert_eq!(good_ormning::mariadb::good_query_opt!(
        dbm,
        "mariadb_gen_delete_where",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             `ba`.`hizat` as `hizat`
           from
             `ba`
           "#;
        &mut db
    ).await?, Some("seeon".to_string()));
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_delete_where",
        //# genemichaels-external: sql-formatter-mariadb
        r#"delete from `ba`
           where
             `ba`.`hizat` = ?
           "#;
        &mut db,
        p1: string = "seeon"
    ).await?;
    assert_eq!(good_ormning::mariadb::good_query_opt!(
        dbm,
        "mariadb_gen_delete_where",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             `ba`.`hizat` as `hizat`
           from
             `ba`
           "#;
        &mut db
    ).await?, None);
    Ok(())
}

#[tokio::test]
async fn test_good_query_combinations() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_default");
    good_module!(dbm_custom, "mariadb_gen_base_insert");
    let db_raw = db().await?;
    let mut client = dbm::migrate(db_raw, None).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_default",
        "insert into default_table (id) values (1)";
        &mut client
    ).await?;
    let db_custom_raw = db().await?;
    let mut client_custom = dbm_custom::migrate(db_custom_raw, None).await?;
    good_ormning::mariadb::good_query!(
        dbm_custom,
        "mariadb_gen_base_insert",
        "insert into bannanana (hizat) values ('test')";
        &mut client_custom
    ).await?;
    let mut client_v0 = dbm::DbMariadbGenDefault0(client.0);
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_default",
        0,
        "insert into default_table (id) values (2)";
        &mut client_v0
    ).await?;
    client.0 = client_v0.0;
    good_ormning::mariadb::good_query!(
        dbm_custom,
        "mariadb_gen_base_insert",
        1,
        "insert into bannanana (hizat) values ('test2')";
        &mut client_custom
    ).await?;
    Ok(())
}

#[tokio::test]
async fn test_inline_param_i32() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_inline_param_i32");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_inline_param_i32",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `bananna` (`hizat`)
           values
             (?)
           "#;
        &mut db,
        p1: i32 = 22
    ).await?;
    assert_eq!(good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_inline_param_i32",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             `bananna`.`hizat` as `hizat`
           from
             `bananna`
           where
             `bananna`.`hizat` = ?
           "#;
        &mut db,
        p1: i32 = 22
    ).await?, 22);
    Ok(())
}

#[tokio::test]
async fn test_inline_param_i32_common_syntax() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_inline_param_i32_common");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_inline_param_i32_common",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `bananna` (`hizat`)
           values
             (${i32 = 22})
           "#;
        &mut db
    ).await?;
    assert_eq!(good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_inline_param_i32_common",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             `bananna`.`hizat` as `hizat`
           from
             `bananna`
           where
             `bananna`.`hizat` = ${i32 = 22}
           "#;
        &mut db
    ).await?, 22);
    Ok(())
}

#[tokio::test]
async fn test_insert_ignore() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_insert_on_conflict_do_nothing");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    assert!(good_ormning::mariadb::good_query_opt!(
        dbm,
        "mariadb_gen_insert_on_conflict_do_nothing",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert ignore into
             `bannanana` (`hizat`)
           values
             (?)
           returning
             1 as `one`
           "#;
        &mut db,
        p1: string = "soy"
    ).await?.is_some());
    assert!(good_ormning::mariadb::good_query_opt!(
        dbm,
        "mariadb_gen_insert_on_conflict_do_nothing",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert ignore into
             `bannanana` (`hizat`)
           values
             (?)
           returning
             1 as `one`
           "#;
        &mut db,
        p1: string = "soy"
    ).await?.is_none());
    Ok(())
}

#[tokio::test]
async fn test_insert_on_duplicate_key_update() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_insert_on_conflict_update");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    assert_eq!(good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_insert_on_conflict_update",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `bannanana` (`hizat`, `two`)
           values
             (?, ?)
           on duplicate key update
             `two` = `bannanana`.`two` + 1
           returning
             `bannanana`.`two` as `two`
           "#;
        &mut db,
        p1: string = "soy",
        p2: i32 = 33
    ).await?, 33);
    assert_eq!(good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_insert_on_conflict_update",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `bannanana` (`hizat`, `two`)
           values
             (?, ?)
           on duplicate key update
             `two` = `bannanana`.`two` + 1
           returning
             `bannanana`.`two` as `two`
           "#;
        &mut db,
        p1: string = "soy",
        p2: i32 = 7
    ).await?, 34);
    assert_eq!(good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_insert_on_conflict_update",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `bannanana` (`hizat`, `two`)
           values
             (?, ?)
           on duplicate key update
             `two` = `bannanana`.`two` + 1
           returning
             `bannanana`.`two` as `two`
           "#;
        &mut db,
        p1: string = "yyyy",
        p2: i32 = 7
    ).await?, 7);
    Ok(())
}

#[tokio::test]
async fn test_migrate_add_field() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_migrate_add_field");
    let db = db().await?;
    let mut db = dbm::migrate(db, Some(&|v| Box::pin(async move {
        match v {
            dbm::DbMariadbGenMigrateAddFieldVersions::V0(db) => {
                good_ormning::mariadb::good_query!(
                    dbm,
                    "mariadb_gen_migrate_add_field",
                    0,
                    //# genemichaels-external: sql-formatter-mariadb
                    r#"insert into
                         `bannna` (`hizat`)
                       values
                         ('nizoot')
                       "#;
                    db
                ).await?;
            },
            _ => { },
        }
        return Ok(());
    }))).await?;
    match good_ormning::mariadb::good_query_opt!(
        dbm,
        "mariadb_gen_migrate_add_field",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             `bannna`.`hizat` as `hizat`,
             `bannna`.`zomzom` as `zomzom`
           from
             `bannna`
           "#;
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
async fn test_migrate_constraints() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_migrate_constraints");
    let db = db().await?;
    let mut db = dbm::migrate(db, Some(&|v| Box::pin(async move {
        match v {
            dbm::DbMariadbGenMigrateConstraintsVersions::V0(db) => {
                good_ormning::mariadb::good_query!(
                    dbm,
                    "mariadb_gen_migrate_constraints",
                    0,
                    //# genemichaels-external: sql-formatter-mariadb
                    r#"insert into
                         `bananna` (`hizat`, `zomzom`)
                       values
                         ('a', 1)
                       "#;
                    db
                ).await?;
                good_ormning::mariadb::good_query!(
                    dbm,
                    "mariadb_gen_migrate_constraints",
                    0,
                    //# genemichaels-external: sql-formatter-mariadb
                    r#"insert into
                         `peel` (`bananna_hizat`, `bananna_zomzom`)
                       values
                         ('a', 1)
                       "#;
                    db
                ).await?;
            },
            dbm::DbMariadbGenMigrateConstraintsVersions::V2(db) => {
                good_ormning::mariadb::good_query!(
                    dbm,
                    "mariadb_gen_migrate_constraints",
                    2,
                    //# genemichaels-external: sql-formatter-mariadb
                    r#"insert into
                         `peel` (`bananna_hizat`, `bananna_zomzom`)
                       values
                         ('a', 1)
                       "#;
                    db
                ).await?;
            },
            _ => { },
        }
        return Ok(());
    }))).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_migrate_constraints",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `banana` (`zomzom`)
           values
             (2)
           "#;
        &mut db
    ).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_migrate_constraints",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `peel` (`bananna_zomzom`)
           values
             (2)
           "#;
        &mut db
    ).await?;
    assert!(good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_migrate_constraints",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `peel` (`bananna_zomzom`)
           values
             (3)
           "#;
        &mut db
    ).await.is_err());
    Ok(())
}

#[tokio::test]
async fn test_migrate_indices() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_migrate_indices");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_migrate_indices",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `banana` (`hizat2`)
           values
             ('a')
           "#;
        &mut db
    ).await?;
    assert!(good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_migrate_indices",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `banana` (`hizat2`)
           values
             ('a')
           "#;
        &mut db
    ).await.is_err());
    Ok(())
}

#[tokio::test]
async fn test_migrate_add_table() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_migrate_add_table");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_migrate_add_table",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `migrate_add_table_two` (`two`)
           values
             (?)
           "#;
        &mut db,
        p1: i32 = 23
    ).await?;
    Ok(())
}

#[tokio::test]
async fn test_migrate_make_field_optional() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_migrate_make_field_optional");
    let db = db().await?;
    let _db = dbm::migrate(db, None).await?;
    Ok(())
}

#[tokio::test]
async fn test_migrate_pre_migration() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_migrate_pre_migration");
    let db = db().await?;
    let _db = dbm::migrate(db, Some(&|v| Box::pin(async move {
        match v {
            dbm::DbMariadbGenMigratePreMigrationVersions::V0(db) => {
                good_ormning::mariadb::good_query!(
                    dbm,
                    "mariadb_gen_migrate_pre_migration",
                    0,
                    //# genemichaels-external: sql-formatter-mariadb
                    r#"insert into
                         `migrate_pre_migration_v0_two` (`two`)
                       values
                         (7)
                       "#;
                    db
                ).await?;
            },
            _ => { },
        }
        return Ok(());
    }))).await?;
    Ok(())
}

#[tokio::test]
async fn test_migrate_remove_field() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_migrate_remove_field");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_migrate_remove_field",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `bnanaa` (`hizat`)
           values
             (?)
           "#;
        &mut db,
        p1: string = "yordol"
    ).await?;
    Ok(())
}

#[tokio::test]
async fn test_migrate_remove_table() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_migrate_remove_table");
    let db = db().await?;
    let _db = dbm::migrate(db, None).await?;
    Ok(())
}

#[tokio::test]
async fn test_migrate_rename_field() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_migrate_rename_field");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_migrate_rename_field",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `bannna` (`hizat`)
           values
             ('nizoot')
           "#;
        &mut db
    ).await?;
    Ok(())
}

#[tokio::test]
async fn test_migrate_rename_table() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_migrate_rename_table");
    let db = db().await?;
    let mut db = dbm::migrate(db, Some(&|v| Box::pin(async move {
        match v {
            dbm::DbMariadbGenMigrateRenameTableVersions::V0(db) => {
                good_ormning::mariadb::good_query!(
                    dbm,
                    "mariadb_gen_migrate_rename_table",
                    0,
                    //# genemichaels-external: sql-formatter-mariadb
                    r#"insert into
                         `migrate_rename_table_bnanana` (`hizat`)
                       values
                         ('survives')
                       "#;
                    db
                ).await?;
            },
            _ => { },
        }
        return Ok(());
    }))).await?;
    assert_eq!(good_ormning::mariadb::good_query_opt!(
        dbm,
        "mariadb_gen_migrate_rename_table",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             `bana`.`hizat` as `hizat`
           from
             `bana`
           "#;
        &mut db
    ).await?, Some("survives".to_string()));
    Ok(())
}

#[tokio::test]
async fn test_nested_paren() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_nested_paren");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    for (val, a, b) in [(1i32, false, true), (2i32, true, false), (3i32, false, false)] {
        good_ormning::mariadb::good_query!(
            dbm,
            "mariadb_gen_nested_paren",
            //# genemichaels-external: sql-formatter-mariadb
            r#"insert into
                 `t` (`val`, `a`, `b`)
               values
                 (?, ?, ?)
               "#;
            &mut db,
            p1: i32 = val,
            p2: bool = a,
            p3: bool = b
        ).await?;
    }
    let rows = good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_nested_paren",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             `t`.`val` as `val`
           from
             `t`
           where
             `t`.`val` > ?
             and (
               `t`.`a`
               or `t`.`b`
             )
           "#;
        &mut db,
        p1: i32 = 1
    ).await?;
    assert_eq!(rows, vec![2i32]);
    Ok(())
}

#[tokio::test]
async fn test_param_custom() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_param_custom");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    let x_0 = integration_tests::MyBool(true);
    let x_1 = integration_tests::MyI32(13);
    let x_2 = integration_tests::MyI64(-22);
    let x_3 = integration_tests::MyU32(14);
    let x_4 = integration_tests::MyF32(12.);
    let x_5 = integration_tests::MyF64(99.);
    let x_6 = integration_tests::MyBytes("hi".as_bytes().to_vec());
    let x_7 = integration_tests::MyString("hogo".to_string());
    let x_8 =
        integration_tests::MyDatetimeChrono(
            NaiveDate::from_ymd_opt(1999, 11, 14).unwrap().and_hms_opt(1, 2, 13).unwrap(),
        );
    let x_9 =
        integration_tests::MyDatetimeChrono(
            NaiveDate::from_ymd_opt(1999, 6, 14).unwrap().and_hms_opt(10, 13, 57).unwrap(),
        );
    let x_10 = integration_tests::MyDatetimeJiff(jiff::civil::DateTime::new(1999, 11, 14, 1, 2, 13, 0).unwrap());
    let x_11 = integration_tests::MyDatetimeJiff(jiff::civil::DateTime::new(1999, 6, 14, 10, 13, 57, 0).unwrap());
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_param_custom",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `bananna` (
               `x_0`,
               `x_1`,
               `x_2`,
               `x_3`,
               `x_4`,
               `x_5`,
               `x_6`,
               `x_7`,
               `x_8`,
               `x_9`,
               `x_10`,
               `x_11`
             )
           values
             (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
        p9: MyDatetimeChrono = & x_8,
        p10: MyDatetimeChrono = & x_9,
        p11: MyDatetimeJiff = & x_10,
        p12: MyDatetimeJiff = & x_11
    ).await?;
    let res = good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_param_custom",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             `bananna`.`x_0` as `x_0`,
             `bananna`.`x_1` as `x_1`,
             `bananna`.`x_2` as `x_2`,
             `bananna`.`x_3` as `x_3`,
             `bananna`.`x_4` as `x_4`,
             `bananna`.`x_5` as `x_5`,
             `bananna`.`x_6` as `x_6`,
             `bananna`.`x_7` as `x_7`,
             `bananna`.`x_8` as `x_8`,
             `bananna`.`x_9` as `x_9`,
             `bananna`.`x_10` as `x_10`,
             `bananna`.`x_11` as `x_11`
           from
             `bananna`
           "#;
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
async fn test_param_i32() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_param_i32");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_param_i32",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `bananna` (`hizat`)
           values
             (?)
           "#;
        &mut db,
        p1: i32 = 22
    ).await?;
    assert_eq!(good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_param_i32",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             `bananna`.`hizat` as `hizat`
           from
             `bananna`
           "#;
        &mut db
    ).await?, 22);
    Ok(())
}

#[tokio::test]
async fn test_param_opt_custom() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_param_opt_custom");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_param_opt_custom",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `bananna` (`hizat`)
           values
             (?)
           "#;
        &mut db,
        p1: opt MyString = Some(&MyString("higgins".into()))
    ).await?;
    assert_eq!(good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_param_opt_custom",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             `bananna`.`hizat` as `hizat`
           from
             `bananna`
           "#;
        &mut db
    ).await?, Some(MyString("higgins".into())));
    Ok(())
}

#[tokio::test]
async fn test_param_opt_i32() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_param_opt_i32");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_param_opt_i32",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `bananna` (`hizat`)
           values
             (?)
           "#;
        &mut db,
        p1: opt i32 = Some(47)
    ).await?;
    assert_eq!(good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_param_opt_i32",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             `bananna`.`hizat` as `hizat`
           from
             `bananna`
           "#;
        &mut db
    ).await?, Some(47));
    Ok(())
}

#[tokio::test]
async fn test_param_opt_i32_null() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_param_opt_i32_null");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_param_opt_i32_null",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `bananna` (`hizat`)
           values
             (null)
           "#;
        &mut db
    ).await?;
    assert_eq!(good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_param_opt_i32_null",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             `bananna`.`hizat` as `hizat`
           from
             `bananna`
           "#;
        &mut db
    ).await?, None);
    Ok(())
}

#[tokio::test]
async fn test_param_datetime_chrono() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_param_datetime_chrono");
    let db = db().await?;
    let ref_date = NaiveDate::from_ymd_opt(1937, 12, 1).unwrap().and_hms_opt(0, 0, 0).unwrap();
    let mut db = dbm::migrate(db, None).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_param_datetime_chrono",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `bananna` (`hizat`)
           values
             (?)
           "#;
        &mut db,
        p1: datetime_chrono = ref_date
    ).await?;
    assert_eq!(good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_param_datetime_chrono",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             `bananna`.`hizat` as `hizat`
           from
             `bananna`
           "#;
        &mut db
    ).await?, ref_date);
    Ok(())
}

#[tokio::test]
async fn test_query_between() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_base_insert");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_base_insert",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `bannanana` (`hizat`, `hizat2`)
           values
             ('a', 5)
           "#;
        &mut db
    ).await?;
    let res = good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_base_insert",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             count(*) as `x`
           from
             `bannanana`
           where
             `hizat2` between 1 and 10
           "#;
        &mut db
    ).await?;
    assert_eq!(res, 1i64);
    Ok(())
}

#[tokio::test]
async fn test_query_case() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_base_insert");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_base_insert",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `bannanana` (`hizat`, `hizat2`)
           values
             ('a', 5)
           "#;
        &mut db
    ).await?;
    let res = good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_base_insert",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             case
               when `hizat2` > 0 then 'positive'
               else 'non-positive'
             end as `res`
           from
             `bannanana`
           "#;
        &mut db
    ).await?;
    assert_eq!(res, "positive");
    Ok(())
}

#[tokio::test]
async fn test_query_collate() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_query_collate");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_query_collate",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `bananna` (`hizat`)
           values
             ('abc')
           "#;
        &mut db
    ).await?;
    let res = good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_query_collate",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             `hizat` as `x`
           from
             `bananna`
           where
             `hizat` collate utf8mb4_bin = 'abc'
           "#;
        &mut db
    ).await?;
    assert_eq!(res, "abc");
    Ok(())
}

#[tokio::test]
async fn test_query_concat() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_query_concat");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_query_concat",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `bananna` (`hizat`)
           values
             (?)
           "#;
        &mut db,
        p1: string = "hello"
    ).await?;
    assert_eq!(good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_query_concat",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             concat(`bananna`.`hizat`, ' world') as `out`
           from
             `bananna`
           "#;
        &mut db
    ).await?, "hello world");
    Ok(())
}

#[tokio::test]
async fn test_query_correlated_subquery() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_query_correlated_subquery");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_query_correlated_subquery",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `b` (`hizat`)
           values
             ('seeon')
           "#;
        &mut db
    ).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_query_correlated_subquery",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `snap` (`hizat`)
           values
             ('seeon')
           "#;
        &mut db
    ).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_query_correlated_subquery",
        //# genemichaels-external: sql-formatter-mariadb
        r#"delete from `b`
           where
             exists (
               select
                 1
               from
                 `snap`
               where
                 `b`.`hizat` = `snap`.`hizat`
             )
           "#;
        &mut db
    ).await?;
    assert_eq!(good_ormning::mariadb::good_query_opt!(
        dbm,
        "mariadb_gen_query_correlated_subquery",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             `b`.`hizat` as `hizat`
           from
             `b`
           "#;
        &mut db
    ).await?, None);
    Ok(())
}

#[tokio::test]
async fn test_query_cte_subquery() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_query_cte_subquery");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_query_cte_subquery",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `bananna` (`hizat`)
           values
             ('a')
           "#;
        &mut db
    ).await?;
    let res = good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_query_cte_subquery",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             (
               with
                 `t` as (
                   select
                     1 as `v`
                 )
               select
                 `v`
               from
                 `t`
             ) as `x`
           "#;
        &mut db
    ).await?;
    assert_eq!(res, Some(1i32));
    Ok(())
}

#[tokio::test]
async fn test_query_conditional_aggregate() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_query_filter");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_query_filter",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `bananna` (`hizat`, `two`)
           values
             (?, ?)
           "#;
        &mut db,
        p1: string = "a",
        p2: i32 = 10
    ).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_query_filter",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `bananna` (`hizat`, `two`)
           values
             (?, ?)
           "#;
        &mut db,
        p1: string = "b",
        p2: i32 = 20
    ).await?;
    let res = good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_query_filter",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             sum(if(`hizat` = 'a', `two`, null)) as `x`
           from
             `bananna`
           "#;
        &mut db
    ).await?;
    assert_eq!(res, Some("10".to_string()));
    Ok(())
}

#[tokio::test]
async fn test_query_having() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_query_having");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_query_having",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `bananna` (`hizat`, `two`)
           values
             ('a', 10)
           "#;
        &mut db
    ).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_query_having",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `bananna` (`hizat`, `two`)
           values
             ('b', 20)
           "#;
        &mut db
    ).await?;
    let res = good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_query_having",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             `hizat` as `x`
           from
             `bananna`
           group by
             `hizat`
           having
             sum(`two`) > 15
           "#;
        &mut db
    ).await?;
    assert_eq!(res, vec!["b".to_string()]);
    Ok(())
}

#[tokio::test]
async fn test_query_null_safe_equals() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_query_is_distinct_from");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_query_is_distinct_from",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `bananna` (`hizat`)
           values
             (null)
           "#;
        &mut db
    ).await?;
    let res = good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_query_is_distinct_from",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             count(*) as `x`
           from
             `bananna`
           where
             not (`hizat` <=> 'abc')
           "#;
        &mut db
    ).await?;
    assert_eq!(res, 1i64);
    let res2 = good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_query_is_distinct_from",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             count(*) as `x`
           from
             `bananna`
           where
             `hizat` <=> null
           "#;
        &mut db
    ).await?;
    assert_eq!(res2, 1i64);
    Ok(())
}

#[tokio::test]
async fn test_query_is_null() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_query_is_null");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_query_is_null",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `bananna` (`hizat`)
           values
             (?)
           "#;
        &mut db,
        p1: string = "not null"
    ).await?;
    assert_eq!(good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_query_is_null",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             count(*) as `count`
           from
             `bananna`
           where
             `bananna`.`hizat` is not null
           "#;
        &mut db
    ).await?, 1i64);
    Ok(())
}

#[tokio::test]
async fn test_query_like() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_query_like");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_query_like",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `bananna` (`hizat`)
           values
             (?)
           "#;
        &mut db,
        p1: string = "apple pie"
    ).await?;
    assert_eq!(good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_query_like",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             `bananna`.`hizat` as `hizat`
           from
             `bananna`
           where
             `bananna`.`hizat` like ?
           "#;
        &mut db,
        p1: string = "apple%"
    ).await?, "apple pie");
    Ok(())
}

#[tokio::test]
async fn test_query_like_escape() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_query_like_escape");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_query_like_escape",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `bananna` (`hizat`)
           values
             ('a%b')
           "#;
        &mut db
    ).await?;
    let res = good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_query_like_escape",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             count(*) as `x`
           from
             `bananna`
           where
             `hizat` like 'a!%b' escape '!'
           "#;
        &mut db
    ).await?;
    assert_eq!(res, 1i64);
    Ok(())
}

#[tokio::test]
async fn test_query_row_number() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_query_row_number");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_query_row_number",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `bananna` (`hizat`)
           values
             (?)
           "#;
        &mut db,
        p1: string = "a"
    ).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_query_row_number",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `bananna` (`hizat`)
           values
             (?)
           "#;
        &mut db,
        p1: string = "b"
    ).await?;
    let res = good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_query_row_number",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             `bananna`.`hizat` as `hizat`,
             row_number() over (
               order by
                 `bananna`.`hizat` asc
             ) as `row_num`
           from
             `bananna`
           "#;
        &mut db
    ).await?;
    assert_eq!(res.len(), 2);
    assert_eq!(res[0].hizat, "a");
    assert_eq!(res[0].row_num, 1i64);
    assert_eq!(res[1].hizat, "b");
    assert_eq!(res[1].row_num, 2i64);
    Ok(())
}

#[tokio::test]
async fn test_query_tuple_cmp() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_base_insert");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    good_ormning::runtime::mariadb::MariadbConnection::execute(
        &mut db.0,
        "insert into bannanana (hizat, hizat2) values ('a', 1), ('b', 2), ('c', 3)",
        vec![],
    ).await?;
    let res = good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_base_insert",
        //# genemichaels-external: sql-formatter-mariadb
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
    ).await?;
    assert_eq!(res.len(), 2);
    assert_eq!(res[0], "a");
    assert_eq!(res[1], "b");
    let res2 = good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_base_insert",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             hizat
           from
             bannanana
           where
             (hizat, hizat2) = ('b', 2)
           "#;
        &mut db
    ).await?;
    assert_eq!(res2.len(), 1);
    assert_eq!(res2[0], "b");
    Ok(())
}

#[tokio::test]
async fn test_query_tuple_in() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_base_insert");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    good_ormning::runtime::mariadb::MariadbConnection::execute(
        &mut db.0,
        "insert into bannanana (hizat, hizat2) values ('a', 1), ('b', 2), ('c', 3)",
        vec![],
    ).await?;
    let res = good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_base_insert",
        //# genemichaels-external: sql-formatter-mariadb
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
    ).await?;
    assert_eq!(res.len(), 2);
    assert_eq!(res[0], "a");
    assert_eq!(res[1], "c");
    Ok(())
}

#[tokio::test]
async fn test_query_union() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_base_insert");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_base_insert",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             `hizat` as `hizat`
           from
             `bannanana`
           union
           select
             `hizat` as `hizat`
           from
             `bannanana`
           "#;
        &mut db
    ).await?;
    Ok(())
}

#[tokio::test]
async fn test_query_window_frame() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_query_window_frame");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    for i in 1 ..= 3 {
        good_ormning::mariadb::good_query!(
            dbm,
            "mariadb_gen_query_window_frame",
            //# genemichaels-external: sql-formatter-mariadb
            r#"insert into
                 `bananna` (`hizat`, `two`)
               values
                 ('key', ?)
               "#;
            &mut db,
            p1: i32 = i
        ).await?;
    }
    let res = good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_query_window_frame",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             sum(`two`) over (
               order by
                 `two` rows between unbounded preceding
                 and current row
             ) as `x`
           from
             `bananna`
           "#;
        &mut db
    ).await?;
    assert_eq!(res, vec![Some("1".to_string()), Some("3".to_string()), Some("6".to_string())]);
    Ok(())
}

#[tokio::test]
async fn test_repeated_param() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_repeated_param");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_repeated_param",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `genrerank` (
               `date`,
               `genre`,
               `secondary`,
               `sort`,
               `rank`,
               `track`
             )
           values
             (?, ?, ?, ?, ?, ?)
           on duplicate key update
             `date` = ?,
             `rank` = ?
           "#;
        &mut db,
        p1: i32 = 20260501,
        p2: string = "rock",
        p3: string = "classic",
        p4: i32 = 1,
        p5: i32 = 10,
        p6: string = "song1",
        p7: i32 = 20260501,
        p8: i32 = 10
    ).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_repeated_param",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `genrerank` (
               `date`,
               `genre`,
               `secondary`,
               `sort`,
               `rank`,
               `track`
             )
           values
             (?, ?, ?, ?, ?, ?)
           on duplicate key update
             `date` = ?,
             `rank` = ?
           "#;
        &mut db,
        p1: i32 = 20260502,
        p2: string = "rock",
        p3: string = "classic",
        p4: i32 = 1,
        p5: i32 = 5,
        p6: string = "song1",
        p7: i32 = 20260502,
        p8: i32 = 5
    ).await?;
    assert_eq!(good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_repeated_param",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             `genrerank`.`date` as `date`
           from
             `genrerank`
           "#;
        &mut db
    ).await?, 20260502);
    assert_eq!(good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_repeated_param",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             `genrerank`.`rank` as `rank`
           from
             `genrerank`
           "#;
        &mut db
    ).await?, 5);
    Ok(())
}

#[tokio::test]
async fn test_returning_wildcard() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_base_insert");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    let res = good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_base_insert",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `bannanana` (`hizat`)
           values
             ('hi')
           returning
             *
           "#;
        &mut db
    ).await?;
    assert_eq!(res.hizat, "hi");
    Ok(())
}

#[tokio::test]
async fn test_select_cte() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_select_cte");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_select_cte",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `select_cte_bannanana` (`hizat`, `hizat2`)
           values
             (?, ?)
           "#;
        &mut db,
        p1: i32 = 1,
        p2: i32 = 7
    ).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_select_cte",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `select_cte_bannanana` (`hizat`, `hizat2`)
           values
             (?, ?)
           "#;
        &mut db,
        p1: i32 = 1,
        p2: i32 = 99
    ).await?;
    let mut res = good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_select_cte",
        //# genemichaels-external: sql-formatter-mariadb
        r#"with
             `hibbo` (`zathi`) as (
               select
                 `select_cte_bannanana`.`hizat2` as `hizat2`
               from
                 `select_cte_bannanana`
             )
           select
             `hibbo`.`zathi` as `zathi`
           from
             `hibbo`
           "#;
        &mut db
    ).await?;
    res.sort();
    assert_eq!(res, vec![7, 99]);
    Ok(())
}

#[tokio::test]
async fn test_select_group_by() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_select_group_by");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_select_group_by",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `bannanana` (`hizat`, `hizat2`)
           values
             (?, ?)
           "#;
        &mut db,
        p1: i32 = 1,
        p2: i32 = 7
    ).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_select_group_by",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `bannanana` (`hizat`, `hizat2`)
           values
             (?, ?)
           "#;
        &mut db,
        p1: i32 = 1,
        p2: i32 = 99
    ).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_select_group_by",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `bannanana` (`hizat`, `hizat2`)
           values
             (?, ?)
           "#;
        &mut db,
        p1: i32 = 2,
        p2: i32 = 3
    ).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_select_group_by",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `bannanana` (`hizat`, `hizat2`)
           values
             (?, ?)
           "#;
        &mut db,
        p1: i32 = 2,
        p2: i32 = 10
    ).await?;
    let mut res = good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_select_group_by",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             sum(`bannanana`.`hizat2`) as `hizat2`
           from
             `bannanana`
           group by
             `bannanana`.`hizat`
           "#;
        &mut db
    ).await?;
    res.sort();
    assert_eq!(res, vec![Some("106".to_string()), Some("13".to_string())]);
    Ok(())
}

#[tokio::test]
async fn test_select_join() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_select_join");
    let db = db().await?;
    let mut db = dbm::migrate(db, Some(&|v| Box::pin(async move {
        match v {
            dbm::DbMariadbGenSelectJoinVersions::V1(db) => {
                good_ormning::mariadb::good_query!(
                    dbm,
                    "mariadb_gen_select_join",
                    //# genemichaels-external: sql-formatter-mariadb
                    r#"insert into
                         `b` (`hizat`, `three`)
                       values
                         ('key', 33)
                       "#;
                    db
                ).await?;
                good_ormning::mariadb::good_query!(
                    dbm,
                    "mariadb_gen_select_join",
                    //# genemichaels-external: sql-formatter-mariadb
                    r#"insert into
                         `select_join_two` (`hizat`, `two`)
                       values
                         ('key', 'no')
                       "#;
                    db
                ).await?;
            },
        }
        return Ok(());
    }))).await?;
    let res = good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_select_join",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             `b`.`three` as `three`,
             `select_join_two`.`two` as `two`
           from
             `b`
             left join `select_join_two` on `b`.`hizat` = `select_join_two`.`hizat`
           "#;
        &mut db
    ).await?;
    assert_eq!(res.three, 33);
    assert_eq!(res.two, Some("no".into()));
    Ok(())
}

#[tokio::test]
async fn test_select_join_cross() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_select_join");
    let db = db().await?;
    let mut db = dbm::migrate(db, Some(&|v| Box::pin(async move {
        match v {
            dbm::DbMariadbGenSelectJoinVersions::V1(db) => {
                good_ormning::mariadb::good_query!(
                    dbm,
                    "mariadb_gen_select_join",
                    //# genemichaels-external: sql-formatter-mariadb
                    r#"insert into
                         `b` (`hizat`, `three`)
                       values
                         ('a', 1)
                       "#;
                    db
                ).await?;
                good_ormning::mariadb::good_query!(
                    dbm,
                    "mariadb_gen_select_join",
                    //# genemichaels-external: sql-formatter-mariadb
                    r#"insert into
                         `b` (`hizat`, `three`)
                       values
                         ('b', 2)
                       "#;
                    db
                ).await?;
                good_ormning::mariadb::good_query!(
                    dbm,
                    "mariadb_gen_select_join",
                    //# genemichaels-external: sql-formatter-mariadb
                    r#"insert into
                         `select_join_two` (`hizat`, `two`)
                       values
                         ('x', 'y')
                       "#;
                    db
                ).await?;
                good_ormning::mariadb::good_query!(
                    dbm,
                    "mariadb_gen_select_join",
                    //# genemichaels-external: sql-formatter-mariadb
                    r#"insert into
                         `select_join_two` (`hizat`, `two`)
                       values
                         ('z', 'w')
                       "#;
                    db
                ).await?;
            },
        }
        return Ok(());
    }))).await?;
    let res = good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_select_join",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             `b`.`three` as `three`,
             `select_join_two`.`two` as `two`
           from
             `b`
             cross join `select_join_two`
           "#;
        &mut db
    ).await?;
    assert_eq!(res.len(), 4);
    Ok(())
}

#[tokio::test]
async fn test_select_join_inner() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_select_join");
    let db = db().await?;
    let mut db = dbm::migrate(db, Some(&|v| Box::pin(async move {
        match v {
            dbm::DbMariadbGenSelectJoinVersions::V1(db) => {
                good_ormning::mariadb::good_query!(
                    dbm,
                    "mariadb_gen_select_join",
                    //# genemichaels-external: sql-formatter-mariadb
                    r#"insert into
                         `b` (`hizat`, `three`)
                       values
                         ('key', 33)
                       "#;
                    db
                ).await?;
                good_ormning::mariadb::good_query!(
                    dbm,
                    "mariadb_gen_select_join",
                    //# genemichaels-external: sql-formatter-mariadb
                    r#"insert into
                         `b` (`hizat`, `three`)
                       values
                         ('nomatch', 44)
                       "#;
                    db
                ).await?;
                good_ormning::mariadb::good_query!(
                    dbm,
                    "mariadb_gen_select_join",
                    //# genemichaels-external: sql-formatter-mariadb
                    r#"insert into
                         `select_join_two` (`hizat`, `two`)
                       values
                         ('key', 'yes')
                       "#;
                    db
                ).await?;
            },
        }
        return Ok(());
    }))).await?;
    let res = good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_select_join",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             `b`.`three` as `three`,
             `select_join_two`.`two` as `two`
           from
             `b`
             inner join `select_join_two` on `b`.`hizat` = `select_join_two`.`hizat`
           "#;
        &mut db
    ).await?;
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].three, 33);
    assert_eq!(res[0].two, "yes".to_string());
    Ok(())
}

#[tokio::test]
async fn test_select_join_right() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_select_join");
    let db = db().await?;
    let mut db = dbm::migrate(db, Some(&|v| Box::pin(async move {
        match v {
            dbm::DbMariadbGenSelectJoinVersions::V1(db) => {
                good_ormning::mariadb::good_query!(
                    dbm,
                    "mariadb_gen_select_join",
                    //# genemichaels-external: sql-formatter-mariadb
                    r#"insert into
                         `b` (`hizat`, `three`)
                       values
                         ('key', 33)
                       "#;
                    db
                ).await?;
                good_ormning::mariadb::good_query!(
                    dbm,
                    "mariadb_gen_select_join",
                    //# genemichaels-external: sql-formatter-mariadb
                    r#"insert into
                         `select_join_two` (`hizat`, `two`)
                       values
                         ('key', 'yes')
                       "#;
                    db
                ).await?;
                good_ormning::mariadb::good_query!(
                    dbm,
                    "mariadb_gen_select_join",
                    //# genemichaels-external: sql-formatter-mariadb
                    r#"insert into
                         `select_join_two` (`hizat`, `two`)
                       values
                         ('nomatch', 'orphan')
                       "#;
                    db
                ).await?;
            },
        }
        return Ok(());
    }))).await?;
    let res = good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_select_join",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             `b`.`three` as `three`,
             `select_join_two`.`two` as `two`
           from
             `b`
             right join `select_join_two` on `b`.`hizat` = `select_join_two`.`hizat`
           "#;
        &mut db
    ).await?;
    assert_eq!(res.len(), 2);
    Ok(())
}

#[tokio::test]
async fn test_select_limit() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_select_limit");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_select_limit",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `bannanana` (`hizat`)
           values
             (?)
           "#;
        &mut db,
        p1: string = "soy"
    ).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_select_limit",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `bannanana` (`hizat`)
           values
             (?)
           "#;
        &mut db,
        p1: string = "soy"
    ).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_select_limit",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `bannanana` (`hizat`)
           values
             (?)
           "#;
        &mut db,
        p1: string = "soy"
    ).await?;
    assert_eq!(good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_select_limit",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             `bannanana`.`hizat` as `hizat`
           from
             `bannanana`
           limit
             2
           "#;
        &mut db
    ).await?.len(), 2);
    Ok(())
}

#[tokio::test]
async fn test_select_offset() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_select_offset");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    for v in [0, 1, 2, 3, 4] {
        good_ormning::mariadb::good_query!(
            dbm,
            "mariadb_gen_select_offset",
            //# genemichaels-external: sql-formatter-mariadb
            r#"insert into
                 `bannanana` (`hizat`)
               values
                 (?)
               "#;
            &mut db,
            p1: i32 = v
        ).await?;
    }
    assert_eq!(good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_select_offset",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             `bannanana`.`hizat` as `hizat`
           from
             `bannanana`
           order by
             `bannanana`.`hizat` asc
           limit
             2
           offset
             ?
           "#;
        &mut db,
        p1: i64 = 2
    ).await?, vec![2, 3]);
    assert_eq!(good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_select_offset",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             `bannanana`.`hizat` as `hizat`
           from
             `bannanana`
           order by
             `bannanana`.`hizat` asc
           limit
             2
           offset
             ?
           "#;
        &mut db,
        p1: i64 = 4
    ).await?, vec![4]);
    Ok(())
}

#[tokio::test]
async fn test_select_order() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_select_order");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_select_order",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `bannanana` (`hizat`)
           values
             (?)
           "#;
        &mut db,
        p1: i32 = 0
    ).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_select_order",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `bannanana` (`hizat`)
           values
             (?)
           "#;
        &mut db,
        p1: i32 = 12
    ).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_select_order",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `bannanana` (`hizat`)
           values
             (?)
           "#;
        &mut db,
        p1: i32 = 9
    ).await?;
    assert_eq!(good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_select_order",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             `bannanana`.`hizat` as `hizat`
           from
             `bannanana`
           order by
             `bannanana`.`hizat` asc
           "#;
        &mut db
    ).await?, vec![0, 9, 12]);
    Ok(())
}

#[tokio::test]
async fn test_select_window() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_select_window");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_select_window",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `select_window_bannanana` (`hizat`, `hizat2`)
           values
             (?, ?)
           "#;
        &mut db,
        p1: i32 = 1,
        p2: i32 = 7
    ).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_select_window",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `select_window_bannanana` (`hizat`, `hizat2`)
           values
             (?, ?)
           "#;
        &mut db,
        p1: i32 = 1,
        p2: i32 = 99
    ).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_select_window",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `select_window_bannanana` (`hizat`, `hizat2`)
           values
             (?, ?)
           "#;
        &mut db,
        p1: i32 = 2,
        p2: i32 = 3
    ).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_select_window",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `select_window_bannanana` (`hizat`, `hizat2`)
           values
             (?, ?)
           "#;
        &mut db,
        p1: i32 = 2,
        p2: i32 = 10
    ).await?;
    let mut res = good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_select_window",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             sum(`select_window_bannanana`.`hizat2`) over (
               partition by
                 `select_window_bannanana`.`hizat`
             ) as `hizat2`
           from
             `select_window_bannanana`
           "#;
        &mut db
    ).await?.into_iter().collect::<Vec<_>>();
    res.sort();
    assert_eq!(
        res,
        vec![Some("106".to_string()), Some("106".to_string()), Some("13".to_string()), Some("13".to_string())]
    );
    Ok(())
}

#[tokio::test]
async fn test_set_ops_all() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_set_ops_all");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_set_ops_all",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `b` (`hizat`)
           values
             ('a')
           "#;
        &mut db
    ).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_set_ops_all",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `b` (`hizat`)
           values
             ('a')
           "#;
        &mut db
    ).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_set_ops_all",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `b` (`hizat`)
           values
             ('b')
           "#;
        &mut db
    ).await?;
    let res = good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_set_ops_all",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             'a' as `h`
           intersect all
           select
             `b`.`hizat` as `h`
           from
             `b`
           "#;
        &mut db
    ).await?;
    assert_eq!(res.len(), 1);
    let res = good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_set_ops_all",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             `b`.`hizat` as `h`
           from
             `b`
           except all
           select
             'a' as `h`
           "#;
        &mut db
    ).await?;
    assert_eq!(res.len(), 2);
    Ok(())
}

#[tokio::test]
async fn test_update() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_update");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_update",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `bananna` (`hizat`)
           values
             ('yog')
           "#;
        &mut db
    ).await?;
    assert_eq!(good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_update",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             `bananna`.`hizat` as `hizat`
           from
             `bananna`
           "#;
        &mut db
    ).await?, "yog");
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_update",
        //# genemichaels-external: sql-formatter-mariadb
        r#"update `bananna`
           set
             `hizat` = 'tep'
           "#;
        &mut db
    ).await?;
    assert_eq!(good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_update",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             `bananna`.`hizat` as `hizat`
           from
             `bananna`
           "#;
        &mut db
    ).await?, "tep");
    Ok(())
}

#[tokio::test]
async fn test_update_where() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_update_where");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_update_where",
        //# genemichaels-external: sql-formatter-mariadb
        r#"insert into
             `ban` (`hizat`)
           values
             ('yog')
           "#;
        &mut db
    ).await?;
    assert_eq!(good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_update_where",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             `ban`.`hizat` as `hizat`
           from
             `ban`
           "#;
        &mut db
    ).await?, "yog");
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_update_where",
        //# genemichaels-external: sql-formatter-mariadb
        r#"update `ban`
           set
             `hizat` = ?
           where
             `ban`.`hizat` = ?
           "#;
        &mut db,
        p1: string = "tep",
        p2: string = "yog2"
    ).await?;
    assert_eq!(good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_update_where",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             `ban`.`hizat` as `hizat`
           from
             `ban`
           "#;
        &mut db
    ).await?, "yog");
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_update_where",
        //# genemichaels-external: sql-formatter-mariadb
        r#"update `ban`
           set
             `hizat` = ?
           where
             `ban`.`hizat` = ?
           "#;
        &mut db,
        p1: string = "tep",
        p2: string = "yog"
    ).await?;
    assert_eq!(good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_update_where",
        //# genemichaels-external: sql-formatter-mariadb
        r#"select
             `ban`.`hizat` as `hizat`
           from
             `ban`
           "#;
        &mut db
    ).await?, "tep");
    Ok(())
}

#[tokio::test]
async fn test_fixes() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_fixes");
    let mut client = db().await?;
    let mut db = dbm::migrate(&mut client, None).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_fixes",
        r#"insert into `t` (`name`, `n`, `f`, `u`) values ('a', 1, 1.5, 4000000000), ('b', 2, 2.5, 1), ('c', 2, 3.5, 2)"#;
        &mut db
    ).await?;
    assert_eq!(good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_fixes",
        r#"select `t`.`name` from `t` where `t`.`n` >= ? and `t`.`n` <= ? order by `t`.`name`"#;
        &mut db,
        p1: i32 = 0,
        p2: i32 = 1
    ).await?, vec!["a".to_string()]);
    assert_eq!(good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_fixes",
        r#"select `t`.`name` from `t` where `t`.`n` = ? and `t`.`name` <> ? order by `t`.`name`"#;
        &mut db,
        p1: i32 = 2,
        p2: string = "b"
    ).await?, vec!["c".to_string()]);
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_fixes",
        r#"insert ignore into `t` (`name`, `n`, `f`, `u`) values ('a', 9, 9.5, ?)"#;
        &mut db,
        p1: u32 = 7
    ).await?;
    assert_eq!(good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_fixes",
        r#"select count(`t`.`id`) as `c` from `t`"#;
        &mut db
    ).await?, 3);
    assert_eq!(good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_fixes",
        r#"select `t`.`u` from `t` where `t`.`name` = 'a'"#;
        &mut db
    ).await?, 4000000000u32);
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_fixes",
        r#"insert into `u` (`uname`, `t_id`) values (${string = &format!("{}", "x")}, ${opt i64 = Some(1)})"#;
        &mut db
    ).await?;
    assert_eq!(good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_fixes",
        r#"select count(distinct `t`.`n`) as `c` from `t`"#;
        &mut db
    ).await?, 2);
    assert_eq!(good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_fixes",
        r#"select (select `t`.`name` from `t` where `t`.`n` = 99) as `x`"#;
        &mut db
    ).await?, None);
    assert_eq!(good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_fixes",
        r#"select `t`.`name` as `h` from `t` union select `u`.`uname` as `h` from `u` order by `h` desc limit 2"#;
        &mut db
    ).await?, vec!["x".to_string(), "c".to_string()]);
    let rows = good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_fixes",
        r#"select * from `t` inner join `u` on `u`.`t_id` = `t`.`id`"#;
        &mut db
    ).await?;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].name, "a");
    assert_eq!(rows[0].uname, "x");
    assert_eq!(rows[0].t_id, Some(1));
    assert_eq!(good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_fixes",
        r#"select `t`.`name` from `t` where `t`.`n` = ? and `t`.`name` <> ? order by `t`.`name`"#;
        &mut db,
        p1: i32 = 2,
        p2: string = "b"
    ).await?, vec!["c".to_string()]);
    assert_eq!(good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_fixes",
        r#"select `t`.`n` div 2 as `h` from `t` order by `t`.`name` limit 1, 2"#;
        &mut db
    ).await?, vec![1, 1]);
    assert_eq!(good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_fixes",
        r#"select `a`.`name` as `an` from `t` as `a` inner join `t` as `b` on `a`.`n` = `b`.`n` and `a`.`id` <> `b`.`id`"#;
        &mut db
    ).await?.len(), 2);
    assert_eq!(good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_fixes",
        r#"select `s`.`name` as `nm` from (select `t`.`name` from `t`) as `s` order by `nm`"#;
        &mut db
    ).await?.len(), 3);
    assert_eq!(good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_fixes",
        r#"with recursive `c` (`n`) as (select 1 union all select `c`.`n` + 1 from `c` where `c`.`n` < 3) select `c`.`n` from `c`"#;
        &mut db
    ).await?, vec![1, 2, 3]);
    assert_eq!(good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_fixes",
        r#"with `c` as (select `t`.`name` from `t`) select `c`.`name` from `c` order by `c`.`name`"#;
        &mut db
    ).await?.len(), 3);
    assert_eq!(
        good_ormning::mariadb::good_query_many!(
            dbm,
            "mariadb_gen_fixes",
            r#"select coalesce(`t`.`x`, 0) as `v`, cast(`t`.`n` as char) as `s`, upper(`t`.`name`) as `up`, length(`t`.`name`) as `l`, abs(`t`.`f` * 2) as `a` from `t` where `t`.`f` > 1.5 and `t`.`f` < 10 order by `t`.`name`"#;
            &mut db
        ).await?.into_iter().map(|r| (r.v, r.s, r.up, r.l, r.a)).collect::<Vec<_>>(),
        vec![
            (0i64, "2".to_string(), "B".to_string(), 1i32, 5.0f64),
            (0i64, "2".to_string(), "C".to_string(), 1i32, 7.0f64),
        ]
    );
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_fixes",
        r#"insert into `t` (`name`, `n`, `f`, `u`) values ('a', 5, 5.5, 5) on duplicate key update `n` = values(`n`)"#;
        &mut db
    ).await?;
    assert_eq!(good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_fixes",
        r#"select `t`.`n` from `t` where `t`.`name` = 'a'"#;
        &mut db
    ).await?, 5);
    assert_eq!(good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_fixes",
        r#"select `t`.`name` from `t`, `u` where `u`.`t_id` = `t`.`id`"#;
        &mut db
    ).await?.len(), 1);
    assert_eq!(good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_fixes",
        r#"select `a`.`name` as `an` from `t` as `a` inner join `t` as `b` using (`n`) where `a`.`id` <> `b`.`id`"#;
        &mut db
    ).await?.len(), 2);
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_fixes",
        r#"update `t` as `a` join `u` on `u`.`t_id` = `a`.`id` set `n` = 7"#;
        &mut db
    ).await?;
    assert_eq!(good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_fixes",
        r#"select `t`.`n` from `t` where `t`.`name` = 'a'"#;
        &mut db
    ).await?, 7);
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_fixes",
        r#"delete `x` from `u` as `x` join `t` on `t`.`id` = `x`.`t_id` where `t`.`n` = 7"#;
        &mut db
    ).await?;
    let r = good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_fixes",
        r#"select substr(`t`.`name`, 1, 1) as `s`, floor(`t`.`f`) as `r`, concat(`t`.`name`, '!') as `c`, now() as `t` from `t` where `t`.`name` = 'a'"#;
        &mut db
    ).await?;
    assert_eq!((r.s, r.r, r.c), ("a".to_string(), 1.0, "a!".to_string()));
    assert!(r.t > jiff::civil::date(2020, 1, 1).at(0, 0, 0, 0));
    assert_eq!(good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_fixes",
        r#"select group_concat(`t`.`name`) as `g` from `t` where `t`.`name` = 'a'"#;
        &mut db
    ).await?, Some("a".to_string()));
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_fixes",
        r#"delete from `t` where `t`.`name` <> 'a' order by `t`.`name` desc limit 1"#;
        &mut db
    ).await?;
    assert_eq!(good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_fixes",
        r#"delete from `t` where `t`.`name` = 'b' returning `t`.`name`"#;
        &mut db
    ).await?, vec!["b".to_string()]);
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_fixes",
        r#"insert into `t` set `name` = 'd', `n` = 4, `f` = 4.5, `u` = 4"#;
        &mut db
    ).await?;
    assert!(good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_fixes",
        r#"select `t`.`name` from `t`"#;
        &mut db
    ).await.is_err());
    Ok(())
}

#[tokio::test]
async fn test_migrate_resume() -> Result<(), loga::Error> {
    use good_ormning::runtime::mariadb::mysql_async::prelude::Queryable;

    good_module!(dbm, "mariadb_gen_migrate_resume");
    let mut conn = db().await?;
    let res = dbm::migrate(&mut conn, Some(&|v| Box::pin(async move {
        match v {
            dbm::DbMariadbGenMigrateResumeVersions::V0(db) => {
                good_ormning::mariadb::good_query!(
                    dbm,
                    "mariadb_gen_migrate_resume",
                    0,
                    "insert into resume_a (x) values (1), (1)";
                    db
                ).await?;
            },
            _ => { },
        }
        return Ok(());
    }))).await;
    assert!(res.is_err());
    let (version, step, hash): (i64, i64, String) =
        conn.query_first("select version, step, hash from __good_version").await.map_err(loga::err)?.unwrap();
    assert_eq!(version, 0);
    assert!(step > 0);
    assert!(!hash.is_empty());
    conn.query_drop("update __good_version set hash = 'xxx'").await.map_err(loga::err)?;
    assert!(dbm::migrate(&mut conn, None).await.is_err());
    conn.exec_drop("update __good_version set hash = ?", (hash,)).await.map_err(loga::err)?;
    conn.query_drop("delete from resume_a limit 1").await.map_err(loga::err)?;
    let mut db = dbm::migrate(&mut conn, None).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_migrate_resume",
        "insert into resume_b (z) values (2)";
        &mut db
    ).await?;
    assert!(good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_migrate_resume",
        "insert into resume_a (x, y) values (1, 3)";
        &mut db
    ).await.is_err());
    assert_eq!(dbm::get_schema_version(&mut db.0).await?, Some(1));
    Ok(())
}

async fn run_graph_migrations(
    conn: &mut good_ormning::runtime::mariadb::mysql_async::Conn,
    versions: Vec<good_ormning::mariadb::Version>,
) -> Result<(), loga::Error> {
    use {
        good_ormning::{
            mariadb::{
                Version,
                graph::utils::MariadbMigrateCtx,
            },
            runtime::mariadb::mysql_async::prelude::Queryable,
            utils::Errs,
        },
        std::collections::HashMap,
    };

    let errs = Errs::new();
    let mut prev_version: Option<Version> = None;
    for (i, version) in versions.into_iter().enumerate() {
        let mut table_sql_names = HashMap::new();
        for (table_id, table) in &version.tables {
            table_sql_names.insert(table_id.clone(), table.id.clone());
        }
        let mut state = MariadbMigrateCtx::new(errs.clone(), table_sql_names, version.clone());
        good_ormning::graphmigrate::migrate(
            &mut state,
            prev_version.take().map(|v| v.to_migrate_nodes()),
            &version.to_migrate_nodes(),
        );
        for statement in &state.statements {
            for attempt in 0 .. 2 {
                conn
                    .query_drop(statement.as_str())
                    .await
                    .map_err(|e| loga::err(format!("Version {}, attempt {}, [{}]: {}", i, attempt, statement, e)))?;
            }
        }
        prev_version = Some(version);
    }
    errs.raise().map_err(|e| loga::err(e.join("\n")))?;
    return Ok(());
}

#[tokio::test]
async fn test_migrate_statements_repeatable() -> Result<(), loga::Error> {
    use {
        good_ormning::{
            mariadb::{
                Version,
                query::expr::SerialExpr,
                schema::field::{
                    field_bool,
                    field_i32,
                    field_i64,
                    field_str,
                },
            },
            runtime::mariadb::mysql_async::prelude::Queryable,
        },
    };

    let versions = vec![{
        let v = Version::new();
        let bananna = v.table("bananna");
        let hizat = bananna.field("hizat", field_str().max_len(50).build());
        let zomzom = bananna.field("zomzom", field_i32().build());
        bananna.primary_key(&[&hizat]);
        bananna.unique_index("bananna_zomzom", &[&zomzom]);
        bananna.index("bananna_zomzom_hizat", &[&zomzom, &hizat]);
        let peel = v.table("peel");
        let peel_hizat = peel.field("bananna_hizat", field_str().max_len(50).build());
        let peel_zomzom = peel.field("bananna_zomzom", field_i32().build());
        peel.foreign_key("peel_bananna", &[(&peel_hizat, &hizat)]);
        peel.foreign_key("peel_bananna_zomzom", &[(&peel_zomzom, &zomzom)]);
        v.table("gone").field("gone", field_i32().build());
        v.build()
    }, {
        let v = Version::new();
        let banana = v.table("banana").renamed_from("bananna");
        let hizat = banana.field("hizat2", field_str().max_len(50).build()).renamed_from("hizat");
        let zomzom = banana.field("zomzom", field_i32().build());
        banana.primary_key(&[&hizat]);
        banana.unique_index("banana_zomzom", &[&zomzom]).renamed_from("bananna_zomzom");
        banana.index("banana_zomzom_hizat", &[&zomzom, &hizat]).renamed_from("bananna_zomzom_hizat");
        let peel = v.table("peel");
        let peel_hizat = peel.field("bananna_hizat", field_str().max_len(50).build());
        let peel_zomzom = peel.field("bananna_zomzom", field_i32().build());
        peel.foreign_key("peel_banana", &[(&peel_hizat, &hizat)]).renamed_from("peel_bananna");
        peel.foreign_key("peel_bananna_zomzom", &[(&peel_zomzom, &zomzom)]);
        v.table("gone").field("gone", field_i32().build());
        v.build()
    }, {
        let v = Version::new();
        let banana = v.table("banana");
        banana.field("hizat2", field_str().max_len(100).opt().build());
        let zomzom = banana.field("zomzom", field_i64().build());
        banana.field("fresh", field_bool().default(SerialExpr::LitBool(true)).build());
        banana.primary_key(&[&zomzom]);
        let peel = v.table("peel");
        let peel_zomzom = peel.field("bananna_zomzom", field_i64().build());
        peel.foreign_key("peel_bananna_zomzom", &[(&peel_zomzom, &zomzom)]);
        v.table("new").field("new", field_i32().build());
        v.build()
    }, {
        let v = Version::new();
        v.table("other").field("other", field_i32().build());
        v.build()
    }];
    let mut conn = db().await?;
    run_graph_migrations(&mut conn, versions).await?;
    let tables: Vec<String> = conn.query("show tables").await.map_err(loga::err)?;
    assert_eq!(tables, vec!["other".to_string()]);
    Ok(())
}

#[tokio::test]
async fn test_migrate_key_column_changes() -> Result<(), loga::Error> {
    use good_ormning::{
        mariadb::{
            Version,
            schema::field::{
                field_i32,
                field_i64,
                field_str,
            },
        },
        runtime::mariadb::mysql_async::prelude::Queryable,
    };

    let versions = vec![{
        let v = Version::new();
        let counter = v.table("counter");
        let id = counter.field("id", field_i64().auto_increment().build());
        let part = counter.field("part", field_i32().build());
        counter.primary_key(&[&id]);
        counter.index("counter_part", &[&part]);
        let parent = v.table("parent");
        let parent_id = parent.field("id", field_i32().build());
        let parent_name = parent.field("name", field_str().max_len(10).build());
        parent.primary_key(&[&parent_id]);
        parent.unique_index("parent_name", &[&parent_name]);
        let child = v.table("child");
        let child_parent = child.field("parent", field_i32().build());
        let child_name = child.field("name", field_str().max_len(10).build());
        child.foreign_key("child_parent", &[(&child_parent, &parent_id)]);
        child.foreign_key("child_name", &[(&child_name, &parent_name)]);
        let later = v.table("later");
        let later_id = later.field("id", field_i32().build());
        later.primary_key(&[&later_id]);
        let same_name = v.table("same_name");
        let same_name_id = same_name.field("id", field_i64().auto_increment().build());
        same_name.field("other", field_i32().build());
        same_name.index("same_name_key", &[&same_name_id]);
        let drop_column = v.table("drop_column");
        let drop_column_id = drop_column.field("id", field_i64().auto_increment().build());
        drop_column.field("other", field_i32().build());
        drop_column.index("drop_column_id", &[&drop_column_id]);
        v.table("grow").field("other", field_i32().build());
        v.build()
    }, {
        let v = Version::new();
        let counter = v.table("counter");
        let id = counter.field("id", field_i64().auto_increment().build());
        let part = counter.field("part", field_i32().build());
        counter.primary_key(&[&id, &part]);
        counter.index("counter_part", &[&part]);
        let parent = v.table("parent");
        let parent_id = parent.field("id", field_i64().build());
        let parent_name = parent.field("name", field_str().max_len(20).build());
        parent.primary_key(&[&parent_id]);
        parent.unique_index("parent_name", &[&parent_name]);
        let child = v.table("child");
        let child_parent = child.field("parent", field_i64().build());
        let child_name = child.field("name", field_str().max_len(20).build());
        child.foreign_key("child_parent", &[(&child_parent, &parent_id)]);
        child.foreign_key("child_name", &[(&child_name, &parent_name)]);
        let later = v.table("later");
        let later_id = later.field("id", field_i32().auto_increment().build());
        later.primary_key(&[&later_id]);
        let same_name = v.table("same_name");
        let same_name_id = same_name.field("id", field_i64().auto_increment().build());
        let same_name_other = same_name.field("other", field_i32().build());
        same_name.index("same_name_key", &[&same_name_id, &same_name_other]);
        v.table("drop_column").field("other", field_i32().build());
        let grow = v.table("grow");
        let grow_id = grow.field("id", field_i64().auto_increment().build());
        grow.field("other", field_i32().build());
        grow.primary_key(&[&grow_id]);
        v.build()
    }, {
        let v = Version::new();
        let counter = v.table("counter");
        let id = counter.field("id", field_i64().auto_increment().build());
        counter.field("part", field_i32().build());
        counter.primary_key(&[&id]);
        let later = v.table("later");
        let later_id = later.field("id", field_i32().auto_increment().build());
        let later_extra = later.field("extra", field_i32().build());
        later.primary_key(&[&later_id, &later_extra]);
        v.build()
    }, {
        let v = Version::new();
        let counter = v.table("counter");
        let id = counter.field("id2", field_i64().auto_increment().build()).renamed_from("id");
        let part = counter.field("part", field_i32().build());
        counter.unique_index("counter_id", &[&id, &part]);
        let later = v.table("later");
        let later_id = later.field("id", field_i32().auto_increment().build());
        let later_extra = later.field("extra", field_i32().build());
        later.primary_key(&[&later_id, &later_extra]);
        v.build()
    }];
    let mut conn = db().await?;
    run_graph_migrations(&mut conn, versions).await?;
    conn.query_drop("insert into `counter` (`part`) values (1), (1)").await.map_err(loga::err)?;
    let ids: Vec<i64> = conn.query("select `id2` from `counter` order by `id2`").await.map_err(loga::err)?;
    assert_eq!(ids, vec![1, 2]);
    conn.query_drop("insert into `later` (`extra`) values (1), (2)").await.map_err(loga::err)?;
    let ids: Vec<i32> = conn.query("select `id` from `later` order by `id`").await.map_err(loga::err)?;
    assert_eq!(ids, vec![1, 2]);
    Ok(())
}

#[tokio::test]
async fn test_migrate_rename_chain() -> Result<(), loga::Error> {
    use good_ormning::{
        mariadb::{
            Version,
            schema::field::field_i32,
        },
        runtime::mariadb::mysql_async::prelude::Queryable,
    };

    let v0 = {
        let v = Version::new();
        let t = v.table("t");
        let b = t.field("b", field_i32().build());
        let c = t.field("c", field_i32().build());
        t.index("ib", &[&b]);
        t.index("ic", &[&c]);
        v.table("tb").field("x", field_i32().build());
        v.table("tc").field("x", field_i32().build());
        v.build()
    };
    let v1 = {
        let v = Version::new();
        let t = v.table("t");
        let a = t.field("a", field_i32().build()).renamed_from("b");
        let b = t.field("b", field_i32().build()).renamed_from("c");
        t.index("ia", &[&a]).renamed_from("ib");
        t.index("ib", &[&b]).renamed_from("ic");
        v.table("ta").renamed_from("tb").field("x", field_i32().build());
        v.table("tb").renamed_from("tc").field("x", field_i32().build());
        v.build()
    };
    let mut conn = db().await?;
    run_graph_migrations(&mut conn, vec![v0.clone()]).await?;
    conn.query_drop("insert into `t` (`b`, `c`) values (1, 2)").await.map_err(loga::err)?;
    conn.query_drop("insert into `tb` (`x`) values (3)").await.map_err(loga::err)?;
    conn.query_drop("insert into `tc` (`x`) values (4)").await.map_err(loga::err)?;
    let errs = good_ormning::utils::Errs::new();
    let mut table_sql_names = std::collections::HashMap::new();
    for (table_id, table) in &v1.tables {
        table_sql_names.insert(table_id.clone(), table.id.clone());
    }
    let mut state =
        good_ormning::mariadb::graph::utils::MariadbMigrateCtx::new(errs.clone(), table_sql_names, v1.clone());
    good_ormning::graphmigrate::migrate(&mut state, Some(v0.to_migrate_nodes()), &v1.to_migrate_nodes());
    errs.raise().map_err(|e| loga::err(e.join("\n")))?;
    for statement in &state.statements {
        for _ in 0 .. 2 {
            conn.query_drop(statement.as_str()).await.map_err(|e| loga::err(format!("[{}]: {}", statement, e)))?;
        }
    }
    let row: Option<(i32, i32)> = conn.query_first("select `a`, `b` from `t`").await.map_err(loga::err)?;
    assert_eq!(row, Some((1, 2)));
    let ta: Option<i32> = conn.query_first("select `x` from `ta`").await.map_err(loga::err)?;
    let tb: Option<i32> = conn.query_first("select `x` from `tb`").await.map_err(loga::err)?;
    assert_eq!((ta, tb), (Some(3), Some(4)));
    let indexes: Vec<(String, String)> =
        conn
            .query(
                "select `index_name`, `column_name` from information_schema.statistics where `table_schema` = database() and `table_name` = 't' order by `index_name`",
            )
            .await
            .map_err(loga::err)?;
    assert_eq!(indexes, vec![("ia".to_string(), "a".to_string()), ("ib".to_string(), "b".to_string())]);
    Ok(())
}

#[tokio::test]
async fn test_migrate_auto_increment_keys() -> Result<(), loga::Error> {
    use good_ormning::runtime::mariadb::mysql_async::prelude::Queryable;

    good_module!(dbm, "mariadb_gen_auto_increment_keys");
    let db = dbm::migrate(db().await?, None).await?;
    let mut conn = db.0;
    for table in ["index_to_pk", "pk_to_index", "same_name", "add_auto", "new_column"] {
        conn.query_drop(format!("insert into `{}` (`other`) values (1), (2)", table)).await.map_err(loga::err)?;
        let ids: Vec<i64> =
            conn.query(format!("select `id` from `{}` order by `id`", table)).await.map_err(loga::err)?;
        assert_eq!(ids, vec![1, 2], "{}", table);
    }
    conn.query_drop("insert into `drop_auto` (`id`, `other`) values (7, 1)").await.map_err(loga::err)?;
    let keys: Vec<(String, String)> =
        conn
            .query(
                "select `table_name`, `index_name` from information_schema.statistics where `table_schema` = database() and `seq_in_index` = 1 and `table_name` <> '__good_version' order by `table_name`, `index_name`",
            )
            .await
            .map_err(loga::err)?;
    assert_eq!(
        keys,
        [
            ("add_auto", "add_auto_id"),
            ("index_to_pk", "PRIMARY"),
            ("new_column", "PRIMARY"),
            ("pk_to_index", "pk_to_index_id"),
            ("same_name", "same_name_key"),
        ]
            .into_iter()
            .map(|(t, i)| (t.to_string(), i.to_string()))
            .collect::<Vec<_>>()
    );
    Ok(())
}

#[tokio::test]
async fn test_type_safety() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_safety");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_safety",
        r#"insert into `v` (`de`, `b`) values (0.10000000000000000001, x'0102ff')"#;
        &mut db
    ).await?;
    let res = good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_safety",
        r#"select coalesce(`ti`, 1000) as `a`, coalesce(`iu`, -1) as `b`, coalesce(`bi`, `bu`) as `c`, coalesce(`f`, 0) as `d`, substring(`b`, 2, 2) as `e`, -`ti` as `g` from `v`"#;
        &mut db
    ).await?;
    let a: i32 = res.a;
    let b: i64 = res.b;
    let c: Option<String> = res.c;
    let d: f64 = res.d;
    let e: Option<Vec<u8>> = res.e;
    let g: Option<i64> = res.g;
    assert_eq!((a, b, c, d, e, g), (1000, -1, None, 0., Some(vec![2, 255]), None));
    let m = good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_safety",
        r#"select `de` * ? as `m` from `v` where `de` = ?"#;
        &mut db,
        p1: decimal = "2",
        p2: decimal = "0.10000000000000000001"
    ).await?;
    assert!(m.as_deref().unwrap_or("").starts_with("0.20000000000000000002"), "{:?}", m);
    let floats = dbm::float_literals(&mut db).await?;
    assert_eq!((floats.f64, floats.f32), (1.0, 1.5));
    Ok(())
}

#[tokio::test]
async fn test_in_list() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_safety");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_safety",
        r#"insert into `v` (`ti`, `s`, `de`) values (1, 'Ab', 1.5), (2, 'b', 2.5), (3, 'c', 3.5), (4, 'd', 4.5), (5, null, null)"#;
        &mut db
    ).await?;
    for (
        low,
        ids,
        high,
        want,
    ) in [
        (0, vec![], 9, vec![]),
        (0, vec![3], 9, vec![3]),
        (2, vec![1, 2, 4], 9, vec![2, 4]),
        (0, vec![5, 4, 3, 2, 1], 4, vec![1, 2, 3, 4]),
    ] {
        let got = good_ormning::mariadb::good_query_many!(
            dbm,
            "mariadb_gen_safety",
            r#"select `ti` from `v` where `ti` >= ? and `ti` in (?) and `ti` <= ? order by `ti`"#;
            &mut db,
            low: i8 = low,
            ids: arr i8 = & ids,
            high: i8 = high
        ).await?;
        assert_eq!(got, want.into_iter().map(Some).collect::<Vec<_>>());
    }
    for (ids, want) in [(vec![], vec![1, 2, 3, 4, 5]), (vec![1, 5], vec![2, 3, 4])] {
        let got = good_ormning::mariadb::good_query_many!(
            dbm,
            "mariadb_gen_safety",
            r#"select `ti` from `v` where `ti` not in (?) order by `ti`"#;
            &mut db,
            ids: arr i8 = & ids
        ).await?;
        assert_eq!(got, want.into_iter().map(Some).collect::<Vec<_>>());
    }
    assert_eq!(dbm::by_names(&mut db, &["ab", "c", "zz"]).await?, vec![Some("Ab".to_string()), Some("c".to_string())]);
    assert_eq!(good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_safety",
        r#"select `ti` from `v` where `de` in (?) order by `ti`"#;
        &mut db,
        amounts: arr decimal = &["2.5", "4.50"]
    ).await?, vec![Some(2), Some(4)]);
    for (
        pairs,
        want,
    ) in [(vec![], vec![]), (vec![(1, "ab")], vec![1]), (vec![(1, "b"), (2, "b"), (3, "c")], vec![2, 3])] {
        let got = good_ormning::mariadb::good_query_many!(
            dbm,
            "mariadb_gen_safety",
            r#"select `ti` from `v` where (`ti`, `s`) in (?) order by `ti`"#;
            &mut db,
            pairs: arr(i8, string) = & pairs
        ).await?;
        assert_eq!(got, want.into_iter().map(Some).collect::<Vec<_>>());
    }
    let pairs = vec![(1, "ab"), (4, "d")];
    assert_eq!(good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_safety",
        r#"select `ti` from `v` where (`ti`, `s`) not in (${arr (i8, string) = &pairs}) and `s` is not null order by `ti`"#;
        &mut db
    ).await?, vec![Some(2), Some(3)]);
    assert_eq!(good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_safety",
        r#"select `ti` from `v` where (`ti`, `de`, `s`) in (?) order by `ti`"#;
        &mut db,
        rows: arr(i8, decimal, opt string) = &[(2, "2.50", Some("b")), (3, "3.5", None)]
    ).await?, vec![Some(2)]);
    Ok(())
}

#[tokio::test]
async fn test_builder_queries() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_builder");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    let a = dbm::upsert(&mut db, "a", 1).await?;
    dbm::upsert(&mut db, "b", 2).await?;
    dbm::upsert(&mut db, "c", 3).await?;
    assert_eq!(dbm::upsert(&mut db, "a", 10).await?, a);
    dbm::insert_u(&mut db, "a", 100).await?;
    dbm::insert_u(&mut db, "c", 1000).await?;
    dbm::insert_u(&mut db, "c", 1000).await?;
    dbm::zero_last(&mut db).await?;
    dbm::add_bonus(&mut db).await?;
    let list = |rows: Vec<dbm::ListRow>| rows.into_iter().map(|r| (r.name, r.n)).collect::<Vec<_>>();
    assert_eq!(
        list(dbm::list(&mut db).await?),
        vec![("a".to_string(), 111), ("b".to_string(), 2), ("c".to_string(), 1000)]
    );
    dbm::delete_by_bonus(&mut db, 100).await?;
    assert_eq!(list(dbm::list(&mut db).await?), vec![("b".to_string(), 2), ("c".to_string(), 1000)]);
    Ok(())
}

#[tokio::test]
async fn test_types() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_types");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_types",
        "insert into t (i8, i16, u8, u16, u64, short, b) values (?, ?, ?, ?, ?, ?, ?)";
        &mut db,
        p1: i8 = - 128,
        p2: i16 = - 32768,
        p3: u8 = 255,
        p4: u16 = 65535,
        p5: u64 = u64:: MAX,
        p6: string = "abc",
        p7: opt bytes = Some(&[0u8, 1, 2, 255][..])
    ).await?;
    let r = good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_types",
        "select i8, i16, u8, u16, u64, short, b from t";
        &mut db
    ).await?;
    assert_eq!(
        (r.i8, r.i16, r.u8, r.u16, r.u64, r.short, r.b),
        (-128, -32768, 255, 65535, u64::MAX, "abc".to_string(), Some(vec![0u8, 1, 2, 255]))
    );
    assert!(good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_types",
        "insert into t (i8, i16, u8, u16, u64, short) values (0, 0, 0, 0, 0, 'abcd')";
        &mut db
    ).await.is_err());
    Ok(())
}

#[tokio::test]
async fn test_pool() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_base_insert");
    let opts = integration_tests::mariadb_server::db_opts().await?;
    let conn = good_ormning::runtime::mariadb::mysql_async::Conn::new(opts.clone()).await.map_err(loga::err)?;
    dbm::migrate(conn, None).await?;
    let mut db = dbm::DbMariadbGenBaseInsert(good_ormning::runtime::mariadb::mysql_async::Pool::new(opts));
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_base_insert",
        "insert into bannanana (hizat) values ('a'), ('b')";
        &mut db
    ).await?;
    assert_eq!(good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_base_insert",
        "select hizat from bannanana order by hizat";
        &mut db
    ).await?, vec!["a".to_string(), "b".to_string()]);
    db.0.disconnect().await.map_err(loga::err)?;
    Ok(())
}

#[tokio::test]
async fn test_native() -> Result<(), loga::Error> {
    good_module!(dbm, "mariadb_gen_native");
    let db = db().await?;
    let mut db = dbm::migrate(db, Some(&|v| Box::pin(async move {
        match v {
            dbm::DbMariadbGenNativeVersions::V0(db) => {
                good_ormning::mariadb::good_query!(
                    dbm,
                    "mariadb_gen_native",
                    0,
                    "insert into t (name, n, f) values ('old', 1, 0.5e0)";
                    db
                ).await?;
                good_ormning::mariadb::good_query!(
                    dbm,
                    "mariadb_gen_native",
                    0,
                    "insert into w (x) values (10), (20)";
                    db
                ).await?;
            },
            _ => { },
        }
        return Ok(());
    }))).await?;
    assert_eq!(good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_native",
        "select d from t where name = 'old'";
        &mut db
    ).await?, "0.00");
    let rows = good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_native",
        "select x, id, label, code, m from w order by x";
        &mut db
    ).await?;
    assert_eq!(
        rows.into_iter().map(|r| (r.x, r.id, r.label, r.code, r.m)).collect::<Vec<_>>(),
        vec![
            (10, 1u64, "none".to_string(), "abc".to_string(), -5),
            (20, 2u64, "none".to_string(), "abc".to_string(), -5)
        ]
    );
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_native",
        "insert into w (x, flag) values (30, 2)";
        &mut db
    ).await?;
    assert_eq!(good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_native",
        "select id, flag, flag + 1 as f1 from w where x = 30";
        &mut db
    ).await.map(|r| (r.id, r.flag, r.f1))?, (3u64, Some(true), Some(3)));
    let at = NaiveDate::from_ymd_opt(2020, 2, 3).unwrap().and_hms_milli_opt(4, 5, 6, 789).unwrap();
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_native",
        "insert into t (name, n, f, d, at) values ('a', 3, 1.5e0, ?, ?), ('b', ${i32 = 4}, 2.5e0, 2.25, null)";
        &mut db,
        p1: decimal = "1.10",
        p2: opt datetime_chrono = Some(at)
    ).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_native",
        "insert into u (name, v) values ('a', 10), ('b', 20)";
        &mut db
    ).await?;
    assert_eq!(good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_native",
        "select at from t where name = 'a'";
        &mut db
    ).await?, Some(at));
    let r = good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_native",
        "select sum(n) as s, avg(n) as a, sum(d) as sd, sum(f) as sf, 7 / 2 as q, 7 div 2 as i, 1.5 as l from t";
        &mut db
    ).await?;
    assert_eq!(r.s, Some("8".to_string()));
    assert_eq!(r.a, Some("2.6667".to_string()));
    assert_eq!(r.sd, Some("3.35".to_string()));
    assert_eq!(r.sf, Some(4.5));
    assert_eq!(r.q, "3.5000");
    assert_eq!(r.i, 3);
    assert_eq!(r.l, "1.5");
    assert_eq!(good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_native",
        "select 5 & 3 as x";
        &mut db
    ).await?, 1u64);
    assert_eq!(good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_native",
        "select name from t where n - 1 and not n - 3 order by name";
        &mut db
    ).await?, vec!["a".to_string()]);
    let r = good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_native",
        "select trim(leading 'x' from 'xxaxx') as a, trim(trailing 'y' from 'byy') as b, trim('x' from 'xcx') as c";
        &mut db
    ).await?;
    assert_eq!((r.a, r.b, r.c), ("axx".to_string(), "b".to_string(), "c".to_string()));
    assert_eq!(good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_native",
        "select t.n, u.v from t join u using (name) order by t.n";
        &mut db
    ).await?.into_iter().map(|r| (r.n, r.v)).collect::<Vec<_>>(), vec![(3, 10), (4, 20)]);
    assert_eq!(good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_native",
        "select name from t order by name offset 1 rows";
        &mut db
    ).await?, vec!["b".to_string(), "old".to_string()]);
    let mut names = good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_native",
        "select name from t where n > 3 union (select name from u order by name limit 1)";
        &mut db
    ).await?;
    names.sort();
    assert_eq!(names, vec!["a".to_string(), "b".to_string()]);
    assert!(good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_native",
        "select now() as x";
        &mut db
    ).await? > NaiveDate::from_ymd_opt(2020, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap());
    let r = good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_native",
        "select cast(n as decimal(10, 2)) as c, cast(name as char(1)) as s, cast(n as datetime(3)) is null as dt from t where name = 'a'";
        &mut db
    ).await?;
    assert_eq!((r.c, r.s), ("3.00".to_string(), "a".to_string()));
    assert_eq!(good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_native",
        "select group_concat(distinct name order by name desc separator '|') as g from t";
        &mut db
    ).await?, Some("old|b|a".to_string()));
    assert_eq!(good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_native",
        "select group_concat(name order by name limit 2) as g from t";
        &mut db
    ).await?, Some("a,b".to_string()));
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_native",
        "replace into w (id, x) values (1, 99)";
        &mut db
    ).await?;
    assert_eq!(good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_native",
        "select x from w where id = 1";
        &mut db
    ).await?, 99);
    assert_eq!(good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_native",
        "select t.n, u.v from t natural join u order by t.n";
        &mut db
    ).await?.into_iter().map(|r| (r.n, r.v)).collect::<Vec<_>>(), vec![(3, 10), (4, 20)]);
    let mut names = good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_native",
        "(select name from t order by name limit 1) union (select name from u order by name desc limit 1)";
        &mut db
    ).await?;
    names.sort();
    assert_eq!(names, vec!["a".to_string(), "b".to_string()]);
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_native",
        "update t join u on u.name = t.name set u.v = u.v + t.n, t.n = 5 where t.name = 'a'";
        &mut db
    ).await?;
    assert_eq!(good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_native",
        "select u.v, t.n from u join t on t.name = u.name where u.name = 'a'";
        &mut db
    ).await.map(|r| (r.v, r.n))?, (13, 5));
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_native",
        "delete u, t from t join u on u.name = t.name where t.name = 'b'";
        &mut db
    ).await?;
    assert_eq!(good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_native",
        "select name from u union all select name from t order by name";
        &mut db
    ).await?, vec!["a".to_string(), "a".to_string(), "old".to_string()]);
    let rows = good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_native",
        "select * from t join u using (name)";
        &mut db
    ).await?;
    assert_eq!(rows.into_iter().map(|r| (r.name, r.n, r.v)).collect::<Vec<_>>(), vec![("a".to_string(), 5, 13)]);
    assert_eq!(good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_native",
        "select name from t natural join u";
        &mut db
    ).await?, vec!["a".to_string()]);
    let mut names = good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_native",
        "((select name from t union select name from u) order by name desc limit 1) union (select name from u)";
        &mut db
    ).await?;
    names.sort();
    assert_eq!(names, vec!["a".to_string(), "old".to_string()]);
    let r = good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_native",
        "select n + 1 as a, f * 2 as b, n / 0 as c, 5000000000 as big, 7 div 2.5 as d, sqrt(-1) as e, case when n > 100 then 1 end as g from t where name = 'a'";
        &mut db
    ).await?;
    assert_eq!((r.a, r.b, r.c, r.big, r.d, r.e, r.g), (6i64, 3.0f64, None, 5000000000i64, 2i64, None, None));
    let mut xs = good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_native",
        "select 1 as x union select null";
        &mut db
    ).await?;
    xs.sort();
    assert_eq!(xs, vec![None, Some(1)]);
    assert_eq!(good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_native",
        "select name from t where name = 'a' for update";
        &mut db
    ).await?, vec!["a".to_string()]);
    assert_eq!(good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_native",
        "select name from t order by name offset 1 rows fetch first 1 rows only";
        &mut db
    ).await?, vec!["old".to_string()]);
    assert_eq!(
        good_ormning::mariadb::good_query_many!(
            dbm,
            "mariadb_gen_native",
            "select name, row_number() over w as r from t window w as (order by name desc) order by name";
            &mut db
        ).await?.into_iter().map(|r| (r.name, r.r)).collect::<Vec<_>>(),
        vec![("a".to_string(), 2), ("old".to_string(), 1)]
    );
    let r = good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_native",
        "select at + interval 1 day as next, extract(year from at) as y, timestamp '2020-01-01 00:00:00' as lit from t where name = 'a'";
        &mut db
    ).await?;
    assert_eq!(r.next, Some(at + chrono::Duration::days(1)));
    assert_eq!(r.y, Some(2020));
    assert_eq!(r.lit, NaiveDate::from_ymd_opt(2020, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap());
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_native",
        "insert ignore into w (id, x) values (1, 5) on duplicate key update x = 6";
        &mut db
    ).await?;
    assert_eq!(good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_native",
        "select x from w where id = 1";
        &mut db
    ).await?, 6);
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_native",
        "insert into parent (id) values (1)";
        &mut db
    ).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_native",
        "insert into child (pid) values (1)";
        &mut db
    ).await?;
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_native",
        "delete from parent";
        &mut db
    ).await?;
    assert_eq!(good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_native",
        "select count(*) as c from child";
        &mut db
    ).await?, 0);
    assert_eq!(good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_native",
        "select n % 2 as k, count(*) as c from t group by k having c > 0 order by k";
        &mut db
    ).await?.into_iter().map(|r| (r.k, r.c)).collect::<Vec<_>>(), vec![(1i64, 2)]);
    assert_eq!(good_ormning::mariadb::good_query_many!(
        dbm,
        "mariadb_gen_native",
        "select name from t where name not like 'o%'";
        &mut db
    ).await?, vec!["a".to_string()]);
    Ok(())
}

#[tokio::test]
async fn test_temporal() -> Result<(), loga::Error> {
    use good_ormning::runtime::mariadb::mysql_async::prelude::Queryable;

    good_module!(dbm, "mariadb_gen_temporal");
    let db = db().await?;
    let mut db = dbm::migrate(db, None).await?;
    let d = jiff::civil::date(2024, 2, 29);
    let dc = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
    let tm = -jiff::SignedDuration::new(800 * 3600 + 62, 250_000_000);
    let tc = chrono::TimeDelta::hours(37);
    let ts = jiff::civil::date(2024, 1, 2).at(3, 4, 5, 123_456_000);
    good_ormning::mariadb::good_query!(
        dbm,
        "mariadb_gen_temporal",
        "insert into e (d, dc, tm, tc, y, ts, s) values (?, ?, ?, ?, ?, ?, ?)";
        &mut db,
        d: opt date_jiff = Some(d),
        dc: opt date_chrono = Some(dc),
        tm: opt time_jiff = Some(tm),
        tc: opt time_chrono = Some(tc),
        y: opt u16 = Some(2024),
        ts: opt datetime_jiff = Some(ts),
        s: opt string = Some("a\\b")
    ).await?;
    let r = good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_temporal",
        "select d, dc, tm, tc, y, ts, s, dd, td, sd from e";
        &mut db
    ).await?;
    assert_eq!(r.d, Some(d));
    assert_eq!(r.dc, Some(dc));
    assert_eq!(r.tm, Some(tm));
    assert_eq!(r.tc, Some(tc));
    assert_eq!(r.y, Some(2024));
    assert_eq!(r.ts, Some(ts));
    assert_eq!(r.s, Some("a\\b".to_string()));
    assert_eq!(r.dd, jiff::civil::date(2020, 1, 2));
    assert_eq!(r.td, -jiff::SignedDuration::from_secs(3723));
    assert_eq!(r.sd, "a\\b'c");
    let r = good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_temporal",
        "select curdate() as cd, date(ts) as dts, year(d) as yr, d + interval 1 day as dn, date_add(d, interval 1 hour) as dh, date_add(tm, interval 1 day) as th, date '2020-01-01' as ld, time '01:00:00' as lt, cast('12:00:00' as time) as ct, timediff(ts, ts) as z from e";
        &mut db
    ).await?;
    assert!(r.cd > NaiveDate::from_ymd_opt(2020, 1, 1).unwrap());
    assert_eq!(r.dts, NaiveDate::from_ymd_opt(2024, 1, 2));
    assert_eq!(r.yr, Some(2024));
    assert_eq!(r.dn, Some(jiff::civil::date(2024, 3, 1)));
    assert_eq!(r.dh, NaiveDate::from_ymd_opt(2024, 2, 29).unwrap().and_hms_opt(1, 0, 0));
    assert_eq!(r.th, Some(tm + jiff::SignedDuration::from_hours(24)));
    assert_eq!(r.ld, NaiveDate::from_ymd_opt(2020, 1, 1).unwrap());
    assert_eq!(r.lt, chrono::TimeDelta::hours(1));
    assert_eq!(r.ct, Some(chrono::TimeDelta::hours(12)));
    assert_eq!(r.z, Some(chrono::TimeDelta::zero()));
    assert_eq!(good_ormning::mariadb::good_query_one!(
        dbm,
        "mariadb_gen_temporal",
        "select count(*) as c from e where d < ts or dc = d";
        &mut db
    ).await?, 0);
    for backslash_escapes in [true, false] {
        if !backslash_escapes {
            db.0.query_drop("set sql_mode = concat(@@sql_mode, ',NO_BACKSLASH_ESCAPES')").await.map_err(loga::err)?;
        }
        good_ormning::mariadb::good_query!(
            dbm,
            "mariadb_gen_temporal",
            "insert into e (s) values ('x\\\\y'), ('z')";
            &mut db
        ).await?;
        assert_eq!(good_ormning::mariadb::good_query_one!(
            dbm,
            "mariadb_gen_temporal",
            "select group_concat(s order by s separator '\\\\') as g from e where s in ('x\\\\y', 'z')";
            &mut db
        ).await?, Some("x\\y\\z".to_string()));
        good_ormning::mariadb::good_query!(
            dbm,
            "mariadb_gen_temporal",
            "delete from e where s in ('x\\\\y', 'z')";
            &mut db
        ).await?;
    }
    Ok(())
}
