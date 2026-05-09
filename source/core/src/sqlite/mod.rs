use {
    crate::{
        sqlite::{
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
                    InsertSource,
                },
                select::{
                    Join,
                    JoinSource,
                    NamedSelectSource,
                    Order,
                    Select,
                },
                update::Update,
                utils::{
                    QueryBody,
                    Returning,
                    SqliteFieldInfo,
                    SqliteQueryCtx,
                    SqliteTableInfo,
                },
            },
            schema::{
                constraint::{
                    Constraint,
                    ConstraintType,
                    ForeignKeyDef,
                    PrimaryKeyDef,
                },
                custom_type::CustomType,
                field::{
                    Field,
                    FieldRef,
                    FieldType,
                },
                table::{
                    Table,
                    TableRef,
                },
                index::Index,
            },
            types::Type,
        },
        utils::Errs,
        QueryResCount,
    },
    serde::{
        Deserialize,
        Serialize,
    },
    std::{
        cell::RefCell,
        collections::{
            BTreeMap,
            HashMap,
            HashSet,
        },
        rc::Rc,
    },
};
pub use crate::sqlite::{
    schema::field::FieldTypeBuilder as SqliteFieldTypeBuilder,
    types::{
        type_bool as sqlite_type_bool,
        type_bytes as sqlite_type_bytes,
        type_f32 as sqlite_type_f32,
        type_f64 as sqlite_type_f64,
        type_i32 as sqlite_type_i32,
        type_i64 as sqlite_type_i64,
        type_str as sqlite_type_str,
        type_u32 as sqlite_type_u32,
        Type as SqliteType,
    },
};
#[cfg(feature = "chrono")]
pub use crate::sqlite::types::type_utctime_s_chrono as sqlite_type_utctime_s_chrono;
#[cfg(feature = "jiff")]
pub use crate::sqlite::types::type_utctime_s_jiff as sqlite_type_utctime_s_jiff;

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

    pub fn build_query(self, name: impl ToString, res_count: QueryResCount) -> Query {
        Query {
            name: name.to_string(),
            body: Box::new(self.q),
            res_count: res_count,
            res_name: None,
        }
    }

    pub fn build_query_named_res(
        self,
        name: impl ToString,
        res_count: QueryResCount,
        res_name: impl ToString,
    ) -> Query {
        Query {
            name: name.to_string(),
            body: Box::new(self.q),
            res_count: res_count,
            res_name: Some(res_name.to_string()),
        }
    }
}

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

    pub fn limit(mut self, v: Expr) -> Self {
        self.q.limit = Some(v);
        self
    }

    pub fn junction(mut self, junction: self::query::select_body::SelectJunction) -> Self {
        self.q.junction.push(junction);
        self
    }

    pub fn build_query(self, name: impl ToString, res_count: QueryResCount) -> Query {
        Query {
            name: name.to_string(),
            body: Box::new(self.q),
            res_count: res_count,
            res_name: None,
        }
    }

    pub fn build_query_named_res(
        self,
        name: impl ToString,
        res_count: QueryResCount,
        res_name: impl ToString,
    ) -> Query {
        Query {
            name: name.to_string(),
            body: Box::new(self.q),
            res_count: res_count,
            res_name: Some(res_name.to_string()),
        }
    }
}

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

    pub fn build_query(self, name: impl ToString, res_count: QueryResCount) -> Query {
        Query {
            name: name.to_string(),
            body: Box::new(self.q),
            res_count: res_count,
            res_name: None,
        }
    }

    pub fn build_query_named_res(
        self,
        name: impl ToString,
        res_count: QueryResCount,
        res_name: impl ToString,
    ) -> Query {
        Query {
            name: name.to_string(),
            body: Box::new(self.q),
            res_count: res_count,
            res_name: Some(res_name.to_string()),
        }
    }
}

pub struct DeleteBuilder {
    pub q: Delete,
}

impl DeleteBuilder {
    pub fn with(mut self, w: crate::sqlite::query::utils::With) -> Self {
        self.q.with = Some(w);
        self
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

    pub fn build_query(self, name: impl ToString, res_count: QueryResCount) -> Query {
        Query {
            name: name.to_string(),
            body: Box::new(self.q),
            res_count: res_count,
            res_name: None,
        }
    }

    pub fn build_query_named_res(
        self,
        name: impl ToString,
        res_count: QueryResCount,
        res_name: impl ToString,
    ) -> Query {
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
    SelectBodyBuilder { q: self::query::select_body::SelectBody {
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
        junctions: vec![],
    } }
}

pub fn new_select_body_from(source: self::query::select_body::NamedSelectSource) -> SelectBodyBuilder {
    SelectBodyBuilder { q: self::query::select_body::SelectBody {
        table: source,
        distinct: false,
        returning: vec![],
        join: vec![],
        where_: None,
        group: vec![],
        order: vec![],
        limit: None,
        junctions: vec![],
    } }
}

pub fn new_insert(table: &TableHandle, values: Vec<(FieldHandle, Expr)>) -> InsertBuilder {
    let mut unique = HashSet::new();
    for v in &values {
        if !unique.insert(v.0.id.clone()) {
            panic!("Duplicate field {:?} in insert", v.0.id);
        }
    }
    InsertBuilder { q: Insert {
        table: table.to_ref(),
        source: InsertSource::Values(values.into_iter().map(|(f, e)| (f.to_ref(), e)).collect()),
        on_conflict: None,
        returning: vec![],
    } }
}

impl InsertBuilder {
    pub fn build_migration(self, version: &VersionHandle) -> String {
        let mut field_lookup: HashMap<TableRef, SqliteTableInfo> = HashMap::new();
        for (table_id, table) in &version.0.borrow().as_ref().unwrap().tables {
            let mut fields: HashMap<FieldRef, SqliteFieldInfo> = HashMap::new();
            for (field_id, field) in &table.fields {
                fields.insert(FieldRef {
                    table_id: table_id.clone(),
                    field_id: field_id.clone(),
                }, SqliteFieldInfo {
                    sql_name: field.id.clone(),
                    type_: field.type_.type_.clone(),
                });
            }
            field_lookup.insert(TableRef(table_id.clone()), SqliteTableInfo {
                sql_name: table.id.clone(),
                fields: fields,
            });
        }
        let mut ctx = SqliteQueryCtx::new(Errs::new(), field_lookup);
        let res = QueryBody::build(&self.q, &mut ctx, &rpds::vector![], QueryResCount::None);
        return res.1.to_string();
    }
}

pub fn new_select(table: &TableHandle) -> SelectBuilder {
    SelectBuilder { q: Select {
        with: None,
        table: NamedSelectSource {
            source: JoinSource::Table(table.to_ref()),
            alias: None,
            index_hint: None,
        },
        returning: vec![],
        junction: vec![],
        join: vec![],
        where_: None,
        group: vec![],
        having: None,
        order: vec![],
        limit: None,
        distinct: false,
    } }
}

pub fn new_select_from(source: NamedSelectSource) -> SelectBuilder {
    SelectBuilder { q: Select {
        with: None,
        table: source,
        returning: vec![],
        junction: vec![],
        join: vec![],
        where_: None,
        group: vec![],
        having: None,
        order: vec![],
        limit: None,
        distinct: false,
    } }
}

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
        index_hint: None,
    } }
}

impl UpdateBuilder {
    pub fn build_migration(self, version: &VersionHandle) -> String {
        let mut field_lookup: HashMap<TableRef, SqliteTableInfo> = HashMap::new();
        for (table_id, table) in &version.0.borrow().as_ref().unwrap().tables {
            let mut fields: HashMap<FieldRef, SqliteFieldInfo> = HashMap::new();
            for (field_id, field) in &table.fields {
                fields.insert(FieldRef {
                    table_id: table_id.clone(),
                    field_id: field_id.clone(),
                }, SqliteFieldInfo {
                    sql_name: field.id.clone(),
                    type_: field.type_.type_.clone(),
                });
            }
            field_lookup.insert(TableRef(table_id.clone()), SqliteTableInfo {
                sql_name: table.id.clone(),
                fields: fields,
            });
        }
        let mut ctx = SqliteQueryCtx::new(Errs::new(), field_lookup);
        let res = QueryBody::build(&self.q, &mut ctx, &rpds::vector![], QueryResCount::None);
        return res.1.to_string();
    }
}

pub fn new_delete(table: &TableHandle) -> DeleteBuilder {
    DeleteBuilder { q: Delete {
        with: None,
        table: table.to_ref(),
        returning: vec![],
        where_: None,
        index_hint: None,
    } }
}

impl DeleteBuilder {
    pub fn build_migration(self, version: &VersionHandle) -> String {
        let mut field_lookup: HashMap<TableRef, SqliteTableInfo> = HashMap::new();
        for (table_id, table) in &version.0.borrow().as_ref().unwrap().tables {
            let mut fields: HashMap<FieldRef, SqliteFieldInfo> = HashMap::new();
            for (field_id, field) in &table.fields {
                fields.insert(FieldRef {
                    table_id: table_id.clone(),
                    field_id: field_id.clone(),
                }, SqliteFieldInfo {
                    sql_name: field.id.clone(),
                    type_: field.type_.type_.clone(),
                });
            }
            field_lookup.insert(TableRef(table_id.clone()), SqliteTableInfo {
                sql_name: table.id.clone(),
                fields: fields,
            });
        }
        let mut ctx = SqliteQueryCtx::new(Errs::new(), field_lookup);
        let res = QueryBody::build(&self.q, &mut ctx, &rpds::vector![], QueryResCount::None);
        return res.1.to_string();
    }
}

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct Version {
    pub tables: BTreeMap<String, Table>,
    pub custom_types: BTreeMap<String, CustomType>,
}

impl Version {
    #[allow(clippy::new_ret_no_self)]
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
    pub version: VersionHandle,
    pub id: String,
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
    pub version: VersionHandle,
    pub id: String,
    pub rust_type: String,
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
                type_: crate::sqlite::types::SimpleType {
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

    pub fn rowid_field(&self, id: Option<&str>) -> FieldHandle {
        let field_id = "rowid";
        let sql_id = id.unwrap_or("rowid");
        self.version.with(|v| {
            v.tables.get_mut(&self.id).unwrap().fields.insert(field_id.into(), Field {
                id: sql_id.into(),
                renamed_from: None,
                type_: FieldType {
                    type_: crate::sqlite::types::type_auto().build(),
                    migration_default: None,
                },
            });
        });
        FieldHandle {
            table: self.clone(),
            id: field_id.into(),
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
        let remote_table = fields.first().unwrap().1.table.id.clone();
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
    pub fn to_migrate_nodes(&self) -> BTreeMap<GraphId, MigrateNode> {
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
