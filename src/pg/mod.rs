use crate::{
    MigrateNodeVersion,
    MigrateNodeVersionComparison,
    BuildContext,
};
use enum_dispatch::{
    enum_dispatch,
};

struct Tokens(String);

impl ToString for Tokens {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl Tokens {
    fn new() -> Tokens {
        Tokens(String::new())
    }

    fn s(&mut self, s: &str) -> &mut Self {
        if !self.0.is_empty() {
            self.0.push(' ');
        }
        self.0.push_str(s);
        self
    }

    fn id(&mut self, i: &str) -> &mut Self {
        if !self.0.is_empty() {
            self.0.push(' ');
        }
        self.0.push_str(&format!("\"{}\"", i));
        self
    }

    fn f(&mut self, f: impl FnOnce(&mut Self)) -> &mut Self {
        f(self);
        self
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum SimpleType {
    Auto,
    U32,
    U64,
    I32,
    I64,
    F32,
    F64,
    Bool,
    String,
    Bytes,
    LocalTime,
    UtcTime,
    Enum(Vec<String>),
    JsonBytes(String),
}

#[derive(Clone, PartialEq, Eq)]
pub enum Type {
    NonOpt(SimpleType, String),
    Opt(SimpleType),
}

impl Type {
    fn simple(&self) -> &SimpleType {
        match self {
            Type::NonOpt(t, _) => t,
            Type::Opt(t) => t,
        }
    }
}

fn to_sql_type(t: &SimpleType) -> &'static str {
    match t {
        SimpleType::Auto => "serial",
        SimpleType::U32 => "int",
        SimpleType::U64 => "bigint",
        SimpleType::I32 => "int",
        SimpleType::I64 => "bigint",
        SimpleType::F32 => "real",
        SimpleType::F64 => "double",
        SimpleType::Bool => "bool",
        SimpleType::String => "text",
        SimpleType::Bytes => "bytea",
        SimpleType::LocalTime => "timestamp",
        SimpleType::UtcTime => "timestamp",
        SimpleType::Enum(_) => "text",
        SimpleType::JsonBytes(_) => "bytea",
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct Table(String);

#[derive(Clone, PartialEq, Eq, Hash)]
struct Field(String, String);

#[derive(Clone, PartialEq, Eq, Hash)]
struct Index(String, String);

#[derive(Clone, Eq, PartialEq, Hash)]
enum Id_ {
    Table(Table),
    Field(Field),
    TableConstraint(String, String),
    TableIndex(Index),
}

#[derive(Clone)]
struct TableDef {
    id: String,
    name: String,
    fields: Vec<FieldDef>,
}

impl MigrateNode_ for TableDef {
    fn update(&self, ctx: &mut BuildContext, old: &Self) {
        unreachable!()
    }
}

impl MigrateNodeDispatch_ for TableDef {
    fn create_coalesce(&mut self, other: &Node_) -> bool {
        match other {
            Node_::Field(f) if f.table_id == self.id => {
                self.fields.push(f.def.clone());
                true
            },
            _ => false,
        }
    }

    fn delete_coalesce(&mut self, other: &Node_) -> bool {
        match other {
            Node_::Table(_) => false,
            Node_::Field(f) if f.table_id == self.id => {
                true
            },
            Node_::TableConstraint(e) if e.table_id == self.id => true,
            Node_::TableIndex(e) if e.table_id == self.id => true,
            _ => false,
        }
    }

    fn create(&self, ctx: &mut BuildContext) {
        let mut stmt = Tokens::new();
        stmt.s("create table").id(&self.id).s("(");
        for (i, f) in self.fields.iter().enumerate() {
            stmt.id(&f.id);
            stmt.s(match &f.type_ {
                Type::NonOpt(t, _) => &format!("{} not null", to_sql_type(t)),
                Type::Opt(t) => to_sql_type(t),
            });
        }
        stmt.s(")");
        ctx.stmt(stmt.to_string());
    }

    fn delete(&self, ctx: &mut BuildContext) {
        ctx.stmt(Tokens::new().s("drop table").id(&self.id).to_string());
    }
}

#[derive(Clone)]
struct FieldDef {
    id: String,
    name: String,
    type_: Type,
}

#[derive(Clone)]
struct PrimaryKeyDef {
    pub fields: Vec<Field>,
}

#[derive(Clone)]
struct ForeignKeyDef {
    pub fields: Vec<(Field, Field)>,
}

#[derive(Clone)]
enum TableConstraintTypeDef {
    PrimaryKey(PrimaryKeyDef),
    ForeignKey(ForeignKeyDef),
}

#[derive(Clone)]
struct TableConstraintDef {
    pub id: String,
    pub type_: TableConstraintTypeDef,
}

#[derive(Clone)]
struct TableIndexDef {
    id: String,
    field_ids: Vec<Field>,
    unique: bool,
}

#[derive(Clone)]
struct NodeField_ {
    table_id: String,
    def: FieldDef,
}

impl MigrateNode_ for NodeField_ {
    fn update(&self, ctx: &mut BuildContext, old: &Self) {
        match (&self.def.type_, &old.def.type_) {
            (Type::NonOpt(t, d), Type::NonOpt(old_t, old_d)) => todo!(),
            (Type::NonOpt(t, _), Type::Opt(old_t)) => todo!(),
            (Type::Opt(t), Type::NonOpt(old_t, _)) => todo!(),
            (Type::Opt(t), Type::Opt(old_t)) => {
                ctx.stmt(
                    Tokens::new()
                        .s("alter table")
                        .id(&self.table_id)
                        .s("alter column")
                        .id(&self.def.id)
                        .s("set type")
                        .s(to_sql_type(t))
                        .to_string(),
                );
            },
        }
    }
}

impl MigrateNodeDispatch_ for NodeField_ {
    fn create(&self, ctx: &mut BuildContext) {
        if matches!(self.def.type_.simple(), SimpleType::Auto) {
            ctx.err(format!("Auto (serial) fields can't be added after table creation"));
        }
        let mut stmt = Tokens::new();
        stmt
            .s("alter table")
            .id(&self.table_id)
            .s("add column")
            .id(&self.def.id)
            .s(to_sql_type(self.def.type_.simple()));
        if let Type::NonOpt(_, d) = &self.def.type_ {
            stmt.s("not null default");
            stmt.s(d);
        }
        ctx.stmt(stmt.to_string());
        if let Type::NonOpt(_, _) = &self.def.type_ {
            let mut stmt = Tokens::new();
            stmt.s("alter table").id(&self.table_id).s("alter column").id(&self.def.id).s("drop default");
        }
    }

    fn delete(&self, ctx: &mut BuildContext) {
        ctx.stmt(
            Tokens::new().s("alter table").id(&self.table_id).s("drop column").id(&self.def.id).to_string(),
        );
    }

    fn create_coalesce(&mut self, other: &Node_) -> bool {
        false
    }

    fn delete_coalesce(&mut self, other: &Node_) -> bool {
        false
    }
}

#[derive(Clone)]
struct NodeTableConstraint_ {
    table_id: String,
    def: TableConstraintDef,
}

impl MigrateNodeDispatch_ for NodeTableConstraint_ {
    fn create_coalesce(&mut self, other: &Node_) -> bool {
        false
    }

    fn create(&self, ctx: &mut BuildContext) {
        let mut stmt = Tokens::new();
        stmt.s("alter table").id(&self.table_id).s("add constraint").id(&self.def.id);
        match &self.def.type_ {
            TableConstraintTypeDef::PrimaryKey(x) => {
                stmt.s("primary key (").f(|t| {
                    for (i, id) in x.fields.iter().enumerate() {
                        if i > 0 {
                            t.s(",");
                        }
                        t.id(&id.1);
                    }
                }).s(")");
            },
            TableConstraintTypeDef::ForeignKey(x) => {
                stmt.s("foreign key (").f(|t| {
                    for (i, id) in x.fields.iter().enumerate() {
                        if i > 0 {
                            t.s(",");
                        }
                        t.id(&id.1.1);
                    }
                }).s(") references ").f(|t| {
                    for (i, id) in x.fields.iter().enumerate() {
                        if i == 0 {
                            t.id(&id.1.0).s("(");
                        } else {
                            t.s(",");
                        }
                        t.id(&id.1.1);
                    }
                }).s(")");
            },
        }
        ctx.stmt(stmt.to_string());
    }

    fn delete_coalesce(&mut self, other: &Node_) -> bool {
        false
    }

    fn delete(&self, ctx: &mut BuildContext) {
        ctx.stmt(
            Tokens::new().s("alter table").id(&self.table_id).s("drop constraint").id(&self.def.id).to_string(),
        );
    }
}

#[derive(Clone)]
struct NodeTableIndex_ {
    table_id: String,
    def: TableIndexDef,
}

impl MigrateNodeDispatch_ for NodeTableIndex_ {
    fn create_coalesce(&mut self, other: &Node_) -> bool {
        false
    }

    fn create(&self, ctx: &mut BuildContext) {
        ctx.stmt(Tokens::new().s("create").f(|t| {
            if self.def.unique {
                t.s("unique");
            }
        }).s("index").id(&self.def.id).s("on").id(&self.table_id).s("(").f(|t| {
            for (i, id) in self.def.field_ids.iter().enumerate() {
                if i > 0 {
                    t.s(",");
                }
                t.id(&id.1);
            }
        }).s(")").to_string());
    }

    fn delete_coalesce(&mut self, other: &Node_) -> bool {
        false
    }

    fn delete(&self, ctx: &mut BuildContext) {
        ctx.stmt(Tokens::new().s("drop index").id(&self.def.id).to_string());
    }
}

#[enum_dispatch]
trait MigrateNodeDispatch_ {
    fn create_coalesce(&mut self, other: &Node_) -> bool;
    fn create(&self, ctx: &mut BuildContext);
    fn delete_coalesce(&mut self, other: &Node_) -> bool;
    fn delete(&self, ctx: &mut BuildContext);
}

trait MigrateNode_: MigrateNodeDispatch_ {
    fn update(&self, ctx: &mut BuildContext, old: &Self);
}

#[derive(Clone)]
#[enum_dispatch(MigrateNodeDispatch_)]
//. #[samevariant(PairwiseNode_)]
enum Node_ {
    Table(TableDef),
    Field(NodeField_),
    TableConstraint(NodeTableConstraint_),
    TableIndex(NodeTableIndex_),
}

type Version = super::Version<Node_, Id_>;

impl Table {
    fn new(v: &mut Version, d: TableDef) -> Table {
        let out = Table(d.id.clone());
        v.schema.insert(Id_::Table(out.clone()), crate::Node {
            deps: vec![],
            body: Node_::Table(d),
        });
        out
    }

    fn field(&self, v: &mut Version, d: FieldDef) -> Field {
        let out = Field(self.0.clone(), d.id.clone());
        v.schema.insert(Id_::Field(out.clone()), crate::Node {
            deps: vec![Id_::Table(self.clone())],
            body: Node_::Field(NodeField_ {
                table_id: self.0.clone(),
                def: d,
            }),
        });
        out
    }

    fn primary_key(&self, v: &mut Version, d: TableConstraintDef) {
        let mut deps = vec![Id_::Table(self.clone())];
        match &d.type_ {
            TableConstraintTypeDef::PrimaryKey(x) => {
                for f in x.fields {
                    if f.0 != self.0 {
                        ctx.err(
                            format!(
                                "Field {} in primary key constraint {} is in table {}, but constraint is in table {}",
                                f.1,
                                d.id,
                                self.0,
                                f.0
                            ),
                        );
                    }
                    deps.push(Id_::Field(f));
                }
            },
            TableConstraintTypeDef::ForeignKey(x) => {
                let last_foreign_table = None;
                for f in x.fields {
                    if f.0.0 != self.0 {
                        ctx.err(
                            format!(
                                "Local field {} in foreign key constraint {} is in table {}, but constraint is in table {}",
                                f.0.1,
                                d.id,
                                self.0,
                                f.1.0
                            ),
                        );
                    }
                    deps.push(Id_::Field(f.0));
                    if let Some(t) = last_foreign_table.take() {
                        if t != f.1.0 {
                            ctx.err(
                                format!(
                                    "Foreign field {} in foreign key constraint {} is in table {}, but constraint is in table {}",
                                    f.1.1,
                                    d.id,
                                    t,
                                    f.1.0
                                ),
                            );
                        }
                    }
                    last_foreign_table = Some(f.1.0);
                    deps.push(Id_::Field(f.1));
                }
            },
        }
        v.schema.insert(Id_::TableConstraint(self.0.clone(), d.id.clone()), crate::Node {
            deps: deps,
            body: Node_::TableConstraint(NodeTableConstraint_ {
                table_id: self.0.clone(),
                def: d,
            }),
        });
    }

    fn index(&self, v: &mut Version, d: TableIndexDef) -> Index {
        let out = Index(self.0.clone(), d.id.clone());
        v.schema.insert(Id_::TableIndex(out.clone()), crate::Node {
            deps: vec![Id_::Table(self.clone())],
            body: Node_::TableIndex(NodeTableIndex_ {
                table_id: self.0.clone(),
                def: d,
            }),
        });
        out
    }
}

impl MigrateNodeVersion for Node_ {
    fn compare(&self, other: &Self) -> crate::MigrateNodeVersionComparison {
        match PairwiseNode_::pairs(self, other) {
            PairwiseNode_::Table(_, _) => crate::MigrateNodeVersionComparison::Keep,
            PairwiseNode_::Field(current, old) => if current.def.type_ == old.def.type_ {
                MigrateNodeVersionComparison::Keep
            } else {
                MigrateNodeVersionComparison::Update
            },
            PairwiseNode_::TablePrimaryKey(current, old) => todo!(),
            PairwiseNode_::TableForeignKey(current, old) => todo!(),
            PairwiseNode_::TableIndex(current, old) => {
                if current.def.field_ids == old.def.field_ids {
                    MigrateNodeVersionComparison::Keep
                } else {
                    MigrateNodeVersionComparison::DeleteCreate
                }
            },
            PairwiseNode_::Nonmatching(_, _) => unreachable!(),
        }
    }

    fn create(&self, ctx: &mut BuildContext) {
        MigrateNodeDispatch_::create(self, stmts)
    }

    fn delete(&self, ctx: &mut BuildContext) {
        MigrateNodeDispatch_::delete(self, stmts)
    }

    fn update(&self, ctx: &mut BuildContext, old: &Self) {
        match PairwiseNode_::pairs(self, old) {
            PairwiseNode_::Table(current, old) => current.update(stmts, &old),
            PairwiseNode_::Field(current, old) => current.update(stmts, &old),
            PairwiseNode_::TablePrimaryKey(current, old) => current.update(stmts, &old),
            PairwiseNode_::TableForeignKey(current, old) => current.update(stmts, &old),
            PairwiseNode_::TableIndex(current, old) => current.update(stmts, &old),
            PairwiseNode_::Nonmatching(_, _) => unreachable!(),
        }
    }

    fn create_coalesce(&mut self, other: &Self) -> bool {
        MigrateNodeDispatch_::create_coalesce(self, other)
    }

    fn delete_coalesce(&mut self, other: &Self) -> bool {
        MigrateNodeDispatch_::delete_coalesce(self, other)
    }
}
