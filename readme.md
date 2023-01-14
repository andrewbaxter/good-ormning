# GOOD-ORMNING

Good-ormning is an ORM, probably?

You

1. Define schemas and queries in `build.rs`
2. `good-ormning` generates a migration function and functions for each queries

**Features**

- No macros
- No generics
- No traits
- No boilerplate duplicating stuff in the schema
- Automatic migrations, no migration-schema mismatches
- Query parameter type checking - no runtime errors due to parameter types, counts, or ordering
- Query logic type checking via a query simulation
- Query result type checking - no runtime errors due to result types, counts, or ordering
- Fast to generate, low runtime overhead

Like other Rust ORMs, Good-ormning doesn't abstract away from actual database workflows, but instead aims to enhance type checking with normal SQL.

See Comparisons, below, for information on how Good-Ormning differs from other Rust ORMs.

**Current status**

Alpha:

- Basic features work
- Incomplete test coverage
- Missing advanced features
- Some ergonomics issues, interfaces may change in upcoming releases

**Supported databases**:

- PostgreSQL
- Sqlite

## Getting started

### First time

1. You'll need the following runtime dependencies:

   - `tokio-postgres` for PostgreSQL
   - `rusqlite` for Sqlite
   - `hex_literal` if you use byte array literals in any queries

   And `build.rs` dependencies:

   - `good-ormning`

2. Create a `build.rs` and define your initial schema version and queries
3. Call `goodormning::generate()` to output the generated code
4. In your code, after creating a database connection, call `migrate`

### Schema changes

1. Copy your previous version schema, leaving the old schema version untouched. Modify the new schema and queries as you wish.
2. Pass both the old and new schema versions to `goodormning::generate()`, which will generate the new migration statements.
3. At runtime, the `migrate` call will make sure the database is updated to the new schema version.

## Example

This `build.rs` file

```rust
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let mut latest_version = Version::default();
    let users = latest_version.table("zQLEK3CT0");
    let id = users.field(&mut latest_version, "zLAPH3H29", "id", field_i64().auto_increment().build());
    let name = users.field(&mut latest_version, "zLQI9HQUQ", "name", field_str().build());
    goodormning::sqlite::generate(&root.join("tests/sqlite_gen_hello_world.rs"), vec![
        // Versions
        (0usize, latest_version)
    ], vec![
        // Queries
        new_insert(&users, vec![(name.id.clone(), Expr::Param {
            name: "text".into(),
            type_: name.def.type_.type_.clone(),
        })]).build_query("create_user", QueryResCount::None),
        new_select(&users).where_(Expr::BinOp {
            left: Box::new(Expr::Field(id.id.clone())),
            op: BinOp::Equals,
            right: Box::new(Expr::Param {
                name: "id".into(),
                type_: id.def.type_.type_.clone(),
            }),
        }).output_fields(&[&id, &name]).build_query("get_user", QueryResCount::One),
        new_select(&users).output_field(&id).build_query("list_users", QueryResCount::Many)
    ]).unwrap();
}
```

Generates this code

```rust

```

And can be used like

```rust
fn main() {
    use sqlite_gen_hello_world as queries;

    let mut db = rusqlite::Connection::open_in_memory()?;
    queries::migrate(&mut db)?;
    queries::new_user(&mut db, "rust human")?;
    for user_id in queries::list_users(&mut db)? {
        let user = queries::get_user(&mut db, user_id)?;
        println!("User {}: {}", user_id, user);
    }
    Ok(())
}
```

```

```

## Advanced usage

### IDs and Names

In general in this library, IDs are SQL table/field/index/constrait/etc ids. Names are what's used in generated Rust functions and structs.

IDs must be stable. Migrations are based around stable ids, so if (for example) a table ID changes, this will be considered a delete of the table with the old id, and a create of a new table with the new id.

In the example above, I used randomly generated IDs which have this property. This has the downside that it makes the SQL CLI harder to use. It's possible this will be improved upon, but if you frequently need to do things from the CLI I suggest creating a custom CLI using generated queries.

### Custom types

When defining a field in the schema, call `.custom("mycrate::MyType")` on the field type builder (or pass it in as `Some("mycreate::MyType".to_string())` if creating the type structure directly).

Custom types need to implement

1. `trait Into<T>` for the corresponding standard type (`i32`, `String`, etc.), if you use the type as a parameter in a query.
2. `fn from_sql(T) -> Result<U, String>` for types used in query results, where `T` is the corresponding standard type and `U` is the custom type. This isn't part of a trait.

If you miss either you'll get a compile error that should indicate clearly what you need to do.

## Comparisons

### Vs Diesel

Good-Ormning is functionally most similar to Diesel.

#### Diesel

- You can define your queries and result structures near where you use them
- You can define new types to use in the schema, which are checked against queries, although this requires significant boilerplate
- Requires many macros, trait implementations
- To synchronize your migrations and in-code schema, you can use the CLI with a live database with migrations applied. However, this resets any custom SQL types in the schema with the built-in SQL types. Alternatively you can maintain the schema by hand (and risk query issues due to typos, mismatches).

#### Good-Ormning

- Queries have to be defined separately, in the `build.rs` file
- You don't have to write any structures, everything is generated from schema and query info
- Custom types can be incorporated into the schema with no boilerplate
- Migrations are automatically derived via a diff between schema versions plus additional migration metadata
- Clear error messages, thanks to no macros, generics, traits
- Code generation is fast, compiling the simple generated code is also fast

### Vs SQLx

#### SQLx

- SQLx has no concept of a schema so it can only perform type-checking on native SQL types (no consideration for new types, blob encodings, etc)
- Requires a running database during all development

#### Good-Ormning

- The same schema used for generating migrations is used for type checking, and natively supports custom types
- A live database is unused during development, but all query syntax must be manually implemented in Good-Ormning so you may encounter missing features

### Vs SeaORM

SeaORM focuses on runtime checks rather than compile time checks, so the focus is quite different.

## A few words on the future

Obviously writing an SQL VM isn't great. The ideal solution would be for popular databases to expose their type checking routines as libraries so they could be imported into external programs, like how Go publishes reusable ast-parsing and type-checking libraries.
