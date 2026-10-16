use {
    crate::{
        mariadb::{
            QueryResCount,
            query::{
                expr::{
                    Expr,
                    ExprType,
                    ExprValName,
                    check_assignable,
                },
                select::Select,
                utils::{
                    MariadbQueryCtx,
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
    std::collections::{
        HashMap,
        HashSet,
    },
};

#[derive(Clone, Debug)]
pub struct Insert {
    pub ignore: bool,
    pub on_conflict: Option<InsertConflict>,
    pub returning: Vec<Returning>,
    pub source: InsertSource,
    pub table: TableRef,
}

impl QueryBody for Insert {
    fn build(
        &self,
        ctx: &mut MariadbQueryCtx,
        path: &rpds::Vector<String>,
        res_count: QueryResCount,
    ) -> (ExprType, Tokens) {
        let table_info = match ctx.tables.get(&self.table) {
            Some(t) => t.clone(),
            None => {
                ctx.errs.err(path, format!("Unknown table {:?} for insert", self.table));
                return (ExprType(vec![]), Tokens::new_mariadb());
            },
        };
        let mut scope = HashMap::new();
        for (field_ref, info) in &table_info.fields {
            scope.insert(ExprValName::field(field_ref), info.type_.clone());
        }
        let mut out = Tokens::new_mariadb();
        match self.on_conflict {
            Some(InsertConflict::Replace) => {
                if self.ignore {
                    ctx.errs.err(path, "`replace` can't be used with `ignore`".to_string());
                }
                out.s("replace");
            },
            Some(InsertConflict::DuplicateKeyUpdate(_)) | None => {
                out.s("insert");
                if self.ignore {
                    out.s("ignore");
                }
            },
        }
        out.s("into").id(&table_info.sql_name);
        let (columns, rows): (Vec<FieldRef>, Vec<Vec<Expr>>) = match &self.source {
            InsertSource::Values(values) => (
                values.iter().map(|(f, _)| f.clone()).collect(),
                vec![values.iter().map(|(_, e)| e.clone()).collect()],
            ),
            InsertSource::Rows(rows) => (rows.columns.clone(), rows.rows.clone()),
            InsertSource::Select { columns, select } => {
                let mut seen = HashSet::new();
                for field_ref in columns {
                    if !seen.insert(field_ref.clone()) {
                        ctx.errs.err(path, format!("Duplicate field {:?} in insert", field_ref));
                    }
                }
                for (field_ref, info) in &table_info.fields {
                    if !info.type_.opt && !info.has_default && !seen.contains(field_ref) {
                        ctx
                            .errs
                            .err(
                                path,
                                format!("Field {:?} is a non-optional field but is missing in insert", field_ref),
                            );
                    }
                }
                out.s("(");
                let mut good = true;
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
                            good = false;
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
                } else if good {
                    for (i, (_, res_col_type)) in res_type.0.iter().enumerate() {
                        let field_info = table_info.fields.get(&columns[i]).unwrap();
                        check_assignable(
                            &mut ctx.errs,
                            &path,
                            &field_info.type_,
                            &ExprType(vec![(ExprValName::empty(), res_col_type.clone())]),
                            if select.returning.len() == res_type.0.len() {
                                Some(&select.returning[i].e)
                            } else {
                                None
                            },
                        );
                    }
                }
                out.s(&res_tokens.to_string());
                (vec![], vec![])
            },
        };
        if !matches!(&self.source, InsertSource::Select { .. }) {
            let mut check_inserting_fields = HashSet::new();
            let mut column_infos = vec![];
            for field_ref in &columns {
                let field_info = match table_info.fields.get(field_ref) {
                    Some(f) => f,
                    None => {
                        ctx
                            .errs
                            .err(path, format!("Unknown field {:?} for insert into {:?}", field_ref, self.table));
                        continue;
                    },
                };
                column_infos.push((field_ref.clone(), field_info.clone()));
                if !check_inserting_fields.insert(field_ref.clone()) {
                    ctx.errs.err(path, format!("Duplicate field {:?} in insert", field_ref));
                }
            }
            for (field_ref, info) in &table_info.fields {
                if !info.type_.opt && !info.has_default && !check_inserting_fields.contains(field_ref) {
                    ctx
                        .errs
                        .err(path, format!("Field {:?} is a non-optional field but is missing in insert", field_ref));
                }
            }
            if column_infos.len() == columns.len() {
                out.s("(");
                for (i, (_, field_info)) in column_infos.iter().enumerate() {
                    if i > 0 {
                        out.s(",");
                    }
                    out.id(&field_info.sql_name);
                }
                out.s(") values");
                for (row_i, row) in rows.iter().enumerate() {
                    if row_i > 0 {
                        out.s(",");
                    }
                    if row.len() != column_infos.len() {
                        ctx
                            .errs
                            .err(
                                path,
                                format!(
                                    "Insert row {} has {} values but {} columns were specified",
                                    row_i,
                                    row.len(),
                                    column_infos.len()
                                ),
                            );
                        continue;
                    }
                    out.s("(");
                    for (i, ((field_ref, field_info), val)) in column_infos.iter().zip(row.iter()).enumerate() {
                        if i > 0 {
                            out.s(",");
                        }
                        let path = path.push_back(format!("Insert row {} value {} ({:?})", row_i, i, field_ref));
                        let res = val.build(ctx, &path, &scope);
                        check_assignable(&mut ctx.errs, &path, &field_info.type_, &res.0, Some(val));
                        out.s(&res.1.to_string());
                    }
                    out.s(")");
                }
            }
        }
        if let Some(InsertConflict::DuplicateKeyUpdate(set)) = &self.on_conflict {
            out.s("on duplicate key update");
            ctx.in_duplicate_key_update = true;
            build_set(ctx, path, &scope, &mut out, set, false);
            ctx.in_duplicate_key_update = false;
        }
        if !self.returning.is_empty() && matches!(res_count, QueryResCount::One) {
            if self.ignore {
                ctx
                    .errs
                    .err(
                        path,
                        "`insert ignore ... returning` returns no row when the insert is ignored, so the result must be optional".to_string(),
                    );
            }
            if rows.len() > 1 || matches!(self.source, InsertSource::Select { .. }) {
                ctx
                    .errs
                    .err(
                        path,
                        "This insert can return more than one row, so the result count can't be one".to_string(),
                    );
            }
        }
        if !self.returning.is_empty() && matches!(res_count, QueryResCount::MaybeOne) && rows.len() > 1 {
            ctx
                .errs
                .err(
                    path,
                    "This insert returns more than one row, so the result count can't be at most one".to_string(),
                );
        }
        let out_type = build_returning(ctx, path, &scope, &mut out, &self.returning, res_count);
        return (out_type, out);
    }
}

#[derive(Clone, Debug)]
pub enum InsertConflict {
    DuplicateKeyUpdate(Vec<(FieldRef, Expr)>),
    Replace,
}

#[derive(Clone, Debug)]
pub struct InsertRows {
    pub columns: Vec<FieldRef>,
    pub rows: Vec<Vec<Expr>>,
}

#[derive(Clone, Debug)]
pub enum InsertSource {
    Rows(InsertRows),
    Select {
        columns: Vec<FieldRef>,
        select: Select,
    },
    Values(Vec<(FieldRef, Expr)>),
}
