use {
    crate::{
        sqlite::{
            QueryResCount,
            query::{
                expr::{
                    Binding,
                    Expr,
                    ExprType,
                    check_bool,
                    check_general_same,
                },
                select_body::{
                    SelectJunction,
                    build_select_junction,
                },
                utils::{
                    QueryBody,
                    Returning,
                    SqliteQueryCtx,
                    With,
                    build_returning_values,
                    build_with,
                },
            },
            schema::table::TableRef,
            types::{
                Type,
                type_i64,
            },
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
pub enum IndexHint {
    IndexedBy(String),
    NotIndexed,
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
    pub index_hint: Option<IndexHint>,
}

impl NamedSelectSource {
    pub fn build(&self, ctx: &mut SqliteQueryCtx, path: &rpds::Vector<String>) -> (ExprType, Tokens) {
        let mut out = Tokens::new();
        let mut new_fields: Vec<(Binding, Type)> = match &self.source {
            JoinSource::Subsel(s) => {
                let res: (ExprType, Tokens) =
                    s.build(ctx, &path.push_back("From subselect".to_string()), QueryResCount::Many);
                out.s("(").s(&res.1.to_string()).s(")");
                res.0.0.clone()
            },
            JoinSource::Table(s) => {
                let table_info = match ctx.tables.get(s) {
                    Some(f) => f,
                    None => {
                        ctx
                            .errs
                            .err(&path.push_back("From".to_string()), format!("No table with id {:?} in version", s));
                        return (ExprType(vec![]), Tokens::new());
                    },
                };
                out.id(&table_info.sql_name);
                if let Some(hint) = &self.index_hint {
                    match hint {
                        IndexHint::IndexedBy(name) => {
                            out.s("indexed by").id(name);
                        },
                        IndexHint::NotIndexed => {
                            out.s("not indexed");
                        },
                    }
                }
                table_info.fields.iter().map(|(id, info)| (Binding::field(id), info.type_.clone())).collect()
            },
            JoinSource::Func(name, args) => {
                if name == "__good_ormning_rarray" {
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 {
                            out.s(",");
                        }
                        let (_, tokens) = arg.build(ctx, &path.push_back(format!("Arg {}", i)), &HashMap::new());
                        out.s(&tokens.to_string());
                    }
                } else {
                    out.s(name).s("(");
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 {
                            out.s(",");
                        }
                        let (_, tokens) = arg.build(ctx, &path.push_back(format!("Arg {}", i)), &HashMap::new());
                        out.s(&tokens.to_string());
                    }
                    out.s(")");
                }
                if name == "rarray" || name == "__good_ormning_rarray" {
                    // Find the type of the argument For now, default to i32 for simplicity or try to
                    // infer? Better to just return a dummy i32 if we can't infer.
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
        return (ExprType(new_fields), out);
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
    pub having: Option<Expr>,
    pub order: Vec<(Expr, Order)>,
    pub limit: Option<Expr>,
    pub distinct: bool,
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
        if self.distinct {
            out.s("distinct");
        }
        if self.returning.is_empty() {
            ctx.errs.err(path, "Select must have at least one output, but outputs are empty".to_string());
        }
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
        if !matches!(self.table.source, JoinSource::Empty) {
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
                let (on_t, on_tokens): (ExprType, Tokens) = j.on.build(ctx, &path.push_back("On".to_string()), &scope);
                check_bool(ctx, &path, &on_t);
                out.s(&on_tokens.to_string());
            }
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
        if let Some(having) = &self.having {
            out.s("having");
            let path = path.push_back("Having".into());
            let (having_t, having_tokens): (ExprType, Tokens) = having.build(ctx, &path, &scope);
            check_bool(ctx, &path, &having_t);
            out.s(&having_tokens.to_string());
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
        return (out_type, out);
    }
}
