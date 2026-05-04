use {
    good_ormning::import::{
        codegen,
        pg,
    },
    good_ormning_core::{
        pg::{
            schema::constraint::ConstraintType,
            types::SimpleSimpleType,
        },
    },
    pglite_oxide::PgliteServer,
};

async fn new_client(server: &PgliteServer) -> tokio_postgres::Client {
    let (client, db_conn) = tokio_postgres::connect(&server.connection_uri(), tokio_postgres::NoTls).await.unwrap();
    tokio::spawn(async move {
        if let Err(e) = db_conn.await {
            eprintln!("connection error: {}", e);
        }
    });
    return client;
}

#[tokio::test]
async fn test_import_pg() {
    let server = PgliteServer::temporary_tcp().unwrap();
    let client = new_client(&server).await;
    client.simple_query("SET search_path TO public").await.unwrap();
    for stmt in [
        "CREATE TABLE users (id bigserial PRIMARY KEY, name text NOT NULL, score double precision, active boolean NOT NULL DEFAULT true, data bytea)",
        "CREATE TABLE posts (id bigserial PRIMARY KEY, user_id bigint NOT NULL REFERENCES users(id), title text NOT NULL, body text)",
        "CREATE INDEX idx_posts_user ON posts(user_id)",
        "CREATE UNIQUE INDEX idx_posts_title ON posts(title)",
    ] {
        client.simple_query(stmt).await.unwrap();
    }
    let version = pg::read_schema(&client).await.unwrap();

    // Verify tables
    assert!(version.tables.contains_key("users"), "missing users table");
    assert!(version.tables.contains_key("posts"), "missing posts table");

    // Users: id is bigserial → Auto type
    let users = &version.tables["users"];
    assert!(users.fields.contains_key("id"), "users missing id field");
    assert!(users.fields.contains_key("name"), "users missing name field");
    assert!(users.fields.contains_key("score"), "users missing score field");
    assert!(users.fields.contains_key("active"), "users missing active field");
    assert!(users.fields.contains_key("data"), "users missing data field");
    assert_eq!(
        users.fields["id"].type_.type_.type_.type_,
        SimpleSimpleType::Auto,
        "id field should be Auto type (bigserial)"
    );

    // PK constraint
    let has_pk = users.constraints.values().any(|c| matches!(&c.type_, ConstraintType::PrimaryKey(_)));
    assert!(has_pk, "users missing PK constraint");

    // Posts: FK and indexes
    let posts = &version.tables["posts"];
    let has_fk = posts.constraints.values().any(|c| matches!(&c.type_, ConstraintType::ForeignKey(_)));
    assert!(has_fk, "posts missing FK constraint");
    assert!(!posts.indices.is_empty(), "posts missing indices");
    let has_unique_idx = posts.indices.values().any(|i| i.unique);
    assert!(has_unique_idx, "posts missing unique index");

    // Generate code and sanity-check
    let code = codegen::generate_pg(&version, "testpg");
    assert!(!code.is_empty());
    assert!(code.contains("fn main()"));
    assert!(code.contains("field_auto"), "generated code missing field_auto for bigserial");
    assert!(code.contains("foreign_key"), "generated code missing foreign_key call");
    assert!(code.contains("unique_index"), "generated code missing unique_index call");
    assert!(code.contains("\"testpg\""), "generated code missing db_name");
}
