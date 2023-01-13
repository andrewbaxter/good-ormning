use integrationtests::MyString;
use testcontainers::{
    images::postgres::Postgres,
    Container,
};
use tokio_postgres::Config;
use anyhow::Result;

pub mod pg_gen_base_insert;
pub mod pg_gen_param_i32;
pub mod pg_gen_param_opt_i32;
pub mod pg_gen_param_opt_i32_null;
pub mod pg_gen_param_custom;
pub mod pg_gen_param_opt_custom;
pub mod pg_gen_migrate_add_field;
pub mod pg_gen_migrate_remove_field;

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
async fn test_migrate_add_field() -> Result<()> {
    let docker = testcontainers::clients::Cli::default();
    let (mut db, _cont) = db(&docker).await?;
    pg_gen_migrate_add_field::migrate(&mut db).await?;
    assert_eq!(pg_gen_migrate_add_field::get_banan(&mut db).await?, Some(MyString("higgins".into())));
    Ok(())
}

#[tokio::test]
async fn test_migrate_remove_field() -> Result<()> {
    let docker = testcontainers::clients::Cli::default();
    let (mut db, _cont) = db(&docker).await?;
    pg_gen_migrate_remove_field::migrate(&mut db).await?;
    pg_gen_migrate_remove_field::insert_banan(&mut db, Some(&MyString("higgins".into()))).await?;
    Ok(())
}
