use std::collections::HashMap;
use crate::{
    utils::Tokens,
    pg::{
        types::Type,
        schema::table::TableId,
    },
};
use super::{
    utils::{
        Query,
        PgQueryCtx,
        build_returning_values,
    },
    expr::{
        Expr,
        ExprType,
        ExprTypeField,
    },
};

pub enum Order {
    Asc,
    Desc,
}

pub enum JoinSource {
    Subsel(Box<Select>),
    Table(TableId),
}

pub struct NamedSelectSource {
    pub source: JoinSource,
    pub alias: Option<String>,
}

impl NamedSelectSource {
    fn build(&self, ctx: &mut PgQueryCtx) -> (Vec<(ExprTypeField, Type)>, Tokens) {
        let mut out = Tokens::new();
        let mut new_fields = match &self.source {
            JoinSource::Subsel(s) => {
                let res = s.build(ctx);
                out.s("(").s(&res.1.to_string()).s(")");
                res.0.0
            },
            JoinSource::Table(s) => {
                let new_fields = match ctx.tables.get(&s) {
                    Some(f) => f,
                    None => {
                        ctx.errs.err(format!("No table with id {} in version", s));
                        return (vec![], Tokens::new());
                    },
                };
                out.id(&s.0);
                new_fields.iter().map(|(x, y)| (x.clone(), y.clone())).collect()
            },
        };
        if let Some(s) = &self.alias {
            out.s("as").id(s);
            let mut new_fields2 = vec![];
            for (k, v) in new_fields {
                new_fields2.push((ExprTypeField {
                    table: s.clone(),
                    field: k.field,
                }, v));
            }
            new_fields = new_fields2;
        }
        (new_fields, out)
    }
}

pub enum JoinType {
    Left,
    Inner,
}

pub struct Join {
    pub source: Box<NamedSelectSource>,
    pub type_: JoinType,
    pub on: Expr,
}

pub struct SelectOutput {
    pub e: Expr,
    pub rename: Option<String>,
}

pub struct Select {
    pub table: NamedSelectSource,
    pub output: Vec<SelectOutput>,
    pub join: Vec<Join>,
    pub where_: Option<Expr>,
    pub group: Vec<Expr>,
    pub order: Option<Vec<(Expr, Order)>>,
    pub limit: Option<usize>,
}

impl Query for Select {
    fn build(&self, ctx: &mut super::utils::PgQueryCtx) -> (ExprType, Tokens) {
        // Prep
        let source = self.table.build(ctx);
        let mut fields = HashMap::new();
        for (k, v) in source.0 {
            fields.insert(k, v);
        }
        let mut all_fields = fields.clone();
        let mut joins = vec![];
        for je in &self.join {
            let mut out = Tokens::new();
            match je.type_ {
                JoinType::Left => out.s("left"),
                JoinType::Inner => out.s("inner"),
            };
            out.s("join");
            let source = je.source.build(ctx);
            out.s(&source.1.to_string());
            out.s("on").s(&je.on.build(ctx, &fields).1.to_string());
            match je.type_ {
                JoinType::Left => {
                    for (k, mut v) in source.0 {
                        if !v.opt {
                            v = Type {
                                opt: true,
                                type_: v.type_,
                            };
                        }
                        all_fields.insert(k, v);
                    }
                },
                JoinType::Inner => {
                    for (k, v) in source.0 {
                        all_fields.insert(k, v);
                    }
                },
            }
            joins.push(out.to_string());
        }

        // Build query
        let mut out = Tokens::new();
        out.s("select");
        let out_type = build_returning_values(ctx, &all_fields, &mut out, &self.output);
        out.s("from");
        out.s(&source.1.to_string());
        for join in joins {
            out.s(&join);
        }
        if let Some(where_) = &self.where_ {
            out.s("where");
            where_.build(ctx, &all_fields);
        }
        if self.group.len() > 0 {
            out.s("group by");
            for (i, g) in self.group.iter().enumerate() {
                if i > 0 {
                    out.s(",");
                }
                g.build(ctx, &all_fields);
            }
        }
        if let Some(o) = &self.order {
            out.s("order by");
            for (i, o) in o.iter().enumerate() {
                if i > 0 {
                    out.s(",");
                }
                o.0.build(ctx, &all_fields);
                out.s(match o.1 {
                    Order::Asc => "asc",
                    Order::Desc => "desc",
                });
            }
        }
        if let Some(l) = self.limit {
            out.s("limit");
            out.s(&format!("{}", l));
        }
        (out_type, out)
    }
}
