# GOOD-ORMNING

- On [crates.io](https://crates.io/crates/good-ormning)
- On [docs.rs](https://docs.rs/good-ormning)

Good-ormning is lightweight end-to-end database management with full static type checking! Do all your development in Rust (no live test database), and know that it'll work in production always.

Dynamic queries are not currently supported. If you want to assemble a query programmatically you can run it against your database connection directly.

## Example

Create this `build.rs` file:

```rust,ignore
use good_ormning::sqlite::{
    Version,
    schema::field::*,
    generate,
    GenerateArgs,
};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let latest_version = Version::new();
    let users = latest_version.table("users");
    users.rowid_field(None);
    users.field("name", field_str().build());
    users.field("points", field_i64().build());
    generate(GenerateArgs {
        versions: vec![
            // Versions
            (1usize, latest_version.build())
        ],
        ..Default::default()
    }).unwrap();
}
```

`generate` will save the version type info in `$OUT_DIR` for use in the proc macros, and generates code to perform database migrations.

Use the database with:

```rust,ignore
use good_ormning::good_module;
use good_ormning::sqlite::good_query;

fn main() {
    good_module!(dbm);

    let mut db = rusqlite::Connection::open_in_memory().unwrap();
    dbm::migrate(&mut db, None).unwrap();

    good_query!("insert into users (name, points) values ($name, $points)"; dbm::Db(&mut db), name: string = "rust human", points: i64 = 0).unwrap();

    let users = good_ormning::sqlite::good_query_many!("select name, points from users"; dbm::Db(&mut db)).unwrap();
    for user in users {
        println!("User: {}, Points: {}", user.name, user.points);
    }
}
```

`migrate`'s second parameter is a callback which is called after each migration, so you can run custom fixup code in migrations.

```markdown
User: rust human, Points: 0
```

### Supported databases

- PostgreSQL (feature `pg`) via `tokio-postgres`
- Sqlite (feature `sqlite`) via `rusqlite`

I think these are both mostly implemented, but if there's a missing language feature you need let me know and I'll try to prioritize it!

## Getting started

### First time

1. You'll need the following runtime dependencies:
   - `good-ormning`
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
4. In your runtime code, call `good_module!(dbm)` to include the generated code.
5. After creating a database connection, call `dbm::migrate(&mut db, None)`
6. Make queries using `good_query!()`.

### Schema changes

1. Copy your previous version schema, leaving the old schema version untouched. Modify the new schema as you wish.
2. Pass both the old and new schema versions to `goodormning::generate()`, which will generate the new migration statements.
3. At runtime, the `migrate` call will make sure the database is updated to the new schema version.

You can get rid of old schema versions once you know there are no existing databases running that version.

## Usage details

### `good_query` macros

These macros are used to execute type-checked queries against the database.

They have the format `good_query_SUFFIX!([DBNAME: string,] [VERSION: usize,] SQL: string, CONN, (PARAM: TYPE = VALUE,)...)`

- `SUFFIX` - This determines the return type.
  - No suffix, no return

  - `_one` - Query will always return one row, or an error

  - `_maybe` - Query will return one or zero rows, or an error. Returns `Option<>`

  - `_many` - Query will return any number of results. Returns `Vec<>`

- `DBNAME` - Optional. If you provided a name in `build.rs` to `generate`, use the same name here. For when you have multiple databases.

- `VERSION` - Optional. Which schema version to execute the query against. You should only need this when running migration post-version code in the callback in `migrate()`.

- `SQL` - The literal SQL query you want to execute. This will be parsed and used to do type checking and return type generation.

- `CONN` - The database connection

- `PARAM: TYPE = VALUE` - The parameter values and their types (because the proc macro doesn't receive type information...).

  `TYPE` takes the format `[arr] [opt] type`. `type` can be any custom type name, or:
  - `i16`, `i32`, `i64`, `u32`, `f32`, `f64`
  - `bool`
  - `string`
  - `bytes`
  - `utctime_s_chrono`, `utctime_ms_chrono`
  - `utctime_s_jiff`, `utctime_ms_jiff`
  - `auto`

Parameters can also be provided inline in the SQL string using `${type = value}` syntax.

Example:

```rust,ignore
good_query!("insert into users (name, points) values (${string = \"rust human\"}, ${i64 = 0})"; dbm::Db(&mut db)).unwrap();
```

### Features

- `pg` - enables generating code for PostgreSQL
- `sqlite` - enables generating code for Sqlite
- `chrono` - enable datetime field/expression types

## A few words on the future

Obviously writing an SQL VM isn't great. The ideal solution would be for popular databases to expose their type checking routines as libraries so they could be imported into external programs, like how Go publishes reusable ast-parsing and type-checking libraries.
