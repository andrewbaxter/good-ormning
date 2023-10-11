use std::{
    collections::HashMap,
};
use proc_macro2::TokenStream;
use crate::{
    pg::{
        types::Type,
        QueryResCount,
        schema::{
            field::Field,
            table::Table,
        },
    },
    utils::{
        Tokens,
        Errs,
    },
};
use super::{
    expr::{
        ExprType,
        ExprValName,
        Expr,
        check_assignable,
    },
    select::Returning,
};

pub struct PgQueryCtx<'a> {
    pub(crate) tables: &'a HashMap<Table, HashMap<Field, Type>>,
    pub errs: Errs,
    pub(crate) rust_arg_lookup: HashMap<String, (usize, Type)>,
    pub(crate) rust_args: Vec<TokenStream>,
    pub(crate) query_args: Vec<TokenStream>,
}

impl<'a> PgQueryCtx<'a> {
    pub(crate) fn new(errs: Errs, tables: &'a HashMap<Table, HashMap<Field, Type>>) -> Self {
        Self {
            tables: tables,
            errs: errs,
            rust_arg_lookup: Default::default(),
            rust_args: Default::default(),
            query_args: Default::default(),
        }
    }
}

pub trait QueryBody {
    fn build(
        &self,
        ctx: &mut PgQueryCtx,
        path: &rpds::Vector<String>,
        res_count: QueryResCount,
    ) -> (ExprType, Tokens);
}

pub fn build_set(
    ctx: &mut PgQueryCtx,
    path: &rpds::Vector<String>,
    scope: &HashMap<ExprValName, Type>,
    out: &mut Tokens,
    values: &Vec<(Field, Expr)>,
) {
    out.s("set");
    for (i, (field, val)) in values.iter().enumerate() {
        let path = path.push_back(format!("Set field {}", i));
        if i > 0 {
            out.s(",");
        }
        out.id(&field.id).s("=");
        let res = val.build(ctx, &path, &scope);
        let field_type = match ctx.tables.get(&field.table).and_then(|t| t.get(&field)) {
            Some(t) => t,
            None => {
                ctx.errs.err(&path, format!("Update destination value field {} is not known", field));
                continue;
            },
        };
        check_assignable(&mut ctx.errs, &path, field_type, &res.0);
        out.s(&res.1.to_string());
    }
}

pub fn build_returning_values(
    ctx: &mut PgQueryCtx,
    path: &rpds::Vector<String>,
    scope: &HashMap<ExprValName, Type>,
    out: &mut Tokens,
    outputs: &Vec<Returning>,
    res_count: QueryResCount,
) -> ExprType {
    if outputs.is_empty() {
        if !matches!(res_count, QueryResCount::None) {
            ctx.errs.err(path, format!("Query has no outputs but res_count is, {:?}, not None", res_count));
        }
    } else {
        if matches!(res_count, QueryResCount::None) {
            ctx.errs.err(&path, format!("Query has outputs so res_count must be not None, but is {:?}", res_count));
        }
    }
    let mut out_rec: Vec<(ExprValName, Type)> = vec![];
    for (i, o) in outputs.iter().enumerate() {
        let path = path.push_back(format!("Result {}", i));
        if i > 0 {
            out.s(",");
        }
        let res = o.e.build(ctx, &path, scope);
        out.s(&res.1.to_string());
        let (res_name, res_type) = match res.0.assert_scalar(&mut ctx.errs, &path) {
            Some(x) => x,
            None => continue,
        };
        if let Some(rename) = &o.rename {
            out.s("as").id(rename);
            out_rec.push((ExprValName::local(rename.clone()), res_type));
        } else {
            out_rec.push((res_name, res_type));
        }
    }
    ExprType(out_rec)
}

pub fn build_returning(
    ctx: &mut PgQueryCtx,
    path: &rpds::Vector<String>,
    scope: &HashMap<ExprValName, Type>,
    out: &mut Tokens,
    outputs: &Vec<Returning>,
    res_count: QueryResCount,
) -> ExprType {
    if !outputs.is_empty() {
        out.s("returning");
    }
    build_returning_values(ctx, path, scope, out, outputs, res_count)
}
