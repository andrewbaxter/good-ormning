use {
    good_ormning::import::{
        codegen,
        sqlite,
    },
    good_ormning_core::sqlite::schema::constraint::ConstraintType,
};

#[test]
fn test_import_sqlite() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    conn.execute_batch(
        "
        CREATE TABLE users (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            score REAL,
            data BLOB
        );
        CREATE TABLE posts (
            post_id INTEGER PRIMARY KEY,
            user_id INTEGER NOT NULL,
            title TEXT NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id)
        );
        CREATE INDEX idx_posts_user ON posts(user_id);
        CREATE UNIQUE INDEX idx_posts_title ON posts(title);
    ",
    )
    .unwrap();

    let version = sqlite::read_schema(&conn).unwrap();

    // Verify tables
    assert!(version.tables.contains_key("users"), "missing users table");
    assert!(version.tables.contains_key("posts"), "missing posts table");

    // Users: id is INTEGER PRIMARY KEY → rowid alias stored as field_id "rowid"
    let users = &version.tables["users"];
    assert!(users.fields.contains_key("rowid"), "users missing rowid field");
    assert!(users.fields.contains_key("name"), "users missing name field");
    assert!(users.fields.contains_key("score"), "users missing score field");
    assert!(users.fields.contains_key("data"), "users missing data field");
    // PK constraint must be present for rowid alias
    let has_pk = users.constraints.values().any(|c| matches!(&c.type_, ConstraintType::PrimaryKey(_)));
    assert!(has_pk, "users missing PK constraint");

    // Posts: FK + indexes
    let posts = &version.tables["posts"];
    let has_fk = posts.constraints.values().any(|c| matches!(&c.type_, ConstraintType::ForeignKey(_)));
    assert!(has_fk, "posts missing FK constraint");
    assert!(!posts.indices.is_empty(), "posts missing indices");
    let has_unique_idx = posts.indices.values().any(|i| i.unique);
    assert!(has_unique_idx, "posts missing unique index");

    // Generate code and do sanity checks
    let code = codegen::generate_sqlite(&version, "testdb");
    assert!(!code.is_empty());
    assert!(code.contains("fn main()"));
    assert!(code.contains("rowid_field"), "generated code missing rowid_field call");
    assert!(code.contains("foreign_key"), "generated code missing foreign_key call");
    assert!(code.contains("unique_index"), "generated code missing unique_index call");
    assert!(code.contains("\"testdb\""), "generated code missing db_name");
}
