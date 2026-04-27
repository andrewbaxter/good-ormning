# GOOD-ORMNING

- On [crates.io](https://crates.io/crates/good-ormning)
- On [docs.rs](https://docs.rs/good-ormning)

Good-ormning is light weight end to end database management with full static type checking! Do all your development in Rust (no live test database), and know that it'll work in production the first time.

Here's how it works:

1. You use `build.rs` to define your database versions. If you want to make changes, copy the last version and make changes to it. Call `generate` with all your versions. (This generates code to perform database setup and migrations, and saves schema types for query type checking.)
2. Use `good_module!(dbm);` to create a `dbm` module containing the generated code.
3. Use `good_query!("select * from mytable where x = $1;", dbm::DbDefault1(&mut db), p1: i32 = 125)` to make queries. `good_query` will return one struct, an `Option<>`, or a list of structs for the query.

SQL dialect support is ongoing - if there's a language feature you need let me know and I'll try to prioritize it!

Dynamic queries are not currently supported. If you want to assemble a query programmatically you can run it against your database connection directly.

### Supported databases

- PostgreSQL (feature `pg`) via `tokio-postgres`
- Sqlite (feature `sqlite`) via `rusqlite`

## Getting started

### First time

1. You'll need the following runtime dependencies:
   - `tokio-postgres` for PostgreSQL
   - `rusqlite` for Sqlite

   And `build.rs` dependencies:
   - `good-ormning`

   And you _must_ enable one (or more) of the database features:
   - `pg`
   - `sqlite`

   plus maybe `chrono` or `jiff` for `DateTime` support.

2. Create a `build.rs` and define your initial schema version using `Version::new()`.
3. Call `goodormning::generate()` to output the generated code
4. In your code, call `good_module!(dbm)` to include the generated code.
5. After creating a database connection, call `dbm::migrate(&mut db, None)`
6. Make queries using `good_query!()`.

### Schema changes

1. Copy your previous version schema, leaving the old schema version untouched. Modify the new schema as you wish.
2. Pass both the old and new schema versions to `goodormning::generate()`, which will generate the new migration statements.
3. At runtime, the `migrate` call will make sure the database is updated to the new schema version.

You can get rid of old schema versions once you know there are no existing databases running that version.

## Example

This `build.rs` file

```rust
use good_ormning::sqlite::{
    Version,
    schema::field::*,
    generate,
};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let latest_version = Version::new();
    let users = latest_version.table("users");
    users.rowid_field(None);
    users.field("name", field_str().build());
    users.field("points", field_i64().build());
    generate(None, vec![
        // Versions
        (1usize, latest_version.build())
    ]).unwrap();
}
```

Can be used like:

```rust,ignore
use good_ormning::good_module;
use good_ormning::sqlite::good_query;

fn main() {
    good_module!(dbm);

    let mut db = rusqlite::Connection::open_in_memory().unwrap();
    dbm::migrate(&mut db, None).unwrap();
    
    good_query!("insert into users (name, points) values ($1, $2)", dbm::DbDefault1(&mut db), p1: string = "rust human", p2: i64 = 0).unwrap();
    
    let users = good_ormning::sqlite::good_query_many!("select name, points from users", dbm::DbDefault1(&mut db)).unwrap();
    for user in users {
        println!("User: {}, Points: {}", user.name, user.points);
    }
}
```

```markdown
User: rust human, Points: 0
```

## Usage details

### Features

- `pg` - enables generating code for PostgreSQL
- `sqlite` - enables generating code for Sqlite
- `chrono` - enable datetime field/expression types

### Queries

There are 4 proc macros for each database engine: `good_query`, `good_query_one`, `good_query_opt`, `good_query_many`.

When defining a field in the schema, you can use `.custom("MyType")` to use a custom Rust type.

The type must have methods to convert to/from the native SQL types. There are traits to guide the implementation:

```rust
pub struct MyString(pub String);

use good_ormning::runtime::pg;
impl pg::GoodOrmningCustomString<MyString> for MyString {
    fn to_sql(value: &MyString) -> &str {
        &value.0
    }

    fn from_sql(s: String) -> Result<MyString, String> {
        Ok(Self(s))
    }
}
```

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

- Queries are defined near where they are used, via macros
- You don't have to write any structures, everything is generated from schema and query info
- Custom types can be incorporated into the schema with no boilerplate
- Migrations are automatically derived via a diff between schema versions plus additional migration metadata
- Clear error messages, thanks to no complex macros or deep generics in the user code
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

SeaORM focuses on runtime checks rather than compile time checks.

## A few words on the future

Obviously writing an SQL VM isn't great. The ideal solution would be for popular databases to expose their type checking routines as libraries so they could be imported into external programs, like how Go publishes reusable ast-parsing and type-checking libraries.
