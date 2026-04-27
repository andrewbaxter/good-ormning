use quote::{
    quote,
    format_ident,
};
use std::{
    collections::{
        HashMap,
        HashSet,
        BTreeMap,
    },
    path::Path,
    fs,
    env,
    rc::Rc,
    cell::RefCell,
};
use serde::{
    Serialize,
    Deserialize,
};
use crate::{
    pg::{
        types::Type,
        graph::utils::PgMigrateCtx,
    },
    utils::Errs,
};
pub use crate::pg::types::{
    Type as PgType,
    type_i32 as pg_type_i32,
    type_i64 as pg_type_i64,
    type_u32 as pg_type_u32,
    type_f32 as pg_type_f32,
    type_f64 as pg_type_f64,
    type_bool as pg_type_bool,
    type_bytes as pg_type_bytes,
    type_str as pg_type_str,
    type_utctime_s_chrono as pg_type_utctime_s_chrono,
    type_utctime_s_jiff as pg_type_utctime_s_jiff,
};
pub use crate::pg::schema::field::FieldTypeBuilder as PgFieldTypeBuilder;
pub use crate::QueryResCount;
use self::{
    query::{
        utils::{
            PgQueryCtx,
            QueryBody,
            PgTableInfo,
            PgFieldInfo,
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
        select_body::SelectBody,
        update::Update,
        delete::Delete,
    },
    schema::{
        field::{
            Field,
            FieldType,
            FieldRef,
        },
        table::{
            Table,
            TableRef,
        },
        constraint::{
            ConstraintType,
            Constraint,
            PrimaryKeyDef,
            ForeignKeyDef,
        },
        index::{
            Index,
        },
        custom_type::{
            CustomType,
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

/// This represents an SQL query. A function will be generated which accepts a db
/// connection and query parameters, and returns the query results. Call the
/// `new_*` functions to get a builder.
pub struct Query {
    pub name: String,
    pub body: Box<dyn QueryBody>,
    pub res_count: QueryResCount,
    pub res_name: Option<String>,
}
pub mod types;
pub mod query;
pub mod schema;
pub mod graph;

/// The number of results this query returns. This determines if the return type is
/// void, `Option`, the value directly, or a `Vec`. It must be a valid value per
/// the query body (e.g. select can't have `None` res count).
#[derive(Debug, Clone)]
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
        let sql_name =
            f
                .table
                .version
                .0
                .borrow()
                .as_ref()
                .unwrap()
                .tables
                .get(&f.table.id)
                .unwrap()
                .fields
                .get(&f.id)
                .unwrap()
                .id
                .clone();
        self.q.returning.push(Returning {
            e: Expr::Field(f.to_ref()),
            rename: Some(sql_name),
        });
        self
    }

    pub fn return_fields(mut self, f: &[&FieldHandle]) -> Self {
        for f in f {
            let sql_name =
                f
                    .table
                    .version
                    .0
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .tables
                    .get(&f.table.id)
                    .unwrap()
                    .fields
                    .get(&f.id)
                    .unwrap()
                    .id
                    .clone();
            self.q.returning.push(Returning {
                e: Expr::Field(f.to_ref()),
                rename: Some(sql_name),
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

pub struct SelectBodyBuilder {
    pub q: SelectBody,
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
        let sql_name =
            f
                .table
                .version
                .0
                .borrow()
                .as_ref()
                .unwrap()
                .tables
                .get(&f.table.id)
                .unwrap()
                .fields
                .get(&f.id)
                .unwrap()
                .id
                .clone();
        self.q.returning.push(Returning {
            e: Expr::Field(f.to_ref()),
            rename: Some(sql_name),
        });
        self
    }

    pub fn return_fields(mut self, f: &[&FieldHandle]) -> Self {
        for f in f {
            let sql_name =
                f
                    .table
                    .version
                    .0
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .tables
                    .get(&f.table.id)
                    .unwrap()
                    .fields
                    .get(&f.id)
                    .unwrap()
                    .id
                    .clone();
            self.q.returning.push(Returning {
                e: Expr::Field(f.to_ref()),
                rename: Some(sql_name),
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

    pub fn distinct(mut self) -> Self {
        self.q.distinct = true;
        self
    }

    pub fn limit(mut self, v: Expr) -> Self {
        self.q.limit = Some(v);
        self
    }

    pub fn build(self) -> SelectBody {
        self.q
    }
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
        let sql_name =
            f
                .table
                .version
                .0
                .borrow()
                .as_ref()
                .unwrap()
                .tables
                .get(&f.table.id)
                .unwrap()
                .fields
                .get(&f.id)
                .unwrap()
                .id
                .clone();
        self.q.returning.push(Returning {
            e: Expr::Field(f.to_ref()),
            rename: Some(sql_name),
        });
        self
    }

    pub fn return_fields(mut self, f: &[&FieldHandle]) -> Self {
        for f in f {
            let sql_name =
                f
                    .table
                    .version
                    .0
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .tables
                    .get(&f.table.id)
                    .unwrap()
                    .fields
                    .get(&f.id)
                    .unwrap()
                    .id
                    .clone();
            self.q.returning.push(Returning {
                e: Expr::Field(f.to_ref()),
                rename: Some(sql_name),
            });
        }
        self
    }

    pub fn returns_from_iter(mut self, f: impl Iterator<Item = Returning>) -> Self {
        self.q.returning.extend(f);
        self
    }

    pub fn with(mut self, with: self::query::utils::With) -> Self {
        self.q.with = Some(with);
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
        let sql_name =
            f
                .table
                .version
                .0
                .borrow()
                .as_ref()
                .unwrap()
                .tables
                .get(&f.table.id)
                .unwrap()
                .fields
                .get(&f.id)
                .unwrap()
                .id
                .clone();
        self.q.returning.push(Returning {
            e: Expr::Field(f.to_ref()),
            rename: Some(sql_name),
        });
        self
    }

    pub fn return_fields(mut self, f: &[&FieldHandle]) -> Self {
        for f in f {
            let sql_name =
                f
                    .table
                    .version
                    .0
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .tables
                    .get(&f.table.id)
                    .unwrap()
                    .fields
                    .get(&f.id)
                    .unwrap()
                    .id
                    .clone();
            self.q.returning.push(Returning {
                e: Expr::Field(f.to_ref()),
                rename: Some(sql_name),
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
        let sql_name =
            f
                .table
                .version
                .0
                .borrow()
                .as_ref()
                .unwrap()
                .tables
                .get(&f.table.id)
                .unwrap()
                .fields
                .get(&f.id)
                .unwrap()
                .id
                .clone();
        self.q.returning.push(Returning {
            e: Expr::Field(f.to_ref()),
            rename: Some(sql_name),
        });
        self
    }

    pub fn return_fields(mut self, f: &[&FieldHandle]) -> Self {
        for f in f {
            let sql_name =
                f
                    .table
                    .version
                    .0
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .tables
                    .get(&f.table.id)
                    .unwrap()
                    .fields
                    .get(&f.id)
                    .unwrap()
                    .id
                    .clone();
            self.q.returning.push(Returning {
                e: Expr::Field(f.to_ref()),
                rename: Some(sql_name),
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

/// This represents an SQL query. A function will be generated which accepts a db
/// connection and query parameters, and returns the query results. Call the
/// `new_*` functions to get a builder. Get a builder for an INSERT query.
///
/// # Arguments
///
/// * `values` - The fields to insert and their corresponding values
pub fn new_insert(table: &TableHandle, values: Vec<(FieldHandle, Expr)>) -> InsertBuilder {
    let mut unique = HashSet::new();
    for v in &values {
        if !unique.insert(v.0.id.clone()) {
            panic!("Duplicate field {:?} in insert", v.0.id);
        }
    }
    InsertBuilder { q: Insert {
        table: table.to_ref(),
        values: values.into_iter().map(|(f, e)| (f.to_ref(), e)).collect(),
        on_conflict: None,
        returning: vec![],
    } }
}

impl InsertBuilder {
    pub fn build_migration(self, version: &VersionHandle) -> String {
        let mut field_lookup: HashMap<TableRef, PgTableInfo> = HashMap::new();
        for (table_id, table) in &version.0.borrow().as_ref().unwrap().tables {
            let mut fields: HashMap<FieldRef, PgFieldInfo> = HashMap::new();
            for (field_id, field) in &table.fields {
                fields.insert(FieldRef {
                    table_id: table_id.clone(),
                    field_id: field_id.clone(),
                }, PgFieldInfo {
                    sql_name: field.id.clone(),
                    type_: field.type_.type_.clone(),
                });
            }
            field_lookup.insert(TableRef(table_id.clone()), PgTableInfo {
                sql_name: table.id.clone(),
                fields: fields,
            });
        }
        let mut ctx = PgQueryCtx::new(Errs::new(), field_lookup);
        let res = QueryBody::build(&self.q, &mut ctx, &rpds::vector![], QueryResCount::None);
        return res.1.to_string();
    }
}

/// Get a builder for a SELECT query.
pub fn new_select(table: &TableHandle) -> SelectBuilder {
    SelectBuilder { q: Select {
        with: None,
        table: NamedSelectSource {
            source: JoinSource::Table(table.to_ref()),
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

/// Get a builder for a SELECT query body. This allows using the query as a
/// subquery or join source.
pub fn new_select_body(table: &TableHandle) -> SelectBodyBuilder {
    SelectBodyBuilder { q: SelectBody {
        table: NamedSelectSource {
            source: JoinSource::Table(table.to_ref()),
            alias: None,
        },
        returning: vec![],
        join: vec![],
        where_: None,
        group: vec![],
        order: vec![],
        limit: None,
        distinct: false,
    } }
}

/// Get a builder for a SELECT query. This allows advanced sources (like selecting
/// from a synthetic table).
pub fn new_select_from(source: NamedSelectSource) -> SelectBuilder {
    SelectBuilder { q: Select {
        with: None,
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
pub fn new_update(table: &TableHandle, values: Vec<(FieldHandle, Expr)>) -> UpdateBuilder {
    let mut unique = HashSet::new();
    for v in &values {
        if !unique.insert(v.0.id.clone()) {
            panic!("Duplicate field {:?} in update", v.0.id);
        }
    }
    UpdateBuilder { q: Update {
        table: table.to_ref(),
        values: values.into_iter().map(|(f, e)| (f.to_ref(), e)).collect(),
        where_: None,
        returning: vec![],
    } }
}

impl UpdateBuilder {
    pub fn build_migration(self, version: &VersionHandle) -> String {
        let mut field_lookup: HashMap<TableRef, PgTableInfo> = HashMap::new();
        for (table_id, table) in &version.0.borrow().as_ref().unwrap().tables {
            let mut fields: HashMap<FieldRef, PgFieldInfo> = HashMap::new();
            for (field_id, field) in &table.fields {
                fields.insert(FieldRef {
                    table_id: table_id.clone(),
                    field_id: field_id.clone(),
                }, PgFieldInfo {
                    sql_name: field.id.clone(),
                    type_: field.type_.type_.clone(),
                });
            }
            field_lookup.insert(TableRef(table_id.clone()), PgTableInfo {
                sql_name: table.id.clone(),
                fields: fields,
            });
        }
        let mut ctx = PgQueryCtx::new(Errs::new(), field_lookup);
        let res = QueryBody::build(&self.q, &mut ctx, &rpds::vector![], QueryResCount::None);
        return res.1.to_string();
    }
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

impl DeleteBuilder {
    pub fn build_migration(self, version: &VersionHandle) -> String {
        let mut field_lookup: HashMap<TableRef, PgTableInfo> = HashMap::new();
        for (table_id, table) in &version.0.borrow().as_ref().unwrap().tables {
            let mut fields: HashMap<FieldRef, PgFieldInfo> = HashMap::new();
            for (field_id, field) in &table.fields {
                fields.insert(FieldRef {
                    table_id: table_id.clone(),
                    field_id: field_id.clone(),
                }, PgFieldInfo {
                    sql_name: field.id.clone(),
                    type_: field.type_.type_.clone(),
                });
            }
            field_lookup.insert(TableRef(table_id.clone()), PgTableInfo {
                sql_name: table.id.clone(),
                fields: fields,
            });
        }
        let mut ctx = PgQueryCtx::new(Errs::new(), field_lookup);
        let res = QueryBody::build(&self.q, &mut ctx, &rpds::vector![], QueryResCount::None);
        return res.1.to_string();
    }
}

/// The version represents the state of a schema at a point in time.
#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct Version {
    pub tables: BTreeMap<String, Table>,
    pub custom_types: BTreeMap<String, CustomType>,
}

impl Version {
    pub fn new() -> VersionHandle {
        VersionHandle(Rc::new(RefCell::new(Some(Version::default()))), Rc::new(std::cell::Cell::new(false)))
    }
}

#[derive(Clone)]
pub struct VersionHandle(pub Rc<RefCell<Option<Version>>>, pub Rc<std::cell::Cell<bool>>);

impl VersionHandle {
    fn with<R>(&self, f: impl FnOnce(&mut Version) -> R) -> R {
        if self.1.get() {
            panic!("Version already built");
        }
        let mut v = self.0.borrow_mut();
        f(v.as_mut().expect("Version already built"))
    }

    pub fn build(&self) -> Version {
        self.1.set(true);
        self.0.borrow().as_ref().expect("Version already built").clone()
    }

    pub fn table(&self, id: &str) -> TableHandle {
        self.with(|v| {
            if v.tables.contains_key(id) {
                panic!("Table {} already exists", id);
            }
            v.tables.insert(id.into(), Table {
                id: id.into(),
                renamed_from: None,
                fields: BTreeMap::new(),
                indices: BTreeMap::new(),
                constraints: BTreeMap::new(),
            });
        });
        TableHandle {
            version: self.clone(),
            id: id.into(),
        }
    }

    pub fn custom_type(&self, id: &str) -> CustomTypeBuilder {
        CustomTypeBuilder {
            version: self.clone(),
            id: id.into(),
        }
    }
}

pub struct CustomTypeBuilder {
    version: VersionHandle,
    id: String,
}

impl CustomTypeBuilder {
    pub fn rust_type(self, rust_type: &str) -> CustomTypeRustBuilder {
        CustomTypeRustBuilder {
            version: self.version,
            id: self.id,
            rust_type: rust_type.into(),
        }
    }
}

pub struct CustomTypeRustBuilder {
    version: VersionHandle,
    id: String,
    rust_type: String,
}

impl CustomTypeRustBuilder {
    pub fn base_type(self, base_type: Type) -> CustomTypeHandle {
        self.version.with(|v| {
            v.custom_types.insert(self.id.clone(), CustomType {
                id: self.id.clone(),
                renamed_from: None,
                rust_type: self.rust_type.clone(),
                base_type: base_type,
            });
        });
        CustomTypeHandle {
            version: self.version,
            id: self.id,
        }
    }
}

#[derive(Clone)]
pub struct CustomTypeHandle {
    pub version: VersionHandle,
    pub id: String,
}

impl CustomTypeHandle {
    pub fn field_type(&self) -> FieldType {
        let (rust_type, base_type) = self.version.with(|v| {
            let ct = v.custom_types.get(&self.id).expect("Custom type missing");
            (ct.rust_type.clone(), ct.base_type.clone())
        });
        FieldType {
            type_: Type {
                type_: crate::pg::types::SimpleType {
                    type_: base_type.type_.type_,
                    custom: Some(rust_type),
                },
                opt: base_type.opt,
                arr: base_type.arr,
            },
            migration_default: None,
        }
    }

    pub fn renamed_from(self, old_name: &str) -> Self {
        self.version.with(|v| {
            v.custom_types.get_mut(&self.id).unwrap().renamed_from = Some(old_name.into());
        });
        self
    }
}

#[derive(Clone)]
pub struct TableHandle {
    pub version: VersionHandle,
    pub id: String,
}

impl TableHandle {
    pub fn to_ref(&self) -> TableRef {
        TableRef(self.id.clone())
    }

    pub fn renamed_from(self, old_name: &str) -> Self {
        self.version.with(|v| {
            v.tables.get_mut(&self.id).unwrap().renamed_from = Some(old_name.into());
        });
        self
    }

    pub fn field(&self, id: &str, type_: FieldType) -> FieldHandle {
        self.version.with(|v| {
            let table = v.tables.get_mut(&self.id).unwrap();
            if table.fields.contains_key(id) {
                panic!("Field {} already exists on table {}", id, self.id);
            }
            table.fields.insert(id.into(), Field {
                id: id.into(),
                renamed_from: None,
                type_: type_,
            });
        });
        FieldHandle {
            table: self.clone(),
            id: id.into(),
        }
    }

    pub fn index(&self, id: &str, fields: &[&FieldHandle]) -> IndexHandle {
        self.version.with(|v| {
            v.tables.get_mut(&self.id).unwrap().indices.insert(id.into(), Index {
                id: id.into(),
                renamed_from: None,
                fields: fields.iter().map(|f| f.id.clone()).collect(),
                unique: false,
            });
        });
        IndexHandle {
            table: self.clone(),
            id: id.into(),
        }
    }

    pub fn unique_index(&self, id: &str, fields: &[&FieldHandle]) -> IndexHandle {
        self.version.with(|v| {
            v.tables.get_mut(&self.id).unwrap().indices.insert(id.into(), Index {
                id: id.into(),
                renamed_from: None,
                fields: fields.iter().map(|f| f.id.clone()).collect(),
                unique: true,
            });
        });
        IndexHandle {
            table: self.clone(),
            id: id.into(),
        }
    }

    pub fn primary_key(&self, id: &str, fields: &[&FieldHandle]) -> ConstraintHandle {
        self.version.with(|v| {
            v.tables.get_mut(&self.id).unwrap().constraints.insert(id.into(), Constraint {
                id: id.into(),
                renamed_from: None,
                type_: ConstraintType::PrimaryKey(
                    PrimaryKeyDef { fields: fields.iter().map(|f| f.id.clone()).collect() },
                ),
            });
        });
        ConstraintHandle {
            table: self.clone(),
            id: id.into(),
        }
    }

    pub fn foreign_key(&self, id: &str, fields: &[(&FieldHandle, &FieldHandle)]) -> ConstraintHandle {
        let remote_table = fields.get(0).unwrap().1.table.id.clone();
        self.version.with(|v| {
            v.tables.get_mut(&self.id).unwrap().constraints.insert(id.into(), Constraint {
                id: id.into(),
                renamed_from: None,
                type_: ConstraintType::ForeignKey(ForeignKeyDef {
                    remote_table: remote_table,
                    fields: fields.iter().map(|(l, r)| (l.id.clone(), r.id.clone())).collect(),
                }),
            });
        });
        ConstraintHandle {
            table: self.clone(),
            id: id.into(),
        }
    }
}

#[derive(Clone)]
pub struct FieldHandle {
    pub table: TableHandle,
    pub id: String,
}

impl FieldHandle {
    pub fn to_ref(&self) -> FieldRef {
        FieldRef {
            table_id: self.table.id.clone(),
            field_id: self.id.clone(),
        }
    }

    pub fn r#type(&self) -> FieldType {
        self.table.version.with(|v| {
            v.tables.get(&self.table.id).unwrap().fields.get(&self.id).unwrap().type_.clone()
        })
    }

    pub fn renamed_from(self, old_name: &str) -> Self {
        self.table.version.with(|v| {
            v.tables.get_mut(&self.table.id).unwrap().fields.get_mut(&self.id).unwrap().renamed_from =
                Some(old_name.into());
        });
        self
    }
}

pub struct IndexHandle {
    pub table: TableHandle,
    pub id: String,
}

impl IndexHandle {
    pub fn renamed_from(self, old_name: &str) -> Self {
        self.table.version.with(|v| {
            v.tables.get_mut(&self.table.id).unwrap().indices.get_mut(&self.id).unwrap().renamed_from =
                Some(old_name.into());
        });
        self
    }
}

pub struct ConstraintHandle {
    pub table: TableHandle,
    pub id: String,
}

impl ConstraintHandle {
    pub fn renamed_from(self, old_name: &str) -> Self {
        self.table.version.with(|v| {
            v.tables.get_mut(&self.table.id).unwrap().constraints.get_mut(&self.id).unwrap().renamed_from =
                Some(old_name.into());
        });
        self
    }
}

impl Version {
    pub(crate) fn to_migrate_nodes(&self) -> BTreeMap<GraphId, MigrateNode> {
        let mut out = BTreeMap::new();
        for (table_id, table) in &self.tables {
            let table_graph_id = GraphId::Table(table_id.clone());
            let table_renamed_from = table.renamed_from.clone();
            out.insert(table_graph_id.clone(), MigrateNode::new(vec![], Node::table(NodeTable_ {
                table_id: table_id.clone(),
                def: table.clone(),
            })));
            for (field_id, field) in &table.fields {
                let field_graph_id = GraphId::Field(table_id.clone(), field_id.clone());
                out.insert(field_graph_id, MigrateNode::new(vec![table_graph_id.clone()], Node::field(NodeField_ {
                    table_id: table_id.clone(),
                    table_renamed_from: table_renamed_from.clone(),
                    def: field.clone(),
                })));
            }
            for (index_id, index) in &table.indices {
                let mut deps = vec![table_graph_id.clone()];
                for f in &index.fields {
                    deps.push(GraphId::Field(table_id.clone(), f.clone()));
                }
                out.insert(
                    GraphId::Index(table_id.clone(), index_id.clone()),
                    MigrateNode::new(deps, Node::table_index(NodeIndex_ {
                        table_id: table_id.clone(),
                        table_renamed_from: table_renamed_from.clone(),
                        def: index.clone(),
                    })),
                );
            }
            for (constraint_id, constraint) in &table.constraints {
                let mut deps = vec![table_graph_id.clone()];
                match &constraint.type_ {
                    ConstraintType::PrimaryKey(x) => {
                        for f in &x.fields {
                            deps.push(GraphId::Field(table_id.clone(), f.clone()));
                        }
                    },
                    ConstraintType::ForeignKey(x) => {
                        deps.push(GraphId::Table(x.remote_table.clone()));
                        for (l, r) in &x.fields {
                            deps.push(GraphId::Field(table_id.clone(), l.clone()));
                            deps.push(GraphId::Field(x.remote_table.clone(), r.clone()));
                        }
                    },
                }
                out.insert(
                    GraphId::Constraint(table_id.clone(), constraint_id.clone()),
                    MigrateNode::new(deps, Node::table_constraint(NodeConstraint_ {
                        table_id: table_id.clone(),
                        table_renamed_from: table_renamed_from.clone(),
                        def: constraint.clone(),
                    })),
                );
            }
        }
        return out;
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
pub fn generate(
    db_name: Option<&str>,
    versions: Vec<(usize, Version)>,
    queries: Vec<Query>,
) -> Result<(), Vec<String>> {
    let db_name = db_name.unwrap_or("default");
    let out_dir = env::var("OUT_DIR").map_err(|e| vec![format!("OUT_DIR not set: {:?}", e)])?;
    let out_dir = Path::new(&out_dir);
    let output = out_dir.join(format!("good_ormning_pg_{}.rs", db_name));
    let json_dir = out_dir.join("good_ormning");
    if let Err(e) = fs::create_dir_all(&json_dir) {
        return Err(vec![format!("Error creating directory {:?}: {:?}", json_dir, e)]);
    }
    let json_path = json_dir.join(format!("pg_{}.json", db_name));

    // Serialize versions for proc macro
    {
        eprintln!("DEBUG: Writing versions to {:?}", json_path);
        let mut versions_map: HashMap<usize, Version> = if json_path.exists() {
            serde_json::from_str(&fs::read_to_string(&json_path).unwrap()).unwrap_or_default()
        } else {
            HashMap::new()
        };
        for (version_i, version) in versions.iter() {
            let entry = versions_map.entry(*version_i).or_insert_with(|| Version::default());
            for (k, v) in &version.tables {
                entry.tables.insert(k.clone(), v.clone());
            }
            for (k, v) in &version.custom_types {
                entry.custom_types.insert(k.clone(), v.clone());
            }
        }
        let _ = fs::write(json_path, serde_json::to_string(&versions_map).unwrap());
    }
    let mut errs = Errs::new();
    let mut migrations = vec![];
    let mut prev_version: Option<Version> = None;
    let mut prev_version_i: Option<i64> = None;
    let mut field_lookup: HashMap<TableRef, PgTableInfo> = HashMap::new();
    for (version_i, version) in &versions {
        let path = rpds::vector![format!("Migration to {}", version_i)];
        let mut migration = vec![];

        // Prep for current version
        field_lookup.clear();
        for (table_id, table) in &version.tables {
            let mut fields: HashMap<FieldRef, PgFieldInfo> = HashMap::new();
            for (field_id, field) in &table.fields {
                fields.insert(FieldRef {
                    table_id: table_id.clone(),
                    field_id: field_id.clone(),
                }, PgFieldInfo {
                    sql_name: field.id.clone(),
                    type_: field.type_.type_.clone(),
                });
            }
            field_lookup.insert(TableRef(table_id.clone()), PgTableInfo {
                sql_name: table.id.clone(),
                fields: fields,
            });
        }
        let version_i = *version_i as i64;
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
            let mut table_sql_names = HashMap::new();
            for (table_id, table) in &version.tables {
                table_sql_names.insert(table_id.clone(), table.id.clone());
            }
            let mut state = PgMigrateCtx::new(errs.clone(), table_sql_names, version.clone());
            let current_nodes = version.to_migrate_nodes();
            let prev_nodes = prev_version.take().map(|s| s.to_migrate_nodes());
            crate::graphmigrate::migrate(&mut state, prev_nodes, &current_nodes);
            for statement in &state.statements {
                migration.push(quote!{
                    {
                        let query = #statement;
                        txn.execute(query, &[]).await.to_good_error_query(query)?;
                    };
                });
            }
            errs = state.errs.clone();
        }

        // Build migration
        let pascal_db_name: String = db_name.split('_').map(|s| {
            let mut c = s.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        }).collect();
        let enum_name = format_ident!("Db{}Versions", pascal_db_name);
        let newtype_name = format_ident!("Db{}{}", pascal_db_name, version_i as usize);
        let enum_variant = format_ident!("V{}", version_i as usize);
        migrations.push(quote!{
            if version < #version_i {
                #(#migration) * {
                    let query = "update __good_version set version = $1";
                    good_ormning:: runtime:: pg:: PgConnection:: execute(
                        &mut txn,
                        query,
                        &[& #version_i]
                    ).await.to_good_error_query(query) ?;
                }
                callback(#enum_name::#enum_variant(#newtype_name(&mut txn))).await ?;
            }
        });

        // Next iter prep
        prev_version = Some(version.clone());
        prev_version_i = Some(version_i);
    }

    // Compile, output
    let last_version_i = prev_version_i.unwrap() as i64;
    let pascal_db_name: String = db_name.split('_').map(|s| {
        let mut c = s.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        }
    }).collect();
    let enum_name = format_ident!("Db{}Versions", pascal_db_name);
    let mut enum_variants = vec![];
    let mut db_types = vec![];
    for (version_i, _) in &versions {
        let newtype_name = format_ident!("Db{}{}", pascal_db_name, version_i);
        let enum_variant = format_ident!("V{}", version_i);
        enum_variants.push(quote!(#enum_variant(#newtype_name <'a >)));
        db_types.push(quote!{
            pub struct #newtype_name <'a >(pub &'a mut dyn good_ormning:: runtime:: pg:: PgConnection);
        });
    }
    let latest_newtype_name = format_ident!("Db{}{}", pascal_db_name, last_version_i as usize);
    let db_alias_name = format_ident!("Db{}", pascal_db_name);
    let db_others =
        self::query::generate::generate_query_functions(
            &mut errs,
            field_lookup,
            queries,
            output.file_stem().unwrap().to_str().unwrap(),
            quote!(#db_alias_name <'_ >),
        );
    let tokens = quote!{
        use good_ormning::runtime::GoodError;
        use good_ormning::runtime::ToGoodError;
        #(#db_types) * pub enum #enum_name <'a > {
            #(#enum_variants,) *
        }
        pub use #latest_newtype_name as #db_alias_name;
        async fn init_db(db: & mut impl good_ormning:: runtime:: pg:: PgConnection) -> Result <(),
        GoodError > {
            {
                let query =
                    "create table if not exists __good_version (rid int primary key, version bigint not null, lock int not null);";
                good_ormning::runtime::pg::PgConnection::execute(db, query, &[]).await.to_good_error_query(query)?;
            }
            {
                let query =
                    "insert into __good_version (rid, version, lock) values (0, -1, 0) on conflict do nothing;";
                good_ormning::runtime::pg::PgConnection::execute(db, query, &[]).await.to_good_error_query(query)?;
            }
            Ok(())
        }
        pub async fn migrate < F >(db: & mut tokio_postgres:: Client, callback: F) -> Result <(),
        GoodError > where F: for <'b > Fn(
            #enum_name <'b >
        ) -> std:: pin:: Pin < Box < dyn std:: future:: Future < Output = Result <(),
        GoodError >> + Send + 'b >> {
            init_db(db).await?;
            loop {
                let mut txn = db.transaction().await.to_good_error(|| "Failed to start transaction".to_string())?;
                let migrated = {
                    let query = "update __good_version set lock = 1 where rid = 0 and lock = 0 returning version";
                    let res =
                        good_ormning::runtime::pg::PgConnection::query(&mut txn, query, &[])
                            .await
                            .to_good_error_query(query)?;
                    let version = match res.first() {
                        Some(r) => {
                            let ver: i64 = r.get(0usize);
                            Some(ver)
                        },
                        None => {
                            None
                        },
                    };
                    if let Some(version) = version {
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
                            let query = "update __good_version set lock = 0";
                            good_ormning::runtime::pg::PgConnection::execute(&mut txn, query, &[])
                                .await
                                .to_good_error_query(query)?;
                        }
                        true
                    }
                    else {
                        false
                    }
                };
                if migrated {
                    txn.commit().await.to_good_error(|| "Failed to commit transaction".to_string())?;
                    return Ok(());
                }
                else {
                    txn.rollback().await.to_good_error(|| "Failed to rollback transaction".to_string())?;
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            }
        }
        pub async fn get_schema_version(
            db: & mut impl good_ormning:: runtime:: pg:: PgConnection
        ) -> Result < Option < i64 >,
        GoodError > {
            init_db(db).await?;
            let query = "select version from __good_version where rid = 0";
            let res = db.query(query, &[]).await.to_good_error_query(query)?;
            if let Some(r) = res.first() {
                let x: i64 = r.get(0usize);
                if x == -1 {
                    return Ok(None);
                } else {
                    return Ok(Some(x));
                }
            }
            Ok(None)
        }
        #(#db_others) *
    };
    match genemichaels_lib::format_str(&tokens.to_string(), &genemichaels_lib::FormatConfig::default()) {
        Ok(src) => {
            match fs::write(&output, src.rendered.as_bytes()) {
                Ok(_) => { },
                Err(e) => errs.err(
                    &rpds::vector![],
                    format!("Failed to write generated code to {:?}: {:?}", output, e),
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
    use crate::pg::{
        new_select,
        QueryResCount,
        new_insert,
        VersionHandle,
        TableHandle,
    };
    use super::{
        schema::field::{
            field_str,
            field_auto,
            field_i32,
        },
        generate,
        query::expr::{
            Expr,
            SerialExpr,
        },
        Version,
    };

    #[test]
    fn test_add_field_serial_bad() {
        assert!(generate(None, vec![
            // Versions (previous)
            (0usize, {
                let v = Version::new();
                v.table("zMOY9YMCK").field("z437INV6D", field_str().build());
                v.build()
            }),
            (1usize, {
                let v = Version::new();
                let bananna = v.table("zMOY9YMCK");
                bananna.field("z437INV6D", field_str().build());
                bananna.field("zPREUVAOD", field_auto().migrate_fill(SerialExpr::LitAuto(0)).build());
                v.build()
            })
        ], vec![]).is_err());
    }

    #[test]
    #[should_panic]
    fn test_add_field_dup_bad() {
        generate(None, vec![
            // Versions (previous)
            (0usize, {
                let v = Version::new();
                v.table("zPAO2PJU4").field("z437INV6D", field_str().build());
                v.build()
            }),
            (1usize, {
                let v = Version::new();
                let bananna = v.table("zQZQ8E2WD");
                bananna.field("z437INV6D", field_str().build());
                bananna.field("z437INV6D", field_i32().build());
                v.build()
            })
        ], vec![]).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_add_table_dup_bad() {
        generate(None, vec![
            // Versions (previous)
            (0usize, {
                let v = Version::new();
                v.table("zSNS34DYI").field("z437INV6D", field_str().build());
                v.build()
            }),
            (1usize, {
                let v = Version::new();
                v.table("zSNS34DYI").field("z437INV6D", field_str().build());
                v.table("zSNS34DYI").field("z437INV6D", field_str().build());
                v.build()
            })
        ], vec![]).unwrap();
    }

    #[test]
    fn test_res_count_none_bad() {
        let v = Version::new();
        let bananna = v.table("z5S18LWQE");
        let hizat = bananna.field("z437INV6D", field_str().build());
        assert!(
            generate(
                None,
                vec![(0usize, v.build())],
                vec![new_select(&bananna).return_field(&hizat).build_query("x", QueryResCount::None)],
            ).is_err()
        );
    }

    #[test]
    fn test_select_nothing_bad() {
        let v = Version::new();
        v.table("zOOR88EQ9").field("z437INV6D", field_str().build());
        let bananna = TableHandle {
            version: v.clone(),
            id: "zOOR88EQ9".into(),
        };
        assert!(
            generate(
                None,
                vec![(0usize, v.build())],
                vec![new_select(&bananna).build_query("x", QueryResCount::None)],
            ).is_err()
        );
    }

    #[test]
    fn test_returning_none_bad() {
        let v = Version::new();
        let bananna = v.table("zZPD1I2EF");
        let hizat = bananna.field("z437INV6D", field_str().build());
        assert!(
            generate(
                None,
                vec![(0usize, v.build())],
                vec![
                    new_insert(&bananna, vec![(hizat.clone(), Expr::LitString("hoy".into()))])
                        .return_field(&hizat)
                        .build_query("x", QueryResCount::None)
                ],
            ).is_err()
        );
    }
}
