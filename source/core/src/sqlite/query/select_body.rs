use {
    crate::{
        sqlite::{
            QueryResCount,
            query::utils::Returning,
            schema::table::TableRef,
            types::{
                Type,
                type_i64,
            },
        },
        utils::Tokens,
    },
    std::collections::HashMap,
    super::{
        expr::{
            Binding,
            Expr,
            ExprType,
            check_assignable,
            check_bool,
            check_general_same,
        },
        utils::{
            QueryBody,
            SqliteQueryCtx,
            build_returning_values,
        },
    },
};

#[derive(Clone, Debug)]
pub enum Order {
    Asc,
    Desc,
}

#[derive(Clone, Debug)]
pub enum JoinSource {
    Subsel(Box<SelectBody>),
    Table(TableRef),
    Func(String, Vec<Expr>),
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
                    s.build_internal(
                        ctx,
                        &HashMap::new(),
                        &path.push_back("From subselect".to_string()),
                        QueryResCount::Many,
                    );
                out.s("(").s(&res.1.to_string()).s(")");
                res.0.0.clone()
            },
            JoinSource::Table(s) => {
                let table_info = match ctx.tables.get(s) {
                    Some(f) => f,
                    None => {
                        ctx
                            .errs
                            .err(&path.push_back("From".to_string()), format!("No known table with id {:?}", s));
                        return (vec![], Tokens::new());
                    },
                };
                out.id(&table_info.sql_name);
                table_info.fields.iter().map(|(id, info)| (Binding::field(id), info.type_.clone())).collect()
            },
            JoinSource::Func(name, args) => {
                out.s(name).s("(");
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        out.s(",");
                    }
                    let (_, tokens) = arg.build(ctx, &path.push_back(format!("Arg {}", i)), &HashMap::new());
                    out.s(&tokens.to_string());
                }
                out.s(")");
                if name == "rarray" {
                    vec![(Binding::local("value".into()), Type {
                        type_: crate::sqlite::types::SimpleType {
                            type_: crate::sqlite::types::SimpleSimpleType::I32,
                            custom: None,
                        },
                        opt: false,
                        arr: false,
                    })]
                } else {
                    vec![]
                }
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
    Inner,
    Left,
    Right,
    Full,
    Cross,
}

#[derive(Clone, Debug)]
pub struct Join {
    pub source: Box<NamedSelectSource>,
    pub type_: JoinType,
    pub on: Option<Expr>,
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
    pub junctions: Vec<SelectJunction>,
}

impl QueryBody for SelectBody {
    fn build(
        &self,
        ctx: &mut SqliteQueryCtx,
        path: &rpds::Vector<String>,
        res_count: QueryResCount,
    ) -> (ExprType, Tokens) {
        return self.build_internal(ctx, &HashMap::new(), path, res_count);
    }
}

impl SelectBody {
    pub fn build_internal(
        &self,
        ctx: &mut SqliteQueryCtx,
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
                JoinType::Inner => out.s("inner"),
                JoinType::Left => out.s("left"),
                JoinType::Right => out.s("right"),
                JoinType::Full => out.s("full outer"),
                JoinType::Cross => out.s("cross"),
            };
            out.s("join");
            let source = je.source.build(ctx, &path);
            out.s(&source.1.to_string());
            match je.type_ {
                JoinType::Inner => {
                    for (k, v) in source.0 {
                        scope.insert(k, v);
                    }
                },
                JoinType::Left => {
                    for (k, mut v) in source.0 {
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
                JoinType::Right => {
                    for v in scope.values_mut() {
                        v.opt = true;
                    }
                    for (k, v) in source.0 {
                        scope.insert(k, v);
                    }
                },
                JoinType::Full => {
                    for v in scope.values_mut() {
                        v.opt = true;
                    }
                    for (k, mut v) in source.0 {
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
                JoinType::Cross => {
                    for (k, v) in source.0 {
                        scope.insert(k, v);
                    }
                },
            }
            if let Some(on) = &je.on {
                out.s("on");
                let (_, on_tokens): (ExprType, Tokens) = on.build(ctx, &path, &scope);
                out.s(&on_tokens.to_string());
            }
            joins.push(out.to_string());
        }

        // Build query
        let mut out = Tokens::new();
        out.s("select");
        if self.distinct {
            out.s("distinct");
        }
        if self.returning.is_empty() {
            ctx.errs.err(path, "Select must have at least one output, but outputs are empty".to_string());
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
            let (where_t, where_tokens): (ExprType, Tokens) = where_.build(ctx, &path, &scope);
            check_bool(ctx, &path, &where_t);
            out.s(&where_tokens.to_string());
        }
        if !self.group.is_empty() {
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
            check_general_same(ctx, &path, &limit_t, &ExprType(vec![(Binding::empty(), type_i64().build())]));
            out.s(&limit_tokens.to_string());
        }
        if !self.junctions.is_empty() {
            let junction_tokens = build_select_junction(ctx, path, &out_type, &self.junctions);
            out.s(&junction_tokens.to_string());
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
    pub body: Box<dyn QueryBody>,
}

pub fn build_select_junction(
    ctx: &mut SqliteQueryCtx,
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
        let (j_body_type, j_body_tokens) = j.body.build(ctx, &path, QueryResCount::Many);
        if j_body_type.0.len() != base_type.0.len() {
            ctx
                .errs
                .err(
                    &path,
                    format!(
                        "Select returns {} columns but the base select has {} columns and these must match exactly",
                        j_body_type.0.len(),
                        base_type.0.len()
                    ),
                );
            continue;
        }
        for (i, ((_, got), (_, want))) in Iterator::zip(j_body_type.0.iter(), base_type.0.iter()).enumerate() {
            let path = path.push_back(format!("Select return {}", i));
            check_assignable(&mut ctx.errs, &path, want, &ExprType(vec![(Binding::empty(), got.clone())]));
        }
        out.s(&j_body_tokens.to_string());
    }
    return out;
}
