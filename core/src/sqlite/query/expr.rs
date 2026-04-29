use {
    serde::{
        Serialize,
        Deserialize,
    },
    chrono::FixedOffset,
    quote::{
        quote,
        format_ident,
        ToTokens,
    },
    syn::Path,
    std::{
        collections::HashMap,
        rc::Rc,
        fmt::Display,
    },
    chrono::{
        DateTime,
        Utc,
    },
    crate::{
        sqlite::{
            types::{
                Type,
                to_rust_types,
                SimpleSimpleType,
                SimpleType,
            },
            query::utils::{
                SqliteQueryCtx,
                QueryBody,
            },
            schema::{
                field::FieldRef,
                table::TableRef,
            },
        },
        utils::{
            Tokens,
            sanitize_ident,
            Errs,
        },
    },
};
#[cfg(feature = "jiff")]
use jiff::Timestamp;
use super::select::{
    Select,
    Order,
};

#[derive(Clone)]
pub struct ExprType(pub Vec<(Binding, Type)>);

impl ExprType {
    pub fn assert_scalar(&self, errs: &mut Errs, path: &rpds::Vector<String>) -> Option<Type> {
        if self.0.len() != 1 {
            errs.err(path, format!("Expected scalar expression but got {} fields", self.0.len()));
            return None;
        }
        return Some(self.0[0].1.clone());
    }
}

pub struct ComputeType(pub Rc<dyn Fn(&mut SqliteQueryCtx, &rpds::Vector<String>, &[ExprType]) -> ExprType>);

impl Clone for ComputeType {
    fn clone(&self) -> Self {
        return ComputeType(self.0.clone());
    }
}

impl std::fmt::Debug for ComputeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return f.write_str("ComputeType");
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SerialWindowFrameType {
    Rows,
    Range,
    Groups,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SerialWindowFrameBound {
    UnboundedPreceding,
    Preceding(Box<SerialExpr>),
    CurrentRow,
    Following(Box<SerialExpr>),
    UnboundedFollowing,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SerialWindowFrameExclude {
    CurrentRow,
    Group,
    Ties,
    NoOthers,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SerialWindowFrame {
    pub type_: SerialWindowFrameType,
    pub start: SerialWindowFrameBound,
    pub end: Option<SerialWindowFrameBound>,
    pub exclude: Option<SerialWindowFrameExclude>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SerialExpr {
    LitArray(Vec<SerialExpr>),
    LitNull(SimpleType),
    LitBool(bool),
    LitAuto(i64),
    LitI16(i16),
    LitI32(i32),
    LitI64(i64),
    LitU32(u32),
    LitF32(f32),
    LitF64(f64),
    LitString(String),
    LitBytes(Vec<u8>),
    #[cfg(feature = "chrono")]
    LitUtcTimeSChrono(DateTime<Utc>),
    #[cfg(feature = "chrono")]
    LitUtcTimeMsChrono(DateTime<Utc>),
    #[cfg(feature = "chrono")]
    LitFixedOffsetTimeChrono(DateTime<FixedOffset>),
    #[cfg(feature = "jiff")]
    LitUtcTimeSJiff(Timestamp),
    #[cfg(feature = "jiff")]
    LitUtcTimeMsJiff(Timestamp),
    BinOp {
        left: Box<SerialExpr>,
        op: BinOp,
        right: Box<SerialExpr>,
    },
    BinOpChain {
        op: BinOp,
        exprs: Vec<SerialExpr>,
    },
    PrefixOp {
        op: PrefixOp,
        right: Box<SerialExpr>,
    },
    Cast(Box<SerialExpr>, Type),
    Collate(Box<SerialExpr>, String),
    Like {
        expr: Box<SerialExpr>,
        pattern: Box<SerialExpr>,
        escape: Option<Box<SerialExpr>>,
        glob: bool,
    },
    Between {
        e: Box<SerialExpr>,
        negated: bool,
        low: Box<SerialExpr>,
        high: Box<SerialExpr>,
    },
}

impl From<SerialWindowFrameType> for WindowFrameType {
    fn from(s: SerialWindowFrameType) -> Self {
        match s {
            SerialWindowFrameType::Rows => WindowFrameType::Rows,
            SerialWindowFrameType::Range => WindowFrameType::Range,
            SerialWindowFrameType::Groups => WindowFrameType::Groups,
        }
    }
}

impl From<SerialWindowFrameBound> for WindowFrameBound {
    fn from(s: SerialWindowFrameBound) -> Self {
        match s {
            SerialWindowFrameBound::UnboundedPreceding => WindowFrameBound::UnboundedPreceding,
            SerialWindowFrameBound::Preceding(e) => WindowFrameBound::Preceding(Box::new(Expr::from(*e))),
            SerialWindowFrameBound::CurrentRow => WindowFrameBound::CurrentRow,
            SerialWindowFrameBound::Following(e) => WindowFrameBound::Following(Box::new(Expr::from(*e))),
            SerialWindowFrameBound::UnboundedFollowing => WindowFrameBound::UnboundedFollowing,
        }
    }
}

impl From<SerialWindowFrameExclude> for WindowFrameExclude {
    fn from(s: SerialWindowFrameExclude) -> Self {
        match s {
            SerialWindowFrameExclude::CurrentRow => WindowFrameExclude::CurrentRow,
            SerialWindowFrameExclude::Group => WindowFrameExclude::Group,
            SerialWindowFrameExclude::Ties => WindowFrameExclude::Ties,
            SerialWindowFrameExclude::NoOthers => WindowFrameExclude::NoOthers,
        }
    }
}

impl From<SerialWindowFrame> for WindowFrame {
    fn from(s: SerialWindowFrame) -> Self {
        WindowFrame {
            type_: WindowFrameType::from(s.type_),
            start: WindowFrameBound::from(s.start),
            end: s.end.map(WindowFrameBound::from),
            exclude: s.exclude.map(WindowFrameExclude::from),
        }
    }
}

impl From<SerialExpr> for Expr {
    fn from(s: SerialExpr) -> Self {
        match s {
            SerialExpr::LitArray(v) => Expr::LitArray(v.into_iter().map(Expr::from).collect()),
            SerialExpr::LitNull(t) => Expr::LitNull(t),
            SerialExpr::LitBool(b) => Expr::LitBool(b),
            SerialExpr::LitAuto(v) => Expr::LitAuto(v),
            SerialExpr::LitI16(v) => Expr::LitI16(v),
            SerialExpr::LitI32(v) => Expr::LitI32(v),
            SerialExpr::LitI64(v) => Expr::LitI64(v),
            SerialExpr::LitU32(v) => Expr::LitU32(v),
            SerialExpr::LitF32(v) => Expr::LitF32(v),
            SerialExpr::LitF64(v) => Expr::LitF64(v),
            SerialExpr::LitString(v) => Expr::LitString(v),
            SerialExpr::LitBytes(v) => Expr::LitBytes(v),
            #[cfg(feature = "chrono")]
            SerialExpr::LitUtcTimeSChrono(v) => Expr::LitUtcTimeSChrono(v),
            #[cfg(feature = "chrono")]
            SerialExpr::LitUtcTimeMsChrono(v) => Expr::LitUtcTimeMsChrono(v),
            #[cfg(feature = "chrono")]
            SerialExpr::LitFixedOffsetTimeChrono(v) => Expr::LitFixedOffsetTimeChrono(v),
            #[cfg(feature = "jiff")]
            SerialExpr::LitUtcTimeSJiff(v) => Expr::LitUtcTimeSJiff(v),
            #[cfg(feature = "jiff")]
            SerialExpr::LitUtcTimeMsJiff(v) => Expr::LitUtcTimeMsJiff(v),
            SerialExpr::BinOp { left, op, right } => Expr::BinOp {
                left: Box::new(Expr::from(*left)),
                op: op,
                right: Box::new(Expr::from(*right)),
            },
            SerialExpr::BinOpChain { op, exprs } => Expr::BinOpChain {
                op: op,
                exprs: exprs.into_iter().map(Expr::from).collect(),
            },
            SerialExpr::PrefixOp { op, right } => Expr::PrefixOp {
                op: op,
                right: Box::new(Expr::from(*right)),
            },
            SerialExpr::Cast(e, t) => Expr::Cast(Box::new(Expr::from(*e)), t),
            SerialExpr::Collate(e, s) => Expr::Collate(Box::new(Expr::from(*e)), s),
            SerialExpr::Like { expr, pattern, escape, glob } => Expr::Like {
                expr: Box::new(Expr::from(*expr)),
                pattern: Box::new(Expr::from(*pattern)),
                escape: escape.map(|e| Box::new(Expr::from(*e))),
                glob,
            },
            SerialExpr::Between { e, negated, low, high } => Expr::Between {
                e: Box::new(Expr::from(*e)),
                negated,
                low: Box::new(Expr::from(*low)),
                high: Box::new(Expr::from(*high)),
            },
        }
    }
}

#[derive(Clone, Debug)]
pub enum WindowFrameType {
    Rows,
    Range,
    Groups,
}

#[derive(Clone, Debug)]
pub enum WindowFrameBound {
    UnboundedPreceding,
    Preceding(Box<Expr>),
    CurrentRow,
    Following(Box<Expr>),
    UnboundedFollowing,
}

#[derive(Clone, Debug)]
pub enum WindowFrameExclude {
    CurrentRow,
    Group,
    Ties,
    NoOthers,
}

#[derive(Clone, Debug)]
pub struct WindowFrame {
    pub type_: WindowFrameType,
    pub start: WindowFrameBound,
    pub end: Option<WindowFrameBound>,
    pub exclude: Option<WindowFrameExclude>,
}

#[derive(Clone, Debug)]
pub enum Expr {
    LitArray(Vec<Expr>),
    // A null value needs a type for type checking purposes. It will always be trated
    // as an optional value.
    LitNull(SimpleType),
    LitBool(bool),
    LitAuto(i64),
    LitI16(i16),
    LitI32(i32),
    LitI64(i64),
    LitU32(u32),
    LitF32(f32),
    LitF64(f64),
    LitString(String),
    LitBytes(Vec<u8>),
    #[cfg(feature = "chrono")]
    LitUtcTimeSChrono(DateTime<Utc>),
    #[cfg(feature = "chrono")]
    LitUtcTimeMsChrono(DateTime<Utc>),
    #[cfg(feature = "chrono")]
    LitFixedOffsetTimeChrono(DateTime<FixedOffset>),
    #[cfg(feature = "jiff")]
    LitUtcTimeSJiff(Timestamp),
    #[cfg(feature = "jiff")]
    LitUtcTimeMsJiff(Timestamp),
    /// A query parameter. This will become a parameter to the generated Rust function
    /// with the specified `name` and `type_`.
    Param {
        name: String,
        type_: Type,
    },
    /// This evaluates to the value of a field in the query main or joined tables. If
    /// you've aliased tables or field names, you'll have to instantiate `FieldId`
    /// yourself with the appropriate values. For synthetic values like function
    /// results you may need a `FieldId` with an empty `TableId` (`""`).
    Field(FieldRef),
    BinOp {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },
    BinOpChain {
        op: BinOp,
        exprs: Vec<Expr>,
    },
    PrefixOp {
        op: PrefixOp,
        right: Box<Expr>,
    },
    Call {
        func: String,
        args: Vec<Expr>,
        compute_type: ComputeType,
        filter: Option<Box<Expr>>,
    },
    Window {
        expr: Box<Expr>,
        partition_by: Vec<Expr>,
        order_by: Vec<(Expr, Order)>,
        frame: Option<WindowFrame>,
    },
    /// A sub SELECT query.
    Select(Box<Select>),
    /// This is a synthetic expression, saying to treat the result of the expression as
    /// having the specified type. Use this for casting between primitive types and
    /// Rust new-types for instance.
    Cast(Box<Expr>, Type),
    Exists(Box<super::select::Select>),
    Collate(Box<Expr>, String),
    Like {
        expr: Box<Expr>,
        pattern: Box<Expr>,
        escape: Option<Box<Expr>>,
        glob: bool,
    },
    Between {
        e: Box<Expr>,
        negated: bool,
        low: Box<Expr>,
        high: Box<Expr>,
    },
    Case {
        operand: Option<Box<Expr>>,
        conditions: Vec<(Expr, Expr)>,
        else_: Option<Box<Expr>>,
    },
}


#[derive(Clone, Hash, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Binding {
    pub table_id: String,
    pub id: String,
}

impl Binding {
    pub fn local(name: String) -> Self {
        Binding {
            table_id: "".into(),
            id: name,
        }
    }

    pub fn empty() -> Self {
        Binding {
            table_id: "".into(),
            id: "".into(),
        }
    }

    pub fn field(f: &FieldRef) -> Self {
        Binding {
            table_id: f.table_id.clone(),
            id: f.field_id.clone(),
        }
    }

    pub fn with_alias(&self, s: &str) -> Binding {
        Binding {
            table_id: s.into(),
            id: self.id.clone(),
        }
    }
}

impl Display for Binding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.table_id.is_empty() {
            return Display::fmt(&self.id, f);
        } else {
            return Display::fmt(&format!("{}.{}", self.table_id, self.id), f);
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BinOp {
    Plus,
    Minus,
    Multiply,
    Divide,
    And,
    Or,
    Equals,
    NotEquals,
    Is,
    IsNot,
    LessThan,
    LessThanEqualTo,
    GreaterThan,
    GreaterThanEqualTo,
    Like,
    In,
    NotIn,
    StringConcat,
    Mod,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    BitwiseShiftLeft,
    BitwiseShiftRight,
    IsDistinctFrom,
    IsNotDistinctFrom,
    Glob,
    Regexp,
    Match,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PrefixOp {
    Not,
    BitwiseNot,
    Minus,
}

macro_rules! empty_type{
    ($out: expr, $t: expr) => {
        (ExprType(vec![(Binding::empty(), Type {
            type_: SimpleType {
                type_: $t,
                custom: None,
            },
            opt: false,
            arr: false,
        })]), $out)
    };
}

impl WindowFrame {
    pub fn build(
        &self,
        ctx: &mut SqliteQueryCtx,
        path: &rpds::Vector<String>,
        scope: &HashMap<Binding, Type>,
    ) -> Tokens {
        let mut out = Tokens::new();
        match self.type_ {
            WindowFrameType::Rows => {
                out.s("rows");
            },
            WindowFrameType::Range => {
                out.s("range");
            },
            WindowFrameType::Groups => {
                out.s("groups");
            },
        }
        if let Some(end) = &self.end {
            out.s("between");
            out.s(&self.start.build(ctx, &path.push_back("Start".into()), scope).to_string());
            out.s("and");
            out.s(&end.build(ctx, &path.push_back("End".into()), scope).to_string());
        } else {
            out.s(&self.start.build(ctx, &path.push_back("Start".into()), scope).to_string());
        }
        if let Some(exclude) = &self.exclude {
            out.s("exclude");
            match exclude {
                WindowFrameExclude::CurrentRow => {
                    out.s("current row");
                },
                WindowFrameExclude::Group => {
                    out.s("group");
                },
                WindowFrameExclude::Ties => {
                    out.s("ties");
                },
                WindowFrameExclude::NoOthers => {
                    out.s("no others");
                },
            }
        }
        out
    }
}

impl WindowFrameBound {
    pub fn build(
        &self,
        ctx: &mut SqliteQueryCtx,
        path: &rpds::Vector<String>,
        scope: &HashMap<Binding, Type>,
    ) -> Tokens {
        let mut out = Tokens::new();
        match self {
            WindowFrameBound::UnboundedPreceding => {
                out.s("unbounded preceding");
            },
            WindowFrameBound::Preceding(e) => {
                let (_, tokens) = e.build(ctx, path, scope);
                out.s(&tokens.to_string()).s("preceding");
            },
            WindowFrameBound::CurrentRow => {
                out.s("current row");
            },
            WindowFrameBound::Following(e) => {
                let (_, tokens) = e.build(ctx, path, scope);
                out.s(&tokens.to_string()).s("following");
            },
            WindowFrameBound::UnboundedFollowing => {
                out.s("unbounded following");
            },
        }
        out
    }
}


impl Expr {
    pub fn build(
        &self,
        ctx: &mut SqliteQueryCtx,
        path: &rpds::Vector<String>,
        scope: &HashMap<Binding, Type>,
    ) -> (ExprType, Tokens) {
        match self {
            Expr::LitArray(res) => {
                let mut out = Tokens::new();
                out.s("(");
                let mut types = vec![];
                for (i, res) in res.iter().enumerate() {
                    if i > 0 {
                        out.s(",");
                    }
                    let (t, tokens) = res.build(ctx, path, scope);
                    out.s(&tokens.to_string());
                    types.push(t);
                }
                out.s(")");
                return (ExprType(types.into_iter().flat_with_index(|i, t| t.0.into_iter().map(move |(mut k, v)| {
                    if k.id.is_empty() {
                        k.id = format!("_{}", i);
                    }
                    (k, v)
                })).collect()), out);
            },
            Expr::LitNull(t) => {
                let mut out = Tokens::new();
                out.s("null");
                return (ExprType(vec![(Binding::empty(), Type {
                    type_: t.clone(),
                    opt: true,
                    arr: false,
                })]), out);
            },
            Expr::LitBool(x) => {
                let mut out = Tokens::new();
                out.s(if *x {
                    "1"
                } else {
                    "0"
                });
                return empty_type!(out, SimpleSimpleType::Bool);
            },
            Expr::LitAuto(x) => {
                let mut out = Tokens::new();
                out.s(&x.to_string());
                return empty_type!(out, SimpleSimpleType::Auto);
            },
            Expr::LitI16(x) => {
                let mut out = Tokens::new();
                out.s(&x.to_string());
                return empty_type!(out, SimpleSimpleType::I16);
            },
            Expr::LitI32(x) => {
                let mut out = Tokens::new();
                out.s(&x.to_string());
                return empty_type!(out, SimpleSimpleType::I32);
            },
            Expr::LitI64(x) => {
                let mut out = Tokens::new();
                out.s(&x.to_string());
                return empty_type!(out, SimpleSimpleType::I64);
            },
            Expr::LitU32(x) => {
                let mut out = Tokens::new();
                out.s(&x.to_string());
                return empty_type!(out, SimpleSimpleType::U32);
            },
            Expr::LitF32(x) => {
                let mut out = Tokens::new();
                out.s(&x.to_string());
                return empty_type!(out, SimpleSimpleType::F32);
            },
            Expr::LitF64(x) => {
                let mut out = Tokens::new();
                out.s(&x.to_string());
                return empty_type!(out, SimpleSimpleType::F64);
            },
            Expr::LitString(x) => {
                let mut out = Tokens::new();
                out.s(&format!("'{}'", x.replace("'", "''")));
                return empty_type!(out, SimpleSimpleType::String);
            },
            Expr::LitBytes(x) => {
                let mut out = Tokens::new();
                let h = hex::encode(&x);
                out.s(&format!("x'{}'", h));
                return empty_type!(out, SimpleSimpleType::Bytes);
            },
            #[cfg(feature = "chrono")]
            Expr::LitUtcTimeSChrono(d) => {
                let mut out = Tokens::new();
                out.s(&format!("{}", d.timestamp()));
                return empty_type!(out, SimpleSimpleType::UtcTimeSChrono);
            },
            #[cfg(feature = "chrono")]
            Expr::LitUtcTimeMsChrono(d) => {
                let mut out = Tokens::new();
                let d = d.to_rfc3339();
                out.s(&format!("'{}'", d));
                return empty_type!(out, SimpleSimpleType::UtcTimeMsChrono);
            },
            #[cfg(feature = "chrono")]
            Expr::LitFixedOffsetTimeChrono(d) => {
                let mut out = Tokens::new();
                let d = d.to_rfc3339();
                out.s(&format!("'{}'", d));
                return empty_type!(out, SimpleSimpleType::FixedOffsetTimeChrono);
            },
            #[cfg(feature = "jiff")]
            Expr::LitUtcTimeSJiff(d) => {
                let mut out = Tokens::new();
                out.s(&format!("{}", d.as_second()));
                return empty_type!(out, SimpleSimpleType::UtcTimeSJiff);
            },
            #[cfg(feature = "jiff")]
            Expr::LitUtcTimeMsJiff(d) => {
                let mut out = Tokens::new();
                let d = d.to_string();
                out.s(&format!("'{}'", d));
                return empty_type!(out, SimpleSimpleType::UtcTimeMsJiff);
            },
            Expr::Param { name: x, type_: t } => {
                let path = path.push_back(format!("Param ({})", x));
                let mut out = Tokens::new();
                let i = match ctx.rust_arg_lookup.entry(x.clone()) {
                    std::collections::hash_map::Entry::Occupied(e) => {
                        let (i, prev_t) = e.get();
                        if t != prev_t {
                            ctx
                                .errs
                                .err(
                                    &path,
                                    format!("Parameter {} specified with multiple types: {:?}, {:?}", x, t, prev_t),
                                );
                        }
                        *i
                    },
                    std::collections::hash_map::Entry::Vacant(e) => {
                        let i = ctx.query_args.len();
                        e.insert((i, t.clone()));
                        let rust_types = to_rust_types(&t.type_.type_);
                        let custom_trait_ident = rust_types.custom_trait;
                        let rust_type = rust_types.arg_type;
                        let ident = format_ident!("{}", sanitize_ident(x).1);
                        let (mut rust_type, mut rust_forward) = if let Some(custom) = &t.type_.custom {
                            let custom_ident = match syn::parse_str::<Path>(custom.as_str()) {
                                Ok(p) => p,
                                Err(e) => {
                                    ctx.errs.err(&path, format!("Couldn't parse custom type {}: {:?}", custom, e));
                                    return (ExprType(vec![]), Tokens::new());
                                },
                            }.to_token_stream();
                            let forward =
                                quote!(< #custom_ident as #custom_trait_ident < #custom_ident >>:: to_sql(& #ident));
                            (quote!(& #custom_ident), forward)
                        } else {
                            (rust_type, quote!(#ident))
                        };
                        if t.opt {
                            rust_type = quote!(Option < #rust_type >);
                            rust_forward = quote!(#ident.map(| #ident | #rust_forward));
                        }
                        rust_forward = match &t.type_.type_ {
                            SimpleSimpleType::UtcTimeSChrono => {
                                if t.opt {
                                    quote!(
                                        #rust_forward.map(
                                            |x| good_ormning::runtime::sqlite::GoodOrmningSqliteTimestamp::I64(
                                                x.timestamp()
                                            )
                                        )
                                    )
                                } else {
                                    quote!(
                                        good_ormning:: runtime:: sqlite:: GoodOrmningSqliteTimestamp:: I64(
                                            #rust_forward.timestamp()
                                        )
                                    )
                                }
                            },
                            SimpleSimpleType::UtcTimeMsChrono => {
                                if t.opt {
                                    quote!(
                                        #rust_forward.map(
                                            |x| good_ormning::runtime::sqlite::GoodOrmningSqliteTimestamp::String(
                                                x.to_rfc3339()
                                            )
                                        )
                                    )
                                } else {
                                    quote!(
                                        good_ormning:: runtime:: sqlite:: GoodOrmningSqliteTimestamp:: String(
                                            #rust_forward.to_rfc3339()
                                        )
                                    )
                                }
                            },
                            SimpleSimpleType::UtcTimeSJiff => {
                                if t.opt {
                                    quote!(
                                        #rust_forward.map(
                                            |x| good_ormning::runtime::sqlite::GoodOrmningSqliteTimestamp::I64(
                                                x.as_second()
                                            )
                                        )
                                    )
                                } else {
                                    quote!(
                                        good_ormning:: runtime:: sqlite:: GoodOrmningSqliteTimestamp:: I64(
                                            #rust_forward.as_second()
                                        )
                                    )
                                }
                            },
                            SimpleSimpleType::UtcTimeMsJiff => {
                                if t.opt {
                                    quote!(
                                        #rust_forward.map(
                                            |x| good_ormning::runtime::sqlite::GoodOrmningSqliteTimestamp::String(
                                                x.to_string()
                                            )
                                        )
                                    )
                                } else {
                                    quote!(
                                        good_ormning:: runtime:: sqlite:: GoodOrmningSqliteTimestamp:: String(
                                            #rust_forward.to_string()
                                        )
                                    )
                                }
                            },
                            _ => rust_forward,
                        };
                        if t.arr {
                            rust_type = quote!(Vec < #rust_type >);
                            rust_forward =
                                quote!(
                                    std:: rc:: Rc:: new(
                                        #ident.into_iter(
                                        ).map(
                                            | #ident | rusqlite:: types:: Value:: from(#rust_forward)
                                        ).collect::< Vec < _ >>()
                                    )
                                );
                        }
                        ctx.rust_args.push(quote!(#ident: #rust_type));
                        ctx.query_args.push(quote!(#rust_forward));
                        i
                    },
                };
                if t.arr {
                    out.s(&format!("rarray(?{})", i + 1));
                } else {
                    out.s(&format!("?{}", i + 1));
                }
                return (ExprType(vec![(Binding::local(x.clone()), t.clone())]), out);
            },
            Expr::Field(x) => {
                let name = Binding::field(x);
                let t = match scope.get(&name) {
                    Some(t) => t.clone(),
                    None => {
                        if x.table_id.is_empty() {
                            let mut found = vec![];
                            for (k, v) in scope {
                                if k.id == x.field_id {
                                    found.push(v.clone());
                                }
                            }
                            if found.len() == 1 {
                                found[0].clone()
                            } else if found.len() > 1 {
                                ctx.errs.err(path, format!("Field {:?} is ambiguous (found in multiple tables)", x.field_id));
                                return (ExprType(vec![]), Tokens::new());
                            } else {
                                ctx.errs.err(path, format!("Field {:?} not found in any table", x.field_id));
                                return (ExprType(vec![]), Tokens::new());
                            }
                        } else {
                            ctx
                                .errs
                                .err(
                                    path,
                                    format!(
                                        "Expression references {:?} but this field isn't available here (available fields: {:?})",
                                        x,
                                        scope.iter().map(|e| e.0.to_string()).collect::<Vec<String>>()
                                    ),
                                );
                            return (ExprType(vec![]), Tokens::new());
                        }
                    },
                };
                let mut out = Tokens::new();
                if x.table_id.is_empty() {
                    out.id(&x.field_id);
                } else {
                    let table_info =
                        ctx
                            .tables
                            .get(&TableRef(x.table_id.clone()))
                            .unwrap_or_else(|| panic!("Table {:?} not found in context", x.table_id));
                    let field_info =
                        table_info
                            .fields
                            .get(x)
                            .unwrap_or_else(|| panic!("Field {:?} not found in table {:?}", x.field_id, x.table_id));
                    out.id(&table_info.sql_name).s(".").id(&field_info.sql_name);
                }
                return (ExprType(vec![(name, t.clone())]), out);
            },
            Expr::BinOp { left, op, right } => {
                return do_bin_op(
                    ctx,
                    &path.push_back(format!("Bin op {:?}", op)),
                    scope,
                    op,
                    &vec![left.as_ref().clone(), right.as_ref().clone()],
                );
            },
            Expr::BinOpChain { op, exprs } => {
                return do_bin_op(ctx, &path.push_back(format!("Bin op chain {:?}", op)), scope, op, exprs);
            },
            Expr::PrefixOp { op, right } => {
                let mut out = Tokens::new();
                match op {
                    PrefixOp::Not => {
                        out.s("not");
                    },
                    PrefixOp::BitwiseNot => {
                        out.s("~");
                    },
                    PrefixOp::Minus => {
                        out.s("-");
                    },
                }
                let (t, tokens) = right.build(ctx, &path.push_back(format!("Prefix op {:?}", op)), scope);
                if matches!(op, PrefixOp::Not) {
                    check_bool(ctx, path, &t);
                }
                out.s(&tokens.to_string());
                return (t, out);
            },
            Expr::Call { func, args, compute_type, filter } => {
                let mut out = Tokens::new();
                out.s(func).s("(");
                let mut arg_types = vec![];
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        out.s(",");
                    }
                    let (t, tokens) = arg.build(ctx, &path.push_back(format!("Call arg {}", i)), scope);
                    out.s(&tokens.to_string());
                    arg_types.push(t);
                }
                out.s(")");
                if let Some(filter) = filter {
                    out.s("filter (where");
                    let (t, tokens) = filter.build(ctx, &path.push_back("Filter".into()), scope);
                    check_bool(ctx, &path.push_back("Filter".into()), &t);
                    out.s(&tokens.to_string()).s(")");
                }
                return (compute_type.0(ctx, path, &arg_types), out);
            },
            Expr::Window { expr, partition_by, order_by, frame } => {
                let mut out = Tokens::new();
                let (t, tokens) = expr.build(ctx, &path.push_back("Window expr".into()), scope);
                out.s(&tokens.to_string()).s("over (");
                let mut first = true;
                if !partition_by.is_empty() {
                    out.s("partition by");
                    for (i, p) in partition_by.iter().enumerate() {
                        if i > 0 {
                            out.s(",");
                        }
                        let (_, tokens) = p.build(ctx, &path.push_back(format!("Partition by {}", i)), scope);
                        out.s(&tokens.to_string());
                    }
                    first = false;
                }
                if !order_by.is_empty() {
                    if !first {
                        out.s(" ");
                    }
                    out.s("order by");
                    for (i, (e, o)) in order_by.iter().enumerate() {
                        if i > 0 {
                            out.s(",");
                        }
                        let (_, tokens): (ExprType, Tokens) =
                            e.build(ctx, &path.push_back(format!("Order by {}", i)), scope);
                        out.s(&tokens.to_string());
                        match o {
                            Order::Asc => {
                                out.s("asc");
                            },
                            Order::Desc => {
                                out.s("desc");
                            },
                        }
                    }
                    first = false;
                }
                if let Some(frame) = frame {
                    if !first {
                        out.s(" ");
                    }
                    out.s(&frame.build(ctx, &path.push_back("Frame".into()), scope).to_string());
                }
                out.s(")");
                return (t, out);
            },
            Expr::Select(s) => {
                let mut out = Tokens::new();
                out.s("(");
                let (t, tokens) =
                    s.build(ctx, &path.push_back("Subselect".into()), crate::QueryResCount::Many);
                out.s(&tokens.to_string()).s(")");
                return (t, out);
            },
            Expr::Cast(e, t) => {
                let mut out = Tokens::new();
                let (got_t, tokens): (ExprType, Tokens) = e.build(ctx, &path.push_back("Cast".into()), scope);
                check_general_same(ctx, path, &got_t, &ExprType(vec![(Binding::empty(), t.clone())]));
                out
                    .s("cast (")
                    .s(&tokens.to_string())
                    .s("as")
                    .s(crate::sqlite::types::to_sql_type(&t.type_.type_))
                    .s(")");
                return (ExprType(vec![(Binding::empty(), t.clone())]), out);
            },
            Expr::Exists(s) => {
                let mut out = Tokens::new();
                out.s("exists (");
                let (_, tokens) =
                    s.build(ctx, &path.push_back("Exists".into()), crate::QueryResCount::Many);
                out.s(&tokens.to_string()).s(")");
                return (ExprType(vec![(Binding::empty(), Type {
                    type_: SimpleType {
                        type_: SimpleSimpleType::Bool,
                        custom: None,
                    },
                    opt: false,
                    arr: false,
                })]), out);
            },
            Expr::Collate(e, s) => {
                let mut out = Tokens::new();
                let (t, tokens) = e.build(ctx, &path.push_back("Collate".into()), scope);
                out.s(&tokens.to_string()).s("collate").s(&format!("\"{}\"", s.replace("\"", "\"\"")));
                return (t, out);
            },
            Expr::Like { expr, pattern, escape, glob } => {
                let mut out = Tokens::new();
                let (t_expr, tokens_expr) = expr.build(ctx, &path.push_back("Like expr".into()), scope);
                let (t_pattern, tokens_pattern) = pattern.build(ctx, &path.push_back("Like pattern".into()), scope);
                let want_t = Type {
                    type_: SimpleType {
                        type_: SimpleSimpleType::String,
                        custom: None,
                    },
                    opt: false,
                    arr: false,
                };
                if let Some(got_t) = t_expr.assert_scalar(&mut ctx.errs, &path.push_back("Like expr".into())) {
                    check_general_same_type(ctx, &path.push_back("Like expr".into()), &got_t, &want_t);
                }
                if let Some(got_t) = t_pattern.assert_scalar(&mut ctx.errs, &path.push_back("Like pattern".into())) {
                    check_general_same_type(ctx, &path.push_back("Like pattern".into()), &got_t, &want_t);
                }
                out.s(&tokens_expr.to_string());
                if *glob {
                    out.s("glob");
                } else {
                    out.s("like");
                }
                out.s(&tokens_pattern.to_string());
                if let Some(escape) = escape {
                    let (t_escape, tokens_escape) = escape.build(ctx, &path.push_back("Like escape".into()), scope);
                    if let Some(got_t) = t_escape.assert_scalar(&mut ctx.errs, &path.push_back("Like escape".into())) {
                        check_general_same_type(ctx, &path.push_back("Like escape".into()), &got_t, &want_t);
                    }
                    out.s("escape").s(&tokens_escape.to_string());
                }
                return (ExprType(vec![(Binding::empty(), Type {
                    type_: SimpleType {
                        type_: SimpleSimpleType::Bool,
                        custom: None,
                    },
                    opt: false,
                    arr: false,
                })]), out);
            },
            Expr::Between { e, negated, low, high } => {
                let mut out = Tokens::new();
                let (t, e_tokens) = e.build(ctx, &path.push_back("Between expr".into()), scope);
                let (t_low, low_tokens) = low.build(ctx, &path.push_back("Between low".into()), scope);
                let (t_high, high_tokens) = high.build(ctx, &path.push_back("Between high".into()), scope);
                check_general_same(ctx, path, &t, &t_low);
                check_general_same(ctx, path, &t, &t_high);
                out.s(&e_tokens.to_string());
                if *negated {
                    out.s("not");
                }
                out.s("between").s(&low_tokens.to_string()).s("and").s(&high_tokens.to_string());
                return (ExprType(vec![(Binding::empty(), Type {
                    type_: SimpleType {
                        type_: SimpleSimpleType::Bool,
                        custom: None,
                    },
                    opt: false,
                    arr: false,
                })]), out);
            },
            Expr::Case { operand, conditions, else_ } => {
                let mut out = Tokens::new();
                out.s("case");
                let op_type = if let Some(operand) = operand {
                    let (t, op_tokens) = operand.build(ctx, &path.push_back("Case operand".into()), scope);
                    out.s(&op_tokens.to_string());
                    Some(t)
                } else {
                    None
                };
                let mut res_type = None;
                for (i, (cond, res)) in conditions.iter().enumerate() {
                    out.s("when");
                    let (cond_t, cond_tokens) = cond.build(ctx, &path.push_back(format!("Case condition {}", i)), scope);
                    if let Some(op_t) = &op_type {
                        check_general_same(ctx, &path.push_back(format!("Case condition {}", i)), op_t, &cond_t);
                    } else {
                        check_bool(ctx, &path.push_back(format!("Case condition {}", i)), &cond_t);
                    }
                    out.s(&cond_tokens.to_string()).s("then");
                    let (r_t, res_tokens) = res.build(ctx, &path.push_back(format!("Case result {}", i)), scope);
                    if let Some(res_t) = &res_type {
                        check_general_same(ctx, &path.push_back(format!("Case result {}", i)), res_t, &r_t);
                    } else {
                        res_type = Some(r_t);
                    }
                    out.s(&res_tokens.to_string());
                }
                if let Some(else_) = else_ {
                    out.s("else");
                    let (else_t, else_tokens) = else_.build(ctx, &path.push_back("Case else".into()), scope);
                    if let Some(res_t) = &res_type {
                        check_general_same(ctx, &path.push_back("Case else".into()), res_t, &else_t);
                    } else {
                        res_type = Some(else_t);
                    }
                    out.s(&else_tokens.to_string());
                }
                out.s("end");
                (res_type.expect("Case must have at least one branch"), out)
            },
        }
    }
}


pub fn check_bool(ctx: &mut SqliteQueryCtx, path: &rpds::Vector<String>, t: &ExprType) {
    check_general_same(ctx, path, t, &ExprType(vec![(Binding::empty(), Type {
        type_: SimpleType {
            type_: SimpleSimpleType::Bool,
            custom: None,
        },
        opt: false,
        arr: false,
    })]));
}

pub fn check_assignable(errs: &mut Errs, path: &rpds::Vector<String>, left: &Type, right: &ExprType) {
    let Some(right) = right.assert_scalar(errs, path) else {
        return;
    };
    check_general_same_type_assignable(errs, path, left, &right);
}

pub fn check_general_same(ctx: &mut SqliteQueryCtx, path: &rpds::Vector<String>, left: &ExprType, right: &ExprType) {
    if left.0.len() != right.0.len() {
        ctx
            .errs
            .err(
                path,
                format!(
                    "Operator arms record type lengths don't match: left has {} fields and right has {}",
                    left.0.len(),
                    right.0.len()
                ),
            );
    } else if left.0.len() == 1 && right.0.len() == 1 {
        check_general_same_type(ctx, path, &left.0[0].1, &right.0[0].1);
    } else {
        for (i, (left, right)) in left.0.iter().zip(right.0.iter()).enumerate() {
            check_general_same_type(ctx, &path.push_back(format!("Record pair {}", i)), &left.1, &right.1);
        }
    }
}

pub fn check_same(errs: &mut Errs, path: &rpds::Vector<String>, left: &ExprType, right: &ExprType) -> Option<Type> {
    let left = left.assert_scalar(errs, &path.push_back("Left".into()))?;
    let right = right.assert_scalar(errs, &path.push_back("Right".into()))?;
    check_general_same_type(&mut SqliteQueryCtx::new(errs.clone(), HashMap::new()), path, &left, &right);
    return Some(left);
}

pub fn check_general_same_type(ctx: &mut SqliteQueryCtx, path: &rpds::Vector<String>, left: &Type, right: &Type) {
    if left.type_.type_ != right.type_.type_ {
        ctx
            .errs
            .err(
                path,
                format!(
                    "Operator arms types don't match: left has type {:?} and right has {:?}",
                    left.type_.type_,
                    right.type_.type_
                ),
            );
    }
}

pub fn check_general_same_type_assignable(errs: &mut Errs, path: &rpds::Vector<String>, left: &Type, right: &Type) {
    if left.type_.type_ != right.type_.type_ {
        errs.err(
            path,
            format!("Expression has type {:?} which is not assignable to {:?}", right.type_.type_, left.type_.type_),
        );
    }
    if !left.opt && right.opt {
        errs.err(path, format!("Expression is optional but destination is not"));
    }
}

fn do_bin_op(
    ctx: &mut SqliteQueryCtx,
    path: &rpds::Vector<String>,
    scope: &HashMap<Binding, Type>,
    op: &BinOp,
    exprs: &Vec<Expr>,
) -> (ExprType, Tokens) {
    let mut out = Tokens::new();
    let mut out_t = None;
    let token = match op {
        BinOp::Plus => "+",
        BinOp::Minus => "-",
        BinOp::Multiply => "*",
        BinOp::Divide => "/",
        BinOp::And => "and",
        BinOp::Or => "or",
        BinOp::Equals => "=",
        BinOp::NotEquals => "<>",
        BinOp::Is => "is",
        BinOp::IsNot => "is not",
        BinOp::LessThan => "<",
        BinOp::LessThanEqualTo => "<=",
        BinOp::GreaterThan => ">",
        BinOp::GreaterThanEqualTo => ">=",
        BinOp::Like => "like",
        BinOp::In => "in",
        BinOp::NotIn => "not in",
        BinOp::StringConcat => "||",
        BinOp::Mod => "%",
        BinOp::BitwiseAnd => "&",
        BinOp::BitwiseOr => "|",
        BinOp::BitwiseXor => "~",
        BinOp::BitwiseShiftLeft => "<<",
        BinOp::BitwiseShiftRight => ">>",
        BinOp::IsDistinctFrom => "is distinct from",
        BinOp::IsNotDistinctFrom => "is not distinct from",
        BinOp::Glob => "glob",
        BinOp::Regexp => "regexp",
        BinOp::Match => "match",
    };
    for (i, res) in exprs.iter().enumerate() {
        if i > 0 {
            out.s(token);
        }
        let (t, tokens): (ExprType, Tokens) = res.build(ctx, &path.push_back(format!("Operand {}", i)), scope);
        let got_t = match t.assert_scalar(&mut ctx.errs, &path.push_back(format!("Operand {}", i))) {
            Some(t) => t,
            None => {
                continue;
            },
        };
        match op {
            BinOp::Plus | BinOp::Minus | BinOp::Multiply | BinOp::Divide | BinOp::Mod | BinOp::BitwiseAnd | BinOp::BitwiseOr | BinOp::BitwiseXor | BinOp::BitwiseShiftLeft | BinOp::BitwiseShiftRight => {
                if !matches!(
                    got_t.type_.type_,
                    SimpleSimpleType::I32 | SimpleSimpleType::I64 | SimpleSimpleType::Auto
                ) && !matches!(op, BinOp::Plus | BinOp::Minus | BinOp::Multiply | BinOp::Divide) {
                     // arithmetic allows floats, bitwise only ints
                } else if !matches!(
                    got_t.type_.type_,
                    SimpleSimpleType::I32 | SimpleSimpleType::I64 | SimpleSimpleType::F32 | SimpleSimpleType::F64 |
                        SimpleSimpleType::Auto
                ) {
                    ctx
                        .errs
                        .err(
                            &path.push_back(format!("Operand {}", i)),
                            format!("Arithmetic/Bitwise operator {:?} not supported for type {:?}", op, got_t.type_.type_),
                        );
                }
            },
            BinOp::And | BinOp::Or => {
                if !matches!(got_t.type_.type_, SimpleSimpleType::Bool) {
                    ctx
                        .errs
                        .err(
                            &path.push_back(format!("Operand {}", i)),
                            format!("Logical operator {:?} not supported for type {:?}", op, got_t.type_.type_),
                        );
                }
            },
            BinOp::Equals |
            BinOp::NotEquals |
            BinOp::Is |
            BinOp::IsNot |
            BinOp::LessThan |
            BinOp::LessThanEqualTo |
            BinOp::GreaterThan |
            BinOp::GreaterThanEqualTo |
            BinOp::Like |
            BinOp::In |
            BinOp::NotIn |
            BinOp::StringConcat |
            BinOp::IsDistinctFrom |
            BinOp::IsNotDistinctFrom |
            BinOp::Glob |
            BinOp::Regexp |
            BinOp::Match => {

            },
        }
        if let Some(out_t) = &mut out_t {
            check_general_same_type(ctx, path, out_t, &got_t);
        } else {
            out_t = Some(got_t);
        }
        out.s(&tokens.to_string());
    }
    let res_t = match op {
        BinOp::Equals |
        BinOp::NotEquals |
        BinOp::Is |
        BinOp::IsNot |
        BinOp::LessThan |
        BinOp::LessThanEqualTo |
        BinOp::GreaterThan |
        BinOp::GreaterThanEqualTo |
        BinOp::Like |
        BinOp::In |
        BinOp::NotIn |
        BinOp::IsDistinctFrom |
        BinOp::IsNotDistinctFrom |
        BinOp::Glob |
        BinOp::Regexp |
        BinOp::Match => Type {
            type_: SimpleType {
                type_: SimpleSimpleType::Bool,
                custom: None,
            },
            opt: false,
            arr: false,
        },
        BinOp::StringConcat => Type {
            type_: SimpleType {
                type_: SimpleSimpleType::String,
                custom: None,
            },
            opt: false,
            arr: false,
        },
        _ => out_t.unwrap_or(Type {
            type_: SimpleType {
                type_: SimpleSimpleType::I32,
                custom: None,
            },
            opt: false,
            arr: false,
        }),
    };
    return (ExprType(vec![(Binding::empty(), res_t)]), out);
}

trait FlatWithIndex<T>: Iterator<Item = T> {
    fn flat_with_index<
        R: Iterator,
        F: FnMut(usize, T) -> R,
    >(self, f: F) -> std::iter::Flatten<WithIndex<Self, F, T, R>>
    where
        Self: Sized;
}

impl<I: Iterator<Item = T>, T> FlatWithIndex<T> for I {
    fn flat_with_index<
        R: Iterator,
        F: FnMut(usize, T) -> R,
    >(self, f: F) -> std::iter::Flatten<WithIndex<Self, F, T, R>>
    where
        Self: Sized {
        WithIndex {
            iter: self,
            f: f,
            i: 0,
            _phantom: std::marker::PhantomData,
        }.flatten()
    }
}

struct WithIndex<I, F, T, R> {
    iter: I,
    f: F,
    i: usize,
    _phantom: std::marker::PhantomData<(T, R)>,
}

impl<I: Iterator<Item = T>, F: FnMut(usize, T) -> R, T, R: Iterator> Iterator for WithIndex<I, F, T, R> {
    type Item = R;

    fn next(&mut self) -> Option<Self::Item> {
        let val = self.iter.next()?;
        let res = (self.f)(self.i, val);
        self.i += 1;
        return Some(res);
    }
}
