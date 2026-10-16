use {
    good_ormning::{
        import::{
            codegen,
            mariadb,
        },
        runtime::mariadb::mysql_async::prelude::Queryable,
    },
    good_ormning_core::mariadb::{
        query::expr::SerialExpr,
        schema::{
            constraint::{
                ConstraintType,
                ForeignKeyAction,
            },
            field::TypeVariant,
        },
        types::SimpleSimpleType,
    },
    integration_tests::mariadb_server::db,
};

fn check_generated(code: &str) {
    syn::parse_file(code).unwrap_or_else(|e| panic!("Generated code doesn't parse: {}\n{}", e, code));
    let last_let = code.rfind("let ").unwrap();
    let first_fk = code.find("foreign_key(").unwrap();
    assert!(first_fk > last_let, "foreign_key emitted before all bindings were defined:\n{}", code);
}

#[tokio::test]
async fn test_import_mariadb() {
    let mut conn = db().await.unwrap();
    for stmt in [
        //# genemichaels-external: sql-formatter-mariadb
        r#"create table users (
             id bigint not null auto_increment primary key,
             name varchar(100) not null,
             email varchar(200) not null unique,
             score double,
             active boolean not null default true,
             small tinyint unsigned,
             data blob,
             notes text,
             code char(5) not null default 'a''b',
             rank mediumint unsigned not null default 7
           )
           "#,
        //# genemichaels-external: sql-formatter-mariadb
        r#"create table posts (
             id int not null auto_increment primary key,
             user_id bigint not null,
             title varchar(100) not null,
             body longtext,
             constraint posts_user foreign key (user_id) references users (id) on delete cascade
           )
           "#,
        //# genemichaels-external: sql-formatter-mariadb
        r#"create table __good_migration_state (x int)
           "#,
        //# genemichaels-external: sql-formatter-mariadb
        r#"create table abcgoodxtable (x int)
           "#,
        //# genemichaels-external: sql-formatter-mariadb
        r#"create index idx_posts_user_title on posts (user_id, title)
           "#,
        //# genemichaels-external: sql-formatter-mariadb
        r#"create unique index idx_posts_title on posts (title)
           "#,
        //# genemichaels-external: sql-formatter-mariadb
        r#"create index idx_posts_prefix on posts (body(10))
           "#,
    ] {
        conn.query_drop(stmt).await.unwrap();
    }
    let (version, failures) = mariadb::read_schema(&mut conn, false).await.unwrap();
    assert_eq!(failures.len(), 1, "{:?}", failures);
    assert!(failures[0].contains("idx_posts_prefix"), "{:?}", failures);
    assert!(version.tables.contains_key("users"), "missing users table");
    assert!(version.tables.contains_key("posts"), "missing posts table");
    assert!(!version.tables.contains_key("__good_migration_state"), "internal table should be excluded");
    assert!(version.tables.contains_key("abcgoodxtable"), "LIKE wildcard excluded a non-internal table");
    let users = &version.tables["users"];
    let t = |f: &str| users.fields[f].type_.type_.type_.type_.clone();
    assert_eq!(t("id"), SimpleSimpleType::I64);
    assert!(users.fields["id"].type_.auto_increment);
    assert_eq!(t("name"), SimpleSimpleType::String);
    assert_eq!(users.fields["name"].type_.max_len, Some(100));
    assert!(!users.fields["name"].type_.type_.opt);
    assert_eq!(t("score"), SimpleSimpleType::F64);
    assert!(users.fields["score"].type_.type_.opt);
    assert_eq!(t("active"), SimpleSimpleType::Bool);
    assert_eq!(t("small"), SimpleSimpleType::U8);
    assert_eq!(users.fields["active"].type_.default, Some(SerialExpr::LitBool(true)));
    assert_eq!(t("data"), SimpleSimpleType::Bytes);
    assert_eq!(users.fields["data"].type_.variant, Some(TypeVariant::Blob));
    assert_eq!(t("notes"), SimpleSimpleType::String);
    assert_eq!(users.fields["notes"].type_.variant, Some(TypeVariant::Text));
    assert_eq!(users.fields["notes"].type_.max_len, None);
    assert_eq!(users.fields["code"].type_.variant, Some(TypeVariant::Char));
    assert_eq!(users.fields["code"].type_.max_len, Some(5));
    assert_eq!(users.fields["code"].type_.default, Some(SerialExpr::LitString("a'b".to_string())));
    assert_eq!(t("rank"), SimpleSimpleType::U32);
    assert_eq!(users.fields["rank"].type_.variant, Some(TypeVariant::Mediumint));
    assert_eq!(users.fields["rank"].type_.default, Some(SerialExpr::LitAnyInt(7)));
    assert!(users.constraints.values().any(|c| matches!(&c.type_, ConstraintType::PrimaryKey(_))));
    let email_idx =
        users
            .indices
            .values()
            .find(|i| i.fields == vec!["email".to_string()])
            .expect("users missing unique index for UNIQUE constraint on email");
    assert!(email_idx.unique);
    let posts = &version.tables["posts"];
    let ConstraintType::ForeignKey(fk) = &posts.constraints["posts_user"].type_ else {
        panic!("posts_user isn't a foreign key");
    };
    assert_eq!(fk.remote_table, "users");
    assert_eq!(fk.fields, vec![("user_id".to_string(), "id".to_string())]);
    assert_eq!(fk.on_delete, Some(ForeignKeyAction::Cascade));
    assert_eq!(fk.on_update, None);
    assert_eq!(posts.indices["idx_posts_user_title"].fields, vec!["user_id".to_string(), "title".to_string()]);
    assert!(posts.indices["idx_posts_title"].unique);
    assert!(!posts.indices.contains_key("idx_posts_prefix"), "prefix index should be skipped");
    assert!(!posts.indices.contains_key("posts_user"), "foreign key index should be skipped");
    assert_eq!(posts.fields["id"].type_.type_.type_.type_, SimpleSimpleType::I32);
    assert!(posts.fields["id"].type_.auto_increment);
    let code = codegen::generate_mariadb(&version, "testmariadb").unwrap();
    check_generated(&code);
    assert!(code.contains("auto_increment"));
    assert!(code.contains("on_delete"));
    assert!(code.contains("field_mediumint_unsigned"));
    assert!(code.contains("field_text"));
    assert!(code.contains("LitString"));
    assert!(code.contains("max_len(100u32)"));
    assert!(code.contains("foreign_key"));
    assert!(code.contains("unique_index"));
    assert!(code.contains("\"testmariadb\""));
    let out_dir = tempfile::tempdir().unwrap();
    std::env::set_var("OUT_DIR", out_dir.path());
    good_ormning::mariadb::generate(good_ormning::mariadb::GenerateArgs {
        db_name: Some("testmariadb".to_string()),
        versions: vec![(1usize, version.clone())],
        ..Default::default()
    }).unwrap();
    conn
        .query_drop(
            "create table events (id bigint auto_increment primary key, at datetime(3) not null, ts timestamp(2) null, d date not null default '2020-01-02', tm time(2) default '-01:02:03.5', y year default 2024, zd date default '0000-00-00', g text, fulltext index events_g (g), index events_ts (id, ts))",
        )
        .await
        .unwrap();
    let (version, failures) = mariadb::read_schema(&mut conn, false).await.unwrap();
    let events = &version.tables["events"];
    assert_eq!(events.fields["at"].type_.type_.type_.type_, SimpleSimpleType::DatetimeJiff);
    assert_eq!(events.fields["at"].type_.precision, Some(3));
    assert_eq!(events.fields["ts"].type_.type_.type_.type_, SimpleSimpleType::DatetimeJiff);
    assert_eq!(events.fields["ts"].type_.variant, Some(TypeVariant::Timestamp));
    assert_eq!(events.fields["ts"].type_.precision, Some(2));
    assert_eq!(events.fields["d"].type_.type_.type_.type_, SimpleSimpleType::DateJiff);
    assert_eq!(events.fields["d"].type_.default, Some(SerialExpr::LitDate("2020-01-02".to_string())));
    assert_eq!(events.fields["tm"].type_.type_.type_.type_, SimpleSimpleType::TimeJiff);
    assert_eq!(events.fields["tm"].type_.precision, Some(2));
    assert_eq!(events.fields["tm"].type_.default, Some(SerialExpr::LitTime("-01:02:03.50".to_string())));
    assert_eq!(events.fields["y"].type_.type_.type_.type_, SimpleSimpleType::U16);
    assert_eq!(events.fields["y"].type_.variant, Some(TypeVariant::Year));
    assert_eq!(events.fields["y"].type_.default, Some(SerialExpr::LitAnyInt(2024)));
    assert!(!events.fields.contains_key("zd"));
    assert!(!events.indices.contains_key("events_g"));
    assert!(events.indices.contains_key("events_ts"));
    assert_eq!(failures.len(), 3, "{:?}", failures);
    assert!(failures.iter().any(|f| f.contains("\"zd\"") && f.contains("zero date")), "{:?}", failures);
    assert!(failures.iter().any(|f| f.contains("\"events_g\"")), "{:?}", failures);
    let code = codegen::generate_mariadb(&version, "testmariadbevents").unwrap();
    syn::parse_file(&code).unwrap_or_else(|e| panic!("Generated code doesn't parse: {}\n{}", e, code));
    assert!(code.contains("field_timestamp_jiff"), "{}", code);
    assert!(code.contains("field_year"), "{}", code);
    assert!(code.contains("LitDate"), "{}", code);
    good_ormning::mariadb::generate(good_ormning::mariadb::GenerateArgs {
        db_name: Some("testmariadbevents".to_string()),
        versions: vec![(1usize, version.clone())],
        ..Default::default()
    }).unwrap();
    let (version, _) = mariadb::read_schema(&mut conn, true).await.unwrap();
    assert_eq!(version.tables["events"].fields["at"].type_.type_.type_.type_, SimpleSimpleType::DatetimeChrono);
    assert_eq!(version.tables["events"].fields["d"].type_.type_.type_.type_, SimpleSimpleType::DateChrono);
    assert_eq!(version.tables["events"].fields["tm"].type_.type_.type_.type_, SimpleSimpleType::TimeChrono);
}
