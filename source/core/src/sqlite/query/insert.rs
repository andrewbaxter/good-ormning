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
        match &self.on_conflict {
            Some(InsertConflict::OrIgnore) => {
                out.s("or ignore");
            },
            Some(InsertConflict::OrReplace) => {
                out.s("or replace");
            },
            Some(InsertConflict::OrAbort) => {
                out.s("or abort");
            },
            Some(InsertConflict::OrFail) => {
                out.s("or fail");
            },
            Some(InsertConflict::OrRollback) => {
                out.s("or rollback");
            },
            _ => { },
        }
        out.s("into").id(&table_info.sql_name);
        let (columns, rows): (Vec<FieldRef>, Vec<Vec<Expr>>) = match &self.source {
            InsertSource::Values(values) => (
                values.iter().map(|(f, _)| f.clone()).collect(),
                vec![values.iter().map(|(_, e)| e.clone()).collect()],
            ),
            InsertSource::Rows(rows) => (rows.columns.clone(), rows.rows.clone()),
            InsertSource::Select { columns, select } => {
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
                if !info.type_.opt && info.type_.type_.type_ != SimpleSimpleType::Auto &&
                    !check_inserting_fields.contains(field_ref) {
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
                        check_assignable(&mut ctx.errs, &path, &field_info.type_, &res.0);
                        out.s(&res.1.to_string());
                    }
                    out.s(")");
                }
            }
        }
        if let Some(conflict) = &self.on_conflict {
            let empty = vec![];
            let (conflict, set) = match conflict {
                InsertConflict::OrIgnore |
                InsertConflict::OrReplace |
                InsertConflict::OrAbort |
                InsertConflict::OrFail |
                InsertConflict::OrRollback => (
                    None,
                    None,
                ),
                InsertConflict::DoNothing => (Some(&empty), None),
                InsertConflict::DoNothingOn { conflict } => (Some(conflict), None),
                InsertConflict::DoUpdate { conflict, set } => (Some(conflict), Some(set)),
            };
            if let Some(conflict) = conflict {
                {
                    out.s("on conflict");
                    if !conflict.is_empty() {
                        out.s("(");
                        for (i, field_ref) in conflict.iter().enumerate() {
                            if i > 0 {
                                out.s(",");
                            }
                            match table_info.fields.get(field_ref) {
                                Some(field_info) => {
                                    out.id(&field_info.sql_name);
                                },
                                None => {
                                    ctx
                                        .errs
                                        .err(
                                            path,
                                            format!(
                                                "Unknown field {:?} in conflict target for insert into {:?}",
                                                field_ref,
                                                self.table
                                            ),
                                        );
                                },
                            }
                        }
                        out.s(")");
                    }
                    match set {
                        Some(set) => {
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
                        None => {
                            out.s("do nothing");
                        },
                    }
                }
            }
        }
        let out_type = build_returning(ctx, path, &scope, &mut out, &self.returning, res_count);
        return (out_type, out);
    }
}

#[derive(Clone, Debug)]
pub enum InsertConflict {
    /// `on conflict do nothing`
    DoNothing,
    /// `on conflict (fields) do nothing`
    DoNothingOn {
        conflict: Vec<FieldRef>,
    },
    /// `insert or ignore` - note this also ignores other constraint violations (not
    /// null, check)
    OrIgnore,
    /// `insert or replace`
    OrReplace,
    /// `insert or abort`
    OrAbort,
    /// `insert or fail`
    OrFail,
    /// `insert or rollback`
    OrRollback,
    DoUpdate {
        conflict: Vec<FieldRef>,
        set: Vec<(FieldRef, Expr)>,
    },
}

/// Multiple rows of values, each row having one value per column.
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
