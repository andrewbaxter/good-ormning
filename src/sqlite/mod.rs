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
        HashSet,
        BTreeMap,
    },
    path::Path,
    fs,
    rc::Rc,
    cell::RefCell,
};
use serde::{
    Serialize,
    Deserialize,
};
use crate::{
    sqlite::{
        types::{
            Type,
            to_rust_types,
        },
        query::expr::Binding,
        graph::utils::SqliteMigrateCtx,
    },
    utils::{
        Errs,
        sanitize_ident,
    },
};
use self::{
    query::{
        utils::{
            SqliteQueryCtx,
            QueryBody,
            SqliteTableInfo,
            SqliteFieldInfo,
            Returning,
        },
        insert::{
            Insert,
            InsertConflict,
        },
        expr::Expr,
        select::{
            Select,
            NamedSelectSource,
            JoinSource,
            Join,
            Order,
        },
        update::Update,
        delete::Delete,
    },
    schema::{
        field::{
            Field,
            SchemaFieldId,
            FieldType,
            FieldRef,
        },
        table::{
            Table,
            SchemaTableId,
            TableRef,
        },
        constraint::{
            ConstraintType,
            Constraint,
            SchemaConstraintId,
            PrimaryKeyDef,
            ForeignKeyDef,
        },
        index::{
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
    pub fn on_conflict_do_update(mut self, f: &[&FieldHandle], v: Vec<(FieldHandle, Expr)>) -> Self {
        self.q.on_conflict = Some(InsertConflict::DoUpdate {
            conflict: f.iter().map(|f| f.to_ref()).collect(),
            set: v.into_iter().map(|(f, e)| (f.to_ref(), e)).collect(),
        });
        self
    }

    pub fn on_conflict_do_nothing(mut self) -> Self {
        self.q.on_conflict = Some(InsertConflict::DoNothing);
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

    pub fn return_field(mut self, f: &FieldHandle) -> Self {
        self.q.returning.push(Returning {
            e: Expr::Field(f.to_ref()),
            rename: None,
        });
        self
    }

    pub fn return_fields(mut self, f: &[&FieldHandle]) -> Self {
        for f in f {
            self.q.returning.push(Returning {
                e: Expr::Field(f.to_ref()),
                rename: None,
            });
        }
        self
    }

    pub fn returns_from_iter(mut self, f: impl Iterator<Item = Returning>) -> Self {
        self.q.returning.extend(f);
        self
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

    pub fn return_field(mut self, f: &FieldHandle) -> Self {
        self.q.returning.push(Returning {
            e: Expr::Field(f.to_ref()),
            rename: None,
        });
        self
    }

    pub fn return_fields(mut self, f: &[&FieldHandle]) -> Self {
        for f in f {
            self.q.returning.push(Returning {
                e: Expr::Field(f.to_ref()),
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

    pub fn junction(mut self, junction: self::query::select_body::SelectJunction) -> Self {
        self.q.junction.push(junction);
        self
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

/// See Update for field descriptions. Call `build()` to get a finished query
/// object.
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

    pub fn return_field(mut self, f: &FieldHandle) -> Self {
        self.q.returning.push(Returning {
            e: Expr::Field(f.to_ref()),
            rename: None,
        });
        self
    }

    pub fn return_fields(mut self, f: &[&FieldHandle]) -> Self {
        for f in f {
            self.q.returning.push(Returning {
                e: Expr::Field(f.to_ref()),
                rename: None,
            });
        }
        self
    }

    pub fn returns_from_iter(mut self, f: impl Iterator<Item = Returning>) -> Self {
        self.q.returning.extend(f);
        self
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

    pub fn return_field(mut self, f: &FieldHandle) -> Self {
        self.q.returning.push(Returning {
            e: Expr::Field(f.to_ref()),
            rename: None,
        });
        self
    }

    pub fn return_fields(mut self, f: &[&FieldHandle]) -> Self {
        for f in f {
            self.q.returning.push(Returning {
                e: Expr::Field(f.to_ref()),
                rename: None,
            });
        }
        self
    }

    pub fn returns_from_iter(mut self, f: impl Iterator<Item = Returning>) -> Self {
        self.q.returning.extend(f);
        self
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

pub struct SelectBodyBuilder {
    pub q: self::query::select_body::SelectBody,
}

impl SelectBodyBuilder {
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

    pub fn return_field(mut self, f: &FieldHandle) -> Self {
        self.q.returning.push(Returning {
            e: Expr::Field(f.to_ref()),
            rename: None,
        });
        self
    }

    pub fn return_fields(mut self, f: &[&FieldHandle]) -> Self {
        for f in f {
            self.q.returning.push(Returning {
                e: Expr::Field(f.to_ref()),
                rename: None,
            });
        }
        self
    }

    pub fn join(mut self, join: self::query::select_body::Join) -> Self {
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

    pub fn order(mut self, expr: Expr, order: self::query::select_body::Order) -> Self {
        self.q.order.push((expr, order));
        self
    }

    pub fn limit(mut self, v: Expr) -> Self {
        self.q.limit = Some(v);
        self
    }

    pub fn build(self) -> self::query::select_body::SelectBody {
        self.q
    }
}

pub fn new_select_body(table: &TableHandle) -> SelectBodyBuilder {
    SelectBodyBuilder {
        q: self::query::select_body::SelectBody {
            table: self::query::select_body::NamedSelectSource {
                source: self::query::select_body::JoinSource::Table(table.to_ref()),
                alias: None,
            },
            distinct: false,
            returning: vec![],
            join: vec![],
            where_: None,
            group: vec![],
            order: vec![],
            limit: None,
        },
    }
}

pub fn new_select_body_from(source: self::query::select_body::NamedSelectSource) -> SelectBodyBuilder {
    SelectBodyBuilder {
        q: self::query::select_body::SelectBody {
            table: source,
            distinct: false,
            returning: vec![],
            join: vec![],
            where_: None,
            group: vec![],
            order: vec![],
            limit: None,
        },
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
pub fn new_insert(table: &TableHandle, values: Vec<(FieldHandle, Expr)>) -> InsertBuilder {
    let mut unique = HashSet::new();
    for v in &values {
        if !unique.insert(v.0.schema_id.clone()) {
            panic!("Duplicate field {:?} in insert", v.0.schema_id);
        }
    }
    InsertBuilder { q: Insert {
        table: table.to_ref(),
        values: values.into_iter().map(|(f, e)| (f.to_ref(), e)).collect(),
        on_conflict: None,
        returning: vec![],
    } }
}

/// Get a builder for a SELECT query.
pub fn new_select(table: &TableHandle) -> SelectBuilder {
    SelectBuilder { q: Select {
        table: NamedSelectSource {
            source: JoinSource::Table(table.to_ref()),
            alias: None,
        },
        returning: vec![],
        junction: vec![],
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
        junction: vec![],
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
pub fn new_update(table: &TableHandle, values: Vec<(FieldHandle, Expr)>) -> UpdateBuilder {
    let mut unique = HashSet::new();
    for v in &values {
        if !unique.insert(v.0.schema_id.clone()) {
            panic!("Duplicate field {:?} in update", v.0.schema_id);
        }
    }
    UpdateBuilder { q: Update {
        table: table.to_ref(),
        values: values.into_iter().map(|(f, e)| (f.to_ref(), e)).collect(),
        where_: None,
        returning: vec![],
    } }
}

/// Get a builder for a DELETE query.
///
/// # Arguments
///
/// * `name` - This becomes the name of the generated rust function.
pub fn new_delete(table: &TableHandle) -> DeleteBuilder {
    DeleteBuilder { q: Delete {
        table: table.to_ref(),
        returning: vec![],
        where_: None,
    } }
}

/// The version represents the state of a schema at a point in time.
#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct Version {
    pub tables: BTreeMap<SchemaTableId, Table>,
}

#[derive(Clone)]
pub struct VersionHandle(pub Rc<RefCell<Version>>);

impl VersionHandle {
    pub fn new() -> Self {
        VersionHandle(Rc::new(RefCell::new(Version::default())))
    }

    pub fn table(&self, schema_id: &str, id: &str) -> TableHandle {
        let schema_id = SchemaTableId(schema_id.into());
        self.0.borrow_mut().tables.insert(schema_id.clone(), Table {
            id: id.into(),
            fields: BTreeMap::new(),
            indices: BTreeMap::new(),
            constraints: BTreeMap::new(),
        });
        TableHandle {
            version: self.clone(),
            schema_id: schema_id,
        }
    }
}

#[derive(Clone)]
pub struct TableHandle {
    pub version: VersionHandle,
    pub schema_id: SchemaTableId,
}

impl TableHandle {
    pub fn to_ref(&self) -> TableRef {
        TableRef(self.schema_id.clone())
    }

    pub fn field(&self, schema_id: &str, id: &str, type_: FieldType) -> FieldHandle {
        let field_schema_id = SchemaFieldId(schema_id.into());
        self
            .version
            .0
            .borrow_mut()
            .tables
            .get_mut(&self.schema_id)
            .unwrap()
            .fields
            .insert(field_schema_id.clone(), Field {
                id: id.into(),
                type_: type_,
            });
        FieldHandle {
            table: self.clone(),
            schema_id: field_schema_id,
        }
    }

    pub fn rowid_field(&self, id: Option<&str>) -> FieldHandle {
        let field_schema_id = SchemaFieldId("rowid".into());
        let id = id.unwrap_or("rowid");
        self
            .version
            .0
            .borrow_mut()
            .tables
            .get_mut(&self.schema_id)
            .unwrap()
            .fields
            .insert(field_schema_id.clone(), Field {
                id: id.into(),
                type_: FieldType {
                    type_: crate::sqlite::types::type_auto().build(),
                    migration_default: None,
                },
            });
        FieldHandle {
            table: self.clone(),
            schema_id: field_schema_id,
        }
    }

    pub fn index(&self, schema_id: &str, id: &str, fields: &[&FieldHandle]) -> IndexHandle {
        let index_schema_id = SchemaIndexId(schema_id.into());
        self
            .version
            .0
            .borrow_mut()
            .tables
            .get_mut(&self.schema_id)
            .unwrap()
            .indices
            .insert(index_schema_id.clone(), Index {
                id: id.into(),
                fields: fields.iter().map(|f| f.schema_id.clone()).collect(),
                unique: false,
            });
        IndexHandle {
            table: self.clone(),
            schema_id: index_schema_id,
        }
    }

    pub fn unique_index(&self, schema_id: &str, id: &str, fields: &[&FieldHandle]) -> IndexHandle {
        let index_schema_id = SchemaIndexId(schema_id.into());
        self
            .version
            .0
            .borrow_mut()
            .tables
            .get_mut(&self.schema_id)
            .unwrap()
            .indices
            .insert(index_schema_id.clone(), Index {
                id: id.into(),
                fields: fields.iter().map(|f| f.schema_id.clone()).collect(),
                unique: true,
            });
        IndexHandle {
            table: self.clone(),
            schema_id: index_schema_id,
        }
    }

    pub fn primary_key(&self, schema_id: &str, id: &str, fields: &[&FieldHandle]) -> ConstraintHandle {
        let constraint_schema_id = SchemaConstraintId(schema_id.into());
        self
            .version
            .0
            .borrow_mut()
            .tables
            .get_mut(&self.schema_id)
            .unwrap()
            .constraints
            .insert(constraint_schema_id.clone(), Constraint {
                id: id.into(),
                type_: ConstraintType::PrimaryKey(PrimaryKeyDef {
                    fields: fields.iter().map(|f| f.schema_id.clone()).collect(),
                }),
            });
        ConstraintHandle {
            table: self.clone(),
            schema_id: constraint_schema_id,
        }
    }

    pub fn foreign_key(
        &self,
        schema_id: &str,
        id: &str,
        fields: &[(&FieldHandle, &FieldHandle)],
    ) -> ConstraintHandle {
        let constraint_schema_id = SchemaConstraintId(schema_id.into());
        let remote_table = fields.get(0).unwrap().1.table.schema_id.clone();
        self
            .version
            .0
            .borrow_mut()
            .tables
            .get_mut(&self.schema_id)
            .unwrap()
            .constraints
            .insert(constraint_schema_id.clone(), Constraint {
                id: id.into(),
                type_: ConstraintType::ForeignKey(ForeignKeyDef {
                    remote_table: remote_table,
                    fields: fields.iter().map(|(l, r)| (l.schema_id.clone(), r.schema_id.clone())).collect(),
                }),
            });
        ConstraintHandle {
            table: self.clone(),
            schema_id: constraint_schema_id,
        }
    }
}

#[derive(Clone)]
pub struct FieldHandle {
    pub table: TableHandle,
    pub schema_id: SchemaFieldId,
}

impl FieldHandle {
    pub fn to_ref(&self) -> FieldRef {
        FieldRef {
            table_id: self.table.schema_id.clone(),
            field_id: self.schema_id.clone(),
        }
    }
}

pub struct IndexHandle {
    pub table: TableHandle,
    pub schema_id: SchemaIndexId,
}

pub struct ConstraintHandle {
    pub table: TableHandle,
    pub schema_id: SchemaConstraintId,
}

impl Version {
    pub(crate) fn to_migrate_nodes(&self) -> BTreeMap<GraphId, MigrateNode> {
        let mut out = BTreeMap::new();
        for (table_schema_id, table) in &self.tables {
            let table_graph_id = GraphId::Table(table_schema_id.clone());
            out.insert(table_graph_id.clone(), MigrateNode::new(vec![], Node::table(NodeTable_ {
                schema_id: table_schema_id.clone(),
                def: table.clone(),
            })));

            let mut local_field_sql_names = HashMap::new();
            for (field_schema_id, field) in &table.fields {
                local_field_sql_names.insert(field_schema_id.clone(), field.id.clone());
                let field_graph_id = GraphId::Field(table_schema_id.clone(), field_schema_id.clone());
                out.insert(field_graph_id, MigrateNode::new(vec![table_graph_id.clone()], Node::field(NodeField_ {
                    table_schema_id: table_schema_id.clone(),
                    table_id: table.id.clone(),
                    schema_id: field_schema_id.clone(),
                    def: field.clone(),
                })));
            }

            for (index_schema_id, index) in &table.indices {
                let mut deps = vec![table_graph_id.clone()];
                for f in &index.fields {
                    deps.push(GraphId::Field(table_schema_id.clone(), f.clone()));
                }
                out.insert(GraphId::Index(table_schema_id.clone(), index_schema_id.clone()), MigrateNode::new(
                    deps,
                    Node::table_index(NodeIndex_ {
                        table_schema_id: table_schema_id.clone(),
                        table_id: table.id.clone(),
                        schema_id: index_schema_id.clone(),
                        def: index.clone(),
                        field_sql_names: local_field_sql_names.clone(),
                    }),
                ));
            }

            for (constraint_schema_id, constraint) in &table.constraints {
                let mut deps = vec![table_graph_id.clone()];
                let mut remote_table_sql_name = None;
                let mut remote_field_sql_names = HashMap::new();

                match &constraint.type_ {
                    ConstraintType::PrimaryKey(x) => {
                        for f in &x.fields {
                            deps.push(GraphId::Field(table_schema_id.clone(), f.clone()));
                        }
                    },
                    ConstraintType::ForeignKey(x) => {
                        deps.push(GraphId::Table(x.remote_table.clone()));
                        remote_table_sql_name =
                            Some(self.tables.get(&x.remote_table).expect("Remote table not found").id.clone());
                        for (l, r) in &x.fields {
                            deps.push(GraphId::Field(table_schema_id.clone(), l.clone()));
                            deps.push(GraphId::Field(x.remote_table.clone(), r.clone()));
                            remote_field_sql_names.insert(
                                r.clone(),
                                self
                                    .tables
                                    .get(&x.remote_table)
                                    .unwrap()
                                    .fields
                                    .get(r)
                                    .expect("Remote field not found")
                                    .id
                                    .clone(),
                            );
                        }
                    },
                }

                out.insert(GraphId::Constraint(table_schema_id.clone(), constraint_schema_id.clone()), MigrateNode::new(
                    deps,
                    Node::table_constraint(NodeConstraint_ {
                        table_schema_id: table_schema_id.clone(),
                        table_sql_name: table.id.clone(),
                        schema_id: constraint_schema_id.clone(),
                        def: constraint.clone(),
                        local_field_sql_names: local_field_sql_names.clone(),
                        remote_table_sql_name,
                        remote_field_sql_names,
                    }),
                ));
            }
        }
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
    let mut errs = Errs::new();
    let mut migrations = vec![];
    let mut prev_version: Option<Version> = None;
    let mut prev_version_i: Option<i64> = None;
    let mut field_lookup: HashMap<TableRef, SqliteTableInfo> = HashMap::new();
    for (version_i, version) in versions {
        let path = rpds::vector![format!("Migration to {}", version_i)];
        let mut migration = vec![];

        // Prep for current version
        field_lookup.clear();
        for (table_schema_id, table) in &version.tables {
            let mut fields = HashMap::new();
            for (field_schema_id, field) in &table.fields {
                fields.insert(FieldRef {
                    table_id: table_schema_id.clone(),
                    field_id: field_schema_id.clone(),
                }, SqliteFieldInfo {
                    sql_name: field.id.clone(),
                    type_: field.type_.type_.clone(),
                });
            }
            field_lookup.insert(TableRef(table_schema_id.clone()), SqliteTableInfo {
                sql_name: table.id.clone(),
                fields: fields,
            });
        }

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

        // Main migrations
        {
            let mut state = SqliteMigrateCtx::new(errs.clone());
            let current_nodes = version.to_migrate_nodes();
            let prev_nodes = prev_version.take().map(|s| s.to_migrate_nodes());
            crate::graphmigrate::migrate(&mut state, prev_nodes, &current_nodes);
            for statement in &state.statements {
                migration.push(quote!{
                    {
                        let query = #statement;
                        db.execute(query, ()).await.to_good_error_query(query)?;
                    };
                });
            }
            errs = state.errs.clone();
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
            let mut ctx = SqliteQueryCtx::new(errs.clone(), &field_lookup);
            let res = QueryBody::build(q.body.as_ref(), &mut ctx, &path, q.res_count.clone());
            let ident = format_ident!("{}", q.name);
            let q_text = res.1.to_string();
            let args = ctx.rust_args.split_off(0);
            let args_forward = ctx.query_args.split_off(0);
            errs = ctx.errs.clone();
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
                    let mut unforward = quote!{
                        let x: #ident = r.get(#i);
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
            let db_arg = quote!(db: &mut impl good_ormning_runtime::sqlite::SqliteConnection);
            match q.res_count {
                QueryResCount::None => {
                    db_others.push(quote!{
                        pub async fn #ident(#db_arg, #(#args,) *) -> Result <(),
                        GoodError > {
                            let query = #q_text;
                            db.execute(query, (#(& #args_forward,) *)).await.to_good_error_query(query) ?;
                            Ok(())
                        }
                    });
                },
                QueryResCount::MaybeOne => {
                    if let Some(res_def) = res_def {
                        db_others.push(res_def);
                    }
                    db_others.push(quote!{
                        pub async fn #ident(#db_arg, #(#args,) *) -> Result < Option < #res_ident >,
                        GoodError > {
                            let query = #q_text;
                            let r = db.query_opt(query, (#(& #args_forward,) *)).await.to_good_error_query(query) ?;
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
                        pub async fn #ident(#db_arg, #(#args,) *) -> Result < #res_ident,
                        GoodError > {
                            let query = #q_text;
                            let r = db.query_one(query, (#(& #args_forward,) *)).await.to_good_error_query(query) ?;
                            Ok(#unforward_res)
                        }
                    });
                },
                QueryResCount::Many => {
                    if let Some(res_def) = res_def {
                        db_others.push(res_def);
                    }
                    db_others.push(quote!{
                        pub async fn #ident(#db_arg, #(#args,) *) -> Result < Vec < #res_ident >,
                        GoodError > {
                            let mut out = vec![];
                            let query = #q_text;
                            for r in db.query(query, (#(& #args_forward,) *)).await.to_good_error_query(query) ? {
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
        pub async fn migrate(db: &mut impl good_ormning_runtime::sqlite::SqliteConnection) -> Result <(),
        GoodError > {
            {
                let query =
                    "create table if not exists __good_version (rid int primary key, version bigint not null, lock int not null);";
                db.execute(query, ()).await.to_good_error_query(query)?;
            }
            {
                let query =
                    "insert into __good_version (rid, version, lock) values (0, -1, 0) on conflict do nothing;";
                db.execute(query, ()).await.to_good_error_query(query)?;
            }
            loop {
                let query =
                    "update __good_version set lock = 1 where rid = 0 and lock = 0 returning version";
                let version = match db.query_opt(query, ()).await.to_good_error_query(query)? {
                    Some(r) => {
                        let ver: i64 = r.get("version");
                        ver
                    },
                    None => {
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                        continue;
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
                #(#migrations) * {
                    let query = "update __good_version set version = ?, lock = 0";
                    db.execute(query, (#last_version_i,)).await.to_good_error_query(query) ?;
                }
                return Ok(());
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
        VersionHandle,
    };
    use super::{
        schema::field::{
            field_str,
            field_auto,
            field_i32,
        },
        generate,
        query::expr::Expr,
    };

    #[test]
    fn test_add_field_serial_bad() {
        assert!(generate(&PathBuf::from_str("/dev/null").unwrap(), vec![
            // Versions (previous)
            (0usize, {
                let v = VersionHandle::new();
                v.table("zMOY9YMCK", "bananna").field("z437INV6D", "hizat", field_str().build());
                v.0.borrow().clone()
            }),
            (1usize, {
                let v = VersionHandle::new();
                let bananna = v.table("zMOY9YMCK", "bananna");
                bananna.field("z437INV6D", "hizat", field_str().build());
                bananna.field("zPREUVAOD", "zomzom", field_auto().migrate_fill(Expr::LitAuto(0)).build());
                v.0.borrow().clone()
            })
        ], vec![]).is_err());
    }

    #[test]
    #[should_panic]
    fn test_add_field_dup_bad() {
        generate(&PathBuf::from_str("/dev/null").unwrap(), vec![
            // Versions (previous)
            (0usize, {
                let v = VersionHandle::new();
                v.table("zPAO2PJU4", "bananna").field("z437INV6D", "hizat", field_str().build());
                v.0.borrow().clone()
            }),
            (1usize, {
                let v = VersionHandle::new();
                let bananna = v.table("zQZQ8E2WD", "bananna");
                bananna.field("z437INV6D", "hizat", field_str().build());
                bananna.field("z437INV6D", "zomzom", field_i32().build());
                v.0.borrow().clone()
            })
        ], vec![]).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_add_table_dup_bad() {
        generate(&PathBuf::from_str("/dev/null").unwrap(), vec![
            // Versions (previous)
            (0usize, {
                let v = VersionHandle::new();
                v.table("zSNS34DYI", "bananna").field("z437INV6D", "hizat", field_str().build());
                v.0.borrow().clone()
            }),
            (1usize, {
                let v = VersionHandle::new();
                v.table("zSNS34DYI", "bananna").field("z437INV6D", "hizat", field_str().build());
                v.table("zSNS34DYI", "bananna").field("z437INV6D", "hizat", field_str().build());
                v.0.borrow().clone()
            })
        ], vec![]).unwrap();
    }

    #[test]
    fn test_res_count_none_bad() {
        let v = VersionHandle::new();
        let bananna = v.table("z5S18LWQE", "bananna");
        let hizat = bananna.field("z437INV6D", "hizat", field_str().build());
        assert!(
            generate(
                &PathBuf::from_str("/dev/null").unwrap(),
                vec![(0usize, v.0.borrow().clone())],
                vec![new_select(&bananna).return_field(&hizat).build_query("x", QueryResCount::None)],
            ).is_err()
        );
    }

    #[test]
    fn test_select_nothing_bad() {
        let v = VersionHandle::new();
        v.table("zOOR88EQ9", "bananna").field("z437INV6D", "hizat", field_str().build());
        let bananna = TableHandle {
            version: v.clone(),
            schema_id: SchemaTableId("zOOR88EQ9".into()),
        };
        assert!(
            generate(
                &PathBuf::from_str("/dev/null").unwrap(),
                vec![(0usize, v.0.borrow().clone())],
                vec![new_select(&bananna).build_query("x", QueryResCount::None)],
            ).is_err()
        );
    }

    #[test]
    fn test_returning_none_bad() {
        let v = VersionHandle::new();
        let bananna = v.table("zZPD1I2EF", "bananna");
        let hizat = bananna.field("z437INV6D", "hizat", field_str().build());
        assert!(
            generate(
                &PathBuf::from_str("/dev/null").unwrap(),
                vec![(0usize, v.0.borrow().clone())],
                vec![
                    new_insert(&bananna, vec![(hizat.clone(), Expr::LitString("hoy".into()))])
                        .return_field(&hizat)
                        .build_query("x", QueryResCount::None)
                ],
            ).is_err()
        );
    }
}
