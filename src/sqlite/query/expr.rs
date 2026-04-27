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
        }
    }
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
    },
    Window {
        expr: Box<Expr>,
        partition_by: Vec<Expr>,
        order_by: Vec<(Expr, Order)>,
    },
    /// A sub SELECT query.
    Select(Box<Select>),
    /// This is a synthetic expression, saying to treat the result of the expression as
    /// having the specified type. Use this for casting between primitive types and
    /// Rust new-types for instance.
    Cast(Box<Expr>, Type),
    Exists(Box<super::select::Select>),
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
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PrefixOp {
    Not,
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
                }
                let (t, tokens) = right.build(ctx, &path.push_back(format!("Prefix op {:?}", op)), scope);
                check_bool(ctx, path, &t);
                out.s(&tokens.to_string());
                return (t, out);
            },
            Expr::Call { func, args, compute_type } => {
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
                return (compute_type.0(ctx, path, &arg_types), out);
            },
            Expr::Window { expr, partition_by, order_by } => {
                let mut out = Tokens::new();
                let (t, tokens) = expr.build(ctx, &path.push_back("Window expr".into()), scope);
                out.s(&tokens.to_string()).s("over (");
                if !partition_by.is_empty() {
                    out.s("partition by");
                    for (i, p) in partition_by.iter().enumerate() {
                        if i > 0 {
                            out.s(",");
                        }
                        let (_, tokens) = p.build(ctx, &path.push_back(format!("Partition by {}", i)), scope);
                        out.s(&tokens.to_string());
                    }
                }
                if !order_by.is_empty() {
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
                }
                out.s(")");
                return (t, out);
            },
            Expr::Select(s) => {
                let mut out = Tokens::new();
                out.s("(");
                let (t, tokens) =
                    s.build(ctx, &path.push_back("Subselect".into()), crate::sqlite::QueryResCount::Many);
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
                    s.build(ctx, &path.push_back("Exists".into()), crate::sqlite::QueryResCount::Many);
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
            BinOp::Plus | BinOp::Minus | BinOp::Multiply | BinOp::Divide => {
                if !matches!(
                    got_t.type_.type_,
                    SimpleSimpleType::I32 | SimpleSimpleType::I64 | SimpleSimpleType::F32 | SimpleSimpleType::F64 |
                        SimpleSimpleType::Auto
                ) {
                    ctx
                        .errs
                        .err(
                            &path.push_back(format!("Operand {}", i)),
                            format!("Arithmetic operator {:?} not supported for type {:?}", op, got_t.type_.type_),
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
            BinOp::NotIn => {

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
        BinOp::NotIn => Type {
            type_: SimpleType {
                type_: SimpleSimpleType::Bool,
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
