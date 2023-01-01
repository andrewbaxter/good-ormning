use std::{
    collections::{
        HashMap,
    },
    hash::Hash,
};
use petgraph::{
    prelude::GraphMap,
    visit::{
        Topo,
        Dfs,
    },
    Directed,
};

pub mod pg;

//. #[derive(Hash, PartialEq, Eq, Clone)]
//. pub struct Id(String);
//. 
//. pub struct Schema {
//.    versions: Vec<Version>,
//. }
//. 
//. pub struct Type {
//.    id: Id,
//.    name: String,
//.    type_: TypeType,
//. }
//. 
//. pub enum TypeType {
//.    Auto,
//.    U32,
//.    U64,
//.    I32,
//.    I64,
//.    F32,
//.    F64,
//.    Bool,
//.    String,
//.    Bytes,
//.    LocalTime,
//.    UtcTime,
//.    Enum(Vec<String>),
//.    JsonBytes(String),
//.    Opt(Box<TypeType>),
//. }
//. 
//. pub struct Version {
//.    pub types: Vec<Type>,
//.    pub tables: Vec<Table>,
//.    pub queries: Vec<Query>,
//. }
//. 
//. pub struct Table {
//.    pub id: Id,
//.    pub name: String,
//.    pub fields: Vec<Field>,
//.    pub primary_key: Vec<Id>,
//.    // table, vec[local, foreign]
//.    pub foreign_key: Option<(Id, Vec<(Id, Id)>)>,
//.    pub indexes: Vec<Index>,
//. }
//. 
//. #[derive(Clone)]
//. pub struct Field {
//.    pub id: Id,
//.    pub name: String,
//.    pub type_: Id,
//.    pub default: Option<String>,
//. }
//. 
//. pub struct Index {
//.    pub id: String,
//.    pub table: Id,
//.    pub unique: bool,
//.    pub fields: Vec<Id>,
//. }
//. 
//. pub struct Select {
//.    pub table: Id,
//.    pub join: Vec<Join>,
//.    pub where_: Expr,
//.    pub fields: Vec<Expr>,
//. }
//. 
//. pub struct Insert {
//.    pub table: Id,
//.    pub values: Vec<Expr>,
//.    pub conflict: InsertConflict,
//.    pub returning: Vec<Expr>,
//. }
//. 
//. pub enum InsertConflict {
//.    Error,
//.    DoNothing,
//.    Replace,
//.    Merge,
//. }
//. 
//. pub struct Delete {
//.    pub table: Id,
//.    pub where_: Expr,
//. }
//. 
//. pub struct Update {
//.    pub table: Id,
//.    pub where_: Expr,
//.    pub values: Vec<Expr>,
//.    pub returning: Vec<Expr>,
//. }
//. 
//. pub enum Expr {
//.    Lit(String),
//.    Param(Id),
//.    Field(Id, Id),
//.    BinOp(ExprBinOp),
//.    PrefixOp(ExprPrefixOp),
//.    Call(ExprCall),
//. }
//. 
//. pub struct ExprBinOp(Box<ExprBinOp_>);
//. 
//. pub struct ExprBinOp_ {
//.    Left: Expr,
//.    Op: BinOp,
//.    Right: Expr,
//. }
//. 
//. pub enum BinOp {
//.    Plus,
//.    Minus,
//.    Multiply,
//.    Divide,
//.    And,
//.    Or,
//.    In,
//.    NotIn,
//.    Equals,
//.    NotEquals,
//.    LessThan,
//.    LessThanEqualTo,
//.    GreaterThan,
//.    GreaterThanEqualTo,
//. }
//. 
//. pub struct ExprPrefixOp(Box<ExprPrefixOp_>);
//. 
//. pub struct ExprPrefixOp_ {
//.    Op: PrefixOp,
//.    Right: Expr,
//. }
//. 
//. pub enum PrefixOp {
//.    Not,
//. }
//. 
//. struct Tokens(String);
//. 
//. impl Tokens {
//.    fn new() -> Tokens {
//.        Tokens(String::new())
//.    }
//. 
//.    fn s(&mut self, s: &str) -> &mut Self {
//.        if !self.0.is_empty() {
//.            self.0.push(' ');
//.        }
//.        self.0.push_str(s);
//.        self
//.    }
//. 
//.    fn id(&mut self, i: &Id) -> &mut Self {
//.        if !self.0.is_empty() {
//.            self.0.push(' ');
//.        }
//.        self.0.push_str(&format!("\"{}\"", i.0));
//.        self
//.    }
//. }
//. 
//. fn generate_pg(s: Schema) {
//.    let bad_type = TypeType::Auto;
//. 
//.    let mut errors = vec![];
//. 
//.    let mut prev: HashMap<Id, HashMap<Id, Field>> = HashMap::new();
//.    for (i, version) in s.versions.iter().enumerate() {
//.        fn to_sql_type(t: &TypeType) -> &'static str {
//.            match t {
//.                TypeType::Auto => "serial",
//.                TypeType::U32 => "int",
//.                TypeType::U64 => "bigint",
//.                TypeType::I32 => "int",
//.                TypeType::I64 => "bigint",
//.                TypeType::F32 => "real",
//.                TypeType::F64 => "double",
//.                TypeType::Bool => "bool",
//.                TypeType::String => "text",
//.                TypeType::Bytes => "bytea",
//.                TypeType::LocalTime => "timestamp",
//.                TypeType::UtcTime => "timestamp",
//.                TypeType::Enum(_) => "text",
//.                TypeType::JsonBytes(_) => "bytea",
//.                TypeType::Opt(t) => to_sql_type(t),
//.            }
//.        }
//.        let mut types = HashMap::new();
//.        for t in version.types {
//.            types.insert(t.id, t.type_);
//.        }
//.        let mut get_type = |id: &Id| -> &TypeType {
//.            match types.get(&id) {
//.                Some(t) => &t,
//.                None => {
//.                    errors.push(format!(
//.                        "Version {}: Reference to undefined type [{}]",
//.                        i, id.0
//.                    ));
//.                    &bad_type
//.                }
//.            }
//.        };
//. 
//.        let mut column_def = |stmt: &mut Tokens, f: &Field| {
//.            stmt.id(&f.id);
//.            stmt.s(match get_type(&f.type_) {
//.                TypeType::Opt(t) => to_sql_type(t),
//.                t => &format!("{} not null", to_sql_type(t)),
//.            });
//.        };
//. 
//.        struct PrevTable {
//.            fields: HashMap<Id, Field>,
//.            indexes: HashMap<Id, Index>,
//.        }
//.        let mut new_prev_version = HashMap::new();
//.        let mut stmt = Tokens("".into());
//.        for table in version.tables {
//.            let mut new_prev_table_fields = HashMap::new();
//.            match prev.remove(&table.id) {
//.                Some(old_table) => {
//.                    for field in table.fields {
//.                        new_prev_table_fields.insert(field.id.clone(), field.clone());
//.                        match old_table.fields.remove(&field.id) {
//.                            Some(old_f) => {
//.                                match (
//.                                    matches!(get_type(&old_f.id), TypeType::Opt(_)),
//.                                    matches!(get_type(&field.id), TypeType::Opt(_)),
//.                                ) {
//.                                    (true, false) => {
//.                                        stmt.s("alter table");
//.                                        stmt.id(&table.id);
//.                                        stmt.s("alter column");
//.                                        stmt.id(&field.id);
//.                                        stmt.s("set not null;");
//.                                    }
//.                                    (false, true) => {
//.                                        stmt.s("alter table");
//.                                        stmt.id(&table.id);
//.                                        stmt.s("alter column");
//.                                        stmt.id(&field.id);
//.                                        stmt.s("drop not null;");
//.                                    }
//.                                    (true, true) | (false, false) => {}
//.                                }
//. 
//.                                let sql_type = to_sql_type(get_type(&field.type_));
//.                                if sql_type != to_sql_type(get_type(&old_f.type_)) {
//.                                    stmt.s("alter table");
//.                                    stmt.id(&table.id);
//.                                    stmt.s("alter column");
//.                                    stmt.id(&field.id);
//.                                    stmt.s("set type");
//.                                    stmt.s(&sql_type);
//.                                    stmt.s(";");
//.                                }
//.                                match (field.default, old_f.default) {
//.                                    (None, None) => {}
//.                                    (Some(_), None) => {
//.                                        stmt.s("alter table");
//.                                        stmt.id(&table.id);
//.                                        stmt.s("alter column");
//.                                        stmt.id(&field.id);
//.                                        stmt.s("drop default;");
//.                                    }
//.                                    (Some(old_d), Some(new_d)) => {
//.                                        if old_d == new_d {
//.                                        } else {
//.                                            stmt.s("alter table");
//.                                            stmt.id(&table.id);
//.                                            stmt.s("alter column");
//.                                            stmt.id(&field.id);
//.                                            stmt.s("set default");
//.                                            stmt.s(&new_d);
//.                                            stmt.s(";");
//.                                        }
//.                                    }
//.                                    (None, Some(new_d)) => {
//.                                        stmt.s("alter table");
//.                                        stmt.id(&table.id);
//.                                        stmt.s("alter column");
//.                                        stmt.id(&field.id);
//.                                        stmt.s("set default");
//.                                        stmt.s(&new_d);
//.                                        stmt.s(";");
//.                                    }
//.                                }
//.                            }
//.                            None => {
//.                                if matches!(get_type(&field.type_), TypeType::Auto) {
//.                                    errors.push(format!("Version {}: Cannot add a serial field ({}) to an existing table", table.id.0, field.id.0));
//.                                }
//.                                // todo err if serial
//.                                stmt.s("alter table");
//.                                stmt.id(&table.id);
//.                                stmt.s("add column");
//.                                stmt.id(&field.id);
//.                                column_def(&mut stmt, &field);
//.                                stmt.s(";");
//.                            }
//.                        }
//.                    }
//.                    for (f_id, _) in old_table.fields {
//.                        stmt.s("alter table")
//.                            .id(&table.id)
//.                            .s("drop column")
//.                            .id(&f_id)
//.                            .s(";");
//.                    }
//.                    for index in table.indexes {
//.                        match old_table.indexes.remove(&index.id) {
//. 
//.                        }
//.                    }
//.                    for (i_id, _) in old_table.indexes {
//.                        stmt.s("drop index").id(&i_id).s(";");
//.                    }
//.                }
//.                None => {
//.                    stmt.s("create").s("table").id(&table.id).s("(");
//.                    for (i, f) in table.fields.iter().enumerate() {
//.                        new_prev_table_fields.insert(f.id.clone(), f.clone());
//.                        if i > 0 {
//.                            stmt.s(",");
//.                        }
//.                        column_def(&mut stmt, f);
//.                    }
//.                    if !table.primary_key.is_empty() {
//.                        stmt.s(", primary key (");
//.                        for (i, k) in table.primary_key.iter().enumerate() {
//.                            if i > 0 {
//.                                stmt.s(",");
//.                            }
//.                            stmt.id(k);
//.                        }
//.                        stmt.s(")");
//.                    }
//.                    if let Some(fk) = table.foreign_key {
//.                        stmt.s(", foreign key (");
//.                        for (i, (f, _)) in fk.1.iter().enumerate() {
//.                            if i > 0 {
//.                                stmt.s(",");
//.                            }
//.                            stmt.id(f);
//.                        }
//.                        stmt.s(") references ");
//.                        stmt.id(&fk.0);
//.                        stmt.s("(");
//.                        for (i, (_, t)) in fk.1.iter().enumerate() {
//.                            if i > 0 {
//.                                stmt.s(",");
//.                            }
//.                            stmt.id(t);
//.                        }
//.                        stmt.s(")");
//.                        stmt.s(";");
//.                    }
//.                    stmt.s(")").s(";");
//.                }
//. 
//.                let mut new_prev_t_indexes = HashMap::new();
//.                    for i in table.indexes {
//.                        stmt.s("create");
//.                        if i.unique {
//.                            stmt.s("unique");
//.                        }
//.                        stmt.s("index");
//.                        s.id(&i.id);
//.                        stmt.s("on");
//.                        stmt.id(&i.table);
//.                        stmt.s("(");
//.                        for (i, f) in i.fields.iter().enumerate() {
//.                            if i > 0 {
//.                                stmt.s(",");
//.                            }
//.                            stmt.id(&f);
//.                        }
//.                        stmt.s(");");
//.                    }
//.            }
//.            new_prev_version.insert(table.id.clone(), PrevTable{
//.                fields: new_prev_table_fields,
//.                indexes: new_prev_t_indexes,
//.            });
//.        }
//.        for (t_id, _) in prev {
//.            stmt.s("drop table").id(&t_id).s(";");
//.        }
//.        prev = new_prev_version;
//.    }
//. }
trait NodeId: Hash + PartialEq + Eq + Clone { }

impl<T: Hash + PartialEq + Eq + Clone> NodeId for T { }

trait MigrateNodeVersion: Clone {
    fn compare(&self, old: &Self) -> MigrateNodeVersionComparison;
    fn create_coalesce(&mut self, other: &Self) -> bool;
    fn create(&self, ctx: &mut BuildContext);
    fn delete_coalesce(&mut self, other: &Self) -> bool;
    fn delete(&self, ctx: &mut BuildContext);
    fn update(&self, ctx: &mut BuildContext, old: &Self);
}

struct Node<T: MigrateNodeVersion, I: NodeId> {
    deps: Vec<I>,
    body: T,
}

enum MigrateNodeVersionComparison {
    Invalid(String),
    Keep,
    Update,
    DeleteCreate,
}

struct Version<T: MigrateNodeVersion, I: NodeId> {
    pre_migration: Option<String>,
    post_migration: Option<String>,
    schema: HashMap<I, Node<T, I>>,
}

struct BuildContext {
    errs: Vec<String>,
    statements: Vec<String>,
}

impl BuildContext {
    fn err(&mut self, text: String) {
        self.errs.push(text);
    }

    fn stmt(&mut self, text: String) {
        self.statements.push(text);
    }
}

fn generate<T: MigrateNodeVersion, I: NodeId>(versions: Vec<Version<T, I>>) {
    #[derive(Clone, Hash, PartialEq, Eq)]
    struct VersionNodeId<I: NodeId>(i32, I);

    enum MigrateNode<T: MigrateNodeVersion> {
        DoNothing,
        Create {
            new: T,
        },
        Delete {
            old: T,
        },
        Update {
            old: T,
            new: T,
        },
    }

    struct Stage<T: MigrateNodeVersion, I: NodeId> {
        nodes: Vec<MigrateNode<T>>,
        node_ids: HashMap<VersionNodeId<I>, usize>,
        g: GraphMap<usize, usize, Directed>,
        ge: usize,
    }

    impl<T: MigrateNodeVersion, I: NodeId> Default for Stage<T, I> {
        fn default() -> Self {
            Stage {
                nodes: vec![],
                node_ids: HashMap::new(),
                g: GraphMap::new(),
                ge: 0,
            }
        }
    }

    impl<T: MigrateNodeVersion, I: NodeId> Stage<T, I> {
        fn add(&mut self, k: VersionNodeId<I>, v: T) -> usize {
            let id = self.nodes.len();
            self.nodes.push(MigrateNode::Create { new: v });
            self.node_ids.insert(k, id);
            self.g.add_node(id);
            id
        }

        fn remove(&mut self, k: VersionNodeId<I>, v: T) -> usize {
            let id = self.nodes.len();
            self.nodes.push(MigrateNode::Delete { old: v });
            self.node_ids.insert(k, id);
            self.g.add_node(id);
            id
        }

        fn edge(&mut self, a: usize, b: usize) {
            let id = self.ge;
            self.ge += 1;
            self.g.add_edge(a, b, id);
        }

        fn get(&self, v: i32, i: &I) -> Option<usize> {
            self.node_ids.get(&VersionNodeId(v, i.clone())).map(|i| *i)
        }
    }

    let mut current = Stage::default();
    let mut has_prev_post_migration = false;
    for (version_i, version) in versions.iter().enumerate() {
        let version_i = version_i as i32;
        let mut next = Stage::default();

        // Update current graph with new elements + merge with old when compatible. Also build next
        // stage graph.
        for (k, n) in &version.schema {
            let vk = VersionNodeId(version_i, k.clone());
            next.remove(vk.clone(), n.body.clone());
            let gk = match current.get(version_i - 1, k) {
                Some(gk) => {
                    let old_n = match current.nodes.get(gk).unwrap() {
                        MigrateNode::Delete { old } => old,
                        MigrateNode::DoNothing |
                        MigrateNode::Create { .. } |
                        MigrateNode::Update { .. } => unreachable!(
                        ),
                    };
                    match n.body.compare(old_n) {
                        MigrateNodeVersionComparison::Keep => {
                            current.nodes[gk] = MigrateNode::DoNothing;
                            gk
                        },
                        MigrateNodeVersionComparison::Update => {
                            current.nodes[gk] = MigrateNode::Update {
                                old: old_n.clone(),
                                new: n.body.clone(),
                            };
                            gk
                        },
                        MigrateNodeVersionComparison::DeleteCreate => {
                            let new_gk = current.add(vk, n.body.clone());
                            current.edge(gk, new_gk);
                            new_gk
                        },
                    }
                },
                None => {
                    current.add(vk, n.body.clone())
                },
            };
        }
        for (k, n) in &version.schema {
            {
                let gk = next.get(version_i, k).unwrap();
                for dep in &n.deps {
                    next.edge(next.get(version_i, dep).unwrap(), gk);
                }
            }
            {
                let gk = current.get(version_i, k).unwrap();
                for dep in &n.deps {
                    current.edge(gk, current.get(version_i, dep).unwrap());
                }
            }
        }

        // Traverse graph, creating migration
        let mut iter = Topo::new(&current.g);
        while let Some(n) = iter.next(&current.g) {
            let mut ctx = BuildContext {
                errs: vec![],
                statements: vec![],
            };
            match current.nodes.get_mut(n).unwrap() {
                MigrateNode::DoNothing => { },
                MigrateNode::Delete { old } => {
                    let dfs = Dfs::new(&current.g, n);
                    dfs.next(&current.g);
                    while let Some(n) = dfs.next(&current.g) {
                        if !match current.nodes.get(n).unwrap() {
                            MigrateNode::Delete { old: v } => old.delete_coalesce(v),
                            _ => {
                                false
                            },
                        } {
                            dfs.stack.pop();
                            current.nodes[n] = MigrateNode::DoNothing;
                        }
                    }
                    old.delete(&mut ctx);
                },
                MigrateNode::Create { new } => {
                    let dfs = Dfs::new(&current.g, n);
                    dfs.next(&current.g);
                    while let Some(n) = dfs.next(&current.g) {
                        if !match current.nodes.get(n).unwrap() {
                            MigrateNode::Create { new: v } => new.create_coalesce(v),
                            _ => {
                                false
                            },
                        } {
                            dfs.stack.pop();
                            current.nodes[n] = MigrateNode::DoNothing;
                        }
                    }
                    new.create(&mut ctx);
                },
                MigrateNode::Update { old, new } => new.update(&mut ctx, old),
            }
        }

        // Generate queries
        if has_prev_post_migration || version.post_migration.is_some() || version_i as usize == versions.len() - 1 { }

        // Next iter prep
        current = next;
        has_prev_post_migration = version.post_migration.is_some();
    }
}
