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
    },
    path::Path,
    fs,
};
use crate::{
    pg::types::Type,
    utils::Errs,
};
use self::{
    schema::{
        TableId,
        TableDef,
        FieldId,
        TableIndexId,
        Node,
        Id,
        NodeTable_,
        FieldDef,
        NodeField_,
        TableConstraintDef,
        TableConstraintId,
        NodeConstraint_,
        TableConstraintTypeDef,
        NodeIndex_,
        IndexDef,
        PgMigrateCtx,
        Node_,
    },
    queries::{
        utils::{
            PgQueryCtx,
            Query,
        },
        expr::ExprTypeField,
    },
};
use {
    schema::{
        MigrateNode,
    },
};

pub mod types;
pub mod schema;
pub mod queries;

pub enum QueryPlural {
    None,
    MaybeOne,
    One,
    Many,
}

struct VersionQuery_ {
    name: String,
    txn: bool,
    plural: QueryPlural,
    q: Box<dyn Query>,
}

#[derive(Default)]
pub struct Version<'a> {
    schema: HashMap<Id, MigrateNode<'a>>,
    pre_migration: Option<String>,
    post_migration: Option<String>,
    queries: Vec<VersionQuery_>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Table(pub TableId);

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Field(pub FieldId);

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Index(pub TableIndexId);

impl<'a> Version<'a> {
    pub fn table(&mut self, id: &str, d: TableDef) -> Table {
        let out = Table(TableId(id.into()));
        self.schema.insert(Id::Table(out.0.clone()), MigrateNode::new(vec![], Node::table(NodeTable_ {
            id: out.0.clone(),
            def: d,
            fields: vec![],
        })));
        out
    }

    pub fn query(&mut self, name: &str, txn: bool, plural: QueryPlural, query: impl Query + 'static) {
        self.queries.push(VersionQuery_ {
            name: name.into(),
            txn: txn,
            plural: plural,
            q: Box::new(query),
        });
    }
}

impl Table {
    pub fn field(&self, v: &mut Version, id: &str, d: FieldDef) -> Field {
        let out = Field(FieldId(self.0.clone(), id.into()));
        v
            .schema
            .insert(
                Id::Field(out.0.clone()),
                MigrateNode::new(vec![Id::Table(self.0.clone())], Node::field(NodeField_ {
                    id: out.0.clone(),
                    def: d,
                })),
            );
        out
    }

    pub fn constraint(&self, errs: &mut Errs, v: &mut Version, id: &str, d: TableConstraintDef) {
        let id = TableConstraintId(self.0.clone(), id.into());
        let mut deps = vec![Id::Table(self.0.clone())];
        match &d.type_ {
            TableConstraintTypeDef::PrimaryKey(x) => {
                for f in &x.fields {
                    if f.0 != self.0 {
                        errs.err(
                            format!(
                                "Field {} in primary key constraint {} is in table {}, but constraint is in table {}",
                                f.1,
                                id,
                                self.0,
                                f.0
                            ),
                        );
                    }
                    deps.push(Id::Field(f.clone()));
                }
            },
            TableConstraintTypeDef::ForeignKey(x) => {
                let mut last_foreign_table = None;
                for f in &x.fields {
                    if f.0.0 != self.0 {
                        errs.err(
                            format!(
                                "Local field {} in foreign key constraint {} is in table {}, but constraint is in table {}",
                                f.0.1,
                                id,
                                self.0,
                                f.1.0
                            ),
                        );
                    }
                    deps.push(Id::Field(f.0.clone()));
                    if let Some(t) = last_foreign_table.take() {
                        if t != f.1.0 {
                            errs.err(
                                format!(
                                    "Foreign field {} in foreign key constraint {} is in table {}, but constraint is in table {}",
                                    f.1.1,
                                    id,
                                    t,
                                    f.1.0
                                ),
                            );
                        }
                    }
                    last_foreign_table = Some(f.1.0.clone());
                    deps.push(Id::Field(f.1.clone()));
                }
            },
        }
        v
            .schema
            .insert(Id::TableConstraint(id.clone()), MigrateNode::new(deps, Node::table_constraint(NodeConstraint_ {
                id: id,
                def: d,
            })));
    }

    pub fn index(&self, v: &mut Version, id: &str, d: IndexDef) -> Index {
        let out = Index(TableIndexId(self.0.clone(), id.into()));
        v
            .schema
            .insert(
                Id::TableIndex(out.0.clone()),
                MigrateNode::new(vec![Id::Table(self.0.clone())], Node::table_index(NodeIndex_ {
                    id: out.0.clone(),
                    def: d,
                })),
            );
        out
    }
}

pub fn generate(output: &Path, versions: Vec<(usize, Version)>) -> Result<(), Vec<String>> {
    let mut errs = Errs::new();
    let mut migrations = vec![];
    let mut ver_wrappers = vec![];
    let mut latest_ver_wrapper: Option<Ident> = None;
    let mut prev_version: Option<&Version> = None;
    let mut prev_ver_txn_wrapper = None;
    let mut prev_version_i: Option<usize> = None;
    for (version_i, version) in &versions {
        let version_i = *version_i;
        if let Some(i) = prev_version_i {
            if version_i != i + 1 {
                errs.err(
                    format!(
                        "Version numbers are not consecutive ({} to {}) - was an intermediate version deleted?",
                        i,
                        version_i
                    ),
                );
            }
        }
        let db_wrapper_ident = format_ident!("DbVer{}", version_i);
        let txn_wrapper_ident = format_ident!("TxnVer{}", version_i);

        // Gather tables for lookup during query generation and check duplicates
        let mut field_lookup = HashMap::new();
        {
            let mut table_lookup = HashSet::new();
            let mut constraint_lookup = HashSet::new();
            let mut index_lookup = HashSet::new();
            for v in version.schema.values() {
                match &v.body.n {
                    Node_::Table(t) => {
                        if !table_lookup.insert(t.id.0.clone()) {
                            errs.err(format!("Duplicate table id {}", t.id));
                        }
                    },
                    Node_::Field(f) => {
                        match field_lookup.entry(f.id.0.clone()) {
                            std::collections::hash_map::Entry::Occupied(_) => { },
                            std::collections::hash_map::Entry::Vacant(e) => {
                                e.insert(HashMap::new());
                            },
                        };
                        let table = field_lookup.get_mut(&f.id.0).unwrap();
                        if table.insert(ExprTypeField {
                            table: f.id.0.0.clone(),
                            field: f.id.1.clone(),
                        }, match &f.def.type_ {
                            schema::FieldType::NonOpt(t, _) => Type {
                                type_: t.clone(),
                                opt: false,
                            },
                            schema::FieldType::Opt(t) => Type {
                                type_: t.clone(),
                                opt: true,
                            },
                        }).is_some() {
                            errs.err(format!("Duplicate field id {}", f.id));
                        }
                    },
                    Node_::Constraint(c) => {
                        if !constraint_lookup.insert(c.id.clone()) {
                            errs.err(format!("Duplicate constraint id {}", c.id));
                        }
                    },
                    Node_::Index(i) => {
                        if !index_lookup.insert(i.id.clone()) {
                            errs.err(format!("Duplicate index id {}", i.id));
                        }
                    },
                };
            }
        }

        // Generate migrations
        {
            let mut state = PgMigrateCtx::new(&mut errs);
            crate::graphmigrate::migrate(&mut state, &prev_version.take().map(|s| &s.schema), &version.schema);
            let mut migration = vec![];
            if let Some(pre) = &version.pre_migration {
                if let Some(prev_ver_txn_wrapper_ident) = prev_ver_txn_wrapper {
                    let pre_ident = format_ident!("{}", pre);
                    migration.push(quote!{
                        txn = #pre_ident(#prev_ver_txn_wrapper_ident(txn)) ?.0;
                    });
                } else {
                    state
                        .errs
                        .err(
                            format!(
                                "Pre-migration specified for version {}, but no previous state to perform migration in",
                                version_i
                            ),
                        );
                }
            }
            for statement in state.statements {
                migration.push(quote!{
                    txn.execute(#statement, &[]) ?;
                });
            }
            if let Some(post) = &version.post_migration {
                let post_ident = format_ident!("{}", post);
                migration.push(quote!{
                    txn = #post_ident(#txn_wrapper_ident(txn)) ?.0;
                });
            }
            migrations.push(quote!{
                if skip <= #version_i {
                    #(#migration) *
                }
            });
        }

        // Generate queries
        {
            let mut db_queries = vec![];
            let mut txn_queries = vec![];
            let mut res_type_idents: HashMap<String, Ident> = HashMap::new();
            let mut res_type_defs = vec![];
            if version_i == versions.len() - 1 || version.post_migration.is_some() ||
                versions.get(version_i + 1).map(|v| v.1.post_migration.is_some()).unwrap_or_default() {
                for q in &version.queries {
                    let mut ctx = PgQueryCtx::new(&mut errs, &field_lookup);
                    ctx.errs.err_ctx.push(vec![("Query", q.name.clone())]);
                    let res = Query::build(q.q.as_ref(), &mut ctx);
                    let ident = format_ident!("{}", q.name);
                    let q_text = res.1.to_string();
                    let args = ctx.args;
                    let args_forward = ctx.args_forward;
                    let out = if q.txn {
                        &mut txn_queries
                    } else {
                        &mut db_queries
                    };

                    struct ConvertResType {
                        type_tokens: TokenStream,
                        unforward: TokenStream,
                    }

                    fn convert_res_type(s: Type) -> Option<ConvertResType> {
                        let mut ident: TokenStream = match s.type_.type_ {
                            types::SimpleSimpleType::Auto => quote!(i64),
                            types::SimpleSimpleType::U32 => quote!(u32),
                            types::SimpleSimpleType::U64 => quote!(u64),
                            types::SimpleSimpleType::I32 => quote!(i32),
                            types::SimpleSimpleType::I64 => quote!(i64),
                            types::SimpleSimpleType::F32 => quote!(f32),
                            types::SimpleSimpleType::F64 => quote!(f64),
                            types::SimpleSimpleType::Bool => quote!(bool),
                            types::SimpleSimpleType::String => quote!(String),
                            types::SimpleSimpleType::Bytes => quote!(Vec < u8 >),
                            types::SimpleSimpleType::LocalTime => quote!(chrono::LocalTime),
                            types::SimpleSimpleType::UtcTime => quote!(chrono:: DateTime < chrono:: Utc >),
                        };
                        if s.opt {
                            ident = quote!(Option < #ident >);
                        }
                        let mut unforward = quote!{
                            let x: #ident = r.get(0);
                        };
                        if let Some(custom) = &s.type_.custom {
                            ident = format_ident!("{}", custom).to_token_stream();
                            if s.opt {
                                unforward = quote!{
                                    #unforward let x = if let Some(x) = x {
                                        #ident:: convert(x) ?
                                    }
                                    else {
                                        None
                                    };
                                };
                                ident = quote!(Option < #ident >);
                            } else {
                                unforward = quote!{
                                    #unforward let x = #ident:: convert(x) ?;
                                };
                            }
                        }
                        Some(ConvertResType {
                            type_tokens: ident,
                            unforward: quote!({
                                #unforward x
                            }),
                        })
                    }

                    let (res_ident, unforward_res) = {
                        let r = res.0.0;
                        let mut fields = vec![];
                        let mut unforward_fields = vec![];
                        for (k, v) in r {
                            let k_ident = format_ident!("{}", k.field);
                            let r = match convert_res_type(v) {
                                Some(x) => x,
                                None => {
                                    continue;
                                },
                            };
                            let type_ident = r.type_tokens;
                            let unforward = r.unforward;
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
                                let ident = format_ident!("ResVer{}_{}", version_i, res_type_count);
                                e.insert(ident.clone());
                                res_type_defs.push(quote!(pub struct #ident #body));
                                ident
                            },
                        };
                        let unforward = quote!(#res_ident {
                            #(#unforward_fields,) *
                        });
                        (res_ident, unforward)
                    };
                    match q.plural {
                        QueryPlural::None => {
                            out.push(quote!{
                                pub async fn #ident(&mut self, #(#args,) *) -> Result <() > {
                                    self.0.execute(#q_text, &[#(#args_forward,) *]) ?;
                                    Ok(())
                                }
                            });
                        },
                        QueryPlural::MaybeOne => {
                            out.push(quote!{
                                pub async fn #ident(&mut self, #(#args,) *) -> Result < Option < #res_ident >> {
                                    let r = self.0.query_opt(#q_text, &[#(#args_forward,) *]) ?;
                                    for r in r {
                                        return Ok(Some(#unforward_res));
                                    }
                                    Ok(None)
                                }
                            });
                        },
                        QueryPlural::One => {
                            out.push(quote!{
                                pub async fn #ident(&mut self, #(#args,) *) -> Result < #res_ident > {
                                    let r = self.0.query_one(#q_text, &[#(#args_forward,) *]) ?;
                                    Ok(#unforward_res)
                                }
                            });
                        },
                        QueryPlural::Many => {
                            out.push(quote!{
                                pub async fn #ident(&mut self, #(#args,) *) -> Result < Vec < #res_ident >> {
                                    let mut out = vec![];
                                    for r in self.0.query(#q_text, &[#(#args_forward,) *]) ? {
                                        out.push(#unforward_res);
                                    }
                                    Ok(out)
                                }
                            });
                        },
                    }
                }
            }
            ver_wrappers.push(quote!{
                pub struct #db_wrapper_ident(tokio_postgres::Client);
                impl #db_wrapper_ident {
                    fn tx < T >(
                        &mut self,
                        cb: impl FnOnce(#txn_wrapper_ident) ->(#txn_wrapper_ident, Result < T >)
                    ) -> Result < T > {
                        let(mut txn, mut res) = cb(#txn_wrapper_ident(self.0.transaction()?));
                        let mut txn = txn.0;
                        match res {
                            Err(e) => {
                                if let Err(e1) = txn.rollback() {
                                    return Err(e1.chain(e));
                                } else {
                                    return Err(e);
                                }
                            },
                            Ok(_) => {
                                if let Err(e) = txn.commit()? {
                                    return Err(e);
                                } else {
                                    return Ok(res);
                                }
                            },
                        }
                    }
                    #(#db_queries) *
                }
                pub struct #txn_wrapper_ident < 'a >(tokio_postgres:: Transaction < 'a >);
                impl #txn_wrapper_ident {
                    #(#txn_queries) *
                }
                #(#res_type_defs) *
            });
        }

        // Next iter prep
        prev_version = Some(version);
        prev_ver_txn_wrapper = Some(txn_wrapper_ident);
        latest_ver_wrapper = Some(db_wrapper_ident);
        prev_version_i = Some(version_i);
    }
    let ver_wrapper = latest_ver_wrapper.unwrap();
    let tokens = quote!{
        fn migrate(db: tokio_postgres::Client) -> Result < #ver_wrapper > {
            let mut txn = db.transaction()?;
            match ||-> Result <() > {
                txn.execute(
                    "create table __good_version if not exists (version bigint not null, hash bytea not null);",
                    &[],
                )?;
                let skip = match txn.query_opt("select * from __good_version limit 1", &[])? {
                    Some(r) => {
                        let ver: i64 = r.get("version");
                        let hash: Vec<u8> = r.get("hash");
                        if let Some(expect_hash) = version_hashes(ver) {
                            if expect_hash != hash {
                                return Err(
                                    anyhow!(
                                        "At version {}, but current version hash {:x} doesn't match schema hash {:x}. Did an old version schema change?",
                                        ver,
                                        hash,
                                        expect_hash
                                    ),
                                );
                            }
                        } else {
                            return Err(
                                anyhow!("At version {} with has {:x} which isn't defined in the schema", ver, hash),
                            );
                        }
                        ver + 1
                    },
                    None => {
                        0
                    },
                };
                #(#migrations) * Ok(())
            }
            () {
                Err(e) => {
                    return txn.rollback().chain(e)?;
                }
                Ok(_) => {
                    txn.commit()?;
                }
            }
            Ok(#ver_wrapper(db))
        }
        #(#ver_wrappers) *
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
