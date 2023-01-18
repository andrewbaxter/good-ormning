use integration_tests::MyString;
use testcontainers::{
    images::postgres::Postgres,
    Container,
};
use tokio_postgres::Config;
use anyhow::Result;

pub mod pg_gen_base_insert;
pub mod pg_gen_param_i32;
pub mod pg_gen_param_utctime;
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
pub mod pg_gen_migrate_detect_1;
pub mod pg_gen_migrate_detect_2;
pub mod pg_gen_migrate_remove_field;
pub mod pg_gen_migrate_add_table;
pub mod pg_gen_migrate_remove_table;

async fn db<
    'a,
>(docker: &testcontainers::clients::Cli) -> Result<(tokio_postgres::Client, Container<'_, Postgres>)> {
    let db_container = docker.run(Postgres::default());
    let mut db_config = Config::new();
    db_config.host("127.0.0.1");
    db_config.dbname("postgres");
    db_config.user("postgres");
    db_config.port(db_container.get_host_port_ipv4(5432));
    let (db, db_conn) = db_config.connect(tokio_postgres::NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = db_conn.await {
            eprintln!("connection error: {}", e);
        }
    });
    Ok((db, db_container))
}

#[tokio::test]
async fn test_base_insert() -> Result<()> {
    let docker = testcontainers::clients::Cli::default();
    let (mut db, _cont) = db(&docker).await?;
    pg_gen_base_insert::migrate(&mut db).await?;
    pg_gen_base_insert::insert_banan(&mut db, "soy").await?;
    assert_eq!(pg_gen_base_insert::get_banan(&mut db).await?, "soy");
    Ok(())
}

#[tokio::test]
async fn test_param_i32() -> Result<()> {
    let docker = testcontainers::clients::Cli::default();
    let (mut db, _cont) = db(&docker).await?;
    pg_gen_param_i32::migrate(&mut db).await?;
    pg_gen_param_i32::insert_banan(&mut db, 22).await?;
    assert_eq!(pg_gen_param_i32::get_banan(&mut db).await?, 22);
    Ok(())
}

#[tokio::test]
async fn test_param_utctime() -> Result<()> {
    let docker = testcontainers::clients::Cli::default();
    let (mut db, _cont) = db(&docker).await?;
    let ref_date = chrono::TimeZone::with_ymd_and_hms(&chrono::Utc, 1937, 12, 1, 0, 0, 0).unwrap();
    pg_gen_param_utctime::migrate(&mut db).await?;
    pg_gen_param_utctime::insert_banan(&mut db, ref_date).await?;
    assert_eq!(pg_gen_param_utctime::get_banan(&mut db).await?, ref_date);
    Ok(())
}

#[tokio::test]
async fn test_param_opt_i32() -> Result<()> {
    let docker = testcontainers::clients::Cli::default();
    let (mut db, _cont) = db(&docker).await?;
    pg_gen_param_opt_i32::migrate(&mut db).await?;
    pg_gen_param_opt_i32::insert_banan(&mut db, Some(47)).await?;
    assert_eq!(pg_gen_param_opt_i32::get_banan(&mut db).await?, Some(47));
    Ok(())
}

#[tokio::test]
async fn test_param_opt_i32_null() -> Result<()> {
    let docker = testcontainers::clients::Cli::default();
    let (mut db, _cont) = db(&docker).await?;
    pg_gen_param_opt_i32_null::migrate(&mut db).await?;
    pg_gen_param_opt_i32_null::insert_banan(&mut db).await?;
    assert_eq!(pg_gen_param_opt_i32_null::get_banan(&mut db).await?, None);
    Ok(())
}

#[tokio::test]
async fn test_param_custom() -> Result<()> {
    let docker = testcontainers::clients::Cli::default();
    let (mut db, _cont) = db(&docker).await?;
    pg_gen_param_custom::migrate(&mut db).await?;
    pg_gen_param_custom::insert_banan(&mut db, &MyString("soy".into())).await?;
    assert_eq!(pg_gen_param_custom::get_banan(&mut db).await?, MyString("soy".into()));
    Ok(())
}

#[tokio::test]
async fn test_param_opt_custom() -> Result<()> {
    let docker = testcontainers::clients::Cli::default();
    let (mut db, _cont) = db(&docker).await?;
    pg_gen_param_opt_custom::migrate(&mut db).await?;
    pg_gen_param_opt_custom::insert_banan(&mut db, Some(&MyString("higgins".into()))).await?;
    assert_eq!(pg_gen_param_opt_custom::get_banan(&mut db).await?, Some(MyString("higgins".into())));
    Ok(())
}

#[tokio::test]
async fn test_insert_on_conflict_do_nothing() -> Result<()> {
    let docker = testcontainers::clients::Cli::default();
    let (mut db, _cont) = db(&docker).await?;
    pg_gen_insert_on_conflict_do_nothing::migrate(&mut db).await?;
    assert!(pg_gen_insert_on_conflict_do_nothing::insert_banan(&mut db, "soy").await?.is_some());
    assert!(pg_gen_insert_on_conflict_do_nothing::insert_banan(&mut db, "soy").await?.is_none());
    Ok(())
}

#[tokio::test]
async fn test_insert_on_conflict_update() -> Result<()> {
    let docker = testcontainers::clients::Cli::default();
    let (mut db, _cont) = db(&docker).await?;
    pg_gen_insert_on_conflict_update::migrate(&mut db).await?;
    assert_eq!(pg_gen_insert_on_conflict_update::insert_banan(&mut db, "soy", 33).await?, 33);
    assert_eq!(pg_gen_insert_on_conflict_update::insert_banan(&mut db, "soy", 7).await?, 34);
    assert_eq!(pg_gen_insert_on_conflict_update::insert_banan(&mut db, "yyyy", 7).await?, 7);
    Ok(())
}

#[tokio::test]
async fn test_update() -> Result<()> {
    let docker = testcontainers::clients::Cli::default();
    let (mut db, _cont) = db(&docker).await?;
    pg_gen_update::migrate(&mut db).await?;
    pg_gen_update::insert_banan(&mut db).await?;
    assert_eq!(pg_gen_update::get_banan(&mut db).await?, "yog");
    pg_gen_update::update_banan(&mut db).await?;
    assert_eq!(pg_gen_update::get_banan(&mut db).await?, "tep");
    Ok(())
}

#[tokio::test]
async fn test_update_where() -> Result<()> {
    let docker = testcontainers::clients::Cli::default();
    let (mut db, _cont) = db(&docker).await?;
    pg_gen_update_where::migrate(&mut db).await?;
    pg_gen_update_where::insert_banan(&mut db).await?;
    assert_eq!(pg_gen_update_where::get_banan(&mut db).await?, "yog");
    pg_gen_update_where::update_banan(&mut db, "tep", "yog2").await?;
    assert_eq!(pg_gen_update_where::get_banan(&mut db).await?, "yog");
    pg_gen_update_where::update_banan(&mut db, "tep", "yog").await?;
    assert_eq!(pg_gen_update_where::get_banan(&mut db).await?, "tep");
    Ok(())
}

#[tokio::test]
async fn test_update_returning() -> Result<()> {
    let docker = testcontainers::clients::Cli::default();
    let (mut db, _cont) = db(&docker).await?;
    pg_gen_update_returning::migrate(&mut db).await?;
    pg_gen_update_returning::insert_banan(&mut db).await?;
    assert_eq!(pg_gen_update_returning::update_banan(&mut db).await?, Some("tep".to_string()));
    Ok(())
}

#[tokio::test]
async fn test_delete() -> Result<()> {
    let docker = testcontainers::clients::Cli::default();
    let (mut db, _cont) = db(&docker).await?;
    pg_gen_delete::migrate(&mut db).await?;
    pg_gen_delete::insert_banan(&mut db).await?;
    assert_eq!(pg_gen_delete::get_banan(&mut db).await?, Some("seeon".to_string()));
    pg_gen_delete::no_banan(&mut db).await?;
    assert_eq!(pg_gen_delete::get_banan(&mut db).await?, None);
    Ok(())
}

#[tokio::test]
async fn test_delete_where() -> Result<()> {
    let docker = testcontainers::clients::Cli::default();
    let (mut db, _cont) = db(&docker).await?;
    pg_gen_delete_where::migrate(&mut db).await?;
    pg_gen_delete_where::insert_banan(&mut db).await?;
    pg_gen_delete_where::no_banan(&mut db, "nozo").await?;
    assert_eq!(pg_gen_delete_where::get_banan(&mut db).await?, Some("seeon".to_string()));
    pg_gen_delete_where::no_banan(&mut db, "seeon").await?;
    assert_eq!(pg_gen_delete_where::get_banan(&mut db).await?, None);
    Ok(())
}

#[tokio::test]
async fn test_delete_returning() -> Result<()> {
    let docker = testcontainers::clients::Cli::default();
    let (mut db, _cont) = db(&docker).await?;
    pg_gen_delete_where::migrate(&mut db).await?;
    pg_gen_delete_where::insert_banan(&mut db).await?;
    assert!(pg_gen_delete_where::get_banan(&mut db).await?.is_some());
    pg_gen_delete_where::no_banan(&mut db, "seeon").await?;
    assert!(pg_gen_delete_where::get_banan(&mut db).await?.is_none());
    Ok(())
}

#[tokio::test]
async fn test_select_join() -> Result<()> {
    let docker = testcontainers::clients::Cli::default();
    let (mut db, _cont) = db(&docker).await?;
    pg_gen_select_join::migrate(&mut db).await?;
    let res = pg_gen_select_join::get_it(&mut db).await?;
    assert_eq!(res.three, 33);
    assert_eq!(res.two, Some("no".into()));
    Ok(())
}

#[tokio::test]
async fn test_select_group_by() -> Result<()> {
    let docker = testcontainers::clients::Cli::default();
    let (mut db, _cont) = db(&docker).await?;
    pg_gen_select_group_by::migrate(&mut db).await?;
    pg_gen_select_group_by::insert_banan(&mut db, 1, 7).await?;
    pg_gen_select_group_by::insert_banan(&mut db, 1, 99).await?;
    pg_gen_select_group_by::insert_banan(&mut db, 2, 3).await?;
    pg_gen_select_group_by::insert_banan(&mut db, 2, 10).await?;
    let mut res = pg_gen_select_group_by::get_banan(&mut db).await?;
    res.sort();
    assert_eq!(res, vec![13, 106]);
    Ok(())
}

#[tokio::test]
async fn test_select_limit() -> Result<()> {
    let docker = testcontainers::clients::Cli::default();
    let (mut db, _cont) = db(&docker).await?;
    pg_gen_select_limit::migrate(&mut db).await?;
    pg_gen_select_limit::insert_banan(&mut db, "soy").await?;
    pg_gen_select_limit::insert_banan(&mut db, "soy").await?;
    pg_gen_select_limit::insert_banan(&mut db, "soy").await?;
    assert_eq!(pg_gen_select_limit::get_banan(&mut db).await?.len(), 2);
    Ok(())
}

#[tokio::test]
async fn test_select_order() -> Result<()> {
    let docker = testcontainers::clients::Cli::default();
    let (mut db, _cont) = db(&docker).await?;
    pg_gen_select_order::migrate(&mut db).await?;
    pg_gen_select_order::insert_banan(&mut db, 0).await?;
    pg_gen_select_order::insert_banan(&mut db, 12).await?;
    pg_gen_select_order::insert_banan(&mut db, 9).await?;
    assert_eq!(pg_gen_select_order::get_banan(&mut db).await?, vec![0, 9, 12]);
    Ok(())
}

#[tokio::test]
async fn test_migrate_add_field() -> Result<()> {
    let docker = testcontainers::clients::Cli::default();
    let (mut db, _cont) = db(&docker).await?;
    pg_gen_migrate_add_field::migrate(&mut db).await?;
    match pg_gen_migrate_add_field::get_banan(&mut db).await? {
        Some(x) => {
            assert_eq!(x.zomzom, true);
            assert_eq!(&x.hizat, "nizoot");
        },
        None => assert!(false),
    };
    Ok(())
}

#[tokio::test]
async fn test_migrate_detect() -> Result<()> {
    let docker = testcontainers::clients::Cli::default();
    let (mut db, _cont) = db(&docker).await?;
    pg_gen_migrate_detect_2::migrate(&mut db).await?;
    assert!(
        matches!(pg_gen_migrate_detect_1::get_banan(&mut db).await, Err(good_ormning::runtime::Error::BadSchema))
    );
    Ok(())
}

#[tokio::test]
async fn test_migrate_remove_field() -> Result<()> {
    let docker = testcontainers::clients::Cli::default();
    let (mut db, _cont) = db(&docker).await?;
    pg_gen_migrate_remove_field::migrate(&mut db).await?;
    pg_gen_migrate_remove_field::new_banan(&mut db, "yordol").await?;
    Ok(())
}

#[tokio::test]
async fn test_migrate_add_table() -> Result<()> {
    let docker = testcontainers::clients::Cli::default();
    let (mut db, _cont) = db(&docker).await?;
    pg_gen_migrate_add_table::migrate(&mut db).await?;
    pg_gen_migrate_add_table::two(&mut db, 23).await?;
    Ok(())
}

#[tokio::test]
async fn test_migrate_remove_table() -> Result<()> {
    let docker = testcontainers::clients::Cli::default();
    let (mut db, _cont) = db(&docker).await?;
    pg_gen_migrate_remove_table::migrate(&mut db).await?;
    Ok(())
}
