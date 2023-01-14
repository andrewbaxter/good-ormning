use std::collections::HashMap;
use crate::{
    utils::Tokens,
    sqlite::{
        types::Type,
        schema::{
            table::TableId,
            field::FieldId,
        },
        QueryResCount,
    },
};
use super::{
    utils::{
        QueryBody,
        SqliteQueryCtx,
        build_returning_values,
    },
    expr::{
        Expr,
        ExprType,
        check_bool,
    },
};

#[derive(Clone, Debug)]
pub enum Order {
    Asc,
    Desc,
}

#[derive(Clone, Debug)]
pub enum JoinSource {
    Subsel(Box<Select>),
    Table(TableId),
}

#[derive(Clone, Debug)]
pub struct NamedSelectSource {
    pub source: JoinSource,
    pub alias: Option<String>,
}

impl NamedSelectSource {
    fn build(
        &self,
        ctx: &mut SqliteQueryCtx,
        path: &rpds::Vector<String>,
    ) -> (Vec<(FieldId, (String, Type))>, Tokens) {
        let mut out = Tokens::new();
        let mut new_fields: Vec<(FieldId, (String, Type))> = match &self.source {
            JoinSource::Subsel(s) => {
                let res = s.build(ctx, &path.push_back(format!("From subselect")), QueryResCount::Many);
                out.s("(").s(&res.1.to_string()).s(")");
                res.0.0.iter().map(|e| (e.0.field.clone(), (e.0.name.clone(), e.1.clone()))).collect()
            },
            JoinSource::Table(s) => {
                let new_fields = match ctx.tables.get(&s) {
                    Some(f) => f,
                    None => {
                        ctx
                            .errs
                            .err(&path.push_back(format!("From")), format!("No table with id {} in version", s));
                        return (vec![], Tokens::new());
                    },
                };
                out.id(&s.0);
                new_fields.iter().map(|e| (e.0.clone(), e.1.clone())).collect()
            },
        };
        if let Some(s) = &self.alias {
            out.s("as").id(s);
            let mut new_fields2 = vec![];
            for (k, v) in new_fields {
                new_fields2.push((FieldId(TableId(s.clone()), k.1.clone()), v));
            }
            new_fields = new_fields2;
        }
        (new_fields, out)
    }
}

#[derive(Clone, Debug)]
pub enum JoinType {
    Left,
    Inner,
}

#[derive(Clone, Debug)]
pub struct Join {
    pub source: Box<NamedSelectSource>,
    pub type_: JoinType,
    pub on: Expr,
}

#[derive(Clone, Debug)]
pub struct SelectOutput {
    pub e: Expr,
    pub rename: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Select {
    pub(crate) table: NamedSelectSource,
    pub(crate) output: Vec<SelectOutput>,
    pub(crate) join: Vec<Join>,
    pub(crate) where_: Option<Expr>,
    pub(crate) group: Vec<Expr>,
    pub(crate) order: Vec<(Expr, Order)>,
    pub(crate) limit: Option<usize>,
}

impl QueryBody for Select {
    fn build(
        &self,
        ctx: &mut super::utils::SqliteQueryCtx,
        path: &rpds::Vector<String>,
        res_count: QueryResCount,
    ) -> (ExprType, Tokens) {
        // Prep
        let source = self.table.build(ctx, path);
        let mut fields = HashMap::new();
        for (k, v) in source.0 {
            fields.insert(k, v);
        }
        let mut scope = fields.clone();
        let mut joins = vec![];
        for (i, je) in self.join.iter().enumerate() {
            let path = path.push_back(format!("Join {}", i));
            let mut out = Tokens::new();
            match je.type_ {
                JoinType::Left => out.s("left"),
                JoinType::Inner => out.s("inner"),
            };
            out.s("join");
            let source = je.source.build(ctx, &path);
            out.s(&source.1.to_string());
            match je.type_ {
                JoinType::Left => {
                    for (k, mut v) in source.0 {
                        if !v.1.opt {
                            v.1 = Type {
                                opt: true,
                                type_: v.1.type_,
                            };
                        }
                        scope.insert(k, v);
                    }
                },
                JoinType::Inner => {
                    for (k, v) in source.0 {
                        scope.insert(k, v);
                    }
                },
            }
            out.s("on").s(&je.on.build(ctx, &path, &scope).1.to_string());
            joins.push(out.to_string());
        }

        // Build query
        let mut out = Tokens::new();
        out.s("select");
        if self.output.is_empty() {
            ctx.errs.err(path, format!("Select must have at least one output, but outputs are empty"));
        }
        let out_type = build_returning_values(ctx, path, &scope, &mut out, &self.output, res_count);
        out.s("from");
        out.s(&source.1.to_string());
        for join in joins {
            out.s(&join);
        }
        if let Some(where_) = &self.where_ {
            out.s("where");
            let path = path.push_back("Where".into());
            let (where_t, where_tokens) = where_.build(ctx, &path, &scope);
            check_bool(ctx, &path, &where_t);
            out.s(&where_tokens.to_string());
        }
        if self.group.len() > 0 {
            out.s("group by");
            for (i, g) in self.group.iter().enumerate() {
                let path = path.push_back(format!("Group by clause {}", i));
                if i > 0 {
                    out.s(",");
                }
                let (_, g_tokens) = g.build(ctx, &path, &scope);
                out.s(&g_tokens.to_string());
            }
        }
        if !self.order.is_empty() {
            out.s("order by");
            for (i, o) in self.order.iter().enumerate() {
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
        if let Some(l) = self.limit {
            out.s("limit");
            out.s(&format!("{}", l));
        }
        (out_type, out)
    }
}
