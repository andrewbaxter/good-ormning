use {
    good_ormning::import::{
        codegen,
        sqlite,
    },
    good_ormning_core::sqlite::schema::constraint::ConstraintType,
};

/// Check that the generated build.rs parses, and that every `foreign_key` call
/// comes after every `let` binding (foreign keys may reference fields of tables
/// emitted later).
fn check_generated(code: &str) {
    syn::parse_file(code).unwrap_or_else(|e| panic!("Generated code doesn't parse: {}\n{}", e, code));
    let last_let = code.rfind("let ").unwrap();
    let first_fk = code.find("foreign_key(").unwrap();
    assert!(first_fk > last_let, "foreign_key emitted before all bindings were defined:\n{}", code);
}

#[test]
fn test_import_sqlite() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    conn.execute_batch(
        //# genemichaels-external: sql-formatter-sqlite
        r#"CREATE TABLE users (
             id INTEGER PRIMARY KEY,
             name TEXT NOT NULL,
             email TEXT NOT NULL UNIQUE,
             score REAL,
             data BLOB
           );

           CREATE TABLE posts (
             post_id INTEGER PRIMARY KEY,
             user_id INTEGER NOT NULL,
             title TEXT NOT NULL,
             FOREIGN KEY (user_id) REFERENCES users (id)
           );

           CREATE TABLE countries (
             code TEXT PRIMARY KEY,
             name TEXT NOT NULL
           );

           CREATE TABLE cities (
             id INTEGER PRIMARY KEY,
             country TEXT NOT NULL REFERENCES Countries,
             name TEXT NOT NULL,
             FOREIGN KEY (COUNTRY) REFERENCES countries (CODE)
           );

           CREATE TABLE __good_migration_state (
             x INTEGER
           );

           CREATE TABLE abcgoodxtable (
             x INTEGER
           );

           CREATE INDEX idx_posts_user ON posts (user_id);

           CREATE UNIQUE INDEX idx_posts_title ON posts (title);

           CREATE INDEX idx_posts_partial ON posts (title)
           WHERE
             user_id > 0;

           CREATE INDEX idx_posts_expr ON posts (lower(title));
           "#,
    ).unwrap();
    let version = sqlite::read_schema(&conn).unwrap();

    // Verify tables
    assert!(version.tables.contains_key("users"), "missing users table");
    assert!(version.tables.contains_key("posts"), "missing posts table");
    assert!(!version.tables.contains_key("__good_migration_state"), "internal table should be excluded");
    assert!(version.tables.contains_key("abcgoodxtable"), "LIKE wildcard excluded a non-internal table");

    // Users: id is INTEGER PRIMARY KEY → rowid alias stored as field_id "rowid"
    let users = &version.tables["users"];
    assert!(users.fields.contains_key("rowid"), "users missing rowid field");
    assert!(users.fields.contains_key("name"), "users missing name field");
    assert!(users.fields.contains_key("score"), "users missing score field");
    assert!(users.fields.contains_key("data"), "users missing data field");

    // PK constraint must be present for rowid alias
    let has_pk = users.constraints.values().any(|c| matches!(&c.type_, ConstraintType::PrimaryKey(_)));
    assert!(has_pk, "users missing PK constraint");

    // UNIQUE constraint imported as a unique index with a recreatable name
    let email_idx = &users.indices["users_email_unique"];
    assert!(email_idx.unique);
    assert_eq!(email_idx.fields, vec!["email".to_string()]);
    assert!(!users.indices.keys().any(|k| k.starts_with("sqlite_autoindex")));

    // Posts: FK + indexes
    let posts = &version.tables["posts"];
    let fk = posts.constraints.values().find_map(|c| match &c.type_ {
        ConstraintType::ForeignKey(fk) => Some(fk),
        _ => None,
    }).expect("posts missing FK constraint");
    assert_eq!(fk.remote_table, "users");
    assert_eq!(fk.fields, vec![("user_id".to_string(), "rowid".to_string())]);
    assert!(posts.indices.contains_key("idx_posts_user"));
    assert!(posts.indices["idx_posts_title"].unique, "posts missing unique index");
    assert!(!posts.indices.contains_key("idx_posts_partial"), "partial index should be skipped");
    assert!(!posts.indices.contains_key("idx_posts_expr"), "expression index should be skipped");

    // Countries: single-column non-INTEGER primary key
    let countries = &version.tables["countries"];
    assert!(countries.fields.contains_key("code"));
    assert!(!countries.fields.contains_key("rowid"));
    let pk = countries.constraints.values().find_map(|c| match &c.type_ {
        ConstraintType::PrimaryKey(pk) => Some(pk),
        _ => None,
    }).expect("countries missing PK constraint");
    assert_eq!(pk.fields, vec!["code".to_string()]);

    // Cities: FK without column list resolves to the parent PK, and FK names match
    // case-insensitively.
    let cities = &version.tables["cities"];
    let fks: Vec<_> = cities.constraints.values().filter_map(|c| match &c.type_ {
        ConstraintType::ForeignKey(fk) => Some(fk),
        _ => None,
    }).collect();
    assert_eq!(fks.len(), 2);
    for fk in fks {
        assert_eq!(fk.remote_table, "countries");
        assert_eq!(fk.fields, vec![("country".to_string(), "code".to_string())]);
    }

    // Generate code and do sanity checks
    let code = codegen::generate_sqlite(&version, "testdb").unwrap();
    check_generated(&code);
    assert!(code.contains("fn main()"));
    assert!(code.contains("rowid_field"), "generated code missing rowid_field call");
    assert!(code.contains("foreign_key"), "generated code missing foreign_key call");
    assert!(code.contains("unique_index"), "generated code missing unique_index call");
    assert!(code.contains("\"testdb\""), "generated code missing db_name");
}

#[test]
fn test_import_sqlite_errors() {
    // Unknown column type
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    conn.execute_batch("CREATE TABLE t (x WEIRDTYPE)").unwrap();
    let err = sqlite::read_schema(&conn).err().expect("unknown type should fail").to_string();
    assert!(err.contains("\"x\"") && err.contains("\"t\"") && err.contains("WEIRDTYPE"), "{}", err);

    // Real column named rowid alongside a rowid alias
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    conn.execute_batch("CREATE TABLE t (id INTEGER PRIMARY KEY, rowid TEXT)").unwrap();
    let err = sqlite::read_schema(&conn).err().expect("column named rowid should fail").to_string();
    assert!(err.contains("rowid") && err.contains("\"t\""), "{}", err);
}

#[test]
fn test_codegen_sqlite_ident_collisions() {
    // Table `a` / field `b_c` and table `a_b` / field `c` would both sanitize to the
    // same binding name; also table `a_b` collides with table `a` field `b`.
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    conn.execute_batch(
        //# genemichaels-external: sql-formatter-sqlite
        r#"CREATE TABLE a (
             b_c INTEGER NOT NULL,
             b INTEGER NOT NULL
           );

           CREATE TABLE a_b (
             c INTEGER NOT NULL,
             FOREIGN KEY (c) REFERENCES a (b_c)
           );

           CREATE TABLE "weird-name" (
             "odd col" INTEGER NOT NULL
           );
           "#,
    ).unwrap();
    let version = sqlite::read_schema(&conn).unwrap();
    let code = codegen::generate_sqlite(&version, "testdb").unwrap();
    check_generated(&code);
    let file = syn::parse_file(&code).unwrap();
    let main = file.items.iter().find_map(|i| match i {
        syn::Item::Fn(f) => Some(f),
        _ => None,
    }).unwrap();
    let mut bound: Vec<String> = vec![];
    for stmt in &main.block.stmts {
        if let syn::Stmt::Local(local) = stmt {
            if let syn::Pat::Ident(p) = &local.pat {
                let name = p.ident.to_string();
                assert!(!bound.contains(&name), "binding {:?} defined twice:\n{}", name, code);
                bound.push(name);
            }
        }
    }

    // v + 3 tables + 4 fields
    assert_eq!(bound.len(), 8, "{:?}", bound);
}
