# GOOD-ORMNING

- On [crates.io](https://crates.io/crates/good-ormning)
- On [docs.rs](https://docs.rs/good-ormning)

Good-ormning is light weight end to end database management with full static type checking! Do all your development in Rust (no live test database), and know that it'll work in production the first time.

Here's how it works:

1. You use `build.rs` to define your database versions. If you want to make changes, copy the last version and make changes to it. Call `generate` with all your versions. (This generates code to perform database setup and migrations, and saves schema types for query type checking.)
2. Use `good_module!(pub db);` to create a `db` module containing the generated code.
3. Use `good_query!("select * from mytable where x = $y;"; y = i32; db, y = 125)` to make queries. `good_query` will return one struct, an `Option<>`, or a list of structs for the query.

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
4. In your code, after creating a database connection, call `migrate`
5. Make queries using `good_query!()`.

### Schema changes

1. Copy your previous version schema, leaving the old schema version untouched. Modify the new schema and queries as you wish.
2. Pass both the old and new schema versions to `goodormning::generate()`, which will generate the new migration statements.
3. At runtime, the `migrate` call will make sure the database is updated to the new schema version.

You can get rid of old schema versions once you know there are no existing databases running that version.

## Example

This `build.rs` file

```rust
use std::{
    path::PathBuf,
    env,
};
use good_ormning::sqlite::{
    Version,
    schema::{
        field::*,
        constraint::*,
    },
    query::{
        expr::*,
        select::*,
    },
    *
};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let latest_version = Version::new();
    let users = latest_version.table("users");
    let id_t = users.rowid_field(None).r#type();
    let name_t = field_str().build();
    let points_t = field_i64().build();
    let id = users.rowid_field(None);
    let name = users.field("name", name_t.clone());
    let points = users.field("points", points_t.clone());
    good_ormning::sqlite::generate(None, vec![
        // Versions
        (0usize, latest_version.build())
    ], vec![
        // Latest version queries
        new_insert(&users, vec![(name.clone(), Expr::Param {
            name: "name".into(),
            type_: name_t.type_.clone(),
        }), (points.clone(), Expr::Param {
            name: "points".into(),
            type_: points_t.type_.clone(),
        })]).build_query("create_user", QueryResCount::None),

        new_select(&users).where_(Expr::BinOp {
            left: Box::new(Expr::Field(id.to_ref())),
            op: BinOp::Equals,
            right: Box::new(Expr::Param {
                name: "id".into(),
                type_: id_t.type_.clone(),
            }),
        }).return_fields(&[&name, &points]).build_query("get_user", QueryResCount::One),

        new_select(&users).return_field(&id).build_query("list_users", QueryResCount::Many)
    ]).unwrap();
}
```

Generates something like:

```rust,ignore
pub fn migrate(db: &mut rusqlite::Connection) -> Result<(), GoodError> {
    // ...
}

pub fn create_user(db: &mut rusqlite::Connection, name: &str, points: i64) -> Result<(), GoodError> {
    // ...
}

pub struct DbRes1 {
    pub name: String,
    pub points: i64,
}

pub fn get_user(db: &mut rusqlite::Connection, id: i64) -> Result<DbRes1, GoodError> {
    // ...
}

pub fn list_users(db: &mut rusqlite::Connection) -> Result<Vec<i64>, GoodError> {
    // ...
}
```

And can be used like:

```rust,ignore
fn main() {
    mod queries {
        include!(concat!(env!("OUT_DIR"), "/good_ormning_sqlite_default.rs"));
    }

    let mut db = rusqlite::Connection::open_in_memory().unwrap();
    queries::migrate(&mut db).unwrap();
    queries::create_user(&mut db, "rust human", 0).unwrap();
    for user_id in queries::list_users(&mut db).unwrap() {
        let user = queries::get_user(&mut db, user_id).unwrap();
        println!("User {}: {}", user_id, user.name);
    }
    Ok(())
}
```

```markdown
User 1: rust human
```

## Usage details

### Features

- `pg` - enables generating code for PostgreSQL
- `sqlite` - enables generating code for Sqlite
- `chrono` - enable datetime field/expression types

### Queries

There are 4 proc macros,

When defining a field in the schema, call `.custom("mycrate::MyString", type_str().build())` on the field type builder (or pass it in as `Some("mycreate::MyType".to_string())` if creating the type structure directly).

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

### Methods

The `Expr::Call` variant allows you to create method call expressions. You must provide in `compute_type` a helper method to type-check the arguments and determine the type of the evaluation of the call.

The first parameter is the evaluation context, which contains `errs` for reporting errors. The second is a path from the evaluation tree root up to the call, for identifying where in a query expression errors occur. The third argument is a vec of arguments passed to the call. Each argument can be a single type or a record consisting of multiple types (like in `()` in `where (x, y, z) < (b.x, b.y, b.z)`). If there are no errors, this must return `Some(...)`.

Error handling is lazy during expression checking - even if an error occurs, processing can continue (and identify more errors before aborting). All errors are fatal, they just don't cause an abort immediately.

If there are errors, record the errors in `ctx.errs.err(path.add(format!("Argument 0")), format!("Error"))`. If evaluation within the call cannot continue, return `None`, otherwise continue.

### Parameters and return types

Parameters with the same name are deduplicated - if you define a query with multiple parameters of the same name but different types you'll get an error.

Different queries with the same multiple-field returns will use the same return type.

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
- Clear error messages, thanks to no macros, generics
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
