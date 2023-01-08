# GOOD-ORMNING

This aims to be a very-safe, minimal-boilerplate ORM for Rust. Like other Rust ORMs, it doesn't abstract away from actual database workflows, but instead aims to enhance type checking with normal SQL.

Good-Ormning works by defining database schema and queries. Migrations are automatically generated between schema versions, and queries are checked against the information in the schema.

See Comparisons, below, for information on how Good-Ormning differs from other Rust ORMs.

The current status is: Alpha - nominally works, but expect bugs. The generated code is easy to check and type safety should prevent most runtime issues, but you may run into yet-unsupported SQL features.

Supported databases:

- PostgreSQL

## Getting started

### First time

1. You'll need the following runtime dependencies:

   - `tokio-postgres` for PostgreSQL
   - `hex_literal` if you use byte array literals in any queries

   And `build.rs` dependencies:

   - `good-ormning`

2. Create a `build.rs` and define your initial schema version and queries
3. Call `goodormning::generate` to output the generated code
4. In your code, after creating a database connection, call `migrate` to get a database handle with the queries you defined as methods

### Schema changes

1. Create a new schema version, leaving the old schema version untouched.

If any of your queries don't work with the new schema, you'll be informed during the build.

## Example

This `build.rs` file

```rust

```

Generates this code

```rust

```

## Advanced usage

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
