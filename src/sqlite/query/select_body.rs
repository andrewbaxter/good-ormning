use {
    super::{
        expr::{
            check_assignable,
            check_bool,
            check_general_same,
            Expr,
            ExprType,
            Binding,
        },
        utils::{
            build_returning_values,
            SqliteQueryCtx,
        },
    },
    crate::{
        sqlite::{
            schema::table::Table,
            types::{
                type_i64,
                Type,
            },
            QueryResCount,
        },
        utils::Tokens,
    },
    std::collections::HashMap,
};

#[derive(Clone, Debug)]
pub enum Order {
    Asc,
    Desc,
}

#[derive(Clone, Debug)]
pub enum JoinSource {
    Subsel(Box<SelectBody>),
    Table(Table),
}

#[derive(Clone, Debug)]
pub struct NamedSelectSource {
    pub source: JoinSource,
    pub alias: Option<String>,
}

impl NamedSelectSource {
    fn build(&self, ctx: &mut SqliteQueryCtx, path: &rpds::Vector<String>) -> (Vec<(Binding, Type)>, Tokens) {
        let mut out = Tokens::new();
        let mut new_fields: Vec<(Binding, Type)> = match &self.source {
            JoinSource::Subsel(s) => {
                let res =
                    s.build(ctx, &HashMap::new(), &path.push_back(format!("From subselect")), QueryResCount::Many);
                out.s("(").s(&res.1.to_string()).s(")");
                res.0.0.clone()
            },
            JoinSource::Table(s) => {
                let new_fields = match ctx.tables.get(&s) {
                    Some(f) => f,
                    None => {
                        ctx.errs.err(&path.push_back(format!("From")), format!("No known table with id {}", s));
                        return (vec![], Tokens::new());
                    },
                };
                out.id(&s.id);
                new_fields.iter().map(|e| (Binding::field(e), e.type_.type_.clone())).collect()
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
pub struct Returning {
    pub e: Expr,
    pub rename: Option<String>,
}

#[derive(Clone, Debug)]
pub struct SelectBody {
    pub table: NamedSelectSource,
    pub distinct: bool,
    pub returning: Vec<Returning>,
    pub join: Vec<Join>,
    pub where_: Option<Expr>,
    pub group: Vec<Expr>,
    pub order: Vec<(Expr, Order)>,
    pub limit: Option<Expr>,
}

impl SelectBody {
    pub fn build(
        &self,
        ctx: &mut super::utils::SqliteQueryCtx,
        inject_scope: &HashMap<Binding, Type>,
        path: &rpds::Vector<String>,
        res_count: QueryResCount,
    ) -> (ExprType, Tokens) {
        // Prep
        let source = self.table.build(ctx, path);
        let mut scope = inject_scope.clone();
        for (k, v) in source.0 {
            scope.insert(k, v);
        }
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
                        if !v.opt {
                            v = Type {
                                opt: true,
                                type_: v.type_,
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
        if self.distinct {
            out.s("distinct");
        }
        if self.returning.is_empty() {
            ctx.errs.err(path, format!("Select must have at least one output, but outputs are empty"));
        }
        let out_type = build_returning_values(ctx, path, &scope, &mut out, &self.returning, res_count);
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
        if let Some(l) = &self.limit {
            out.s("limit");
            let path = path.push_back("Limit".into());
            let (limit_t, limit_tokens) = l.build(ctx, &path, &scope);
            check_general_same(ctx, &path, &limit_t, &ExprType(vec![(Binding::empty(), type_i64().build())]));
            out.s(&limit_tokens.to_string());
        }
        (out_type, out)
    }
}

#[derive(Clone, Debug, Copy)]
pub enum SelectJunctionOperator {
    Union,
    UnionAll,
    Intersect,
    Except,
}

#[derive(Clone, Debug)]
pub struct SelectJunction {
    pub op: SelectJunctionOperator,
    pub body: SelectBody,
}

pub fn build_select_junction(
    ctx: &mut super::utils::SqliteQueryCtx,
    path: &rpds::Vector<String>,
    base_type: &ExprType,
    body_junctions: &[SelectJunction],
) -> Tokens {
    let mut out = Tokens::new();
    for (i, j) in body_junctions.iter().enumerate() {
        let path = path.push_back(format!("Junction clause {} - {:?}", i, j.op));
        match j.op {
            SelectJunctionOperator::Union => {
                out.s("union");
            },
            SelectJunctionOperator::UnionAll => {
                out.s("union all");
            },
            SelectJunctionOperator::Intersect => {
                out.s("intersect");
            },
            SelectJunctionOperator::Except => {
                out.s("except");
            },
        }
        let j_body = j.body.build(ctx, &HashMap::new(), &path, QueryResCount::Many);
        if j_body.0.0.len() != base_type.0.len() {
            ctx
                .errs
                .err(
                    &path,
                    format!(
                        "Select returns {} columns but the base select has {} columns and these must match exactly",
                        j_body.0.0.len(),
                        base_type.0.len()
                    ),
                );
            continue;
        }
        for (i, ((_, got), (_, want))) in Iterator::zip(j_body.0.0.iter(), base_type.0.iter()).enumerate() {
            let path = path.push_back(format!("Select return {}", i));
            check_assignable(&mut ctx.errs, &path, want, &ExprType(vec![(Binding::empty(), got.clone())]));
        }
        out.s(&j_body.1.to_string());
    }
    return out;
}
