use {
    crate::{
        sqlite::{
            QueryResCount,
            query::{
                expr::{
                    Binding,
                    Expr,
                    ExprType,
                    check_assignable,
                },
                select::Select,
                utils::{
                    QueryBody,
                    Returning,
                    SqliteQueryCtx,
                    build_returning,
                    build_set,
                },
            },
            schema::{
                field::FieldRef,
                table::TableRef,
            },
            types::SimpleSimpleType,
        },
        utils::Tokens,
    },
    std::collections::{
        HashMap,
        HashSet,
    },
};

#[derive(Clone, Debug)]
pub struct Insert {
    pub on_conflict: Option<InsertConflict>,
    pub returning: Vec<Returning>,
    pub source: InsertSource,
    pub table: TableRef,
}

impl QueryBody for Insert {
    fn build(
        &self,
        ctx: &mut SqliteQueryCtx,
        path: &rpds::Vector<String>,
        res_count: QueryResCount,
    ) -> (ExprType, Tokens) {
        // Prep
        let table_info = match ctx.tables.get(&self.table) {
            Some(t) => t.clone(),
            None => {
                ctx.errs.err(path, format!("Unknown table {:?} for insert", self.table));
                return (ExprType(vec![]), Tokens::new());
            },
        };
        let mut scope = HashMap::new();
        for (field_ref, info) in &table_info.fields {
            scope.insert(Binding::field(field_ref), info.type_.clone());
        }
        let mut out = Tokens::new();
        out.s("insert");
        if let Some(InsertConflict::DoNothing) = &self.on_conflict {
            out.s("or ignore");
        }
        out.s("into").id(&table_info.sql_name);
        match &self.source {
            InsertSource::Values(values) => {
                let mut check_inserting_fields = HashSet::new();
                for p in values {
                    let field_info = match table_info.fields.get(&p.0) {
                        Some(f) => f,
                        None => {
                            ctx.errs.err(path, format!("Unknown field {:?} for insert into {:?}", p.0, self.table));
                            continue;
                        },
                    };
                    if field_info.type_.opt {
                        continue;
                    }
                    if !check_inserting_fields.insert(p.0.clone()) {
                        ctx.errs.err(path, format!("Duplicate field {:?} in insert", p.0));
                    }
                }
                for (field_ref, info) in &table_info.fields {
                    if !info.type_.opt && info.type_.type_.type_ != SimpleSimpleType::Auto &&
                        !check_inserting_fields.remove(field_ref) {
                        ctx
                            .errs
                            .err(
                                path,
                                format!("Field {:?} is a non-optional field but is missing in insert", field_ref),
                            );
                    }
                }
                out.s("(");
                for (i, (field_ref, _)) in values.iter().enumerate() {
                    if i > 0 {
                        out.s(",");
                    }
                    let field_info = table_info.fields.get(field_ref).unwrap().clone();
                    out.id(&field_info.sql_name);
                }
                out.s(") values (");
                for (i, (field_ref, val)) in values.iter().enumerate() {
                    if i > 0 {
                        out.s(",");
                    }
                    let field_info = table_info.fields.get(field_ref).unwrap().clone();
                    let path = path.push_back(format!("Insert value {} ({:?})", i, field_ref));
                    let res = val.build(ctx, &path, &scope);
                    check_assignable(&mut ctx.errs, &path, &field_info.type_, &res.0);
                    out.s(&res.1.to_string());
                }
                out.s(")");
            },
            InsertSource::Select { columns, select } => {
                out.s("(");
                for (i, field_ref) in columns.iter().enumerate() {
                    if i > 0 {
                        out.s(",");
                    }
                    let field_info = match table_info.fields.get(field_ref) {
                        Some(f) => f,
                        None => {
                            ctx
                                .errs
                                .err(path, format!("Unknown field {:?} for insert into {:?}", field_ref, self.table));
                            continue;
                        },
                    };
                    out.id(&field_info.sql_name);
                }
                out.s(")");
                let (res_type, res_tokens): (ExprType, Tokens) =
                    select.build(ctx, &path.push_back("Insert select".to_string()), QueryResCount::Many);
                if res_type.0.len() != columns.len() {
                    ctx
                        .errs
                        .err(
                            path,
                            format!(
                                "Insert select returns {} columns but {} columns were specified",
                                res_type.0.len(),
                                columns.len()
                            ),
                        );
                } else {
                    for (i, (_, res_col_type)) in res_type.0.iter().enumerate() {
                        let field_info = table_info.fields.get(&columns[i]).unwrap();
                        let src_type: crate::sqlite::types::Type = res_col_type.clone();
                        check_assignable(
                            &mut ctx.errs,
                            &path,
                            &field_info.type_,
                            &ExprType(vec![(Binding::empty(), src_type)]),
                        );
                    }
                }
                out.s(&res_tokens.to_string());
            },
        }
        if let Some(conflict) = &self.on_conflict {
            match conflict {
                InsertConflict::DoNothing => (),
                InsertConflict::DoUpdate { conflict, set } => {
                    out.s("on conflict");
                    if !conflict.is_empty() {
                        out.s("(");
                        for (i, field_ref) in conflict.iter().enumerate() {
                            if i > 0 {
                                out.s(",");
                            }
                            let field_info = table_info.fields.get(field_ref).unwrap();
                            out.id(&field_info.sql_name);
                        }
                        out.s(")");
                    }
                    out.s("do update");
                    let mut set_scope = scope.clone();
                    for (field_ref, info) in &table_info.fields {
                        set_scope.insert(Binding {
                            table_id: "excluded".into(),
                            id: field_ref.field_id.clone(),
                        }, info.type_.clone());
                    }
                    ctx.table_aliases.insert("excluded".to_string(), self.table.clone());
                    build_set(ctx, path, &set_scope, &mut out, set);
                },
            }
        }
        let out_type = build_returning(ctx, path, &scope, &mut out, &self.returning, res_count);
        return (out_type, out);
    }
}

#[derive(Clone, Debug)]
pub enum InsertConflict {
    DoNothing,
    DoUpdate {
        conflict: Vec<FieldRef>,
        set: Vec<(FieldRef, Expr)>,
    },
}

#[derive(Clone, Debug)]
pub enum InsertSource {
    Select {
        columns: Vec<FieldRef>,
        select: Select,
    },
    Values(Vec<(FieldRef, Expr)>),
}
