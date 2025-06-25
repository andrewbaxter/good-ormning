use {
    super::{
        select_body::{
            Order,
            SelectBody,
            SelectJunction,
        },
        utils::SqliteQueryCtx,
    },
    crate::{
        sqlite::{
            query::select_body::build_select_junction,
            schema::field::Field,
            types::{
                to_rust_types,
                SimpleSimpleType,
                SimpleType,
                Type,
            },
            QueryResCount,
        },
        utils::{
            sanitize_ident,
            Errs,
            Tokens,
        },
    },
    quote::{
        format_ident,
        quote,
        ToTokens,
    },
    samevariant::samevariant,
    std::{
        collections::HashMap,
        fmt::Display,
        rc::Rc,
    },
    syn::Path,
};
#[cfg(feature = "chrono")]
use {
    chrono::{
        DateTime,
        Utc,
    },
};

/// This is used for function expressions, to check the argument types and compute
/// a result type from them.  See readme for details.
#[derive(Clone)]
pub struct ComputeType(Rc<dyn Fn(&mut SqliteQueryCtx, &rpds::Vector<String>, Vec<ExprType>) -> Option<Type>>);

impl ComputeType {
    pub fn new(
        f: impl Fn(&mut SqliteQueryCtx, &rpds::Vector<String>, Vec<ExprType>) -> Option<Type> + 'static,
    ) -> ComputeType {
        return ComputeType(Rc::new(f));
    }
}

impl std::fmt::Debug for ComputeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return f.write_str("ComputeType");
    }
}

#[derive(Clone, Debug)]
pub enum Expr {
    LitArray(Vec<Expr>),
    // A null value needs a type for type checking purposes. It will always be trated
    // as an optional value.
    LitNull(SimpleType),
    LitBool(bool),
    LitI32(i32),
    LitI64(i64),
    LitU32(u32),
    LitF32(f32),
    LitF64(f64),
    LitString(String),
    LitBytes(Vec<u8>),
    #[cfg(feature = "chrono")]
    LitUtcTimeS(DateTime<Utc>),
    #[cfg(feature = "chrono")]
    LitUtcTimeMs(DateTime<Utc>),
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
    Binding(Binding),
    BinOp {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },
    /// This is the same as `BinOp` but allows chaining multiple expressions with the
    /// same operator. This can be useful if you have many successive `AND`s or similar.
    BinOpChain {
        op: BinOp,
        exprs: Vec<Expr>,
    },
    PrefixOp {
        op: PrefixOp,
        right: Box<Expr>,
    },
    /// Represents a call to an SQL function, like `collate()`. You must provide the
    /// type of the result since we don't have a table of functions and their return
    /// types at present.
    Call {
        func: String,
        args: Vec<Expr>,
        /// Checks the input types and computes the resulting type
        compute_type: ComputeType,
    },
    // This is an `OVER` windowing function. If neither `partition_by` nor `order_by`
    // have elements it'll be rendered as `OVER()` (all rows).
    Window {
        expr: Box<Expr>,
        partition_by: Vec<Expr>,
        order_by: Vec<(Expr, Order)>,
    },
    /// A sub SELECT query.
    Select {
        body: Box<SelectBody>,
        body_junctions: Vec<SelectJunction>,
    },
    Exists {
        not: bool,
        body: Box<SelectBody>,
        body_junctions: Vec<SelectJunction>,
    },
    /// This is a synthetic expression, saying to treat the result of the expression as
    /// having the specified type. Use this for casting between primitive types and
    /// Rust new-types for instance.
    Cast(Box<Expr>, Type),
}

impl Expr {
    pub fn field(f: &Field) -> Expr {
        return Expr::Binding(Binding::field(f));
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct Binding {
    pub table_id: String,
    pub id: String,
}

impl Binding {
    /// Create an expression field/value name for a select-local (tableless) field, for
    /// instance `WINDOW` fields.
    pub fn local(name: impl AsRef<str>) -> Self {
        Binding {
            table_id: "".into(),
            id: name.as_ref().to_string(),
        }
    }

    pub(crate) fn empty() -> Self {
        Binding {
            table_id: "".into(),
            id: "".into(),
        }
    }

    /// Create an expression field/value name from a table field.
    pub fn field(f: &Field) -> Self {
        Binding {
            table_id: f.table.id.clone(),
            id: f.id.clone(),
        }
    }

    /// Derive an expression field/value name from a different name, with a new alias.
    pub fn with_alias(&self, s: &str) -> Binding {
        Binding {
            table_id: s.into(),
            id: self.id.clone(),
        }
    }
}

impl Display for Binding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&format!("{}.{}", self.table_id, self.id), f)
    }
}

pub struct ExprType(pub Vec<(Binding, Type)>);

impl ExprType {
    pub fn assert_scalar(&self, errs: &mut Errs, path: &rpds::Vector<String>) -> Option<(Binding, Type)> {
        if self.0.len() != 1 {
            errs.err(path, format!("Select outputs must be scalars, but got result with {} fields", self.0.len()));
            return None;
        }
        Some(self.0[0].clone())
    }
}

pub fn check_general_same_type(ctx: &mut SqliteQueryCtx, path: &rpds::Vector<String>, left: &Type, right: &Type) {
    if left.opt != right.opt {
        ctx.errs.err(path, format!("Operator arms have differing optionality"));
    }
    if left.array != right.array {
        ctx.errs.err(path, format!("Operator arms are either not both arrays or not both scalars"));
    }

    #[derive(Debug)]
    #[samevariant(GeneralTypePairs)]
    enum GeneralType {
        Bool,
        Numeric,
        Blob,
    }

    fn general_type(t: &Type) -> GeneralType {
        match t.type_.type_ {
            SimpleSimpleType::U32 => GeneralType::Numeric,
            SimpleSimpleType::I32 => GeneralType::Numeric,
            SimpleSimpleType::I64 => GeneralType::Numeric,
            SimpleSimpleType::F32 => GeneralType::Numeric,
            SimpleSimpleType::F64 => GeneralType::Numeric,
            SimpleSimpleType::Bool => GeneralType::Bool,
            SimpleSimpleType::String => GeneralType::Blob,
            SimpleSimpleType::Bytes => GeneralType::Blob,
            #[cfg(feature = "chrono")]
            SimpleSimpleType::UtcTimeS => GeneralType::Numeric,
            #[cfg(feature = "chrono")]
            SimpleSimpleType::UtcTimeMs => GeneralType::Blob,
            #[cfg(feature = "chrono")]
            SimpleSimpleType::FixedOffsetTimeMs => GeneralType::Blob,
        }
    }

    match GeneralTypePairs::pairs(&general_type(left), &general_type(right)) {
        GeneralTypePairs::Nonmatching(left, right) => {
            ctx.errs.err(path, format!("Operator arms have incompatible types: {:?} and {:?}", left, right));
        },
        _ => { },
    }
}

pub(crate) fn check_general_same(
    ctx: &mut SqliteQueryCtx,
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
        check_general_same_type(ctx, path, &left.0[0].1, &left.0[0].1);
    } else {
        for (i, (left, right)) in left.0.iter().zip(right.0.iter()).enumerate() {
            check_general_same_type(ctx, &path.push_back(format!("Record pair {}", i)), &left.1, &right.1);
        }
    }
}

pub(crate) fn check_same(
    errs: &mut Errs,
    path: &rpds::Vector<String>,
    left: &ExprType,
    right: &ExprType,
) -> Option<Type> {
    let left = match left.assert_scalar(errs, &path.push_back("Left".into())) {
        Some(t) => t,
        None => {
            return None;
        },
    };
    let right = match right.assert_scalar(errs, &path.push_back("Right".into())) {
        Some(t) => t,
        None => {
            return None;
        },
    };
    if left.1.opt != right.1.opt {
        errs.err(
            path,
            format!(
                "Expected same types, but left nullability is {} but right nullability is {}",
                left.1.opt,
                right.1.opt
            ),
        );
    }
    if left.1.type_.custom != right.1.type_.custom {
        errs.err(
            path,
            format!(
                "Expected same types, but left rust type is {:?} while right rust type is {:?}",
                left.1.type_.custom,
                right.1.type_.custom
            ),
        );
    }
    if left.1.type_.type_ != right.1.type_.type_ {
        errs.err(
            path,
            format!(
                "Expected same types, but left base type is {:?} while right base type is {:?}",
                left.1.type_.type_,
                right.1.type_.type_
            ),
        );
    }
    Some(left.1.clone())
}

pub(crate) fn check_bool(ctx: &mut SqliteQueryCtx, path: &rpds::Vector<String>, a: &ExprType) {
    let t = match a.assert_scalar(&mut ctx.errs, path) {
        Some(t) => t,
        None => {
            return;
        },
    };
    if t.1.opt {
        ctx.errs.err(path, format!("Expected bool type but is nullable: got {:?}", t));
    }
    if !matches!(t.1.type_.type_, SimpleSimpleType::Bool) {
        ctx.errs.err(path, format!("Expected bool but type is non-bool: got {:?}", t.1.type_.type_));
    }
}

#[cfg(feature = "chrono")]
pub(crate) fn check_utc_if_time(ctx: &mut SqliteQueryCtx, path: &rpds::Vector<String>, t: &ExprType) {
    for (i, el) in t.0.iter().enumerate() {
        if matches!(el.1.type_.type_, SimpleSimpleType::FixedOffsetTimeMs) {
            ctx.errs.err(
                &if t.0.len() == 1 {
                    path.clone()
                } else {
                    path.push_back(format!("Record pair {}", i))
                },
                format!(
                    "Datetimes with non-utc offsets may not be used in normal binary operators - use the `Tz` operators instead"
                ),
            );
        }
    }
}

pub(crate) fn check_assignable(errs: &mut Errs, path: &rpds::Vector<String>, a: &Type, b: &ExprType) {
    check_same(errs, path, &ExprType(vec![(Binding::empty(), a.clone())]), b);
}

impl Expr {
    pub(crate) fn build(
        &self,
        ctx: &mut SqliteQueryCtx,
        path: &rpds::Vector<String>,
        scope: &HashMap<Binding, Type>,
    ) -> (ExprType, Tokens) {
        macro_rules! empty_type{
            ($o: expr, $t: expr) => {
                (ExprType(vec![(Binding::empty(), Type {
                    type_: SimpleType {
                        type_: $t,
                        custom: None,
                    },
                    opt: false,
                    array: false,
                })]), $o)
            };
        }

        fn do_bin_op(
            ctx: &mut SqliteQueryCtx,
            path: &rpds::Vector<String>,
            scope: &HashMap<Binding, Type>,
            op: &BinOp,
            exprs: &Vec<Expr>,
        ) -> (ExprType, Tokens) {
            let operand_lower_limit;
            match op {
                BinOp::Plus | BinOp::Minus | BinOp::Multiply | BinOp::Divide | BinOp::And | BinOp::Or => {
                    operand_lower_limit = 1;
                },
                BinOp::Equals |
                BinOp::NotEquals |
                BinOp::Is |
                BinOp::IsNot |
                BinOp::TzEquals |
                BinOp::TzNotEquals |
                BinOp::TzIs |
                BinOp::TzIsNot |
                BinOp::LessThan |
                BinOp::LessThanEqualTo |
                BinOp::GreaterThan |
                BinOp::GreaterThanEqualTo |
                BinOp::Like |
                BinOp::In |
                BinOp::NotIn => {
                    operand_lower_limit = 2;
                },
            };
            if exprs.len() < operand_lower_limit {
                ctx
                    .errs
                    .err(
                        path,
                        format!(
                            "{:?} must have at least {} operand(s), but got {}",
                            op,
                            operand_lower_limit,
                            exprs.len()
                        ),
                    );
            }
            let mut res = vec![];
            for (i, e) in exprs.iter().enumerate() {
                res.push(e.build(ctx, &path.push_back(format!("Operand {}", i)), scope));
            }
            let t = match op {
                BinOp::Plus | BinOp::Minus | BinOp::Multiply | BinOp::Divide => {
                    let base = res.get(0).unwrap();
                    let t =
                        match check_same(
                            &mut ctx.errs,
                            &path.push_back(format!("Operands 0, 1")),
                            &base.0,
                            &res.get(0).unwrap().0,
                        ) {
                            Some(t) => t,
                            None => {
                                return (ExprType(vec![]), Tokens::new());
                            },
                        };
                    for (i, res) in res.iter().enumerate().skip(2) {
                        match check_same(
                            &mut ctx.errs,
                            &path.push_back(format!("Operands 0, {}", i)),
                            &base.0,
                            &res.0,
                        ) {
                            Some(_) => { },
                            None => {
                                return (ExprType(vec![]), Tokens::new());
                            },
                        };
                    }
                    t
                },
                BinOp::And | BinOp::Or => {
                    for (i, res) in res.iter().enumerate() {
                        check_bool(ctx, &path.push_back(format!("Operand {}", i)), &res.0);
                    }
                    Type {
                        type_: SimpleType {
                            type_: SimpleSimpleType::Bool,
                            custom: None,
                        },
                        array: false,
                        opt: false,
                    }
                },
                BinOp::Equals |
                BinOp::NotEquals |
                BinOp::Is |
                BinOp::IsNot |
                BinOp::TzEquals |
                BinOp::TzNotEquals |
                BinOp::TzIs |
                BinOp::TzIsNot |
                BinOp::LessThan |
                BinOp::LessThanEqualTo |
                BinOp::GreaterThan |
                BinOp::GreaterThanEqualTo |
                BinOp::Like => {
                    #[cfg(feature = "chrono")]
                    if match op {
                        BinOp::TzEquals | BinOp::TzNotEquals | BinOp::TzIs | BinOp::TzIsNot => false,
                        _ => true,
                    } {
                        for (i, el) in res.iter().enumerate() {
                            check_utc_if_time(ctx, &path.push_back(format!("Operand {}", i)), &el.0);
                        }
                    }
                    let base = res.get(0).unwrap();
                    check_general_same(
                        ctx,
                        &path.push_back(format!("Operands 0, 1")),
                        &base.0,
                        &res.get(1).unwrap().0,
                    );
                    for (i, res) in res.iter().enumerate().skip(2) {
                        check_general_same(ctx, &path.push_back(format!("Operands 0, {}", i)), &base.0, &res.0);
                    }
                    Type {
                        type_: SimpleType {
                            type_: SimpleSimpleType::Bool,
                            custom: None,
                        },
                        opt: false,
                        array: false,
                    }
                },
                BinOp::In | BinOp::NotIn => {
                    #[cfg(feature = "chrono")]
                    if match op {
                        BinOp::TzEquals | BinOp::TzNotEquals | BinOp::TzIs | BinOp::TzIsNot => false,
                        _ => true,
                    } {
                        for (i, el) in res.iter().enumerate() {
                            check_utc_if_time(ctx, &path.push_back(format!("Operand {}", i)), &el.0);
                        }
                    }
                    let base = res.get(0).unwrap();
                    check_general_same(
                        ctx,
                        &path.push_back(format!("Operands 0, 1")),
                        &base.0,
                        &res.get(1).unwrap().0,
                    );
                    for (i, res) in res.iter().enumerate().skip(2) {
                        check_general_same(ctx, &path.push_back(format!("Operands 0, {}", i)), &base.0, &res.0);
                    }
                    Type {
                        type_: SimpleType {
                            type_: SimpleSimpleType::Bool,
                            custom: None,
                        },
                        opt: false,
                        array: false,
                    }
                },
            };
            let token = match op {
                BinOp::Plus => "+",
                BinOp::Minus => "-",
                BinOp::Multiply => "*",
                BinOp::Divide => "/",
                BinOp::And => "and",
                BinOp::Or => "or",
                BinOp::Equals => "=",
                BinOp::NotEquals => "!=",
                BinOp::Is => "is",
                BinOp::IsNot => "is not",
                BinOp::TzEquals => "=",
                BinOp::TzNotEquals => "!=",
                BinOp::TzIs => "is",
                BinOp::TzIsNot => "is not",
                BinOp::LessThan => "<",
                BinOp::LessThanEqualTo => "<=",
                BinOp::GreaterThan => ">",
                BinOp::GreaterThanEqualTo => ">=",
                BinOp::Like => "like",
                BinOp::In => "in",
                BinOp::NotIn => "not in",
            };
            let mut out = Tokens::new();
            out.s("(");
            for (i, res) in res.iter().enumerate() {
                if i > 0 {
                    out.s(token);
                }
                out.s(&res.1.to_string());
            }
            out.s(")");
            (ExprType(vec![(Binding::empty(), t)]), out)
        }

        match self {
            Expr::LitArray(t) => {
                let mut out = Tokens::new();
                let mut child_types = vec![];
                out.s("(");
                for (i, child) in t.iter().enumerate() {
                    if i > 0 {
                        out.s(", ");
                    }
                    let (child_type, child_tokens) = child.build(ctx, path, scope);
                    out.s(&child_tokens.to_string());
                    child_types.extend(child_type.0);
                }
                out.s(")");
                return (ExprType(child_types), out);
            },
            Expr::LitNull(t) => {
                let mut out = Tokens::new();
                out.s("null");
                return (ExprType(vec![(Binding::empty(), Type {
                    type_: t.clone(),
                    opt: true,
                    array: false,
                })]), out);
            },
            Expr::LitBool(x) => {
                let mut out = Tokens::new();
                out.s(if *x {
                    "true"
                } else {
                    "false"
                });
                return empty_type!(out, SimpleSimpleType::Bool);
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
            Expr::LitUtcTimeS(d) => {
                let mut out = Tokens::new();
                let d = d.timestamp();
                out.s(&format!("{}", d));
                return empty_type!(out, SimpleSimpleType::UtcTimeS);
            },
            #[cfg(feature = "chrono")]
            Expr::LitUtcTimeMs(d) => {
                let mut out = Tokens::new();
                let d = d.to_rfc3339();
                out.s(&format!("'{}'", d));
                return empty_type!(out, SimpleSimpleType::UtcTimeMs);
            },
            Expr::Param { name: x, type_: t } => {
                let path = path.push_back(format!("Param ({})", x));
                let mut out = Tokens::new();
                let mut errs = vec![];
                let i = match ctx.rust_arg_lookup.entry(x.clone()) {
                    std::collections::hash_map::Entry::Occupied(e) => {
                        let (i, prev_t) = e.get();
                        if t != prev_t {
                            errs.push(
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
                        rust_forward = match t.type_.type_ {
                            SimpleSimpleType::U32 => rust_forward,
                            SimpleSimpleType::I32 => rust_forward,
                            SimpleSimpleType::I64 => rust_forward,
                            SimpleSimpleType::F32 => rust_forward,
                            SimpleSimpleType::F64 => rust_forward,
                            SimpleSimpleType::Bool => rust_forward,
                            SimpleSimpleType::String => rust_forward,
                            SimpleSimpleType::Bytes => rust_forward,
                            #[cfg(feature = "chrono")]
                            SimpleSimpleType::UtcTimeS => quote!(#rust_forward.timestamp()),
                            #[cfg(feature = "chrono")]
                            SimpleSimpleType::UtcTimeMs => quote!(#rust_forward.to_rfc3339()),
                            #[cfg(feature = "chrono")]
                            SimpleSimpleType::FixedOffsetTimeMs => quote!(#rust_forward.to_rfc3339()),
                        };
                        if t.array {
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
                        if t.opt {
                            rust_type = quote!(Option < #rust_type >);
                            rust_forward = quote!(#ident.map(| #ident | #rust_forward));
                        }
                        ctx.rust_args.push(quote!(#ident: #rust_type));
                        ctx.query_args.push(quote!(#rust_forward));
                        i
                    },
                };
                for e in errs {
                    ctx.errs.err(&path, e);
                }
                if t.array {
                    out.s(&format!("rarray(${})", i + 1));
                } else {
                    out.s(&format!("${}", i + 1));
                }
                return (ExprType(vec![(Binding::local(x.clone()), t.clone())]), out);
            },
            Expr::Binding(name) => {
                let t = match scope.get(&name) {
                    Some(t) => t.clone(),
                    None => {
                        ctx
                            .errs
                            .err(
                                path,
                                format!(
                                    "Expression references {} but this field isn't available here (available fields: {:?})",
                                    name,
                                    scope.iter().map(|e| e.0.to_string()).collect::<Vec<String>>()
                                ),
                            );
                        return (ExprType(vec![]), Tokens::new());
                    },
                };
                let mut out = Tokens::new();
                if name.table_id != "" {
                    out.id(&name.table_id).s(".");
                }
                out.id(&name.id);
                return (ExprType(vec![(name.clone(), t.clone())]), out);
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
                return do_bin_op(ctx, &path.push_back(format!("Chain bin op {:?}", op)), scope, op, exprs);
            },
            Expr::PrefixOp { op, right } => {
                let path = path.push_back(format!("Prefix op {:?}", op));
                let mut out = Tokens::new();
                let res = right.build(ctx, &path, scope);
                let (op_text, op_type) = match op {
                    PrefixOp::Not => {
                        check_bool(ctx, &path, &res.0);
                        ("not", SimpleSimpleType::Bool)
                    },
                };
                out.s(op_text).s(&res.1.to_string());
                return empty_type!(out, op_type);
            },
            Expr::Call { func, args, compute_type } => {
                let mut types = vec![];
                let mut out = Tokens::new();
                out.s(func);
                out.s("(");
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        out.s(",");
                    }
                    let (arg_type, tokens) =
                        arg.build(ctx, &path.push_back(format!("Call [{}] arg {}", func, i)), scope);
                    types.push(arg_type);
                    out.s(&tokens.to_string());
                }
                out.s(")");
                let type_ = match (compute_type.0)(ctx, &path, types) {
                    Some(t) => t,
                    None => {
                        return (ExprType(vec![]), Tokens::new());
                    },
                };
                return (ExprType(vec![(Binding::empty(), type_)]), out);
            },
            Expr::Window { expr, partition_by, order_by } => {
                let mut out = Tokens::new();
                let expr = expr.build(ctx, &path, &scope);
                out.s(&expr.1.to_string());
                out.s("over");
                out.s("(");
                if !partition_by.is_empty() {
                    out.s("partition by");
                    for (i, e) in partition_by.iter().enumerate() {
                        let path = path.push_back(format!("Partition by {}", i));
                        if i > 0 {
                            out.s(",");
                        }
                        let (_, p) = e.build(ctx, &path, &scope);
                        out.s(&p.to_string());
                    }
                }
                if !order_by.is_empty() {
                    out.s("order by");
                    for (i, o) in order_by.iter().enumerate() {
                        let path = path.push_back(format!("Order by clause {}", i));
                        if i > 0 {
                            out.s(",");
                        }
                        let (_, o_tokens) = o.0.build(ctx, &path, &scope);
                        out.s(&o_tokens.to_string());
                        out.s(match o.1 {
                            Order::Asc => "asc",
                            Order::Desc => "desc",
                        });
                    }
                }
                out.s(")");
                return (expr.0, out);
            },
            Expr::Select { body, body_junctions } => {
                let path = path.push_back(format!("Subselect"));
                let mut out = Tokens::new();
                let base = body.build(ctx, scope, &path, QueryResCount::Many);
                out.s(&base.1.to_string());
                out.s(&build_select_junction(ctx, &path, &base.0, &body_junctions).to_string());
                return (base.0, out);
            },
            Expr::Exists { not, body, body_junctions } => {
                let path = path.push_back(format!("(Not)Exists"));
                let mut out = Tokens::new();
                if *not {
                    out.s("not");
                }
                out.s("exists");
                out.s("(");
                let base = body.build(ctx, scope, &path, QueryResCount::Many);
                out.s(&base.1.to_string());
                out.s(&build_select_junction(ctx, &path, &base.0, &body_junctions).to_string());
                out.s(")");
                return (ExprType(vec![(Binding::empty(), Type {
                    type_: SimpleType {
                        type_: SimpleSimpleType::Bool,
                        custom: None,
                    },
                    opt: false,
                    array: false,
                })]), out);
            },
            Expr::Cast(e, t) => {
                let path = path.push_back(format!("Cast"));
                let out = e.build(ctx, &path, scope);
                let got_t = match out.0.assert_scalar(&mut ctx.errs, &path) {
                    Some(t) => t,
                    None => {
                        return (ExprType(vec![]), Tokens::new());
                    },
                };
                check_general_same_type(ctx, &path, t, &got_t.1);
                return (ExprType(vec![(got_t.0, t.clone())]), out.1);
            },
        };
    }
}

/// Datetimes with fixed offsets must be converted to utc before comparison.
///
/// The Tz operators are for working with datetimes with fied offsets where you
/// _want_ to not consider datetimes referring to the same instant but with
/// different timezones equal (that is, to be equal both the time and timezone must
/// match). I think this is probably a rare use case.
#[derive(Clone, Debug)]
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
    TzEquals,
    TzNotEquals,
    TzIs,
    TzIsNot,
    LessThan,
    LessThanEqualTo,
    GreaterThan,
    GreaterThanEqualTo,
    Like,
    In,
    NotIn,
}

#[derive(Clone, Debug)]
pub enum PrefixOp {
    Not,
}
