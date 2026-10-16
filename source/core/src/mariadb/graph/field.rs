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
            schema::constraint::ConstraintType,
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
        Node,
        NodeData,
        NodeDataDispatch,
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

fn add_auto_increment_key(ctx: &MariadbMigrateCtx, stmt: &mut Tokens, table_id: &str, sql_name: &str) {
    let table = ctx.version.tables.get(table_id).unwrap();
    let field_key = table.fields.iter().find(|(_, f)| f.id == sql_name).map(|(k, _)| k.clone()).unwrap();
    let mut key = None;
    for c in table.constraints.values() {
        if let ConstraintType::PrimaryKey(x) = &c.type_ {
            if x.fields.first() == Some(&field_key) {
                key = Some(&x.fields);
                stmt.s(", drop index if exists").id("PRIMARY").s(", add primary key");
            }
        }
    }
    if key.is_none() {
        for i in table.indices.values() {
            if i.fields.first() == Some(&field_key) {
                key = Some(&i.fields);
                stmt.s(if i.unique {
                    ", add unique index if not exists"
                } else {
                    ", add index if not exists"
                }).id(&i.id);
                break;
            }
        }
    }
    let Some(fields) = key else {
        return;
    };
    stmt.s("(");
    for (i, f) in fields.iter().enumerate() {
        if i > 0 {
            stmt.s(",");
        }
        stmt.id(&table.fields.get(f).unwrap().id);
    }
    stmt.s(")");
}

impl NodeData for NodeField_ {
    fn update(&self, ctx: &mut MariadbMigrateCtx, old: &Self) {
        if self.def.id != old.def.id {
            let mut stmt = Tokens::new_mariadb();
            stmt
                .s("alter table")
                .id(&self.table_id)
                .s("rename column if exists")
                .id(&old.def.id)
                .s("to")
                .id(&self.def.id);
            ctx.statements.push(stmt.to_string());
        }
        if self.column_changed(old) {
            let definition = column_definition(ctx, &self.table_id, &self.def);
            let mut stmt = Tokens::new_mariadb();
            stmt.s("alter table").id(&self.table_id).s("modify column if exists").id(&self.def.id).s(&definition);
            if self.def.type_.auto_increment && !old.def.type_.auto_increment {
                add_auto_increment_key(ctx, &mut stmt, &self.table_id, &self.def.id);
            }
            ctx.statements.push(stmt.to_string());
        }
    }
}

impl NodeDataDispatch for NodeField_ {
    fn create(&self, ctx: &mut MariadbMigrateCtx) {
        let definition = column_definition(ctx, &self.table_id, &self.def);
        let mut stmt = Tokens::new_mariadb();
        stmt.s("alter table").id(&self.table_id).s("add column if not exists").id(&self.def.id).s(&definition);
        if self.def.type_.auto_increment {
            add_auto_increment_key(ctx, &mut stmt, &self.table_id, &self.def.id);
        }
        ctx.statements.push(stmt.to_string());
    }

    fn create_coalesce(&mut self, other: Node) -> Option<Node> {
        return Some(other);
    }

    fn delete(&self, ctx: &mut MariadbMigrateCtx) {
        ctx
            .statements
            .push(
                Tokens::new_mariadb()
                    .s("alter table")
                    .id(&self.table_id)
                    .s("drop column if exists")
                    .id(&self.def.id)
                    .to_string(),
            );
    }

    fn delete_coalesce(&mut self, other: Node) -> Option<Node> {
        return Some(other);
    }
}
