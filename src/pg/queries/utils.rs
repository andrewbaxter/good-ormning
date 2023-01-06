use std::collections::HashMap;
use proc_macro2::TokenStream;
use crate::{
    pg::{
        schema::{
            TableId,
            FieldId,
        },
        types::Type,
    },
    utils::Tokens,
};
use super::{
    expr::{
        ExprType,
        ExprTypeField,
        Expr,
    },
    select::SelectOutput,
};

pub(crate) struct QueryCtx<'a> {
    pub(crate) tables: HashMap<TableId, HashMap<ExprTypeField, Type>>,
    errs: &'a mut Vec<String>,
    pub(crate) err_ctx: Vec<Vec<(&'static str, String)>>,
    pub(crate) param_count: usize,
    pub(crate) args: Vec<TokenStream>,
    pub(crate) args_forward: Vec<TokenStream>,
    pub(crate) params: Vec<TokenStream>,
}

impl<'a> QueryCtx<'a> {
    pub(crate) fn new(errs: &'a mut Vec<String>) -> Self {
        Self {
            tables: Default::default(),
            errs: errs,
            err_ctx: vec![],
            param_count: 0,
            args: Default::default(),
            args_forward: Default::default(),
            params: Default::default(),
        }
    }

    pub fn err(&mut self, t: String) {
        let mut out = String::new();
        for (i, (k, v)) in self.err_ctx.iter().rev().flatten().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            out.push_str(&format!("{}: {}", k, v));
        }
        out.push_str(" - ");
        out.push_str(&t);
        self.errs.push(out);
    }
}

pub(crate) trait Query {
    fn build(&self, ctx: &mut QueryCtx) -> (ExprType, Tokens);
}

pub fn build_set(
    ctx: &mut QueryCtx,
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
    ctx: &mut QueryCtx,
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
    ctx: &mut QueryCtx,
    all_fields: &HashMap<ExprTypeField, Type>,
    out: &mut Tokens,
    outputs: &Vec<SelectOutput>,
) -> ExprType {
    if !outputs.is_empty() {
        out.s("returning");
    }
    build_returning_values(ctx, all_fields, out, outputs)
}
