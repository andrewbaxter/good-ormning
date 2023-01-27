# GOOD-ORMNING

Good-ormning is an ORM, probably? In a nutshell:

1. Define schemas and queries in `build.rs`
2. Good-ormning generates a function to set up/migrate the database
3. Good-ormning generates functions for each query

### Features

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

See Comparisons, below, for information on how Good-ormning differs from other Rust ORMs.

### Current status

Alpha:

- Basic features work
- Moderate test coverage
- Missing advanced features
- Some ergonomics issues, interfaces may change in upcoming releases

### Supported databases

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
4. In your code, after creating a database connection, call `initialize`

### Schema changes

1. Copy your previous version schema, leaving the old schema version untouched. Modify the new schema and queries as you wish.
2. Pass both the old and new schema versions to `goodormning::generate()`, which will generate the new migration statements.
3. At runtime, call `migrate` when you're ready to perform the migration (ex: after upgrading all servers to support the new schema)

## Example

This `build.rs` file

```rust
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let mut latest_version = Version::default();
    let users = latest_version.table("zQLEK3CT0", "users");
    let id = users.rowid_field(&mut latest_version, None);
    let name = users.field(&mut latest_version, "zLQI9HQUQ", "name", field_str().build());
    let points = users.field(&mut latest_version, "zLAPH3H29", "points", field_i64().build());
    goodormning::sqlite::generate(&root.join("tests/sqlite_gen_hello_world.rs"), vec![
        // Versions
        (0, latest_version)
    ], vec![
        // Queries
        new_insert(&users, vec![(name.clone(), Expr::Param {
            name: "name".into(),
            type_: name.type_.type_.clone(),
        }), (points.clone(), Expr::Param {
            name: "points".into(),
            type_: points.type_.type_.clone(),
        })]).build_query("create_user", QueryResCount::None),
        new_select(&users).where_(Expr::BinOp {
            left: Box::new(Expr::Field(id.clone())),
            op: BinOp::Equals,
            right: Box::new(Expr::Param {
                name: "id".into(),
                type_: id.type_.type_.clone(),
            }),
        }).return_fields(&[&name, &points]).build_query("get_user", QueryResCount::One),
        new_select(&users).return_field(&id).build_query("list_users", QueryResCount::Many)
    ]).unwrap();
}
```

Generates this code <integration_tests/tests/hello_world.rs> and can be used like:

```rust
fn main() {
    use hello_world as queries;

    let mut db = rusqlite::Connection::open_in_memory().unwrap();
    queries::initialize(&mut db).unwrap();
    queries::create_user(&mut db, "rust human", 0).unwrap();
    for user_id in queries::list_users(&mut db).unwrap() {
        let user = queries::get_user(&mut db, user_id).unwrap();
        println!("User {}: {}", user_id, user.name);
    }
    Ok(())
}
```

```
User 1: rust human
```

## Usage details

### `initialize` and `migrate`

The normal way to use these is

1. Call `initialize` every time the server starts
2. Trigger `migrate` after server upgrades, but not automatically

`initialize` sets up the latest database version in new environments but does nothing otherwise, so it's safe to run at every startup.

`migrate` performs incremental migrations from any intermediate version to the latest version. While you can use this instead of (rather than in addition to) `initialize` in single-server environments, if you're doing migrations with a fleet of servers you'd do this after all the servers have been updated to backwards-compatibly support the new schema. Once all the servers are upgraded you trigger `migrate` and the servers will start using new TODO

### Schema IDs and IDs

IDs are used both in SQL and Rust, so must be valid in both (however, some munging is applied to ids in Rust if they clash with keywords). Depending on the database, you can change IDs arbitrarily between schema versions but swapping IDs in consecutive versions isn't currently supported - if you need to do swaps do it over three different versions (like `v0`: `A` and `B`, `v1`: `A_` and `B`, `v2`: `B` and `A`).

Schema IDs are internal ids used for matching fields across versions, to identify renames, deletes, etc. Schema IDs must not change once used in a version. I recommend using randomly generated IDs, via a macro.

### Types and queries

Use `type_*` `field_*` functions to get expression/field type builders. Use `new_insert/select/update/delete` to get a query builder for the associated query type.

### Custom types

When defining a field in the schema, call `.custom("mycrate::MyString", type_str().build())` on the field type builder (or pass it in as `Some("mycreate::MyType".to_string())` if creating the type structure directly).

Custom types need to implement functions like this:

```rust
pub struct MyString(pub String);

impl MyString {
    pub fn to_sql(&self) -> &str {
        &self.0
    }

    pub fn from_sql(s: String) -> Result<Self, MyErr> {
        Ok(Self(s))
    }
}
```

Any `std::err::Error` can be used for the error. The `to_sql` result and `from_sql` arguments should correspond to the base type you specified. If you're not sure what type that is, guess, and when you compile you'll get an compiler error saying which type you need.

## Comparisons

### Vs Diesel

Good-ormning is functionally most similar to Diesel.

#### Diesel

- You can define your queries and result structures near where you use them
- You can dynamically define queries (i.e. swap operators depending on the input, etc.)
- Result structures must be manually defined, and care must be taken to get the field order to match the query
- You can define new types to use in the schema, which are checked against queries, although this requires significant boilerplate
- Requires many macros, trait implementations
- To synchronize your migrations and in-code schema, you can use the CLI with a live database with migrations applied. However, this resets any custom SQL types in the schema with the built-in SQL types. Alternatively you can maintain the schema by hand (and risk query issues due to typos, mismatches).
- Column count limitations, slow build times
- Supports more syntax, withstood test of time

#### Good-ormning

- Queries have to be defined separately, in the `build.rs` file
- All queries have to be defined up front in `build.rs`
- You don't have to write any structures, everything is generated from schema and query info
- Custom types can be incorporated into the schema with no boilerplate
- Migrations are automatically derived via a diff between schema versions plus additional migration metadata
- Clear error messages, thanks to no macros, generics, traits
- Code generation is fast, compiling the simple generated code is also fast
- Alpha

### Vs SQLx

#### SQLx

- SQLx has no concept of a schema so it can only perform type-checking on native SQL types (no consideration for new types, blob encodings, etc)
- Requires a running database during development

#### Good-ormning

- The same schema used for generating migrations is used for type checking, and natively supports custom types
- A live database is unused during development, but all query syntax must be manually implemented in Good-ormning so you may encounter missing features

### Vs SeaORM

SeaORM focuses on runtime checks rather than compile time checks, so the focus is quite different.

## A few words on the future

Obviously writing an SQL VM isn't great. The ideal solution would be for popular databases to expose their type checking routines as libraries so they could be imported into external programs, like how Go publishes reusable ast-parsing and type-checking libraries.

It would be great to provider more flexibility in migrations, but for downtime-less migrations with complex migrations the code also needs to be adjusted significantly. Common advice appears to be to make smaller, incremental, backward-compatible migrations and make larger changes over multiple versions and deploys, which seems a reasonable solution.
