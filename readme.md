# GOOD-ORMNING

- On [crates.io](https://crates.io/crates/good-ormning)
- On [docs.rs](https://docs.rs/good-ormning)

Good-ormning is an ORM, probably? In a nutshell:

1. Define schemas and queries in `build.rs`
2. Good-ormning generates a function to set up/migrate the database
3. Good-ormning generates functions for each query

### Features

- No macros
- No generics
- No traits (okay, simple traits for custom types to help guide implementations _only_)
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

- PostgreSQL (feature `pg`)
- Sqlite (feature `sqlite`)

## Getting started

### First time

1. You'll need the following runtime dependencies:

   - `good-ormning-traits` if you use any custom types (non-plain types)
   - `tokio-postgres` for PostgreSQL
   - `rusqlite` for Sqlite

   And `build.rs` dependencies:

   - `good-ormning`

   And you _must_ enable one (or more) of the database features:

   - `pg`
   - `sqlite`

   plus maybe `chrono` for `DateTime` support.

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
    let users = latest_version.table("zQLEK3CT0", "users");
    let id = users.rowid_field(&mut latest_version, None);
    let name = users.field(&mut latest_version, "zLQI9HQUQ", "name", field_str().build());
    let points = users.field(&mut latest_version, "zLAPH3H29", "points", field_i64().build());
    goodormning::sqlite::generate(&root.join("tests/sqlite_gen_hello_world.rs"), vec![
        // Versions
        (0usize, latest_version)
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

Generates this code

```rust
#[derive(Debug)]
pub struct GoodError(pub String);

impl std::fmt::Display for GoodError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for GoodError { }

impl From<rusqlite::Error> for GoodError {
    fn from(value: rusqlite::Error) -> Self {
        GoodError(value.to_string())
    }
}

pub fn migrate(db: &mut rusqlite::Connection) -> Result<(), GoodError> {
    db.execute(
        "create table if not exists __good_version (rid int primary key, version bigint not null, lock int not null);",
        (),
    )?;
    db.execute("insert into __good_version (rid, version, lock) values (0, -1, 0) on conflict do nothing;", ())?;
    loop {
        let txn = db.transaction()?;
        match (|| {
            let mut stmt =
                txn.prepare("update __good_version set lock = 1 where rid = 0 and lock = 0 returning version")?;
            let mut rows = stmt.query(())?;
            let version = match rows.next()? {
                Some(r) => {
                    let ver: i64 = r.get(0usize)?;
                    ver
                },
                None => return Ok(false),
            };
            drop(rows);
            stmt.finalize()?;
            if version > 0i64 {
                return Err(
                    GoodError(
                        format!(
                            "The latest known version is {}, but the schema is at unknown version {}",
                            0i64,
                            version
                        ),
                    ),
                );
            }
            if version < 0i64 {
                txn.execute("create table \"users\" ( \"name\" text not null , \"points\" integer not null )", ())?;
            }
            txn.execute("update __good_version set version = $1, lock = 0", rusqlite::params![0i64])?;
            let out: Result<bool, GoodError> = Ok(true);
            out
        })() {
            Err(e) => {
                match txn.rollback() {
                    Err(e1) => {
                        return Err(
                            GoodError(
                                format!("{}\n\nRolling back the transaction due to the above also failed: {}", e, e1),
                            ),
                        );
                    },
                    Ok(_) => {
                        return Err(e);
                    },
                };
            },
            Ok(migrated) => {
                match txn.commit() {
                    Err(e) => {
                        return Err(GoodError(format!("Error committing the migration transaction: {}", e)));
                    },
                    Ok(_) => {
                        if migrated {
                            return Ok(())
                        } else {
                            std::thread::sleep(std::time::Duration::from_millis(5 * 1000));
                        }
                    },
                };
            },
        }
    }
}

pub fn create_user(db: &mut rusqlite::Connection, name: &str, points: i64) -> Result<(), GoodError> {
    db
        .execute("insert into \"users\" ( \"name\" , \"points\" ) values ( $1 , $2 )", rusqlite::params![name, points])
        .map_err(|e| GoodError(e.to_string()))?;
    Ok(())
}

pub struct DbRes1 {
    pub name: String,
    pub points: i64,
}

pub fn get_user(db: &mut rusqlite::Connection, id: i64) -> Result<DbRes1, GoodError> {
    let mut stmt =
        db.prepare(
            "select \"users\" . \"name\" , \"users\" . \"points\" from \"users\" where ( \"users\" . \"rowid\" = $1 )",
        )?;
    let mut rows = stmt.query(rusqlite::params![id]).map_err(|e| GoodError(e.to_string()))?;
    let r = rows.next()?.ok_or_else(|| GoodError("Query expected to return one row but returned no rows".into()))?;
    Ok(DbRes1 {
        name: {
            let x: String = r.get(0usize)?;
            x
        },
        points: {
            let x: i64 = r.get(1usize)?;
            x
        },
    })
}

pub fn list_users(db: &mut rusqlite::Connection) -> Result<Vec<i64>, GoodError> {
    let mut out = vec![];
    let mut stmt = db.prepare("select \"users\" . \"rowid\" from \"users\"")?;
    let mut rows = stmt.query(rusqlite::params![]).map_err(|e| GoodError(e.to_string()))?;
    while let Some(r) = rows.next()? {
        out.push({
            let x: i64 = r.get(0usize)?;
            x
        });
    }
    Ok(out)
}

```

And can be used like

```rust
fn main() {
    use sqlite_gen_hello_world as queries;

    let mut db = rusqlite::Connection::open_in_memory().unwrap();
    queries::migrate(&db).unwrap();
    queries::create_user(&db, "rust human", 0).unwrap();
    for user_id in queries::list_users(&db).unwrap() {
        let user = queries::get_user(&db, user_id).unwrap();
        println!("User {}: {}", user_id, user.name);
    }
    Ok(())
}
```

```
User 1: rust human
```

## Usage details

### Features

- `pg` - enables generating code for PostgreSQL
- `sqlite` - enables generating code for Sqlite
- `chrono` - enable datetime field/expression types

### Schema IDs and IDs

"Schema IDs" are internal ids used for matching fields across versions, to identify renames, deletes, etc. Schema IDs must not change once used in a version. I recommend using randomly generated IDs, via a macro. Changing Schema IDs is treated like a delete followed by a create.

"IDs" are used both in SQL (for fields) and Rust (in parameters and returned data structures), so must be valid in both (however, some munging is automatically applied to ids in Rust if they clash with keywords). Depending on the database, you can change IDs arbitrarily between schema versions but swapping IDs in consecutive versions isn't currently supported - if you need to do swaps do it over three different versions (like `v0`: `A` and `B`, `v1`: `A_` and `B`, `v2`: `B` and `A`).

### Query, expression and fields types

Use `type_*` `field_*` functions to get type builders for use in expressions/fields. Use `new_insert/select/update/delete` to get a query builder for the associated query type.

There are also some helper functions for building queries, see

- `field_param`, a shortcut for a parameter matching the type and name of a field
- `set_field`, a shortcut for setting field values in INSERT and UPDATE
- `eq_field`, `gt_field`, `gte_field`, `lt_field`, `lte_field` are shortcuts for expressions comparing a field and a parameter with the same type
- `expr_and`, a shortcut for AND expressions

for the database you're using.

### Custom types

When defining a field in the schema, call `.custom("mycrate::MyString", type_str().build())` on the field type builder (or pass it in as `Some("mycreate::MyType".to_string())` if creating the type structure directly).

The type must have methods to convert to/from the native SQL types. There are traits to guide the implementation:

```rust
pub struct MyString(pub String);

impl pg::GoodOrmningCustomString<MyString> for MyString {
    fn to_sql(value: &MyString) -> &str {
        &value.0
    }

    fn from_sql(s: String) -> Result<MyString, String> {
        Ok(Self(s))
    }
}
```

### Parameters and return types

Parameters with the same name are deduplicated - if you define a query with multiple parameters of the same name but different types you'll get an error.

Return types with the same contents are similarly deduplicated (methods to make two queries that return the same fields will return the same type).

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
