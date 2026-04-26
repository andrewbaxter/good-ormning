use std::collections::HashMap;
use crate::{
    utils::Tokens,
    sqlite::{
        types::{
            Type,
            type_i64,
        },
        QueryResCount,
        schema::{
            table::TableRef,
        },
    },
};
use super::{
    utils::{
        SqliteQueryCtx,
        QueryBody,
        build_returning,
        build_returning_values,
        Returning,
        With,
        build_with,
    },
    expr::{
        Expr,
        ExprType,
        check_bool,
        check_general_same,
        Binding,
    },
    select_body::{
        SelectJunction,
        build_select_junction,
    },
};

#[derive(Clone, Debug)]
pub enum Order {
    Asc,
    Desc,
}

#[derive(Clone, Debug)]
pub enum JoinType {
    Inner,
    Left,
}

#[derive(Clone, Debug)]
pub struct Join {
    pub type_: JoinType,
    pub source: NamedSelectSource,
    pub on: Expr,
}

#[derive(Clone, Debug)]
pub enum JoinSource {
    Subsel(Box<Select>),
    Table(TableRef),
}

#[derive(Clone, Debug)]
pub struct NamedSelectSource {
    pub source: JoinSource,
    pub alias: Option<String>,
}

impl NamedSelectSource {
    pub fn build(&self, ctx: &mut SqliteQueryCtx, path: &rpds::Vector<String>) -> (ExprType, Tokens) {
        let mut out = Tokens::new();
        let mut new_fields: Vec<(Binding, Type)> = match &self.source {
            JoinSource::Subsel(s) => {
                let res: (ExprType, Tokens) =
                    s.build(ctx, &path.push_back(format!("From subselect")), QueryResCount::Many);
                out.s("(").s(&res.1.to_string()).s(")");
                res.0.0.clone()
            },
            JoinSource::Table(s) => {
                let table_info = match ctx.tables.get(&s) {
                    Some(f) => f,
                    None => {
                        ctx
                            .errs
                            .err(&path.push_back(format!("From")), format!("No table with id {:?} in version", s));
                        return (ExprType(vec![]), Tokens::new());
                    },
                };
                out.id(&table_info.sql_name);
                table_info.fields.iter().map(|(id, info)| (Binding::field(id), info.type_.clone())).collect()
            },
        };
        if let Some(s) = &self.alias {
            out.s("as").id(s);
            let mut new_fields2 = vec![];
            for (k, v) in new_fields {
                new_fields2.push((k.with_alias(s), v));
            }
            new_fields = new_fields2;
        }
        (ExprType(new_fields), out)
    }
}

#[derive(Clone, Debug)]
pub struct Select {
    pub with: Option<With>,
    pub table: NamedSelectSource,
    pub returning: Vec<Returning>,
    pub junction: Vec<SelectJunction>,
    pub join: Vec<Join>,
    pub where_: Option<Expr>,
    pub group: Vec<Expr>,
    pub order: Vec<(Expr, Order)>,
    pub limit: Option<Expr>,
}

impl QueryBody for Select {
    fn build(
        &self,
        ctx: &mut SqliteQueryCtx,
        path: &rpds::Vector<String>,
        res_count: QueryResCount,
    ) -> (ExprType, Tokens) {
        let mut scope = HashMap::new();
        let mut out = Tokens::new();
        if let Some(with) = &self.with {
            out.s(&build_with(ctx, path, with).to_string());
        }
        out.s("select");
        let (fields, table_tokens): (ExprType, Tokens) = self.table.build(ctx, path);
        for (k, v) in fields.0 {
            scope.insert(k, v);
        }
        for (i, j) in self.join.iter().enumerate() {
            let path = path.push_back(format!("Join {}", i));
            let (fields, _): (ExprType, Tokens) = j.source.build(ctx, &path);
            match j.type_ {
                JoinType::Left => {
                    for (k, mut v) in fields.0 {
                        if !v.opt {
                            v = Type {
                                opt: true,
                                arr: false,
                                type_: v.type_,
                            };
                        }
                        scope.insert(k, v);
                    }
                },
                JoinType::Inner => {
                    for (k, v) in fields.0 {
                        scope.insert(k, v);
                    }
                },
            }
        }
        let out_type = build_returning_values(ctx, path, &scope, &mut out, &self.returning, res_count);
        out.s("from").s(&table_tokens.to_string());
        for (i, j) in self.join.iter().enumerate() {
            let path = path.push_back(format!("Join {}", i));
            match j.type_ {
                JoinType::Inner => {
                    out.s("inner join");
                },
                JoinType::Left => {
                    out.s("left join");
                },
            }
            let (_, source_tokens): (ExprType, Tokens) = j.source.build(ctx, &path);
            out.s(&source_tokens.to_string()).s("on");
            let (on_t, on_tokens): (ExprType, Tokens) = j.on.build(ctx, &path.push_back(format!("On")), &scope);
            check_bool(ctx, &path, &on_t);
            out.s(&on_tokens.to_string());
        }
        if let Some(where_) = &self.where_ {
            out.s("where");
            let path = path.push_back("Where".into());
            let (where_t, where_tokens): (ExprType, Tokens) = where_.build(ctx, &path, &scope);
            check_bool(ctx, &path, &where_t);
            out.s(&where_tokens.to_string());
        }
        if !self.group.is_empty() {
            out.s("group by");
            for (i, g) in self.group.iter().enumerate() {
                if i > 0 {
                    out.s(",");
                }
                let path = path.push_back(format!("Group by {}", i));
                let (_, tokens): (ExprType, Tokens) = g.build(ctx, &path, &scope);
                out.s(&tokens.to_string());
            }
        }
        if !self.order.is_empty() {
            out.s("order by");
            for (i, (e, o)) in self.order.iter().enumerate() {
                if i > 0 {
                    out.s(",");
                }
                let path = path.push_back(format!("Order by {}", i));
                let (_, tokens): (ExprType, Tokens) = e.build(ctx, &path, &scope);
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
        if let Some(l) = &self.limit {
            out.s("limit");
            let path = path.push_back("Limit".into());
            let (limit_t, limit_tokens): (ExprType, Tokens) = l.build(ctx, &path, &scope);
            check_general_same(ctx, &path, &limit_t, &ExprType(vec![(Binding::empty(), type_i64().build())]));
            out.s(&limit_tokens.to_string());
        }
        if !self.junction.is_empty() {
            let junction_tokens = build_select_junction(ctx, path, &out_type, &self.junction);
            out.s(&junction_tokens.to_string());
        }
        (out_type, out)
    }
}
