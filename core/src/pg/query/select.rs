use std::collections::HashMap;
use crate::{
    utils::Tokens,
    pg::{
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
        QueryBody,
        PgQueryCtx,
        build_returning_values,
        Returning,
        With,
        build_with,
    },
    expr::{
        Expr,
        ExprType,
        check_bool,
        ExprValName,
        check_general_same,
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
    Table(TableRef),
    Func(String, Vec<Expr>),
    Empty,
}

#[derive(Clone, Debug)]
pub struct NamedSelectSource {
    pub source: JoinSource,
    pub alias: Option<String>,
}

impl NamedSelectSource {
    pub fn build(&self, ctx: &mut PgQueryCtx, path: &rpds::Vector<String>) -> (ExprType, Tokens) {
        let mut out = Tokens::new();
        let mut new_fields: Vec<(ExprValName, Type)> = match &self.source {
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
                table_info.fields.iter().map(|(id, info)| (ExprValName::field(id), info.type_.clone())).collect()
            },
            JoinSource::Func(name, args) => {
                out.id(name).s("(");
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        out.s(",");
                    }
                    let (_, tokens) = arg.build(ctx, &path.push_back(format!("Arg {}", i)), &HashMap::new());
                    out.s(&tokens.to_string());
                }
                out.s(")");
                vec![]
            },
            JoinSource::Empty => {
                vec![]
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
pub struct Select {
    pub with: Option<With>,
    pub table: NamedSelectSource,
    pub returning: Vec<Returning>,
    pub join: Vec<Join>,
    pub where_: Option<Expr>,
    pub group: Vec<Expr>,
    pub having: Option<Expr>,
    pub order: Vec<(Expr, Order)>,
    pub limit: Option<Expr>,
    pub distinct: bool,
    pub junctions: Vec<super::select_body::SelectJunction>,
}

impl QueryBody for Select {
    fn build(
        &self,
        ctx: &mut super::utils::PgQueryCtx,
        path: &rpds::Vector<String>,
        res_count: QueryResCount,
    ) -> (ExprType, Tokens) {
        let mut out = Tokens::new();
        if let Some(with) = &self.with {
            out.s(&build_with(ctx, path, with).to_string());
        }

        // Prep
        let source: (ExprType, Tokens) = self.table.build(ctx, path);
        let mut fields = HashMap::new();
        for (k, v) in source.0.0 {
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
            let source: (ExprType, Tokens) = je.source.build(ctx, &path);
            out.s(&source.1.to_string());
            match je.type_ {
                JoinType::Left => {
                    for (k, mut v) in source.0.0 {
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
                    for (k, v) in source.0.0 {
                        scope.insert(k, v);
                    }
                },
            }
            out.s("on");
            let (_on_t, on_tokens): (ExprType, Tokens) = je.on.build(ctx, &path, &scope);
            out.s(&on_tokens.to_string());
            joins.push(out.to_string());
        }

        // Build query
        out.s("select");
        if self.distinct {
            out.s("distinct");
        }
        if self.returning.is_empty() {
            ctx.errs.err(path, format!("Select must have at least one output, but outputs are empty"));
        }
        let out_type = build_returning_values(ctx, path, &scope, &mut out, &self.returning, res_count);
        if !matches!(self.table.source, JoinSource::Empty) {
            out.s("from");
            out.s(&source.1.to_string());
            for join in joins {
                out.s(&join);
            }
        }
        if let Some(where_) = &self.where_ {
            out.s("where");
            let path = path.push_back("Where".into());
            let (where_t, where_tokens): (ExprType, Tokens) = where_.build(ctx, &path, &scope);
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
                let (_, g_tokens): (ExprType, Tokens) = g.build(ctx, &path, &scope);
                out.s(&g_tokens.to_string());
            }
        }
        if let Some(having) = &self.having {
            out.s("having");
            let path = path.push_back("Having".into());
            let (having_t, having_tokens): (ExprType, Tokens) = having.build(ctx, &path, &scope);
            check_bool(ctx, &path, &having_t);
            out.s(&having_tokens.to_string());
        }
        if !self.order.is_empty() {
            out.s("order by");
            for (i, o) in self.order.iter().enumerate() {
                let path = path.push_back(format!("Order by clause {}", i));
                if i > 0 {
                    out.s(",");
                }
                let (_, o_tokens): (ExprType, Tokens) = o.0.build(ctx, &path, &scope);
                out.s(&o_tokens.to_string());
                match o.1 {
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
            check_general_same(ctx, &path, &limit_t, &ExprType(vec![(ExprValName::empty(), type_i64().build())]));
            out.s(&limit_tokens.to_string());
        }
        for (i, j) in self.junctions.iter().enumerate() {
            let path = path.push_back(format!("Junction {}", i));
            match j.op {
                super::select_body::SelectJunctionOperator::Union => out.s("union"),
                super::select_body::SelectJunctionOperator::UnionAll => out.s("union all"),
                super::select_body::SelectJunctionOperator::Intersect => out.s("intersect"),
                super::select_body::SelectJunctionOperator::Except => out.s("except"),
            };
            let (j_body_type, j_body_tokens) = j.body.build(ctx, &path, QueryResCount::Many);
            check_general_same(ctx, &path, &out_type, &j_body_type);
            out.s(&j_body_tokens.to_string());
        }
        (out_type, out)
    }
}
