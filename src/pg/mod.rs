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
    collections::HashMap,
    path::Path,
    fs,
};
use crate::{
    pg::{
        types::Type,
        queries::expr::ExprValName,
    },
    utils::Errs,
};
use self::{
    queries::{
        utils::{
            PgQueryCtx,
            QueryBody,
        },
        insert::{
            Insert,
            InsertConflict,
        },
        expr::Expr,
        select::{
            SelectOutput,
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
        node::{
            Id,
            Node,
        },
        utils::{
            MigrateNode,
            PgMigrateCtx,
        },
        table::{
            TableId,
            NodeTable_,
        },
        field::{
            FieldId,
            FieldDef,
            NodeField_,
            FieldType,
        },
        index::{
            IndexId,
            IndexDef,
            NodeIndex_,
        },
        constraint::{
            ConstraintDef,
            ConstraintId,
            ConstraintType,
            NodeConstraint_,
        },
    },
};

pub mod types;
pub mod queries;
pub mod schema;
pub mod utils;

/// The number of results this query returns. This determines if the return type is
/// void, `Option`, the value directly, or a `Vec`. It must be a valid value per the
/// query body (e.g. select can't have `None` res count).
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

    pub fn returning(mut self, v: Expr) -> Self {
        self.q.returning.push(SelectOutput {
            e: v,
            rename: None,
        });
        self
    }

    pub fn named_returning(mut self, name: impl ToString, v: Expr) -> Self {
        self.q.returning.push(SelectOutput {
            e: v,
            rename: Some(name.to_string()),
        });
        self
    }

    pub fn returning_field(mut self, f: &Field) -> Self {
        self.q.returning.push(SelectOutput {
            e: Expr::Field(f.id.clone()),
            rename: None,
        });
        self
    }

    pub fn returning_fields(mut self, f: &[&Field]) -> Self {
        for f in f {
            self.q.returning.push(SelectOutput {
                e: Expr::Field(f.id.clone()),
                rename: None,
            });
        }
        self
    }

    pub fn returning_from_iter(mut self, f: impl Iterator<Item = SelectOutput>) -> Self {
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
    pub fn output(mut self, v: Expr) -> Self {
        self.q.output.push(SelectOutput {
            e: v,
            rename: None,
        });
        self
    }

    pub fn named_output(mut self, name: impl ToString, v: Expr) -> Self {
        self.q.output.push(SelectOutput {
            e: v,
            rename: Some(name.to_string()),
        });
        self
    }

    pub fn output_field(mut self, f: &Field) -> Self {
        self.q.output.push(SelectOutput {
            e: Expr::Field(f.id.clone()),
            rename: None,
        });
        self
    }

    pub fn output_fields(mut self, f: &[&Field]) -> Self {
        for f in f {
            self.q.output.push(SelectOutput {
                e: Expr::Field(f.id.clone()),
                rename: None,
            });
        }
        self
    }

    pub fn output_from_iter(mut self, f: impl Iterator<Item = SelectOutput>) -> Self {
        self.q.output.extend(f);
        self
    }

    pub fn join(mut self, v: Join) -> Self {
        self.q.join.push(v);
        self
    }

    pub fn where_(mut self, v: Expr) -> Self {
        self.q.where_ = Some(v);
        self
    }

    pub fn group(mut self, v: Vec<Expr>) -> Self {
        self.q.group = v;
        self
    }

    pub fn order(mut self, v: Vec<(Expr, Order)>) -> Self {
        self.q.order = v;
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

    pub fn returning(mut self, v: Expr) -> Self {
        self.q.returning.push(SelectOutput {
            e: v,
            rename: None,
        });
        self
    }

    pub fn named_returning(mut self, name: impl ToString, v: Expr) -> Self {
        self.q.returning.push(SelectOutput {
            e: v,
            rename: Some(name.to_string()),
        });
        self
    }

    pub fn returning_field(mut self, f: &Field) -> Self {
        self.q.returning.push(SelectOutput {
            e: Expr::Field(f.id.clone()),
            rename: None,
        });
        self
    }

    pub fn returning_fields(mut self, f: &[&Field]) -> Self {
        for f in f {
            self.q.returning.push(SelectOutput {
                e: Expr::Field(f.id.clone()),
                rename: None,
            });
        }
        self
    }

    pub fn returning_from_iter(mut self, f: impl Iterator<Item = SelectOutput>) -> Self {
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

    pub fn returning(mut self, v: Expr) -> Self {
        self.q.returning.push(SelectOutput {
            e: v,
            rename: None,
        });
        self
    }

    pub fn named_returning(mut self, name: impl ToString, v: Expr) -> Self {
        self.q.returning.push(SelectOutput {
            e: v,
            rename: Some(name.to_string()),
        });
        self
    }

    pub fn returning_field(mut self, f: &Field) -> Self {
        self.q.returning.push(SelectOutput {
            e: Expr::Field(f.id.clone()),
            rename: None,
        });
        self
    }

    pub fn returning_fields(mut self, f: &[&Field]) -> Self {
        for f in f {
            self.q.returning.push(SelectOutput {
                e: Expr::Field(f.id.clone()),
                rename: None,
            });
        }
        self
    }

    pub fn returning_from_iter(mut self, f: impl Iterator<Item = SelectOutput>) -> Self {
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
pub fn new_insert(table: &Table, values: Vec<(FieldId, Expr)>) -> InsertBuilder {
    InsertBuilder { q: Insert {
        table: table.0.clone(),
        values: values,
        on_conflict: None,
        returning: vec![],
    } }
}

/// Get a builder for a SELECT query.
pub fn new_select(table: &Table) -> SelectBuilder {
    SelectBuilder { q: Select {
        table: NamedSelectSource {
            source: JoinSource::Table(table.0.clone()),
            alias: None,
        },
        output: vec![],
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
        output: vec![],
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
pub fn new_update(table: &Table, values: Vec<(FieldId, Expr)>) -> UpdateBuilder {
    UpdateBuilder { q: Update {
        table: table.0.clone(),
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
        table: table.0.clone(),
        returning: vec![],
        where_: None,
    } }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Table(pub TableId);

#[derive(Clone)]
pub struct Field {
    pub id: FieldId,
    pub def: FieldDef,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Index(pub IndexId);

/// The version represents the state of a schema at a point in time.
#[derive(Default)]
pub struct Version {
    schema: HashMap<Id, MigrateNode>,
    pre_migration: Vec<Box<dyn QueryBody>>,
    post_migration: Vec<Box<dyn QueryBody>>,
}

impl Version {
    /// Define a table in the version
    pub fn table(&mut self, id: &str) -> Table {
        let out = Table(TableId(id.into()));
        if self.schema.insert(Id::Table(out.0.clone()), MigrateNode::new(vec![], Node::table(NodeTable_ {
            id: out.0.clone(),
            fields: vec![],
        }))).is_some() {
            panic!("Table with id {} already exists", out.0);
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
    pub fn field(&self, v: &mut Version, id: impl ToString, name: impl ToString, type_: FieldType) -> Field {
        let out_id = FieldId(self.0.clone(), id.to_string());
        let d = FieldDef {
            name: name.to_string(),
            type_: type_,
        };
        if v
            .schema
            .insert(
                Id::Field(out_id.clone()),
                MigrateNode::new(vec![Id::Table(self.0.clone())], Node::field(NodeField_ {
                    id: out_id.clone(),
                    def: d.clone(),
                })),
            )
            .is_some() {
            panic!("Field with id {} already exists", out_id.0);
        };
        Field {
            id: out_id,
            def: d.clone(),
        }
    }

    /// Define a constraint
    pub fn constraint(&self, v: &mut Version, id: &str, type_: ConstraintType) {
        let id = ConstraintId(self.0.clone(), id.into());
        let d = ConstraintDef { type_: type_ };
        let mut deps = vec![Id::Table(self.0.clone())];
        match &d.type_ {
            ConstraintType::PrimaryKey(x) => {
                for f in &x.fields {
                    if f.0 != self.0 {
                        panic!(
                            "Field {} in primary key constraint {} is in table {}, but constraint is in table {}",
                            f.1,
                            id,
                            self.0,
                            f.0
                        );
                    }
                    deps.push(Id::Field(f.clone()));
                }
            },
            ConstraintType::ForeignKey(x) => {
                let mut last_foreign_table = None;
                for f in &x.fields {
                    if f.0.0 != self.0 {
                        panic!(
                            "Local field {} in foreign key constraint {} is in table {}, but constraint is in table {}",
                            f.0.1,
                            id,
                            self.0,
                            f.1.0
                        );
                    }
                    deps.push(Id::Field(f.0.clone()));
                    if let Some(t) = last_foreign_table.take() {
                        if t != f.1.0 {
                            panic!(
                                "Foreign field {} in foreign key constraint {} is in table {}, but constraint is in table {}",
                                f.1.1,
                                id,
                                t,
                                f.1.0
                            );
                        }
                    }
                    last_foreign_table = Some(f.1.0.clone());
                    deps.push(Id::Field(f.1.clone()));
                }
            },
        }
        if v.schema.insert(Id::Constraint(id.clone()), MigrateNode::new(deps, Node::table_constraint(NodeConstraint_ {
            id: id.clone(),
            def: d,
        }))).is_some() {
            panic!("Constraint with id {} aleady exists", id.0)
        };
    }

    /// Define an index
    pub fn index(&self, id: impl ToString, fields: Vec<FieldId>) -> IndexBuilder {
        IndexBuilder {
            id: IndexId(self.0.clone(), id.to_string()),
            d: IndexDef {
                field_ids: fields,
                unique: false,
            },
        }
    }
}

pub struct IndexBuilder {
    id: IndexId,
    d: IndexDef,
}

impl IndexBuilder {
    pub fn unique(mut self) -> Self {
        self.d.unique = true;
        self
    }

    pub fn build(self, v: &mut Version) -> Index {
        if v
            .schema
            .insert(
                Id::Index(self.id.clone()),
                MigrateNode::new(vec![Id::Table(self.id.0.clone())], Node::table_index(NodeIndex_ {
                    id: self.id.clone(),
                    def: self.d,
                })),
            )
            .is_some() {
            panic!("Index with id {} already exists", self.id);
        };
        Index(self.id)
    }
}

/// Generate Rust code for migrations and queries.
///
/// # Arguments
///
/// * `output` - the path to a single rust source file where the output will be written
///
/// # Returns
///
/// * Error - a list of validation or generation errors that occurred
pub fn generate(output: &Path, versions: Vec<(usize, Version)>, queries: Vec<Query>) -> Result<(), Vec<String>> {
    let mut errs = Errs::new();
    let mut migrations = vec![];
    let mut prev_version: Option<Version> = None;
    let mut prev_version_i: Option<i64> = None;
    let mut field_lookup = HashMap::new();
    for (version_i, version) in versions {
        errs.push_ctx(vec![("Migration to", version_i.to_string())]);
        let mut migration = vec![];

        fn do_migration_query(
            errs: &mut Errs,
            migration: &mut Vec<TokenStream>,
            field_lookup: &HashMap<TableId, HashMap<FieldId, (String, Type)>>,
            q: &dyn QueryBody,
        ) {
            let mut qctx = PgQueryCtx::new(errs.clone(), &field_lookup);
            let e_res = q.build(&mut qctx);
            if !qctx.rust_args.is_empty() {
                qctx.errs.err(format!("Migration statements can't receive arguments"));
            }
            let statement = e_res.1.to_string();
            let args = qctx.query_args;
            migration.push(quote!{
                txn.execute(#statement, &[#(& #args,) *]).await ?;
            });
        }

        // Do pre-migrations
        for (i, q) in version.pre_migration.iter().enumerate() {
            errs.push_ctx(vec![("Pre-migration statement", i.to_string())]);
            do_migration_query(&mut errs, &mut migration, &field_lookup, q.as_ref());
            errs.pop_ctx();
        }

        // Prep for current version
        field_lookup.clear();
        let version_i = version_i as i64;
        if let Some(i) = prev_version_i {
            if version_i != i as i64 + 1 {
                errs.err(
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
                    match field_lookup.entry(f.id.0.clone()) {
                        std::collections::hash_map::Entry::Occupied(_) => { },
                        std::collections::hash_map::Entry::Vacant(e) => {
                            e.insert(HashMap::new());
                        },
                    };
                    let table = field_lookup.get_mut(&f.id.0).unwrap();
                    if table.insert(f.id.clone(), (f.def.name.clone(), f.def.type_.type_.clone())).is_some() {
                        errs.err(format!("Duplicate field id {}", f.id));
                    }
                },
                _ => { },
            };
        }

        // Main migrations
        {
            let mut state = PgMigrateCtx::new(errs.clone());
            crate::graphmigrate::migrate(&mut state, prev_version.take().map(|s| s.schema), &version.schema);
            for statement in &state.statements {
                migration.push(quote!{
                    txn.execute(#statement, &[]).await ?;
                });
            }
        }

        // Post-migration
        for (i, q) in version.post_migration.iter().enumerate() {
            errs.push_ctx(vec![("Post-migration statement", i.to_string())]);
            do_migration_query(&mut errs, &mut migration, &field_lookup, q.as_ref());
            errs.pop_ctx();
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
        errs.pop_ctx();
    }

    // Generate queries
    let mut db_others = Vec::new();
    {
        let mut res_type_idents: HashMap<String, Ident> = HashMap::new();
        for q in queries {
            let mut ctx = PgQueryCtx::new(errs.clone(), &field_lookup);
            ctx.errs.push_ctx(vec![("Query", q.name.clone())]);
            let res = QueryBody::build(q.body.as_ref(), &mut ctx);
            let ident = format_ident!("{}", q.name);
            let q_text = res.1.to_string();
            let args = ctx.rust_args.split_off(0);
            let args_forward = ctx.query_args.split_off(0);
            drop(ctx);
            let (res_ident, unforward_res) = {
                fn convert_one_res(
                    errs: &mut Errs,
                    i: usize,
                    k: &ExprValName,
                    v: &Type,
                ) -> Option<(Ident, TokenStream, TokenStream)> {
                    if k.name.is_empty() {
                        errs.err(
                            format!("Result element {} has no name; name it using `rename` if this is intentional", i),
                        );
                        return None;
                    }
                    let mut ident: TokenStream = match v.type_.type_ {
                        types::SimpleSimpleType::Auto => quote!(i64),
                        types::SimpleSimpleType::U32 => quote!(u32),
                        types::SimpleSimpleType::I32 => quote!(i32),
                        types::SimpleSimpleType::I64 => quote!(i64),
                        types::SimpleSimpleType::F32 => quote!(f32),
                        types::SimpleSimpleType::F64 => quote!(f64),
                        types::SimpleSimpleType::Bool => quote!(bool),
                        types::SimpleSimpleType::String => quote!(String),
                        types::SimpleSimpleType::Bytes => quote!(Vec < u8 >),
                        types::SimpleSimpleType::UtcTime => quote!(chrono:: DateTime < chrono:: Utc >),
                    };
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
                    return Some((format_ident!("{}", utils::sanitize(&k.name).1), ident, quote!({
                        #unforward x
                    })));
                }

                if res.0.0.len() == 1 {
                    let e = &res.0.0[0];
                    let (_, type_ident, unforward) = match convert_one_res(&mut errs, 0, &e.0, &e.1) {
                        None => {
                            continue;
                        },
                        Some(x) => x,
                    };
                    (type_ident, unforward)
                } else {
                    let mut fields = vec![];
                    let mut unforward_fields = vec![];
                    for (i, (k, v)) in res.0.0.into_iter().enumerate() {
                        let (k_ident, type_ident, unforward) = match convert_one_res(&mut errs, i, &k, &v) {
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
                    let res_ident = match res_type_idents.entry(body.to_string()) {
                        std::collections::hash_map::Entry::Occupied(e) => {
                            e.get().clone()
                        },
                        std::collections::hash_map::Entry::Vacant(e) => {
                            let ident = if let Some(name) = q.res_name {
                                format_ident!("{}", name)
                            } else {
                                format_ident!("DbRes{}", res_type_count)
                            };
                            e.insert(ident.clone());
                            db_others.push(quote!(pub struct #ident #body));
                            ident
                        },
                    };
                    let unforward = quote!(#res_ident {
                        #(#unforward_fields,) *
                    });
                    (res_ident.to_token_stream(), unforward)
                }
            };
            let db_arg = quote!(db: &mut impl tokio_postgres::GenericClient);
            match q.res_count {
                QueryResCount::None => {
                    db_others.push(quote!{
                        pub async fn #ident(#db_arg, #(#args,) *) -> Result <(),
                        GoodError > {
                            db.execute(
                                #q_text,
                                &[#(& #args_forward,) *]
                            ).await.map_err(|e| GoodError(e.to_string())) ?;
                            Ok(())
                        }
                    });
                },
                QueryResCount::MaybeOne => {
                    db_others.push(quote!{
                        pub async fn #ident(#db_arg, #(#args,) *) -> Result < Option < #res_ident >,
                        GoodError > {
                            let r = db.query_opt(
                                #q_text,
                                &[#(& #args_forward,) *]
                            ).await.map_err(|e| GoodError(e.to_string())) ?;
                            if let Some(r) = r {
                                return Ok(Some(#unforward_res));
                            }
                            Ok(None)
                        }
                    });
                },
                QueryResCount::One => {
                    db_others.push(quote!{
                        pub async fn #ident(#db_arg, #(#args,) *) -> Result < #res_ident,
                        GoodError > {
                            let r = db.query_one(
                                #q_text,
                                &[#(& #args_forward,) *]
                            ).await.map_err(|e| GoodError(e.to_string())) ?;
                            Ok(#unforward_res)
                        }
                    });
                },
                QueryResCount::Many => {
                    db_others.push(quote!{
                        pub async fn #ident(#db_arg, #(#args,) *) -> Result < Vec < #res_ident >,
                        GoodError > {
                            let mut out = vec![];
                            for r in db.query(
                                #q_text,
                                &[#(& #args_forward,) *]
                            ).await.map_err(|e| GoodError(e.to_string())) ? {
                                out.push(#unforward_res);
                            }
                            Ok(out)
                        }
                    });
                },
            }
            errs.pop_ctx();
        }
    }

    // Compile, output
    let last_version_i = prev_version_i.unwrap() as i64;
    let tokens = quote!{
        #[derive(Debug)]
        pub struct GoodError(pub String);
        impl std::fmt::Display for GoodError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
        impl std::error::Error for GoodError { }
        pub async fn migrate(db: &mut tokio_postgres::Client) -> Result <(),
        GoodError > {
            let txn = db.transaction().await.map_err(|e| GoodError(e.to_string()))?;
            match(|| {
                async {
                    txn.execute("create table if not exists __good_version (version bigint not null);", &[]).await?;
                    let version = match txn.query_opt("select * from __good_version limit 1", &[]).await? {
                        Some(r) => {
                            let ver: i64 = r.get("version");
                            ver
                        },
                        None => {
                            let ver: i64 =
                                txn
                                    .query_one(
                                        "insert into __good_version (version) values (-1) returning version",
                                        &[],
                                    )
                                    .await?
                                    .get("version");
                            ver
                        },
                    };
                    #(
                        #migrations
                    ) * txn.execute("update __good_version set version = $1", &[& #last_version_i]).await ?;
                    let out: Result <(),
                    tokio_postgres::Error >= Ok(());
                    out
                }
            })().await {
                Err(e) => {
                    match txn.rollback().await {
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
                            return Err(GoodError(e.to_string()));
                        },
                    };
                }
                Ok(_) => {
                    match txn.commit().await {
                        Err(e) => {
                            return Err(GoodError(format!("Error committing the migration transaction: {}", e)));
                        },
                        Ok(_) => { },
                    };
                }
            }
            Ok(())
        }
        #(#db_others) *
    };
    if let Some(p) = output.parent() {
        if let Err(e) = fs::create_dir_all(&p) {
            errs.err(format!("Error creating output parent directories {}: {:?}", p.to_string_lossy(), e));
        }
    }
    match genemichaels::format_str(&tokens.to_string(), &genemichaels::FormatConfig::default()) {
        Ok(src) => {
            match fs::write(output, src.rendered.as_bytes()) {
                Ok(_) => { },
                Err(e) => errs.err(
                    format!("Failed to write generated code to {}: {:?}", output.to_string_lossy(), e),
                ),
            };
        },
        Err(e) => {
            errs.err(format!("Error formatting generated code: {:?}\n{}", e, tokens));
        },
    };
    errs.raise()?;
    Ok(())
}
