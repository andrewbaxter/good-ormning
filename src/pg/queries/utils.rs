use std::collections::HashMap;
use proc_macro2::TokenStream;
use crate::{
    pg::{
        types::Type,
        schema::{
            table::TableId,
            field::FieldId,
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
        ExprTypeField,
        Expr,
    },
    select::SelectOutput,
};

pub struct PgQueryCtx<'a> {
    pub(crate) tables: &'a HashMap<TableId, HashMap<ExprTypeField, Type>>,
    pub(crate) errs: &'a mut Errs,
    pub(crate) arg_lookup: HashMap<String, (usize, Type)>,
    pub(crate) args: Vec<TokenStream>,
    pub(crate) args_forward: Vec<TokenStream>,
}

impl<'a> PgQueryCtx<'a> {
    pub(crate) fn new(errs: &'a mut Errs, tables: &'a HashMap<TableId, HashMap<ExprTypeField, Type>>) -> Self {
        Self {
            tables: tables,
            errs: errs,
            arg_lookup: Default::default(),
            args: Default::default(),
            args_forward: Default::default(),
        }
    }
}

pub trait Query {
    fn build(&self, ctx: &mut PgQueryCtx) -> (ExprType, Tokens);
}

pub fn build_set(
    ctx: &mut PgQueryCtx,
    all_fields: &HashMap<ExprTypeField, Type>,
    out: &mut Tokens,
    values: &Vec<(FieldId, Expr)>,
) {
    out.s("set");
    for (i, (k, v)) in values.iter().enumerate() {
        if i > 0 {
            out.s(",");
        }
        out.id(&k.1).s("=");
        let res = v.build(ctx, &all_fields);
        res.0.assert_scalar(ctx);
        out.s(&res.1.to_string());
    }
}

pub fn build_returning_values(
    ctx: &mut PgQueryCtx,
    all_fields: &HashMap<ExprTypeField, Type>,
    out: &mut Tokens,
    outputs: &Vec<SelectOutput>,
) -> ExprType {
    let mut out_rec: Vec<(ExprTypeField, Type)> = vec![];
    for (i, o) in outputs.iter().enumerate() {
        if i > 0 {
            out.s(",");
        }
        let res = o.e.build(ctx, all_fields);
        let (res_name, res_type) = match res.0.assert_scalar(ctx) {
            Some(x) => x,
            None => continue,
        };
        if let Some(rename) = &o.rename {
            out.s("as").id(rename);
            out_rec.push((ExprTypeField {
                table: "".into(),
                field: rename.clone(),
            }, res_type));
        } else {
            out_rec.push((ExprTypeField {
                table: "".into(),
                field: res_name.field,
            }, res_type));
        }
    }
    ExprType(out_rec)
}

pub fn build_returning(
    ctx: &mut PgQueryCtx,
    all_fields: &HashMap<ExprTypeField, Type>,
    out: &mut Tokens,
    outputs: &Vec<SelectOutput>,
) -> ExprType {
    if !outputs.is_empty() {
        out.s("returning");
    }
    build_returning_values(ctx, all_fields, out, outputs)
}
