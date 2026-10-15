use {
    good_ormning::import::{
        codegen,
        pg,
    },
    good_ormning_core::pg::{
        schema::constraint::ConstraintType,
        types::SimpleSimpleType,
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

/// Check that the generated build.rs parses, and that every `foreign_key` call
/// comes after every `let` binding (foreign keys may reference fields of tables
/// emitted later).
fn check_generated(code: &str) {
    syn::parse_file(code).unwrap_or_else(|e| panic!("Generated code doesn't parse: {}\n{}", e, code));
    let last_let = code.rfind("let ").unwrap();
    let first_fk = code.find("foreign_key(").unwrap();
    assert!(first_fk > last_let, "foreign_key emitted before all bindings were defined:\n{}", code);
}

#[tokio::test]
async fn test_import_pg() {
    let server = PgliteServer::temporary_tcp().unwrap();
    let client = new_client(&server).await;
    client.simple_query("SET search_path TO public").await.unwrap();
    for stmt in [
        //# genemichaels-external: sql-formatter-pg
        r#"CREATE TABLE users (
             id bigserial PRIMARY KEY,
             name text NOT NULL,
             email text NOT NULL UNIQUE,
             score double precision,
             active boolean NOT NULL DEFAULT true,
             data bytea
           )
           "#,
        //# genemichaels-external: sql-formatter-pg
        r#"CREATE TABLE posts (
             id bigserial PRIMARY KEY,
             user_id bigint NOT NULL REFERENCES users (id),
             seq serial NOT NULL,
             title text NOT NULL,
             body text
           )
           "#,
        //# genemichaels-external: sql-formatter-pg
        r#"CREATE TABLE __good_migration_state (
             x int
           )
           "#,
        //# genemichaels-external: sql-formatter-pg
        r#"CREATE TABLE abcgoodxtable (
             x int
           )
           "#,
        //# genemichaels-external: sql-formatter-pg
        r#"CREATE INDEX idx_posts_user ON posts (user_id)
           "#,
        //# genemichaels-external: sql-formatter-pg
        r#"CREATE UNIQUE INDEX idx_posts_title ON posts (title)
           "#,
        //# genemichaels-external: sql-formatter-pg
        r#"CREATE INDEX idx_posts_partial ON posts (title)
           WHERE
             body IS NOT NULL
           "#,
        //# genemichaels-external: sql-formatter-pg
        r#"CREATE INDEX idx_posts_expr ON posts (lower(title))
           "#,
    ] {
        client.simple_query(stmt).await.unwrap();
    }
    let version = pg::read_schema(&client).await.unwrap();

    // Verify tables
    assert!(version.tables.contains_key("users"), "missing users table");
    assert!(version.tables.contains_key("posts"), "missing posts table");
    assert!(!version.tables.contains_key("__good_migration_state"), "internal table should be excluded");
    assert!(version.tables.contains_key("abcgoodxtable"), "LIKE wildcard excluded a non-internal table");

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

    // UNIQUE constraint imported as unique index
    let email_idx =
        users
            .indices
            .values()
            .find(|i| i.fields == vec!["email".to_string()])
            .expect("users missing unique index for UNIQUE constraint on email");
    assert!(email_idx.unique);

    // Posts: FK and indexes
    let posts = &version.tables["posts"];
    let fk = posts.constraints.values().find_map(|c| match &c.type_ {
        ConstraintType::ForeignKey(fk) => Some(fk),
        _ => None,
    }).expect("posts missing FK constraint");
    assert_eq!(fk.remote_table, "users");
    assert_eq!(fk.fields, vec![("user_id".to_string(), "id".to_string())]);
    assert!(posts.indices.contains_key("idx_posts_user"));
    assert!(posts.indices["idx_posts_title"].unique, "posts missing unique index");
    assert!(!posts.indices.contains_key("idx_posts_partial"), "partial index should be skipped");
    assert!(!posts.indices.contains_key("idx_posts_expr"), "expression index should be skipped");
    assert_eq!(
        posts.fields["seq"].type_.type_.type_.type_,
        SimpleSimpleType::Auto,
        "serial field should be widened to Auto"
    );

    // Generate code and sanity-check
    let code = codegen::generate_pg(&version, "testpg").unwrap();
    check_generated(&code);
    assert!(code.contains("fn main()"));
    assert!(code.contains("field_auto"), "generated code missing field_auto for bigserial");
    assert!(code.contains("foreign_key"), "generated code missing foreign_key call");
    assert!(code.contains("unique_index"), "generated code missing unique_index call");
    assert!(code.contains("\"testpg\""), "generated code missing db_name");

    // Unsupported types produce an error naming the column
    client.simple_query("CREATE TABLE events (id bigserial PRIMARY KEY, at timestamptz NOT NULL)").await.unwrap();
    let err = pg::read_schema(&client).await.err().expect("timestamptz column should fail to import").to_string();
    assert!(err.contains("\"at\"") && err.contains("\"events\"") && err.contains("timestamptz"), "{}", err);
}
