use proc_macro2::{
    TokenStream,
    Ident,
};
use quote::{
    quote,
    format_ident,
    ToTokens,
};
use std::{
    collections::{
        HashMap,
    },
    path::Path,
    fs,
    rc::Rc,
};
use crate::{
    sqlite::{
        types::{
            Type,
        },
        query::expr::ExprValName,
        graph::utils::SqliteMigrateCtx,
    },
    utils::{
        Errs,
        sanitize_ident,
        DOCSTRING_MIGRATE,
        DOCSTRING_INITIALIZE,
    },
};
use self::{
    query::{
        utils::{
            SqliteQueryCtx,
            QueryBody,
        },
        insert::{
            Insert,
            InsertConflict,
        },
        expr::Expr,
        select::{
            Returning,
            Select,
            NamedSelectSource,
            JoinSource,
            Join,
            Order,
        },
        update::Update,
        delete::Delete,
    },
    types::{
        SimpleType,
        SimpleSimpleType,
    },
    schema::{
        field::{
            Field,
            Field_,
            SchemaFieldId,
            FieldType,
        },
        table::{
            Table,
            Table_,
            SchemaTableId,
        },
        constraint::{
            ConstraintType,
            Constraint_,
            Constraint,
            SchemaConstraintId,
        },
        index::{
            Index_,
            Index,
            SchemaIndexId,
        },
    },
    graph::{
        table::NodeTable_,
        GraphId,
        utils::MigrateNode,
        Node,
        field::NodeField_,
        constraint::NodeConstraint_,
        index::NodeIndex_,
    },
};

pub mod types;
pub mod query;
pub mod schema;
pub mod graph;

/// The number of results this query returns. This determines if the return type is
/// void, `Option`, the value directly, or a `Vec`. It must be a valid value per the
/// query body (e.g. select can't have `None` res count).
#[derive(Debug, Clone)]
pub enum QueryResCount {
    None,
    MaybeOne,
    One,
    Many,
}

/// See Insert for field descriptions. Call `build()` to get a finished query object.
pub struct InsertBuilder {
    pub q: Insert,
}

impl InsertBuilder {
    pub fn on_conflict(mut self, v: InsertConflict) -> Self {
        self.q.on_conflict = Some(v);
        self
    }

    pub fn return_(mut self, v: Expr) -> Self {
        self.q.returning.push(Returning {
            e: v,
            rename: None,
        });
        self
    }

    pub fn return_named(mut self, name: impl ToString, v: Expr) -> Self {
        self.q.returning.push(Returning {
            e: v,
            rename: Some(name.to_string()),
        });
        self
    }

    pub fn return_field(mut self, f: &Field) -> Self {
        self.q.returning.push(Returning {
            e: Expr::Field(f.clone()),
            rename: None,
        });
        self
    }

    pub fn return_fields(mut self, f: &[&Field]) -> Self {
        for f in f {
            self.q.returning.push(Returning {
                e: Expr::Field((*f).clone()),
                rename: None,
            });
        }
        self
    }

    pub fn returns_from_iter(mut self, f: impl Iterator<Item = Returning>) -> Self {
        self.q.returning.extend(f);
        self
    }

    // Produce a migration for use in version pre/post-migration.
    pub fn build_migration(self) -> Insert {
        self.q
    }

    // Produce a query object.
    //
    // # Arguments
    //
    // * `name` - This is used as the name of the rust function.
    pub fn build_query(self, name: impl ToString, res_count: QueryResCount) -> Query {
        Query {
            name: name.to_string(),
            body: Box::new(self.q),
            res_count: res_count,
            res_name: None,
        }
    }

    // Same as `build_query`, but specify a name for the result structure. Only valid if
    // result is a record (not a single value).
    pub fn build_query_named_res(self, name: impl ToString, res_count: QueryResCount, res_name: impl ToString) -> Query {
        Query {
            name: name.to_string(),
            body: Box::new(self.q),
            res_count: res_count,
            res_name: Some(res_name.to_string()),
        }
    }
}

/// See Select for field descriptions. Call `build()` to get a finished query object.
pub struct SelectBuilder {
    pub q: Select,
}

impl SelectBuilder {
    pub fn return_(mut self, v: Expr) -> Self {
        self.q.returning.push(Returning {
            e: v,
            rename: None,
        });
        self
    }

    pub fn return_named(mut self, name: impl ToString, v: Expr) -> Self {
        self.q.returning.push(Returning {
            e: v,
            rename: Some(name.to_string()),
        });
        self
    }

    pub fn return_field(mut self, f: &Field) -> Self {
        self.q.returning.push(Returning {
            e: Expr::Field(f.clone()),
            rename: None,
        });
        self
    }

    pub fn return_fields(mut self, f: &[&Field]) -> Self {
        for f in f {
            self.q.returning.push(Returning {
                e: Expr::Field((*f).clone()),
                rename: None,
            });
        }
        self
    }

    pub fn returns_from_iter(mut self, f: impl Iterator<Item = Returning>) -> Self {
        self.q.returning.extend(f);
        self
    }

    pub fn join(mut self, join: Join) -> Self {
        self.q.join.push(join);
        self
    }

    pub fn where_(mut self, predicate: Expr) -> Self {
        self.q.where_ = Some(predicate);
        self
    }

    pub fn group(mut self, clauses: Vec<Expr>) -> Self {
        self.q.group = clauses;
        self
    }

    pub fn order(mut self, expr: Expr, order: Order) -> Self {
        self.q.order.push((expr, order));
        self
    }

    pub fn order_from_iter(mut self, clauses: impl Iterator<Item = (Expr, Order)>) -> Self {
        self.q.order.extend(clauses);
        self
    }

    pub fn limit(mut self, v: usize) -> Self {
        self.q.limit = Some(v);
        self
    }

    // Produce a migration for use in version pre/post-migration.
    pub fn build_migration(self) -> Select {
        self.q
    }

    // Produce a query object.
    //
    // # Arguments
    //
    // * `name` - This is used as the name of the rust function.
    pub fn build_query(self, name: impl ToString, res_count: QueryResCount) -> Query {
        Query {
            name: name.to_string(),
            body: Box::new(self.q),
            res_count: res_count,
            res_name: None,
        }
    }

    // Same as `build_query`, but specify a name for the result structure. Only valid if
    // result is a record (not a single value).
    pub fn build_query_named_res(self, name: impl ToString, res_count: QueryResCount, res_name: impl ToString) -> Query {
        Query {
            name: name.to_string(),
            body: Box::new(self.q),
            res_count: res_count,
            res_name: Some(res_name.to_string()),
        }
    }
}

/// See Update for field descriptions. Call `build()` to get a finished query object.
pub struct UpdateBuilder {
    pub q: Update,
}

impl UpdateBuilder {
    pub fn where_(mut self, v: Expr) -> Self {
        self.q.where_ = Some(v);
        self
    }

    pub fn return_(mut self, v: Expr) -> Self {
        self.q.returning.push(Returning {
            e: v,
            rename: None,
        });
        self
    }

    pub fn return_named(mut self, name: impl ToString, v: Expr) -> Self {
        self.q.returning.push(Returning {
            e: v,
            rename: Some(name.to_string()),
        });
        self
    }

    pub fn return_field(mut self, f: &Field) -> Self {
        self.q.returning.push(Returning {
            e: Expr::Field(f.clone()),
            rename: None,
        });
        self
    }

    pub fn return_fields(mut self, f: &[&Field]) -> Self {
        for f in f {
            self.q.returning.push(Returning {
                e: Expr::Field((*f).clone()),
                rename: None,
            });
        }
        self
    }

    pub fn returns_from_iter(mut self, f: impl Iterator<Item = Returning>) -> Self {
        self.q.returning.extend(f);
        self
    }

    // Produce a migration for use in version pre/post-migration.
    pub fn build_migration(self) -> Update {
        self.q
    }

    // Produce a query object.
    //
    // # Arguments
    //
    // * `name` - This is used as the name of the rust function.
    pub fn build_query(self, name: impl ToString, res_count: QueryResCount) -> Query {
        Query {
            name: name.to_string(),
            body: Box::new(self.q),
            res_count: res_count,
            res_name: None,
        }
    }

    // Same as `build_query`, but specify a name for the result structure. Only valid if
    // result is a record (not a single value).
    pub fn build_query_named_res(self, name: impl ToString, res_count: QueryResCount, res_name: impl ToString) -> Query {
        Query {
            name: name.to_string(),
            body: Box::new(self.q),
            res_count: res_count,
            res_name: Some(res_name.to_string()),
        }
    }
}

/// See Delete for field descriptions. Call `build()` to get a finished query object.
pub struct DeleteBuilder {
    pub q: Delete,
}

impl DeleteBuilder {
    pub fn where_(mut self, v: Expr) -> Self {
        self.q.where_ = Some(v);
        self
    }

    pub fn return_(mut self, v: Expr) -> Self {
        self.q.returning.push(Returning {
            e: v,
            rename: None,
        });
        self
    }

    pub fn return_named(mut self, name: impl ToString, v: Expr) -> Self {
        self.q.returning.push(Returning {
            e: v,
            rename: Some(name.to_string()),
        });
        self
    }

    pub fn return_field(mut self, f: &Field) -> Self {
        self.q.returning.push(Returning {
            e: Expr::Field(f.clone()),
            rename: None,
        });
        self
    }

    pub fn return_fields(mut self, f: &[&Field]) -> Self {
        for f in f {
            self.q.returning.push(Returning {
                e: Expr::Field((*f).clone()),
                rename: None,
            });
        }
        self
    }

    pub fn returns_from_iter(mut self, f: impl Iterator<Item = Returning>) -> Self {
        self.q.returning.extend(f);
        self
    }

    // Produce a migration for use in version pre/post-migration.
    pub fn build_migration(self) -> Delete {
        self.q
    }

    // Produce a query object.
    //
    // # Arguments
    //
    // * `name` - This is used as the name of the rust function.
    pub fn build_query(self, name: impl ToString, res_count: QueryResCount) -> Query {
        Query {
            name: name.to_string(),
            body: Box::new(self.q),
            res_count: res_count,
            res_name: None,
        }
    }

    // Same as `build_query`, but specify a name for the result structure. Only valid if
    // result is a record (not a single value).
    pub fn build_query_named_res(self, name: impl ToString, res_count: QueryResCount, res_name: impl ToString) -> Query {
        Query {
            name: name.to_string(),
            body: Box::new(self.q),
            res_count: res_count,
            res_name: Some(res_name.to_string()),
        }
    }
}

/// This represents an SQL query. A function will be generated which accepts a db
/// connection and query parameters, and returns the query results. Call the `new_*`
/// functions to get a builder.
pub struct Query {
    pub name: String,
    pub body: Box<dyn QueryBody>,
    pub res_count: QueryResCount,
    pub res_name: Option<String>,
}

/// Get a builder for an INSERT query.
///
/// # Arguments
///
/// * `values` - The fields to insert and their corresponding values
pub fn new_insert(table: &Table, values: Vec<(Field, Expr)>) -> InsertBuilder {
    InsertBuilder { q: Insert {
        table: table.clone(),
        values: values,
        on_conflict: None,
        returning: vec![],
    } }
}

/// Get a builder for a SELECT query.
pub fn new_select(table: &Table) -> SelectBuilder {
    SelectBuilder { q: Select {
        table: NamedSelectSource {
            source: JoinSource::Table(table.clone()),
            alias: None,
        },
        returning: vec![],
        join: vec![],
        where_: None,
        group: vec![],
        order: vec![],
        limit: None,
    } }
}

/// Get a builder for a SELECT query. This allows advanced sources (like selecting
/// from a synthetic table).
pub fn new_select_from(source: NamedSelectSource) -> SelectBuilder {
    SelectBuilder { q: Select {
        table: source,
        returning: vec![],
        join: vec![],
        where_: None,
        group: vec![],
        order: vec![],
        limit: None,
    } }
}

/// Get a builder for an UPDATE query.
///
/// # Arguments
///
/// * `values` - The fields to update and their corresponding values
pub fn new_update(table: &Table, values: Vec<(Field, Expr)>) -> UpdateBuilder {
    UpdateBuilder { q: Update {
        table: table.clone(),
        values: values,
        where_: None,
        returning: vec![],
    } }
}

/// Get a builder for a DELETE query.
///
/// # Arguments
///
/// * `name` - This becomes the name of the generated rust function.
pub fn new_delete(table: &Table) -> DeleteBuilder {
    DeleteBuilder { q: Delete {
        table: table.clone(),
        returning: vec![],
        where_: None,
    } }
}

/// The version represents the state of a schema at a point in time.
#[derive(Default)]
pub struct Version {
    schema: HashMap<GraphId, MigrateNode>,
    pre_migration: Vec<Box<dyn QueryBody>>,
    post_migration: Vec<Box<dyn QueryBody>>,
}

impl Version {
    /// Define a table in the version
    pub fn table(&mut self, schema_id: &str, id: &str) -> Table {
        let out = Table(Rc::new(Table_ {
            schema_id: SchemaTableId(schema_id.into()),
            id: id.into(),
        }));
        if self.schema.insert(GraphId::Table(out.schema_id.clone()), MigrateNode::new(vec![], Node::table(NodeTable_ {
            def: out.clone(),
            fields: vec![],
        }))).is_some() {
            panic!("Table with schema id {} already exists", out.schema_id);
        };
        out
    }

    /// Add a query to execute before before migrating to this schema (applied immediately
    /// before migration).
    pub fn pre_migration(&mut self, q: impl QueryBody + 'static) {
        self.pre_migration.push(Box::new(q));
    }

    /// Add a query to execute after migrating to this schema version (applied immediately
    /// after migration).
    pub fn post_migration(&mut self, q: impl QueryBody + 'static) {
        self.post_migration.push(Box::new(q));
    }
}

impl Table {
    /// Define a field
    pub fn field(&self, v: &mut Version, schema_id: impl ToString, id: impl ToString, type_: FieldType) -> Field {
        let out = Field(Rc::new(Field_ {
            table: self.clone(),
            schema_id: SchemaFieldId(schema_id.to_string()),
            id: id.to_string(),
            type_: type_,
        }));
        if &out.id == "rowid" {
            panic!("Use rowid_field to define a rowid field");
        }
        if v
            .schema
            .insert(
                GraphId::Field(self.schema_id.clone(), out.schema_id.clone()),
                MigrateNode::new(
                    vec![GraphId::Table(self.schema_id.clone())],
                    Node::field(NodeField_ { def: out.clone() }),
                ),
            )
            .is_some() {
            panic!("Field with schema id {}.{} already exists", self.schema_id, out.schema_id);
        };
        out
    }

    pub fn rowid_field(&self, v: &mut Version, custom_type: Option<String>) -> Field {
        let out = Field(Rc::new(Field_ {
            table: self.clone(),
            schema_id: SchemaFieldId("rowid".into()),
            id: "rowid".into(),
            type_: FieldType {
                type_: Type {
                    type_: SimpleType {
                        type_: SimpleSimpleType::I64,
                        custom: custom_type,
                    },
                    opt: false,
                },
                migration_default: None,
            },
        }));
        if v
            .schema
            .insert(
                GraphId::Field(self.schema_id.clone(), out.schema_id.clone()),
                MigrateNode::new(
                    vec![GraphId::Table(self.schema_id.clone())],
                    Node::field(NodeField_ { def: out.clone() }),
                ),
            )
            .is_some() {
            panic!("Field with schema id {}.{} already exists", self.schema_id, out.schema_id);
        };
        out
    }

    /// Define a constraint
    pub fn constraint(&self, v: &mut Version, schema_id: impl ToString, id: impl ToString, type_: ConstraintType) {
        let out = Constraint(Rc::new(Constraint_ {
            table: self.clone(),
            schema_id: SchemaConstraintId(schema_id.to_string()),
            id: id.to_string(),
            type_: type_,
        }));
        let mut deps = vec![GraphId::Table(self.schema_id.clone())];
        match &out.type_ {
            ConstraintType::PrimaryKey(x) => {
                for f in &x.fields {
                    if &f.table != self {
                        panic!(
                            "Field {} in primary key constraint {} is in table {}, but constraint is in table {}",
                            f,
                            out.id,
                            f.table,
                            self
                        );
                    }
                    deps.push(GraphId::Field(self.schema_id.clone(), f.schema_id.clone()));
                }
            },
            ConstraintType::ForeignKey(x) => {
                let mut last_foreign_table: Option<Field> = None;
                for f in &x.fields {
                    if &f.0.table != self {
                        panic!(
                            "Local field {} in foreign key constraint {} is in table {}, but constraint is in table {}",
                            f.0,
                            out.id,
                            f.0.table,
                            self
                        );
                    }
                    deps.push(GraphId::Field(f.0.table.schema_id.clone(), f.0.schema_id.clone()));
                    if let Some(t) = last_foreign_table.take() {
                        if t.table != f.1.table {
                            panic!(
                                "Foreign field {} in foreign key constraint {} is in table {}, but constraint is in table {}",
                                f.1,
                                out.id,
                                f.1.table,
                                self
                            );
                        }
                    }
                    last_foreign_table = Some(f.1.clone());
                    deps.push(GraphId::Field(f.1.table.schema_id.clone(), f.1.schema_id.clone()));
                }
            },
        }
        if v
            .schema
            .insert(
                GraphId::Constraint(self.schema_id.clone(), out.schema_id.clone()),
                MigrateNode::new(deps, Node::table_constraint(NodeConstraint_ { def: out.clone() })),
            )
            .is_some() {
            panic!("Constraint with schema id {}.{} aleady exists", self.schema_id, out.schema_id)
        };
    }

    /// Define an index
    pub fn index(&self, schema_id: impl ToString, id: impl ToString, fields: &[&Field]) -> IndexBuilder {
        IndexBuilder {
            table: self.clone(),
            schema_id: schema_id.to_string(),
            id: id.to_string(),
            fields: fields.iter().map(|e| (*e).clone()).collect(),
            unique: false,
        }
    }
}

pub struct IndexBuilder {
    table: Table,
    schema_id: String,
    id: String,
    fields: Vec<Field>,
    unique: bool,
}

impl IndexBuilder {
    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }

    pub fn build(self, v: &mut Version) -> Index {
        let out = Index(Rc::new(Index_ {
            table: self.table,
            schema_id: SchemaIndexId(self.schema_id),
            id: self.id,
            fields: self.fields,
            unique: self.unique,
        }));
        if v
            .schema
            .insert(
                GraphId::Index(out.table.schema_id.clone(), out.schema_id.clone()),
                MigrateNode::new(
                    vec![GraphId::Table(out.table.schema_id.clone())],
                    Node::table_index(NodeIndex_ { def: out.clone() }),
                ),
            )
            .is_some() {
            panic!("Index with schema id {}.{} already exists", out.table.schema_id, out.schema_id);
        };
        out
    }
}

/// Generate Rust code for migrations and queries.
///
/// # Arguments
///
/// * `output` - the path to a single rust source file where the output will be written
///
/// * `versions` - a list of database version ids and schema versions. The ids must be
///    consecutive but can start from any number. Once a version has been applied to a
///    production database it shouldn't be modified again (modifications should be done in a
///    new version).
///
///    These will be turned into migrations as part of the `migrate` function.
///
/// * `queries` - a list of queries against the schema in the latest version. These will be
///    turned into functions.
///
/// # Returns
///
/// * Error - a list of validation or generation errors that occurred
pub fn generate(output: &Path, versions: Vec<(u32, Version)>, queries: Vec<Query>) -> Result<(), Vec<String>> {
    // Initial correctness checks
    {
        if versions.is_empty() {
            return Err(vec![format!("Generate called with empty versions")]);
        }
        let mut prev_relations: HashMap<&String, String> = HashMap::new();
        let mut prev_fields = HashMap::new();
        let mut prev_constraints = HashMap::new();
        for (v_i, v) in &versions {
            let mut relations = HashMap::new();
            let mut fields = HashMap::new();
            let mut constraints = HashMap::new();
            for n in v.schema.values() {
                match &n.body {
                    Node::Table(t) => {
                        let id = &t.def.id;
                        let comp_id = format!("table {}", t.def.schema_id);
                        if relations.insert(id, comp_id.clone()).is_some() {
                            panic!("Duplicate table id {} -- {}", t.def.id, t.def);
                        }
                        if let Some(schema_id) = prev_relations.get(id) {
                            if schema_id != &comp_id {
                                panic!(
                                    "Table {} id in version {} swapped with another relation since previous version; unsupported",
                                    t.def,
                                    v_i
                                );
                            }
                        }
                    },
                    Node::Field(f) => {
                        let id = (&f.def.table.schema_id, &f.def.id);
                        if fields.insert(id, f.def.schema_id.clone()).is_some() {
                            panic!("Duplicate field id {} -- {}", f.def.id, f.def);
                        }
                        if let Some(schema_id) = prev_fields.get(&id) {
                            if schema_id != &f.def.schema_id {
                                panic!(
                                    "Field {} id in version {} swapped with another field since previous version; unsupported",
                                    f.def,
                                    v_i
                                );
                            }
                        }
                    },
                    Node::Constraint(c) => {
                        let id = (&c.def.table.schema_id, &c.def.id);
                        if constraints.insert(id, c.def.schema_id.clone()).is_some() {
                            panic!("Duplicate constraint id {} -- {}", c.def.id, c.def);
                        }
                        if let Some(schema_id) = prev_constraints.get(&id) {
                            if schema_id != &c.def.schema_id {
                                panic!(
                                    "Constraint {} id in version {} swapped with another constraint since previous version; unsupported",
                                    c.def,
                                    v_i
                                );
                            }
                        }
                    },
                    Node::Index(i) => {
                        let id = &i.def.id;
                        let comp_id = format!("index {}", i.def.schema_id);
                        if relations.insert(id, comp_id.clone()).is_some() {
                            panic!("Duplicate index id {} -- {}", i.def.id, i.def);
                        }
                        if let Some(schema_id) = prev_relations.get(&id) {
                            if schema_id != &comp_id {
                                panic!(
                                    "Index {} id in version {} swapped with another relation since previous version; unsupported",
                                    i.def,
                                    v_i
                                );
                            }
                        }
                    },
                }
            }
            prev_relations = relations;
            prev_fields = fields;
            prev_constraints = constraints;
        }
    }
    let mut errs = Errs::new();

    // Generate initialization and incremental migrations
    fn generate_migration(
        errs: &mut Errs,
        field_lookup: &mut HashMap<Table, HashMap<Field, Type>>,
        path: rpds::Vector<String>,
        prev_version: Option<(&Version, i64)>,
        version: &Version,
        version_i: i64,
    ) -> TokenStream {
        let mut migration = vec![];

        fn do_migration_query(
            errs: &mut Errs,
            path: &rpds::Vector<String>,
            migration: &mut Vec<TokenStream>,
            field_lookup: &HashMap<Table, HashMap<Field, Type>>,
            q: &dyn QueryBody,
        ) {
            let mut qctx = SqliteQueryCtx::new(errs.clone(), &field_lookup);
            let e_res = q.build(&mut qctx, path, QueryResCount::None);
            if !qctx.rust_args.is_empty() {
                qctx.errs.err(path, format!("Migration statements can't receive arguments"));
            }
            let statement = e_res.1.to_string();
            let args = qctx.query_args;
            migration.push(quote!{
                txn.execute(#statement, rusqlite::params![#(#args,) *]) ?;
            });
        }

        // Do pre-migrations
        if prev_version.is_some() {
            for (i, q) in version.pre_migration.iter().enumerate() {
                do_migration_query(
                    errs,
                    &path.push_back(format!("Pre-migration statement {}", i)),
                    &mut migration,
                    &field_lookup,
                    q.as_ref(),
                );
            }
        }

        // Prep for current version
        field_lookup.clear();
        let version_i = version_i as i64;
        if let Some((_, i)) = prev_version {
            if version_i != i as i64 + 1 {
                errs.err(
                    &path,
                    format!(
                        "Version numbers are not consecutive ({} to {}) - was an intermediate version deleted?",
                        i,
                        version_i
                    ),
                );
            }
        }

        // Gather tables for lookup during query generation and check duplicates
        for v in version.schema.values() {
            match &v.body {
                Node::Field(f) => {
                    match field_lookup.entry(f.def.table.clone()) {
                        std::collections::hash_map::Entry::Occupied(_) => { },
                        std::collections::hash_map::Entry::Vacant(e) => {
                            e.insert(HashMap::new());
                        },
                    };
                    let table = field_lookup.get_mut(&f.def.table).unwrap();
                    table.insert(f.def.clone(), f.def.type_.type_.clone());
                },
                _ => { },
            };
        }

        // Main migrations
        {
            let mut state = SqliteMigrateCtx::new(errs.clone());
            crate::graphmigrate::migrate(&mut state, prev_version.map(|s| &s.0.schema), &version.schema);
            for statement in &state.statements {
                migration.push(quote!{
                    txn.execute(#statement, ()) ?;
                });
            }
        }

        // Post-migration
        for (i, q) in version.post_migration.iter().enumerate() {
            do_migration_query(
                errs,
                &path.push_back(format!("Post-migration statement {}", i)),
                &mut migration,
                &field_lookup,
                q.as_ref(),
            );
        }

        quote!{
            if version < #version_i {
                #(#migration) *
            }
        }
    }

    let initialization_statements = {
        let (version_i, version) = versions.last().unwrap();
        let mut field_lookup = HashMap::new();
        generate_migration(
            &mut errs,
            &mut field_lookup,
            rpds::vector![format!("Initialization to {}", version_i)],
            None,
            &version,
            *version_i as i64,
        )
    };
    let mut field_lookup = HashMap::new();
    let mut migration_statements = vec![];
    {
        let mut prev_version: Option<(&Version, i64)> = None;
        for (version_i, version) in &versions {
            let version_i = *version_i as i64;
            migration_statements.push(
                generate_migration(
                    &mut errs,
                    &mut field_lookup,
                    rpds::vector![
                        format!("Migration from {} to {}", prev_version.map(|v| v.1).unwrap_or(-1), version_i)
                    ],
                    prev_version,
                    &version,
                    version_i,
                ),
            );
            prev_version = Some((version, version_i));
        }
    }

    // Generate queries
    let mut db_others = Vec::new();
    {
        let mut res_type_idents: HashMap<String, Ident> = HashMap::new();
        for q in queries {
            let path = rpds::vector![format!("Query {}", q.name)];
            let mut ctx = SqliteQueryCtx::new(errs.clone(), &field_lookup);
            let res = QueryBody::build(q.body.as_ref(), &mut ctx, &path, q.res_count.clone());
            let ident = format_ident!("{}", q.name);
            let q_text = res.1.to_string();
            let args = ctx.rust_args.split_off(0);
            let args_forward = ctx.query_args.split_off(0);
            drop(ctx);
            let (res_ident, res_def, unforward_res) = {
                fn convert_one_res(
                    errs: &mut Errs,
                    path: &rpds::Vector<String>,
                    i: usize,
                    k: &ExprValName,
                    v: &Type,
                ) -> Option<(Ident, TokenStream, TokenStream)> {
                    if k.id.is_empty() {
                        errs.err(
                            path,
                            format!("Result element {} has no name; name it using `rename` if this is intentional", i),
                        );
                        return None;
                    }
                    let mut ident: TokenStream = match v.type_.type_ {
                        types::SimpleSimpleType::U32 => quote!(u32),
                        types::SimpleSimpleType::I32 => quote!(i32),
                        types::SimpleSimpleType::I64 => quote!(i64),
                        types::SimpleSimpleType::F32 => quote!(f32),
                        types::SimpleSimpleType::F64 => quote!(f64),
                        types::SimpleSimpleType::Bool => quote!(bool),
                        types::SimpleSimpleType::String => quote!(String),
                        types::SimpleSimpleType::Bytes => quote!(Vec < u8 >),
                        types::SimpleSimpleType::UtcTimeS => quote!(chrono:: DateTime < chrono:: Utc >),
                        types::SimpleSimpleType::UtcTimeMs => quote!(chrono:: DateTime < chrono:: Utc >),
                    };
                    if v.opt {
                        ident = quote!(Option < #ident >);
                    }
                    let mut unforward = match v.type_.type_ {
                        types::SimpleSimpleType::U32 |
                        types::SimpleSimpleType::I32 |
                        types::SimpleSimpleType::I64 |
                        types::SimpleSimpleType::F32 |
                        types::SimpleSimpleType::F64 |
                        types::SimpleSimpleType::Bool |
                        types::SimpleSimpleType::String |
                        types::SimpleSimpleType::Bytes => {
                            quote!{
                                let x: #ident = r.get(#i) ?;
                            }
                        },
                        types::SimpleSimpleType::UtcTimeS => {
                            quote!{
                                let x: i64 = r.get(#i) ?;
                                let x = chrono::TimeZone::timestamp_opt(&chrono::Utc, x, 0).unwrap();
                            }
                        },
                        types::SimpleSimpleType::UtcTimeMs => {
                            quote!{
                                let x: String = r.get(#i) ?;
                                let x =
                                    chrono::DateTime::<chrono::Utc>::from(
                                        chrono::DateTime::<chrono::FixedOffset>::parse_from_rfc3339(
                                            &x,
                                        ).map_err(|e| GoodError(e.to_string()))?,
                                    );
                            }
                        },
                    };
                    if let Some(custom) = &v.type_.custom {
                        ident = match syn::parse_str::<syn::Path>(&custom) {
                            Ok(i) => i.to_token_stream(),
                            Err(e) => {
                                errs.err(
                                    path,
                                    format!(
                                        "Couldn't parse provided custom type name [{}] as identifier path: {:?}",
                                        custom,
                                        e
                                    ),
                                );
                                return None;
                            },
                        };
                        if v.opt {
                            unforward = quote!{
                                #unforward let x = if let Some(x) = x {
                                    Some(#ident:: from_sql(x).map_err(|e| GoodError(e.to_string())) ?)
                                }
                                else {
                                    None
                                };
                            };
                            ident = quote!(Option < #ident >);
                        } else {
                            unforward = quote!{
                                #unforward let x = #ident:: from_sql(x).map_err(|e| GoodError(e.to_string())) ?;
                            };
                        }
                    }
                    return Some((format_ident!("{}", sanitize_ident(&k.id).1), ident, quote!({
                        #unforward x
                    })));
                }

                if res.0.0.len() == 1 {
                    let e = &res.0.0[0];
                    let (_, type_ident, unforward) = match convert_one_res(&mut errs, &path, 0, &e.0, &e.1) {
                        None => {
                            continue;
                        },
                        Some(x) => x,
                    };
                    (type_ident, None, unforward)
                } else {
                    let mut fields = vec![];
                    let mut unforward_fields = vec![];
                    for (i, (k, v)) in res.0.0.into_iter().enumerate() {
                        let (k_ident, type_ident, unforward) = match convert_one_res(&mut errs, &path, i, &k, &v) {
                            Some(x) => x,
                            None => continue,
                        };
                        fields.push(quote!{
                            pub #k_ident: #type_ident
                        });
                        unforward_fields.push(quote!{
                            #k_ident: #unforward
                        });
                    }
                    let body = quote!({
                        #(#fields,) *
                    });
                    let res_type_count = res_type_idents.len();
                    let (res_ident, res_def) = match res_type_idents.entry(body.to_string()) {
                        std::collections::hash_map::Entry::Occupied(e) => {
                            (e.get().clone(), None)
                        },
                        std::collections::hash_map::Entry::Vacant(e) => {
                            let ident = if let Some(name) = q.res_name {
                                format_ident!("{}", name)
                            } else {
                                format_ident!("DbRes{}", res_type_count)
                            };
                            e.insert(ident.clone());
                            let res_def = quote!(pub struct #ident #body);
                            (ident, Some(res_def))
                        },
                    };
                    let unforward = quote!(#res_ident {
                        #(#unforward_fields,) *
                    });
                    (res_ident.to_token_stream(), res_def, unforward)
                }
            };
            let db_arg = quote!(db: &mut rusqlite::Connection);
            match q.res_count {
                QueryResCount::None => {
                    db_others.push(quote!{
                        pub fn #ident(#db_arg, #(#args,) *) -> Result <(),
                        GoodError > {
                            db.execute(
                                #q_text,
                                rusqlite::params![#(#args_forward,) *]
                            ).map_err(|e| GoodError(e.to_string())) ?;
                            Ok(())
                        }
                    });
                },
                QueryResCount::MaybeOne => {
                    if let Some(res_def) = res_def {
                        db_others.push(res_def);
                    }
                    db_others.push(quote!{
                        pub fn #ident(#db_arg, #(#args,) *) -> Result < Option < #res_ident >,
                        GoodError > {
                            let mut stmt = db.prepare(#q_text) ?;
                            let mut rows =
                                stmt
                                    .query(rusqlite::params![#(#args_forward,) *])
                                    .map_err(|e| GoodError(e.to_string()))?;
                            let r = rows.next()?;
                            if let Some(r) = r {
                                return Ok(Some(#unforward_res));
                            }
                            Ok(None)
                        }
                    });
                },
                QueryResCount::One => {
                    if let Some(res_def) = res_def {
                        db_others.push(res_def);
                    }
                    db_others.push(quote!{
                        pub fn #ident(#db_arg, #(#args,) *) -> Result < #res_ident,
                        GoodError > {
                            let mut stmt = db.prepare(#q_text) ?;
                            let mut rows =
                                stmt
                                    .query(rusqlite::params![#(#args_forward,) *])
                                    .map_err(|e| GoodError(e.to_string()))?;
                            let r =
                                rows
                                    .next()?
                                    .ok_or_else(
                                        || GoodError("Query expected to return one row but returned no rows".into()),
                                    )?;
                            Ok(#unforward_res)
                        }
                    });
                },
                QueryResCount::Many => {
                    if let Some(res_def) = res_def {
                        db_others.push(res_def);
                    }
                    db_others.push(quote!{
                        pub fn #ident(#db_arg, #(#args,) *) -> Result < Vec < #res_ident >,
                        GoodError > {
                            let mut out = vec![];
                            let mut stmt = db.prepare(#q_text) ?;
                            let mut rows =
                                stmt
                                    .query(rusqlite::params![#(#args_forward,) *])
                                    .map_err(|e| GoodError(e.to_string()))?;
                            while let Some(r) = rows.next() ? {
                                out.push(#unforward_res);
                            }
                            Ok(out)
                        }
                    });
                },
            }
        }
    }

    fn txn_body(
        name: Ident,
        docstring: String,
        last_version_i: i64,
        prebody: TokenStream,
        body: TokenStream,
    ) -> TokenStream {
        quote!{
            #[doc = #docstring] pub fn #name(db: &mut rusqlite::Connection) -> Result <(),
            GoodError > {
                #prebody loop {
                    let txn = db.transaction()?;
                    match(|| {
                        let version = match lock_version(&txn)? {
                            Some(v) => v,
                            None => {
                                return Ok(false);
                            },
                        };
                        if version > #last_version_i {
                            return Err(
                                GoodError(
                                    format!(
                                        "The latest known version is {}, but the schema is at unknown version {}",
                                        #last_version_i,
                                        version
                                    ),
                                ),
                            );
                        }
                        #body unlock_version(&txn, #last_version_i) ?;
                        let out: Result < bool,
                        GoodError >= Ok(true);
                        out
                    })() {
                        Err(e) => {
                            match txn.rollback() {
                                Err(e1) => {
                                    return Err(
                                        GoodError(
                                            format!(
                                                "{}\n\nIn addition to above query error during the transaction, rollback failed: {}",
                                                e,
                                                e1
                                            ),
                                        ),
                                    );
                                },
                                Ok(_) => {
                                    return Err(e);
                                },
                            };
                        }
                        Ok(migrated) => {
                            match txn.commit() {
                                Err(e) => {
                                    return Err(
                                        GoodError(format!("Error committing the migration transaction: {}", e)),
                                    );
                                },
                                Ok(_) => {
                                    if migrated {
                                        return Ok(())
                                    } else {
                                        std::thread::sleep(std::time::Duration::from_millis(5 * 1000));
                                    }
                                },
                            };
                        }
                    }
                }
            }
        }
    }

    let last_version_i = versions.last().unwrap().0 as i64;
    let initialize = txn_body(format_ident!("initialize"), DOCSTRING_INITIALIZE.into(), last_version_i, quote!{
        prep_metadata_table(db)?;
    }, initialization_statements);
    let migrate = txn_body(format_ident!("migrate"), DOCSTRING_MIGRATE.into(), last_version_i, quote!{
        prep_metadata_table(db)?;
    }, quote!{
        #(#migration_statements) *
    });

    // Compile, output
    let tokens = quote!{
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
        fn lock_version(txn: &rusqlite::Transaction) -> Result < Option < i64 >,
        GoodError > {
            let mut stmt =
                txn.prepare("update __good_version set lock = 1 where rid = 0 and lock = 0 returning version")?;
            let mut rows = stmt.query(())?;
            let version = match rows.next()? {
                Some(r) => {
                    let ver: i64 = r.get(0usize)?;
                    ver
                },
                None => return Ok(None),
            };
            drop(rows);
            stmt.finalize()?;
            Ok(Some(version))
        }
        fn unlock_version(txn: &rusqlite::Transaction, v: i64) -> Result <(),
        GoodError > {
            txn.execute("update __good_version set version = $1, lock = 0", rusqlite::params![v])?;
            Ok(())
        }
        fn prep_metadata_table(db: &mut rusqlite::Connection) -> Result <(),
        GoodError > {
            db.execute(
                "create table if not exists __good_version (rid int primary key, version bigint not null, lock int not null);",
                (),
            )?;
            db.execute(
                "insert into __good_version (rid, version, lock) values (0, -1, 0) on conflict do nothing;",
                (),
            )?;
            Ok(())
        }
        #initialize #migrate #(#db_others) *
    };
    if let Some(p) = output.parent() {
        if let Err(e) = fs::create_dir_all(&p) {
            errs.err(
                &rpds::vector![],
                format!("Error creating output parent directories {}: {:?}", p.to_string_lossy(), e),
            );
        }
    }
    match genemichaels::format_str(&tokens.to_string(), &genemichaels::FormatConfig::default()) {
        Ok(src) => {
            match fs::write(output, src.rendered.as_bytes()) {
                Ok(_) => { },
                Err(e) => errs.err(
                    &rpds::vector![],
                    format!("Failed to write generated code to {}: {:?}", output.to_string_lossy(), e),
                ),
            };
        },
        Err(e) => {
            errs.err(&rpds::vector![], format!("Error formatting generated code: {:?}\n{}", e, tokens));
        },
    };
    errs.raise()?;
    Ok(())
}

#[cfg(test)]
mod test {
    use std::{
        path::PathBuf,
        str::FromStr,
    };
    use crate::sqlite::{
        new_select,
        QueryResCount,
        new_insert,
    };
    use super::{
        schema::field::{
            field_str,
            field_i32,
        },
        generate,
        Version,
        query::expr::Expr,
    };

    #[test]
    #[should_panic]
    fn test_add_field_dup_bad() {
        generate(&PathBuf::from_str("/dev/null").unwrap(), vec![
            // Versions (previous)
            (0, {
                let mut v = Version::default();
                let bananna = v.table("zPAO2PJU4", "bananna");
                bananna.field(&mut v, "z437INV6D", "hizat", field_str().build());
                v
            }),
            (1, {
                let mut v = Version::default();
                let bananna = v.table("zQZQ8E2WD", "bananna");
                bananna.field(&mut v, "z437INV6D", "hizat", field_str().build());
                bananna.field(&mut v, "z437INV6D", "zomzom", field_i32().build());
                v
            })
        ], vec![]).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_add_table_dup_bad() {
        generate(&PathBuf::from_str("/dev/null").unwrap(), vec![
            // Versions (previous)
            (0, {
                let mut v = Version::default();
                let bananna = v.table("zSNS34DYI", "bananna");
                bananna.field(&mut v, "z437INV6D", "hizat", field_str().build());
                v
            }),
            (1, {
                let mut v = Version::default();
                let bananna = v.table("zSNS34DYI", "bananna");
                bananna.field(&mut v, "z437INV6D", "hizat", field_str().build());
                let bananna = v.table("zSNS34DYI", "bananna");
                bananna.field(&mut v, "z437INV6D", "hizat", field_str().build());
                v
            })
        ], vec![]).unwrap();
    }

    #[test]
    fn test_res_count_none_bad() {
        let mut v = Version::default();
        let bananna = v.table("z5S18LWQE", "bananna");
        let hizat = bananna.field(&mut v, "z437INV6D", "hizat", field_str().build());
        assert!(
            generate(
                &PathBuf::from_str("/dev/null").unwrap(),
                vec![(0, v)],
                vec![new_select(&bananna).return_field(&hizat).build_query("x", QueryResCount::None)],
            ).is_err()
        );
    }

    #[test]
    fn test_select_nothing_bad() {
        let mut v = Version::default();
        let bananna = v.table("zOOR88EQ9", "bananna");
        bananna.field(&mut v, "z437INV6D", "hizat", field_str().build());
        assert!(
            generate(
                &PathBuf::from_str("/dev/null").unwrap(),
                vec![(0, v)],
                vec![new_select(&bananna).build_query("x", QueryResCount::None)],
            ).is_err()
        );
    }

    #[test]
    fn test_returning_none_bad() {
        let mut v = Version::default();
        let bananna = v.table("zZPD1I2EF", "bananna");
        let hizat = bananna.field(&mut v, "z437INV6D", "hizat", field_str().build());
        assert!(
            generate(
                &PathBuf::from_str("/dev/null").unwrap(),
                vec![(0, v)],
                vec![
                    new_insert(&bananna, vec![(hizat.clone(), Expr::LitString("hoy".into()))])
                        .return_field(&hizat)
                        .build_query("x", QueryResCount::None)
                ],
            ).is_err()
        );
    }
}
