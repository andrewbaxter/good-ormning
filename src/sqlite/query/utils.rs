use {
    std::{
        collections::{
            HashMap,
            HashSet,
        },
        rc::Rc,
    },
    proc_macro2::TokenStream,
    crate::{
        sqlite::{
            schema::{
                field::{
                    Field,
                    FieldType,
                    Field_,
                    SchemaFieldId,
                },
                table::{
                    SchemaTableId,
                    Table,
                    Table_,
                },
            },
            types::{
                Type,
            },
            QueryResCount,
        },
        utils::{
            Errs,
            Tokens,
        },
    },
    super::{
        expr::{
            check_assignable,
            Expr,
            ExprType,
            Binding,
        },
        select_body::{
            Returning,
            SelectBody,
            SelectJunction,
            SelectJunctionOperator,
        },
    },
};

pub struct SqliteQueryCtx {
    pub(crate) tables: HashMap<Table, HashSet<Field>>,
    pub errs: Errs,
    pub(crate) rust_arg_lookup: HashMap<String, (usize, Type)>,
    pub(crate) rust_args: Vec<TokenStream>,
    pub(crate) query_args: Vec<TokenStream>,
}

impl<'a> SqliteQueryCtx {
    pub(crate) fn new(errs: Errs, tables: HashMap<Table, HashSet<Field>>) -> Self {
        Self {
            tables: tables,
            errs: errs,
            rust_arg_lookup: Default::default(),
            rust_args: Default::default(),
            query_args: Default::default(),
        }
    }
}

pub trait QueryBody {
    fn build(
        &self,
        ctx: &mut SqliteQueryCtx,
        path: &rpds::Vector<String>,
        res_count: QueryResCount,
    ) -> (ExprType, Tokens);
}

pub fn build_set(
    ctx: &mut SqliteQueryCtx,
    path: &rpds::Vector<String>,
    scope: &HashMap<Binding, Type>,
    out: &mut Tokens,
    values: &Vec<(Field, Expr)>,
) {
    out.s("set");
    for (i, (field, val)) in values.iter().enumerate() {
        let path = path.push_back(format!("Set field {}", i));
        if i > 0 {
            out.s(",");
        }
        out.id(&field.id).s("=");
        let res = val.build(ctx, &path, &scope);
        let field = match ctx.tables.get(&field.table).and_then(|t| t.get(&field)) {
            Some(t) => t,
            None => {
                ctx.errs.err(&path, format!("Update destination value field {} is not known", field));
                continue;
            },
        };
        check_assignable(&mut ctx.errs, &path, &field.type_.type_, &res.0);
        out.s(&res.1.to_string());
    }
}

pub fn build_returning_values(
    ctx: &mut SqliteQueryCtx,
    path: &rpds::Vector<String>,
    scope: &HashMap<Binding, Type>,
    out: &mut Tokens,
    outputs: &Vec<Returning>,
    res_count: QueryResCount,
) -> ExprType {
    if outputs.is_empty() {
        if !matches!(res_count, QueryResCount::None) {
            ctx.errs.err(path, format!("Query has no outputs but res_count is, {:?}, not None", res_count));
        }
    } else {
        if matches!(res_count, QueryResCount::None) {
            ctx.errs.err(&path, format!("Query has outputs so res_count must be not None, but is {:?}", res_count));
        }
    }
    let mut out_rec: Vec<(Binding, Type)> = vec![];
    for (i, o) in outputs.iter().enumerate() {
        let path = path.push_back(format!("Result {}", i));
        if i > 0 {
            out.s(",");
        }
        let res = o.e.build(ctx, &path, scope);
        out.s(&res.1.to_string());
        let (res_name, res_type) = match res.0.assert_scalar(&mut ctx.errs, &path) {
            Some(x) => x,
            None => continue,
        };
        if let Some(rename) = &o.rename {
            out.s("as").id(rename);
            out_rec.push((Binding::local(rename.clone()), res_type));
        } else {
            out_rec.push((res_name, res_type));
        }
    }
    ExprType(out_rec)
}

pub fn build_returning(
    ctx: &mut SqliteQueryCtx,
    path: &rpds::Vector<String>,
    scope: &HashMap<Binding, Type>,
    out: &mut Tokens,
    outputs: &Vec<Returning>,
    res_count: QueryResCount,
) -> ExprType {
    if !outputs.is_empty() {
        out.s("returning");
    }
    build_returning_values(ctx, path, scope, out, outputs, res_count)
}

#[derive(Clone, Debug)]
pub struct With {
    pub recursive: bool,
    pub ctes: Vec<Cte>,
}

#[derive(Clone, Debug)]
pub struct Cte {
    pub table: Table,
    pub columns: Vec<Field>,
    pub body: SelectBody,
    pub body_junctions: Vec<SelectJunction>,
}

pub struct CteBuilder {
    table: Table,
    cte: Cte,
}

impl CteBuilder {
    pub fn new(id: impl AsRef<str>, body: SelectBody) -> Self {
        let table = Table(Rc::new(Table_ {
            schema_id: SchemaTableId("".to_string()),
            id: id.as_ref().to_string(),
        }));
        return Self {
            table: table.clone(),
            cte: Cte {
                table: table,
                columns: vec![],
                body: body,
                body_junctions: vec![],
            },
        };
    }

    pub fn body_junction(&mut self, j: SelectJunction) {
        self.cte.body_junctions.push(j);
    }

    pub fn field(&mut self, id: impl AsRef<str>, type_: Type) -> Field {
        let f = Field(Rc::new(Field_ {
            table: self.table.clone(),
            schema_id: SchemaFieldId(id.as_ref().to_string()),
            id: id.as_ref().to_string(),
            type_: FieldType {
                type_: type_,
                migration_default: None,
            },
        }));
        if self.cte.columns.contains(&f) {
            panic!("Duplicate field {} in CTE definition", id.as_ref());
        }
        self.cte.columns.push(f.clone());
        return f;
    }

    pub fn build(self) -> (Table, Cte) {
        return (self.table, self.cte);
    }
}

pub fn build_with(ctx: &mut SqliteQueryCtx, path: &rpds::Vector<String>, with: &With) -> Tokens {
    let mut out = Tokens::new();
    out.s("with");
    for (i, cte) in with.ctes.iter().enumerate() {
        if i > 0 {
            out.s(",");
        }
        let path = path.push_back(format!("CTE {}", i));
        out.s(&cte.table.id);
        out.s("(");
        for (i, c) in cte.columns.iter().enumerate() {
            if i > 0 {
                out.s(",");
            }
            out.s(&c.id);
        }
        out.s(")");
        out.s("as");
        out.s("(");
        let body = cte.body.build(ctx, &HashMap::new(), &path, QueryResCount::Many);
        for (i, ((_, got), want)) in Iterator::zip(body.0.0.iter(), cte.columns.iter()).enumerate() {
            let path = path.push_back(format!("Select return {}", i));
            check_assignable(
                &mut ctx.errs,
                &path,
                &want.type_.type_,
                &ExprType(vec![(Binding::empty(), got.clone())]),
            );
        }
        out.s(&body.1.to_string());
        if body.0.0.len() != cte.columns.len() {
            ctx
                .errs
                .err(
                    &path,
                    format!(
                        "Select returns {} columns but the CTE needs exactly {} columns",
                        body.0.0.len(),
                        cte.columns.len()
                    ),
                );
            continue;
        }
        ctx.tables.insert(cte.table.clone(), cte.columns.iter().cloned().collect());
        for (i, j) in cte.body_junctions.iter().enumerate() {
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
            if j_body.0.0.len() != cte.columns.len() {
                ctx
                    .errs
                    .err(
                        &path,
                        format!(
                            "Select returns {} columns but the CTE needs exactly {} columns",
                            j_body.0.0.len(),
                            cte.columns.len()
                        ),
                    );
                continue;
            }
            for (i, ((_, got), want)) in Iterator::zip(j_body.0.0.iter(), cte.columns.iter()).enumerate() {
                let path = path.push_back(format!("Select return {}", i));
                check_assignable(
                    &mut ctx.errs,
                    &path,
                    &want.type_.type_,
                    &ExprType(vec![(Binding::empty(), got.clone())]),
                );
            }
        }
        out.s(")");
    }
    return out;
}
