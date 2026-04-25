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
    proc_macro2::TokenStream,
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
        pg::{
            types::{
                Type,
                to_rust_types,
                SimpleSimpleType,
                SimpleType,
            },
            query::utils::{
                PgQueryCtx,
                PgTableInfo,
                PgFieldInfo,
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
pub struct ExprType(pub Vec<(ExprValName, Type)>);

impl ExprType {
    pub fn assert_scalar(&self, errs: &mut Errs, path: &rpds::Vector<String>) -> Option<Type> {
        if self.0.len() != 1 {
            errs.err(
                path,
                format!("Select outputs must be scalars, but got result with more than one field: {}", self.0.len()),
            );
            return None;
        }
        Some(self.0[0].1.clone())
    }
}

pub struct ComputeType(pub Rc<dyn Fn(&mut PgQueryCtx, &rpds::Vector<String>, &[ExprType]) -> ExprType>);

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
    LitUtcTimeChrono(DateTime<Utc>),
    #[cfg(feature = "chrono")]
    LitFixedOffsetTimeChrono(DateTime<FixedOffset>),
    #[cfg(feature = "jiff")]
    LitUtcTimeJiff(Timestamp),
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
            SerialExpr::LitUtcTimeChrono(v) => Expr::LitUtcTimeChrono(v),
            #[cfg(feature = "chrono")]
            SerialExpr::LitFixedOffsetTimeChrono(v) => Expr::LitFixedOffsetTimeChrono(v),
            #[cfg(feature = "jiff")]
            SerialExpr::LitUtcTimeJiff(v) => Expr::LitUtcTimeJiff(v),
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
    LitUtcTimeChrono(DateTime<Utc>),
    #[cfg(feature = "chrono")]
    LitFixedOffsetTimeChrono(DateTime<FixedOffset>),
    #[cfg(feature = "jiff")]
    LitUtcTimeJiff(Timestamp),
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
}

#[derive(Clone, Hash, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct ExprValName {
    pub table_id: String,
    pub id: String,
}

impl ExprValName {
    pub(crate) fn local(name: String) -> Self {
        ExprValName {
            table_id: "".into(),
            id: name,
        }
    }

    pub fn empty() -> Self {
        ExprValName {
            table_id: "".into(),
            id: "".into(),
        }
    }

    pub fn field(f: &FieldRef) -> Self {
        ExprValName {
            table_id: f.table_id.0.clone(),
            id: f.field_id.0.clone(),
        }
    }

    pub(crate) fn with_alias(&self, s: &str) -> ExprValName {
        ExprValName {
            table_id: s.into(),
            id: self.id.clone(),
        }
    }
}

impl Display for ExprValName {
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
    In,
    NotIn,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PrefixOp {
    Not,
}

pub(crate) fn check_same(
    errs: &mut Errs,
    path: &rpds::Vector<String>,
    left: &ExprType,
    right: &ExprType,
) -> Option<Type> {
    let left = left.assert_scalar(errs, &path.push_back("Left".into())) ?;
    let right = right.assert_scalar(errs, &path.push_back("Right".into())) ?;
    if left.opt != right.opt {
        errs.err(
            path,
            format!("Operator arms optionality don't match: left has {} and right has {}", left.opt, right.opt),
        );
    }
    if left.type_.custom != right.type_.custom {
        errs.err(
            path,
            format!(
                "Operator arms custom types don't match: left has type {:?} and right has {:?}",
                left.type_.custom,
                right.type_.custom
            ),
        );
    }
    if left.type_.type_ != right.type_.type_ {
        errs.err(
            path,
            format!(
                "Operator arms types don't match: left has type {:?} and right has {:?}",
                left.type_.type_,
                right.type_.type_
            ),
        );
    }
    Some(left.clone())
}

pub(crate) fn check_bool(
    ctx: &mut PgQueryCtx,
    path: &rpds::Vector<String>,
    t: &ExprType,
) {
    let Some(t) = t.assert_scalar(&mut ctx.errs, path) else {
        return;
    };
    if t.opt {
        ctx.errs.err(path, format!("Expected non-optional bool but got optional bool"));
    }
    if !matches!(t.type_.type_, SimpleSimpleType::Bool) {
        ctx.errs.err(path, format!("Expected bool but type is non-bool: got {:?}", t.type_.type_));
    }
}

pub(crate) fn check_assignable(
    errs: &mut Errs,
    path: &rpds::Vector<String>,
    left: &Type,
    right: &ExprType,
) {
    let Some(right) = right.assert_scalar(errs, path) else {
        return;
    };
    if left.type_.type_ != right.type_.type_ {
        errs.err(
            path,
            format!(
                "Expression has type {:?} which is not assignable to {:?}",
                right.type_.type_,
                left.type_.type_
            ),
        );
    }
    if !left.opt && right.opt {
        errs.err(path, format!("Expression is optional but destination is not"));
    }
}

pub(crate) fn check_general_same(
    ctx: &mut PgQueryCtx,
    path: &rpds::Vector<String>,
    left: &ExprType,
    right: &ExprType,
) {
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

pub(crate) fn check_general_same_type(
    ctx: &mut PgQueryCtx,
    path: &rpds::Vector<String>,
    left: &Type,
    right: &Type,
) {
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

impl Expr {
    pub fn build(
        &self,
        ctx: &mut PgQueryCtx,
        path: &rpds::Vector<String>,
        scope: &HashMap<ExprValName, Type>,
    ) -> (ExprType, Tokens) {
        match self {
            Expr::LitArray(v) => {
                let mut out = Tokens::new();
                out.s("array [");
                let mut res_types = vec![];
                for (i, e) in v.iter().enumerate() {
                    if i > 0 {
                        out.s(",");
                    }
                    let res = e.build(ctx, path, scope);
                    out.s(&res.1.to_string());
                    res_types.push(res.0);
                }
                out.s("]");
                let mut out_type = None;
                for (i, t) in res_types.iter().enumerate() {
                    if let Some(prev) = &out_type {
                        check_general_same(ctx, &path.push_back(format!("Array element {}", i)), prev, t);
                    } else {
                        out_type = Some(t.clone());
                    }
                }
                return (out_type.unwrap_or(ExprType(vec![])), out);
            },
            Expr::LitNull(t) => {
                let mut out = Tokens::new();
                out.s("null");
                return (
                    ExprType(vec![(ExprValName::empty(), Type {
                        type_: t.clone(),
                        opt: true,
                        arr: false,
                    })]),
                    out
                );
            },
            Expr::LitBool(b) => {
                let mut out = Tokens::new();
                out.s(if *b { "true" } else { "false" });
                return (
                    ExprType(vec![(ExprValName::empty(), Type {
                        type_: SimpleType {
                            type_: SimpleSimpleType::Bool,
                            custom: None,
                        },
                        opt: false,
                        arr: false,
                    })]),
                    out
                );
            },
            Expr::LitAuto(v) => {
                let mut out = Tokens::new();
                out.s(&v.to_string());
                return (
                    ExprType(vec![(ExprValName::empty(), Type {
                        type_: SimpleType {
                            type_: SimpleSimpleType::Auto,
                            custom: None,
                        },
                        opt: false,
                        arr: false,
                    })]),
                    out
                );
            },
            Expr::LitI16(v) => {
                let mut out = Tokens::new();
                out.s(&v.to_string());
                return (
                    ExprType(vec![(ExprValName::empty(), Type {
                        type_: SimpleType {
                            type_: SimpleSimpleType::I16,
                            custom: None,
                        },
                        opt: false,
                        arr: false,
                    })]),
                    out
                );
            },
            Expr::LitI32(v) => {
                let mut out = Tokens::new();
                out.s(&v.to_string());
                return (
                    ExprType(vec![(ExprValName::empty(), Type {
                        type_: SimpleType {
                            type_: SimpleSimpleType::I32,
                            custom: None,
                        },
                        opt: false,
                        arr: false,
                    })]),
                    out
                );
            },
            Expr::LitI64(v) => {
                let mut out = Tokens::new();
                out.s(&v.to_string());
                return (
                    ExprType(vec![(ExprValName::empty(), Type {
                        type_: SimpleType {
                            type_: SimpleSimpleType::I64,
                            custom: None,
                        },
                        opt: false,
                        arr: false,
                    })]),
                    out
                );
            },
            Expr::LitU32(v) => {
                let mut out = Tokens::new();
                out.s(&v.to_string());
                return (
                    ExprType(vec![(ExprValName::empty(), Type {
                        type_: SimpleType {
                            type_: SimpleSimpleType::U32,
                            custom: None,
                        },
                        opt: false,
                        arr: false,
                    })]),
                    out
                );
            },
            Expr::LitF32(v) => {
                let mut out = Tokens::new();
                out.s(&v.to_string());
                return (
                    ExprType(vec![(ExprValName::empty(), Type {
                        type_: SimpleType {
                            type_: SimpleSimpleType::F32,
                            custom: None,
                        },
                        opt: false,
                        arr: false,
                    })]),
                    out
                );
            },
            Expr::LitF64(v) => {
                let mut out = Tokens::new();
                out.s(&v.to_string());
                return (
                    ExprType(vec![(ExprValName::empty(), Type {
                        type_: SimpleType {
                            type_: SimpleSimpleType::F64,
                            custom: None,
                        },
                        opt: false,
                        arr: false,
                    })]),
                    out
                );
            },
            Expr::LitString(v) => {
                let mut out = Tokens::new();
                out.s(&format!("'{}'", v.replace("'", "''")));
                return (
                    ExprType(vec![(ExprValName::empty(), Type {
                        type_: SimpleType {
                            type_: SimpleSimpleType::String,
                            custom: None,
                        },
                        opt: false,
                        arr: false,
                    })]),
                    out
                );
            },
            Expr::LitBytes(v) => {
                let mut out = Tokens::new();
                out.s(&format!("'\\x{}'", hex::encode(v)));
                return (
                    ExprType(vec![(ExprValName::empty(), Type {
                        type_: SimpleType {
                            type_: SimpleSimpleType::Bytes,
                            custom: None,
                        },
                        opt: false,
                        arr: false,
                    })]),
                    out
                );
            },
            #[cfg(feature = "chrono")]
            Expr::LitUtcTimeChrono(v) => {
                let mut out = Tokens::new();
                out.s(&format!("'{}'", v.to_rfc3339()));
                return (
                    ExprType(vec![(ExprValName::empty(), Type {
                        type_: SimpleType {
                            type_: SimpleSimpleType::UtcTimeSChrono,
                            custom: None,
                        },
                        opt: false,
                        arr: false,
                    })]),
                    out
                );
            },
            #[cfg(feature = "chrono")]
            Expr::LitFixedOffsetTimeChrono(v) => {
                let mut out = Tokens::new();
                out.s(&format!("'{}'", v.to_rfc3339()));
                return (
                    ExprType(vec![(ExprValName::empty(), Type {
                        type_: SimpleType {
                            type_: SimpleSimpleType::FixedOffsetTimeChrono,
                            custom: None,
                        },
                        opt: false,
                        arr: false,
                    })]),
                    out
                );
            },
            #[cfg(feature = "jiff")]
            Expr::LitUtcTimeJiff(v) => {
                let mut out = Tokens::new();
                out.s(&format!("'{}'", v.to_string()));
                return (
                    ExprType(vec![(ExprValName::empty(), Type {
                        type_: SimpleType {
                            type_: SimpleSimpleType::UtcTimeSJiff,
                            custom: None,
                        },
                        opt: false,
                        arr: false,
                    })]),
                    out
                );
            },
            Expr::Param { name, type_ } => {
                let mut out = Tokens::new();
                let path = path.push_back(format!("Param ({})", name));
                let i = match ctx.rust_arg_lookup.entry(name.clone()) {
                    std::collections::hash_map::Entry::Occupied(e) => {
                        let (i, prev_t) = e.get();
                        if type_ != prev_t {
                            ctx
                                .errs
                                .err(
                                    &path,
                                    format!("Parameter {} specified with multiple types: {:?}, {:?}", name, type_, prev_t),
                                );
                        }
                        *i
                    },
                    std::collections::hash_map::Entry::Vacant(e) => {
                        let i = ctx.rust_args.len() + 1;
                        e.insert((i, type_.clone()));
                        let rust_types = to_rust_types(&type_.type_.type_);
                        let custom_trait_ident = rust_types.custom_trait;
                        let rust_type = rust_types.arg_type;
                        let ident = format_ident!("{}", sanitize_ident(name).1);
                        let (mut rust_type, mut rust_forward) = if let Some(custom) = &type_.type_.custom {
                            let custom_ident = match syn::parse_str::<syn::Path>(custom.as_str()) {
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
                        if type_.opt {
                            rust_type = quote!(Option < #rust_type >);
                            rust_forward = quote!(#ident.map(| #ident | #rust_forward));
                        }
                        if type_.arr {
                            rust_type = quote!(Vec < #rust_type >);
                        }
                        ctx.rust_args.push(quote!(#ident: #rust_type));
                        ctx.query_args.push(quote!(#rust_forward));
                        i
                    },
                };
                out.s(&format!("${}", i));
                return (ExprType(vec![(ExprValName::local(name.clone()), type_.clone())]), out);
            },
            Expr::Field(x) => {
                let name = ExprValName::field(x);
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
                let table_info = ctx.tables.get(&TableRef(x.table_id.clone())).unwrap();
                let field_info = table_info.fields.get(x).unwrap();
                out.id(&table_info.sql_name).s(".").id(&field_info.sql_name);
                return (ExprType(vec![(name, t.clone())]), out);
            },
            Expr::BinOp { left, op, right } => {
                let mut out = Tokens::new();
                let l_res = left.build(ctx, &path.push_back("Bin op left".into()), scope);
                let r_res = right.build(ctx, &path.push_back("Bin op right".into()), scope);
                let t = check_same(&mut ctx.errs, path, &l_res.0, &r_res.0);
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
                    BinOp::In => "in",
                    BinOp::NotIn => "not in",
                };
                out.s(&l_res.1.to_string()).s(token).s(&r_res.1.to_string());
                let mut res_t = t.unwrap_or(Type {
                    type_: SimpleType {
                        type_: SimpleSimpleType::I32,
                        custom: None,
                    },
                    opt: false,
                    arr: false,
                });
                match op {
                    BinOp::Equals |
                    BinOp::NotEquals |
                    BinOp::Is |
                    BinOp::IsNot |
                    BinOp::LessThan |
                    BinOp::LessThanEqualTo |
                    BinOp::GreaterThan |
                    BinOp::GreaterThanEqualTo |
                    BinOp::In |
                    BinOp::NotIn => {
                        res_t = Type {
                            type_: SimpleType {
                                type_: SimpleSimpleType::Bool,
                                custom: None,
                            },
                            opt: false,
                            arr: false,
                        };
                    },
                    _ => { },
                }
                return (ExprType(vec![(ExprValName::empty(), res_t)]), out);
            },
            Expr::BinOpChain { op, exprs } => {
                let mut out = Tokens::new();
                let token = match op {
                    BinOp::And => "and",
                    BinOp::Or => "or",
                    _ => panic!("Chain only supported for and/or"),
                };
                let mut out_t = None;
                for (i, e) in exprs.iter().enumerate() {
                    if i > 0 {
                        out.s(token);
                    }
                    let res = e.build(ctx, &path.push_back(format!("Chain element {}", i)), scope);
                    check_bool(ctx, &path.push_back(format!("Chain element {}", i)), &res.0);
                    out.s(&res.1.to_string());
                    out_t = Some(res.0);
                }
                return (out_t.unwrap_or(ExprType(vec![])), out);
            },
            Expr::PrefixOp { op, right } => {
                let mut out = Tokens::new();
                let token = match op {
                    PrefixOp::Not => "not",
                };
                let res = right.build(ctx, &path.push_back("Prefix op".into()), scope);
                check_bool(ctx, path, &res.0);
                out.s(token).s(&res.1.to_string());
                return (res.0, out);
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
                        let (_, tokens): (ExprType, Tokens) = e.build(ctx, &path.push_back(format!("Order by {}", i)), scope);
                        out.s(&tokens.to_string());
                        match o {
                            Order::Asc => { out.s("asc"); },
                            Order::Desc => { out.s("desc"); },
                        }
                    }
                }
                out.s(")");
                return (t, out);
            },
            Expr::Select(s) => {
                let mut out = Tokens::new();
                out.s("(");
                let (t, tokens) = s.build(ctx, &path.push_back("Subselect".into()), crate::pg::QueryResCount::Many);
                out.s(&tokens.to_string()).s(")");
                return (t, out);
            },
            Expr::Cast(e, t) => {
                let mut out = Tokens::new();
                let (got_t, tokens) = e.build(ctx, &path.push_back("Cast".into()), scope);
                check_general_same(ctx, path, &got_t, &ExprType(vec![(ExprValName::empty(), t.clone())]));
                out.s("(").s(&tokens.to_string()).s("::").s(crate::pg::types::to_sql_type(&t.type_.type_)).s(")");
                return (ExprType(vec![(ExprValName::empty(), t.clone())]), out);
            },
        }
    }
}
