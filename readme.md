# GOOD-ORMNING

Good-ormning is an ORM, probably? In a nutshell:

1. Define schemas and queries in `build.rs`
2. `good-ormning` generates a function to set up/migrate the database and for each query

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

See Comparisons, below, for information on how Good-ormning differs from other Rust ORMs.

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
    let id = users.rowid();
    let name = users.field(&mut latest_version, "zLQI9HQUQ", "name", field_str().build());
    let points = users.field(&mut latest_version, "zLAPH3H29", "points", field_i64().build());
    goodormning::sqlite::generate(&root.join("tests/sqlite_gen_hello_world.rs"), vec![
        // Versions
        (0usize, latest_version)
    ], vec![
        // Queries
        new_insert(&users, vec![(name.id.clone(), Expr::Param {
            name: "name".into(),
            type_: name.def.type_.type_.clone(),
        }), (points.id.clone(), Expr::Param {
            name: "points".into(),
            type_: points.def.type_.type_.clone(),
        })]).build_query("create_user", QueryResCount::None),
        new_select(&users).where_(Expr::BinOp {
            left: Box::new(Expr::Field(id.id.clone())),
            op: BinOp::Equals,
            right: Box::new(Expr::Param {
                name: "id".into(),
                type_: id.def.type_.type_.clone(),
            }),
        }).output_fields(&[&name, &points]).build_query("get_user", QueryResCount::One),
        new_select(&users).output_field(&id).build_query("list_users", QueryResCount::Many)
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
    let txn = db.transaction().map_err(|e| GoodError(e.to_string()))?;
    match (|| {
        txn.execute("create table if not exists __good_version (version bigint not null);", ())?;
        let mut stmt = txn.prepare("select version from __good_version limit 1")?;
        let mut rows = stmt.query(())?;
        let version = match rows.next()? {
            Some(r) => {
                let ver: i64 = r.get(0usize)?;
                ver
            },
            None => {
                let mut stmt = txn.prepare("insert into __good_version (version) values (-1) returning version")?;
                let mut rows = stmt.query(())?;
                let ver: i64 =
                    rows
                        .next()?
                        .ok_or_else(|| GoodError("Insert version failed to return any values".into()))?
                        .get(0usize)?;
                ver
            },
        };
        if version < 0i64 {
            txn.execute(
                "create table \"zQLEK3CT0\" ( \"zLQI9HQUQ\" text not null , \"zLAPH3H29\" integer not null )",
                (),
            )?;
        }
        txn.execute("update __good_version set version = $1", rusqlite::params![0i64])?;
        let out: Result<(), GoodError> = Ok(());
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
        Ok(_) => {
            match txn.commit() {
                Err(e) => {
                    return Err(GoodError(format!("Error committing the migration transaction: {}", e)));
                },
                Ok(_) => { },
            };
        },
    }
    Ok(())
}

pub fn create_user(db: &mut rusqlite::Connection, name: &str, points: i64) -> Result<(), GoodError> {
    db
        .execute(
            "insert into \"zQLEK3CT0\" ( \"zLQI9HQUQ\" , \"zLAPH3H29\" ) values ( $1 , $2 )",
            rusqlite::params![name, points],
        )
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
            "select \"zQLEK3CT0\" . \"zLQI9HQUQ\" , \"zQLEK3CT0\" . \"zLAPH3H29\" from \"zQLEK3CT0\" where ( \"zQLEK3CT0\" . \"rowid\" = $1 )",
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
    let mut stmt = db.prepare("select \"zQLEK3CT0\" . \"rowid\" from \"zQLEK3CT0\"")?;
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
    queries::migrate(&mut db).unwrap();
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

### IDs and Names

In general in this library, IDs are SQL table/field/index/constrait/etc ids, and names are what's used in generated Rust functions and structs.

IDs must be stable. Migrations are based around stable ids, so if (for example) a table ID changes, this will be considered a delete of the table with the old id, and a create of a new table with the new id.

In the example above, I used randomly generated IDs which have this property. This has the downside that it makes the SQL CLI harder to use. It's possible this will be improved upon, but if you frequently need to do things from the CLI I suggest creating a custom CLI using generated queries.

### Types and queries

Use `type_*` `field_*` functions to get expression/field type builders. Use `new_insert/select/update/delete` to get a query builder for the associated query type.

### Custom types

When defining a field in the schema, call `.custom("mycrate::MyType")` on the field type builder (or pass it in as `Some("mycreate::MyType".to_string())` if creating the type structure directly).

Custom types need to implement

1. `trait Into<T>` for the corresponding standard type (`i32`, `String`, etc.), if you use the type as a parameter in a query.
2. `fn from_sql(T) -> Result<U, String>` for types used in query results, where `T` is the corresponding standard type and `U` is the custom type. This isn't part of a trait.

If you miss either you'll get a compile error that should indicate clearly what you need to do.

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

#### Good-ormning

- Queries have to be defined separately, in the `build.rs` file
- All queries have to be defined up front in `build.rs`
- You don't have to write any structures, everything is generated from schema and query info
- Custom types can be incorporated into the schema with no boilerplate
- Migrations are automatically derived via a diff between schema versions plus additional migration metadata
- Clear error messages, thanks to no macros, generics, traits
- Code generation is fast, compiling the simple generated code is also fast

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
