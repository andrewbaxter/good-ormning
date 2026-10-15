use {
    crate::{
        pg::{
            QueryResCount,
            query::{
                select::NamedSelectSource,
                expr::{
                    Expr,
                    ExprType,
                    ExprValName,
                    check_bool,
                },
                utils::{
                    PgQueryCtx,
                    QueryBody,
                    Returning,
                    build_returning,
                    build_set,
                },
            },
            schema::{
                field::FieldRef,
                table::TableRef,
            },
        },
        utils::Tokens,
    },
    std::collections::HashMap,
};

#[derive(Clone, Debug)]
pub struct Update {
    /// Alias for the updated table, for use in `from` joins and `where`
    pub alias: Option<String>,
    /// Additional tables (`update ... from ...`) whose fields can be used in `set` and
    /// `where`
    pub from: Vec<NamedSelectSource>,
    pub returning: Vec<Returning>,
    pub table: TableRef,
    pub values: Vec<(FieldRef, Expr)>,
    pub where_: Option<Expr>,
}

impl QueryBody for Update {
    fn build(
        &self,
        ctx: &mut PgQueryCtx,
        path: &rpds::Vector<String>,
        res_count: QueryResCount,
    ) -> (ExprType, Tokens) {
        // Prep
        let table_info = match ctx.tables.get(&self.table) {
            Some(t) => t.clone(),
            None => {
                ctx.errs.err(path, format!("Unknown table {:?} for update", self.table));
                return (ExprType(vec![]), Tokens::new());
            },
        };
        let mut scope = HashMap::new();
        for (k, info) in &table_info.fields {
            let mut name = ExprValName::field(k);
            if let Some(alias) = &self.alias {
                name = name.with_alias(alias);
            }
            scope.insert(name, info.type_.clone());
        }
        if let Some(alias) = &self.alias {
            ctx.table_aliases.insert(alias.clone(), self.table.clone());
        }
        let mut from_tokens = vec![];
        for (i, f) in self.from.iter().enumerate() {
            let (fields, tokens) = f.build(ctx, &path.push_back(format!("From {}", i)));
            for (k, v) in fields.0 {
                scope.insert(k, v);
            }
            from_tokens.push(tokens);
        }

        // Build query
        let mut out = Tokens::new();
        out.s("update").id(&table_info.sql_name);
        if let Some(alias) = &self.alias {
            out.s("as").id(alias);
        }
        build_set(ctx, path, &scope, &mut out, &self.values);
        if !from_tokens.is_empty() {
            out.s("from");
            for (i, t) in from_tokens.iter().enumerate() {
                if i > 0 {
                    out.s(",");
                }
                out.s(&t.to_string());
            }
        }
        if let Some(where_) = &self.where_ {
            out.s("where");
            let path = path.push_back("Where".into());
            let (where_t, where_tokens) = where_.build(ctx, &path, &scope);
            check_bool(ctx, &path, &where_t);
            out.s(&where_tokens.to_string());
        }
        let out_type = build_returning(ctx, path, &scope, &mut out, &self.returning, res_count);
        return (out_type, out);
    }
}
