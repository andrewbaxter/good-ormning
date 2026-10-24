use {
    crate::{
        graphmigrate::Comparison,
        mariadb::{
            MariadbFieldInfo,
            MariadbQueryCtx,
            MariadbTableInfo,
            query::expr::{
                Expr,
                check_assignable,
            },
            schema::{
                field::{
                    Field,
                    FieldRef,
                },
                table::TableRef,
            },
            types::to_sql_type,
        },
        utils::Tokens,
    },
    std::collections::{
        HashMap,
        HashSet,
    },
    super::{
        GraphId,
        utils::MariadbMigrateCtx,
    },
};

#[derive(Clone)]
pub struct NodeField_ {
    pub def: Field,
    pub table_id: String,
    pub table_renamed_from: Option<String>,
}

impl NodeField_ {
    pub fn compare(&self, old: &Self, created: &HashSet<GraphId>) -> Comparison {
        if created.contains(&GraphId::Table(self.table_id.clone())) {
            return Comparison::Recreate;
        }
        return if self.def.id != old.def.id || self.table_id != old.table_id || self.column_changed(old) {
            Comparison::Update
        } else {
            Comparison::DoNothing
        };
    }
}

impl NodeField_ {
    fn column_changed(&self, old: &Self) -> bool {
        let t = &self.def.type_;
        let old_t = &old.def.type_;
        return t.type_.opt != old_t.type_.opt || t.type_.type_.type_ != old_t.type_.type_.type_ ||
            t.auto_increment != old_t.auto_increment ||
            t.default != old_t.default ||
            t.max_len != old_t.max_len ||
            t.precision != old_t.precision ||
            t.scale != old_t.scale ||
            t.variant != old_t.variant;
    }
}

pub fn column_definition(ctx: &mut MariadbMigrateCtx, table_id: &str, f: &Field) -> String {
    let path = rpds::vector![format!("{}.{}", table_id, f.id)];
    let mut out = Tokens::new_mariadb();
    out.s(&to_sql_type(&f.type_));
    if f.type_.type_.opt {
        out.s("null");
    } else {
        out.s("not null");
    }
    if let Some(d) = &f.type_.default {
        let mut qctx_tables = HashMap::new();
        let mut fields = HashMap::new();
        fields.insert(FieldRef {
            table_id: table_id.to_string(),
            field_id: f.id.clone(),
        }, MariadbFieldInfo {
            sql_name: f.id.clone(),
            type_: f.type_.type_.clone(),
            has_default: true,
        });
        qctx_tables.insert(TableRef(table_id.to_string()), MariadbTableInfo {
            sql_name: table_id.to_string(),
            fields: fields,
        });
        let mut qctx = MariadbQueryCtx::new(ctx.errs.clone(), qctx_tables);
        let expr: Expr = Expr::from(d.clone());
        let e_res = expr.build(&mut qctx, &path, &HashMap::new());
        check_assignable(&mut qctx.errs, &path, &f.type_.type_, &e_res.0, Some(&expr));
        if !qctx.rust_args.is_empty() {
            qctx
                .errs
                .err(
                    &path,
                    format!(
                        "Default expressions must not have any parameters, but this has {} parameters",
                        qctx.rust_args.len()
                    ),
                );
        }
        out.s("default (").s(&e_res.1.to_string()).s(")");
    }
    return out.to_string();
}

impl NodeField_ {
    pub fn create_clauses(&self, ctx: &mut MariadbMigrateCtx) -> Vec<Vec<String>> {
        let definition = column_definition(ctx, &self.table_id, &self.def);
        let mut clause = Tokens::new_mariadb();
        clause.s("add column if not exists").id(&self.def.id).s(&definition);
        return vec![vec![clause.to_string()]];
    }

    pub fn delete_clauses(&self) -> Vec<Vec<String>> {
        let mut clause = Tokens::new_mariadb();
        clause.s("drop column if exists").id(&self.def.id);
        return vec![vec![clause.to_string()]];
    }

    pub fn update_clauses(&self, ctx: &mut MariadbMigrateCtx, old: &Self, merged: bool) -> Vec<Vec<String>> {
        let renamed = self.def.id != old.def.id;
        let changed = self.column_changed(old);
        let mut out = vec![];
        if merged && renamed && changed {
            let definition = column_definition(ctx, &self.table_id, &self.def);
            let mut clause = Tokens::new_mariadb();
            clause.s("change column if exists").id(&old.def.id).id(&self.def.id).s(&definition);
            out.push(vec![clause.to_string()]);
            return out;
        }
        if renamed {
            let mut clause = Tokens::new_mariadb();
            clause.s("rename column if exists").id(&old.def.id).s("to").id(&self.def.id);
            out.push(vec![clause.to_string()]);
        }
        if changed {
            let definition = column_definition(ctx, &self.table_id, &self.def);
            let mut clause = Tokens::new_mariadb();
            clause.s("modify column if exists").id(&self.def.id).s(&definition);
            out.push(vec![clause.to_string()]);
        }
        return out;
    }
}
