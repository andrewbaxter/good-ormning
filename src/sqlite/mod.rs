use {
    self::{
        graph::{
            constraint::NodeConstraint_,
            field::NodeField_,
            index::NodeIndex_,
            table::NodeTable_,
            utils::MigrateNode,
            GraphId,
            Node,
        },
        query::{
            delete::Delete,
            expr::Expr,
            insert::{
                Insert,
                InsertConflict,
            },
            select::Select,
            select_body::{
                Join,
                JoinSource,
                NamedSelectSource,
                Order,
                Returning,
            },
            update::Update,
            utils::{
                QueryBody,
                SqliteQueryCtx,
            },
        },
        schema::{
            constraint::{
                Constraint,
                ConstraintType,
                Constraint_,
                SchemaConstraintId,
            },
            field::{
                Field,
                FieldType,
                Field_,
                SchemaFieldId,
            },
            index::{
                Index,
                Index_,
                SchemaIndexId,
            },
            table::{
                SchemaTableId,
                Table,
                Table_,
            },
        },
        types::{
            SimpleSimpleType,
            SimpleType,
        },
    },
    crate::{
        sqlite::{
            graph::utils::SqliteMigrateCtx,
            query::expr::Binding,
            types::{
                to_rust_types,
                Type,
            },
        },
        utils::{
            sanitize_ident,
            Errs,
        },
    },
    proc_macro2::{
        Ident,
        TokenStream,
    },
    query::{
        select_body::{
            SelectBody,
            SelectJunction,
        },
        utils::With,
    },
    quote::{
        format_ident,
        quote,
        ToTokens,
    },
    std::{
        collections::{
            BTreeMap,
            HashMap,
            HashSet,
        },
        fs,
        path::Path,
        rc::Rc,
    },
};

pub mod types;
pub mod query;
pub mod schema;
pub mod graph;

/// The number of results this query returns. This determines if the return type is
/// void, `Option`, the value directly, or a `Vec`. It must be a valid value per
/// the query body (e.g. select can't have `None` res count).
#[derive(Debug, Clone)]
pub enum QueryResCount {
    None,
    MaybeOne,
    One,
    Many,
}

/// See Insert for field descriptions. Call `build()` to get a finished query
/// object.
pub struct InsertBuilder {
    pub q: Insert,
}

impl InsertBuilder {
    // Add a `WITH`/CTE to the query.
    pub fn with(mut self, w: With) -> Self {
        self.q.with = Some(w);
        return self;
    }

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
            e: Expr::Binding(Binding::field(f)),
            rename: None,
        });
        self
    }

    pub fn return_fields(mut self, f: &[&Field]) -> Self {
        for f in f {
            self.q.returning.push(Returning {
                e: Expr::Binding(Binding::field(f)),
                rename: None,
            });
        }
        self
    }

    pub fn returns_from_iter(mut self, f: impl Iterator<Item = Returning>) -> Self {
        self.q.returning.extend(f);
        self
    }

    /// Produce a migration for use in version pre/post-migration.
    pub fn build_migration(self) -> Insert {
        self.q
    }

    /// Produce a query object.
    ///
    /// # Arguments
    ///
    /// * `name` - This is used as the name of the rust function.
    pub fn build_query(self, name: impl ToString, res_count: QueryResCount) -> Query {
        Query {
            name: name.to_string(),
            body: Box::new(self.q),
            res_count: res_count,
            res_name: None,
        }
    }

    /// Same as `build_query`, but specify a name for the result structure. Only valid
    /// if result is a record (not a single value).
    pub fn build_query_named_res(self, name: impl ToString, res_count: QueryResCount, res_name: impl ToString) -> Query {
        Query {
            name: name.to_string(),
            body: Box::new(self.q),
            res_count: res_count,
            res_name: Some(res_name.to_string()),
        }
    }
}

/// See Select for field descriptions. Call `build()` to get a finished query
/// object.
pub struct SelectBuilder {
    pub q: Select,
}

impl SelectBuilder {
    pub fn distinct(mut self) -> Self {
        self.q.body.distinct = true;
        return self;
    }

    // Add a `WITH`/CTE to the query.
    pub fn with(mut self, w: With) -> Self {
        self.q.with = Some(w);
        return self;
    }

    pub fn return_(mut self, v: Expr) -> Self {
        self.q.body.returning.push(Returning {
            e: v,
            rename: None,
        });
        self
    }

    pub fn return_named(mut self, name: impl ToString, v: Expr) -> Self {
        self.q.body.returning.push(Returning {
            e: v,
            rename: Some(name.to_string()),
        });
        self
    }

    pub fn return_field(mut self, f: &Field) -> Self {
        self.q.body.returning.push(Returning {
            e: Expr::Binding(Binding::field(f)),
            rename: None,
        });
        self
    }

    pub fn return_fields(mut self, f: &[&Field]) -> Self {
        for f in f {
            self.q.body.returning.push(Returning {
                e: Expr::Binding(Binding::field(f)),
                rename: None,
            });
        }
        self
    }

    pub fn returns_from_iter(mut self, f: impl Iterator<Item = Returning>) -> Self {
        self.q.body.returning.extend(f);
        self
    }

    pub fn join(mut self, join: Join) -> Self {
        self.q.body.join.push(join);
        self
    }

    pub fn where_(mut self, predicate: Expr) -> Self {
        self.q.body.where_ = Some(predicate);
        self
    }

    pub fn group(mut self, clauses: Vec<Expr>) -> Self {
        self.q.body.group = clauses;
        self
    }

    pub fn order(mut self, expr: Expr, order: Order) -> Self {
        self.q.body.order.push((expr, order));
        self
    }

    pub fn order_from_iter(mut self, clauses: impl Iterator<Item = (Expr, Order)>) -> Self {
        self.q.body.order.extend(clauses);
        self
    }

    /// Sets `LIMIT`. `v` must evaluate to a number.
    pub fn limit(mut self, v: Expr) -> Self {
        self.q.body.limit = Some(v);
        self
    }

    /// Add a UNION/INTERSECT/EXCEPT junction to the query.
    pub fn junction(mut self, j: SelectJunction) -> Self {
        self.q.body_junctions.push(j);
        return self;
    }

    /// Produce a migration for use in version pre/post-migration.
    pub fn build_migration(self) -> Select {
        self.q
    }

    /// Produce a query object.
    ///
    /// # Arguments
    ///
    /// * `name` - This is used as the name of the rust function.
    pub fn build_query(self, name: impl ToString, res_count: QueryResCount) -> Query {
        Query {
            name: name.to_string(),
            body: Box::new(self.q),
            res_count: res_count,
            res_name: None,
        }
    }

    // Same as `build_query`, but specify a name for the result structure. Only valid
    // if result is a record (not a single value).
    pub fn build_query_named_res(self, name: impl ToString, res_count: QueryResCount, res_name: impl ToString) -> Query {
        Query {
            name: name.to_string(),
            body: Box::new(self.q),
            res_count: res_count,
            res_name: Some(res_name.to_string()),
        }
    }
}

/// See SelectBody for field descriptions. Call `build()` to get a finished query
/// object.
pub struct SelectBodyBuilder {
    pub q: SelectBody,
}

impl SelectBodyBuilder {
    pub fn distinct(mut self) -> Self {
        self.q.distinct = true;
        return self;
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
            e: Expr::Binding(Binding::field(f)),
            rename: None,
        });
        self
    }

    pub fn return_fields(mut self, f: &[&Field]) -> Self {
        for f in f {
            self.q.returning.push(Returning {
                e: Expr::Binding(Binding::field(f)),
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

    /// Sets `LIMIT`. `v` must evaluate to a number.
    pub fn limit(mut self, v: Expr) -> Self {
        self.q.limit = Some(v);
        self
    }

    /// Produce a select body object.
    pub fn build(self) -> SelectBody {
        return self.q;
    }
}

/// See Update for field descriptions. Call `build()` to get a finished query
/// object.
pub struct UpdateBuilder {
    pub q: Update,
}

impl UpdateBuilder {
    // Add a `WITH`/CTE to the query.
    pub fn with(mut self, w: With) -> Self {
        self.q.with = Some(w);
        return self;
    }

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
            e: Expr::Binding(Binding::field(f)),
            rename: None,
        });
        self
    }

    pub fn return_fields(mut self, f: &[&Field]) -> Self {
        for f in f {
            self.q.returning.push(Returning {
                e: Expr::Binding(Binding::field(f)),
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

    // Same as `build_query`, but specify a name for the result structure. Only valid
    // if result is a record (not a single value).
    pub fn build_query_named_res(self, name: impl ToString, res_count: QueryResCount, res_name: impl ToString) -> Query {
        Query {
            name: name.to_string(),
            body: Box::new(self.q),
            res_count: res_count,
            res_name: Some(res_name.to_string()),
        }
    }
}

/// See Delete for field descriptions. Call `build()` to get a finished query
/// object.
pub struct DeleteBuilder {
    pub q: Delete,
}

impl DeleteBuilder {
    // Add a `WITH`/CTE to the query.
    pub fn with(mut self, w: With) -> Self {
        self.q.with = Some(w);
        return self;
    }

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
            e: Expr::Binding(Binding::field(f)),
            rename: None,
        });
        self
    }

    pub fn return_fields(mut self, f: &[&Field]) -> Self {
        for f in f {
            self.q.returning.push(Returning {
                e: Expr::Binding(Binding::field(f)),
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

    // Same as `build_query`, but specify a name for the result structure. Only valid
    // if result is a record (not a single value).
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
/// connection and query parameters, and returns the query results. Call the
/// `new_*` functions to get a builder.
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
    let mut unique = HashSet::new();
    for v in &values {
        if !unique.insert(&v.0) {
            panic!("Duplicate field {} in insert", v.0);
        }
    }
    InsertBuilder { q: Insert {
        with: None,
        table: table.clone(),
        values: values,
        on_conflict: None,
        returning: vec![],
    } }
}

/// Get a builder for a SELECT query.
pub fn new_select(table: &Table) -> SelectBuilder {
    SelectBuilder { q: Select {
        with: None,
        body: SelectBody {
            distinct: false,
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
        },
        body_junctions: vec![],
    } }
}

/// Get a builder for a SELECT query. This allows advanced sources (like selecting
/// from a synthetic table).
pub fn new_select_from(source: NamedSelectSource) -> SelectBuilder {
    SelectBuilder { q: Select {
        with: None,
        body: SelectBody {
            distinct: false,
            table: source,
            returning: vec![],
            join: vec![],
            where_: None,
            group: vec![],
            order: vec![],
            limit: None,
        },
        body_junctions: vec![],
    } }
}

/// Get a builder for an inner SELECT, such as in a CTE, subquery, JOIN, etc.
pub fn new_select_body(table: &Table) -> SelectBodyBuilder {
    SelectBodyBuilder { q: SelectBody {
        distinct: false,
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

/// Get a builder for an UPDATE query.
///
/// # Arguments
///
/// * `values` - The fields to update and their corresponding values
pub fn new_update(table: &Table, values: Vec<(Field, Expr)>) -> UpdateBuilder {
    let mut unique = HashSet::new();
    for v in &values {
        if !unique.insert(&v.0) {
            panic!("Duplicate field {} in update", v.0);
        }
    }
    UpdateBuilder { q: Update {
        with: None,
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
        with: None,
        table: table.clone(),
        returning: vec![],
        where_: None,
    } }
}

/// The version represents the state of a schema at a point in time.
#[derive(Default)]
pub struct Version {
    schema: BTreeMap<GraphId, MigrateNode>,
    pre_migration: Vec<Box<dyn QueryBody>>,
    post_migration: Vec<Box<dyn QueryBody>>,
}

impl Version {
    /// Define a table in this version
    pub fn table(&mut self, schema_id: &str, id: &str) -> Table {
        let out = Table(Rc::new(Table_ {
            schema_id: SchemaTableId(schema_id.into()),
            id: id.into(),
        }));
        if self.schema.insert(GraphId::Table(out.schema_id.clone()), MigrateNode::new(vec![], Node::table(NodeTable_ {
            def: out.clone(),
            fields: vec![],
            constraints: vec![],
        }))).is_some() {
            panic!("Table with schema id {} already exists", out.schema_id);
        };
        out
    }

    /// Add a query to execute before before migrating to this schema (applied
    /// immediately before migration).  Note that these may not run on new databases or
    /// if you later delete early migrations, so these should only modify existing data
    /// and not create new data (singleton rows, etc).  If you need those, do it with a
    /// normal query executed manually against the latest version.
    pub fn pre_migration(&mut self, q: impl QueryBody + 'static) {
        self.pre_migration.push(Box::new(q));
    }

    /// Add a query to execute after migrating to this schema version (applied
    /// immediately after migration). See other warnings from `pre_migration`.
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
                    array: false,
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
        let mut deps = vec![GraphId::Table(self.table.schema_id.clone())];
        for field in &self.fields {
            deps.push(GraphId::Field(field.table.schema_id.clone(), field.schema_id.clone()));
        }
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
                MigrateNode::new(deps, Node::table_index(NodeIndex_ { def: out.clone() })),
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
///   consecutive but can start from any number. Once a version has been applied to a
///   production database it shouldn't be modified again (modifications should be done
///   in a new version).
///
///   These will be turned into migrations as part of the `migrate` function.
///
/// * `queries` - a list of queries against the schema in the latest version. These
///   will be turned into functions.
///
/// # Returns
///
/// * Error - a list of validation or generation errors that occurred
pub fn generate(output: &Path, versions: Vec<(usize, Version)>, queries: Vec<Query>) -> Result<(), Vec<String>> {
    {
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
    let mut migrations = vec![];
    let mut prev_version: Option<Version> = None;
    let mut prev_version_i: Option<i64> = None;
    let mut field_lookup = HashMap::new();
    for (version_i, version) in versions {
        let path = rpds::vector![format!("Migration to {}", version_i)];
        let mut migration = vec![];

        fn do_migration_query(
            errs: &mut Errs,
            path: &rpds::Vector<String>,
            migration: &mut Vec<TokenStream>,
            field_lookup: &HashMap<Table, HashSet<Field>>,
            q: &dyn QueryBody,
        ) {
            let mut qctx = SqliteQueryCtx::new(errs.clone(), field_lookup.clone());
            let e_res = q.build(&mut qctx, path, QueryResCount::None);
            if !qctx.rust_args.is_empty() {
                qctx.errs.err(path, format!("Migration statements can't receive arguments"));
            }
            let statement = e_res.1.to_string();
            let args = qctx.query_args;
            migration.push(quote!{
                {
                    let query = #statement;
                    txn.execute(query, rusqlite::params![#(#args,) *]).to_good_error_query(query)?
                };
            });
        }

        // Do pre-migrations
        for (i, q) in version.pre_migration.iter().enumerate() {
            do_migration_query(
                &mut errs,
                &path.push_back(format!("Pre-migration statement {}", i)),
                &mut migration,
                &field_lookup,
                q.as_ref(),
            );
        }

        // Prep for current version
        field_lookup.clear();
        let version_i = version_i as i64;
        if let Some(i) = prev_version_i {
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
                            e.insert(HashSet::new());
                        },
                    };
                    let table = field_lookup.get_mut(&f.def.table).unwrap();
                    table.insert(f.def.clone());
                },
                _ => { },
            };
        }

        // Main migrations
        {
            let mut state = SqliteMigrateCtx::new(errs.clone());
            crate::graphmigrate::migrate(&mut state, prev_version.take().map(|s| s.schema), &version.schema);
            for statement in &state.statements {
                migration.push(quote!{
                    {
                        let query = #statement;
                        txn.execute(query, ()).to_good_error_query(query)?
                    };
                });
            }
        }

        // Post-migration
        for (i, q) in version.post_migration.iter().enumerate() {
            do_migration_query(
                &mut errs,
                &path.push_back(format!("Post-migration statement {}", i)),
                &mut migration,
                &field_lookup,
                q.as_ref(),
            );
        }

        // Build migration
        migrations.push(quote!{
            if version < #version_i {
                #(#migration) *
            }
        });

        // Next iter prep
        prev_version = Some(version);
        prev_version_i = Some(version_i);
    }

    // Generate queries
    let mut db_others = Vec::new();
    {
        let mut res_type_idents: HashMap<String, Ident> = HashMap::new();
        for q in queries {
            let path = rpds::vector![format!("Query {}", q.name)];
            let mut ctx = SqliteQueryCtx::new(errs.clone(), field_lookup.clone());
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
                    k: &Binding,
                    v: &Type,
                ) -> Option<(Ident, TokenStream, TokenStream)> {
                    if k.id.is_empty() {
                        errs.err(
                            path,
                            format!("Result element {} has no name; name it using `rename` if this is intentional", i),
                        );
                        return None;
                    }
                    let rust_types = to_rust_types(&v.type_.type_);
                    let custom_trait_ident = rust_types.custom_trait;
                    let mut ident = rust_types.ret_type;
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
                                let x: #ident = r.get(#i).to_good_error(|| format!("Getting result {}", #i)) ?;
                            }
                        },
                        #[cfg(feature = "chrono")]
                        types::SimpleSimpleType::UtcTimeS => {
                            quote!{
                                let x: i64 = r.get(#i).to_good_error(|| format!("Getting result {}", #i)) ?;
                                let x = chrono::TimeZone::timestamp_opt(&chrono::Utc, x, 0).unwrap();
                            }
                        },
                        #[cfg(feature = "chrono")]
                        types::SimpleSimpleType::UtcTimeMs => {
                            quote!{
                                let x: String = r.get(#i).to_good_error(|| format!("Getting result {}", #i)) ?;
                                let x =
                                    chrono::DateTime::<chrono::Utc>::from(
                                        chrono::DateTime::<chrono::FixedOffset>::parse_from_rfc3339(
                                            &x,
                                        ).to_good_error(|| format!("Getting result {}", #i))?,
                                    );
                            }
                        },
                        #[cfg(feature = "chrono")]
                        types::SimpleSimpleType::FixedOffsetTimeMs => {
                            quote!{
                                let x: String = r.get(#i).to_good_error(|| format!("Getting result {}", #i)) ?;
                                let x =
                                    chrono::DateTime::<chrono::FixedOffset>::from(
                                        chrono::DateTime::<chrono::FixedOffset>::parse_from_rfc3339(
                                            &x,
                                        ).to_good_error(|| format!("Getting result {}", #i))?,
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
                                    Some(
                                        < #ident as #custom_trait_ident < #ident >>:: from_sql(
                                            x
                                        ).to_good_error(|| format!("Parsing result {}", #i)) ?
                                    )
                                }
                                else {
                                    None
                                };
                            };
                            ident = quote!(Option < #ident >);
                        } else {
                            unforward = quote!{
                                #unforward let x =< #ident as #custom_trait_ident < #ident >>:: from_sql(
                                    x
                                ).to_good_error(|| format!("Parsing result {}", #i)) ?;
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
            let db_arg = quote!(db:& rusqlite:: Connection);
            match q.res_count {
                QueryResCount::None => {
                    db_others.push(quote!{
                        pub fn #ident(#db_arg, #(#args,) *) -> Result <(),
                        GoodError > {
                            let query = #q_text;
                            db.execute(query, rusqlite::params![#(#args_forward,) *]).to_good_error_query(query)?;
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
                            let query = #q_text;
                            let mut stmt = db.prepare(query).to_good_error_query(query)?;
                            let mut rows =
                                stmt.query(rusqlite::params![#(#args_forward,) *]).to_good_error_query(query)?;
                            let r = rows.next().to_good_error(|| format!("Getting row in query [{}]", query))?;
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
                            let query = #q_text;
                            let mut stmt = db.prepare(query).to_good_error_query(query)?;
                            let mut rows =
                                stmt.query(rusqlite::params![#(#args_forward,) *]).to_good_error_query(query)?;
                            let r =
                                rows
                                    .next()
                                    .to_good_error(|| format!("Getting row in query [{}]", query))?
                                    .ok_or_else(
                                        || GoodError(
                                            format!(
                                                "Expected to return one row but returned no rows in query [{}]",
                                                query
                                            ),
                                        ),
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
                            let query = #q_text;
                            let mut stmt = db.prepare(query).to_good_error_query(query)?;
                            let mut rows =
                                stmt.query(rusqlite::params![#(#args_forward,) *]).to_good_error_query(query)?;
                            while let Some(
                                r
                            ) = rows.next().to_good_error(|| format!("Getting row in query [{}]", query)) ? {
                                out.push(#unforward_res);
                            }
                            Ok(out)
                        }
                    });
                },
            }
        }
    }

    // Compile, output
    let last_version_i = prev_version_i.unwrap() as i64;
    let tokens = quote!{
        use good_ormning_runtime::GoodError;
        use good_ormning_runtime::ToGoodError;
        pub fn migrate(db:& mut rusqlite:: Connection) -> Result <(),
        GoodError > {
            rusqlite::vtab::array::load_module(
                &db,
            ).to_good_error(|| "Error loading array extension for array values".to_string())?;
            {
                let query =
                    "create table if not exists __good_version (rid int primary key, version bigint not null, lock int not null);";
                db.execute(query, ()).to_good_error_query(query)?;
            }
            {
                let query =
                    "insert into __good_version (rid, version, lock) values (0, -1, 0) on conflict do nothing;";
                db.execute(query, ()).to_good_error_query(query)?;
            }
            loop {
                let txn = db.transaction().to_good_error(|| "Starting transaction".to_string())?;
                match(|| {
                    let query = "update __good_version set lock = 1 where rid = 0 and lock = 0 returning version";
                    let mut stmt = txn.prepare(query).to_good_error_query(query)?;
                    let mut rows = stmt.query(()).to_good_error_query(query)?;
                    let version = match rows.next().to_good_error_query(query)? {
                        Some(r) => {
                            let ver: i64 = r.get(0usize).to_good_error_query(query)?;
                            ver
                        },
                        None => return Ok(false),
                    };
                    drop(rows);
                    stmt.finalize().to_good_error_query(query)?;
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
                    #(#migrations) * let query = "update __good_version set version = $1, lock = 0";
                    txn.execute(query, rusqlite::params![#last_version_i]).to_good_error_query(query)?;
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
                                            "{}\n\nRolling back the transaction due to the above also failed: {}",
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
                    }
                }
            }
        }
        #(#db_others) *
    };
    if let Some(p) = output.parent() {
        if let Err(e) = fs::create_dir_all(&p) {
            errs.err(
                &rpds::vector![],
                format!("Error creating output parent directories {}: {:?}", p.to_string_lossy(), e),
            );
        }
    }
    match genemichaels_lib::format_str(&tokens.to_string(), &genemichaels_lib::FormatConfig::default()) {
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
            (0usize, {
                let mut v = Version::default();
                let bananna = v.table("zPAO2PJU4", "bananna");
                bananna.field(&mut v, "z437INV6D", "hizat", field_str().build());
                v
            }),
            (1usize, {
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
            (0usize, {
                let mut v = Version::default();
                let bananna = v.table("zSNS34DYI", "bananna");
                bananna.field(&mut v, "z437INV6D", "hizat", field_str().build());
                v
            }),
            (1usize, {
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
                vec![(0usize, v)],
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
                vec![(0usize, v)],
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
                vec![(0usize, v)],
                vec![
                    new_insert(&bananna, vec![(hizat.clone(), Expr::LitString("hoy".into()))])
                        .return_field(&hizat)
                        .build_query("x", QueryResCount::None)
                ],
            ).is_err()
        );
    }
}
