use good_ormning::pg::Version as PgVersion;
use good_ormning::sqlite::Version as SqliteVersion;
use good_ormning::{QueryResCount, sqlite_type_i32 as type_i32, sqlite_type_i64 as type_i64, sqlite_type_u32 as type_u32, sqlite_type_f32 as type_f32, sqlite_type_f64 as type_f64, sqlite_type_bool as type_bool, sqlite_type_bytes as type_bytes, sqlite_type_str as type_str, sqlite_type_utctime_s_chrono as type_utctime_s_chrono, sqlite_type_utctime_s_jiff as type_utctime_s_jiff, SqliteType as Type, SqliteFieldTypeBuilder as FieldTypeBuilder};
use good_ormning::sqlite::VersionHandle;
use {
    std::path::Path,
    std::rc::Rc,
    good_ormning::{
        sqlite::{
            generate,
            new_delete,
            new_insert,
            new_select,
            new_select_body,
            new_update,
            query::{
                expr::{
                    BinOp,
                    Binding,
                    ComputeType,
                    Expr,
                    ExprType,
                },
                helpers::{
                    set_field,
                },
                select::{
                    Join,
                    NamedSelectSource,
                    JoinSource,
                    JoinType,
                    Order,
                },
                select_body::{
                    SelectJunction,
                },
                utils::{
                    CteBuilder,
                    With,
                },
            },
            schema::{
                field::{
                    field_bool,
                    field_bytes,
                    field_f32,
                    field_f64,
                    field_i32,
                    field_i64,
                    field_u32,
                    field_str,
                    field_utctime_ms_chrono,
                    field_utctime_s_chrono,
                    field_utctime_ms_jiff,
                    field_utctime_s_jiff,
                },
            },
            TableHandle,
            FieldHandle,
        },
    },
    flowcontrol::shed,
};

fn get_type(f: &FieldHandle) -> Type {
    let version = f.table.version.0.borrow();
    version
        .as_ref()
        .unwrap()
        .tables
        .get(&f.table.id)
        .unwrap()
        .fields
        .get(&f.id)
        .unwrap()
        .type_
        .type_
        .clone()
}
pub fn build() {
    // # Hello world example
    {
        let v = SqliteVersion::new();
        let users = v.table("hello_world_users");
        let id = users.rowid_field(None);
        let name = users.field("name", field_str().build());
        let points = users.field("points", field_i64().build());
        generate(Some("sqlite_gen_hello_world"), vec![
            // Versions
            (1usize, v.build())
        ], vec![]).unwrap();
    }

    // # Base: create table, insert, select
    {
        let v = SqliteVersion::new();
        let bananna = v.table("base_insert_bannanana");
        let hizat = bananna.field("hizat", field_str().build());
        generate(Some("sqlite_gen_base_insert"), vec![(1usize, v.build())], vec![]).unwrap();
    }

    // # Primary key
    {
        let v = SqliteVersion::new();
        let bananna = v.table("constraint_bannanana");
        let hizat = bananna.field("hizat", field_str().build());
        bananna.primary_key("hizat_pk", &[&hizat]);
        generate(Some("sqlite_gen_constraint"), vec![(1usize, v.build())], vec![]).unwrap();
    }

    // # (insert) Param: i32
    {
        let v = SqliteVersion::new();
        let bananna = v.table("param_i32_bananna");
        let hizat = bananna.field("hizat", field_i32().build());
        generate(Some("sqlite_gen_param_i32"), vec![(1usize, v.build())], vec![]).unwrap();
    }

    // # (insert) Param: datetime (seconds) (chrono)
    {
        let v = SqliteVersion::new();
        let bananna = v.table("param_utctime_s_chrono_bananna");
        let hizat = bananna.field("hizat", field_utctime_s_chrono().build());
        generate(Some("sqlite_gen_param_utctime_s_chrono"), vec![(1usize, v.build())], vec![]).unwrap();
    }

    // # (insert) Param: datetime (ms) (chrono)
    {
        let v = SqliteVersion::new();
        let bananna = v.table("param_utctime_ms_chrono_bananna");
        let hizat = bananna.field("hizat", field_utctime_ms_chrono().build());
        generate(Some("sqlite_gen_param_utctime_ms_chrono"), vec![(1usize, v.build())], vec![]).unwrap();
    }

    // # (insert) Param: datetime (seconds) (jiff)
    {
        let v = SqliteVersion::new();
        let bananna = v.table("param_utctime_s_jiff_bananna");
        let hizat = bananna.field("hizat", field_utctime_s_jiff().build());
        generate(Some("sqlite_gen_param_utctime_s_jiff"), vec![(1usize, v.build())], vec![]).unwrap();
    }

    // # (insert) Param: datetime (ms) (jiff)
    {
        let v = SqliteVersion::new();
        let bananna = v.table("param_utctime_ms_jiff_bananna");
        let hizat = bananna.field("hizat", field_utctime_ms_jiff().build());
        generate(Some("sqlite_gen_param_utctime_ms_jiff"), vec![(1usize, v.build())], vec![]).unwrap();
    }

    // # (insert) Param: Opt`<i32>`
    {
        let v = SqliteVersion::new();
        let bananna = v.table("param_opt_i32_bananna");
        let hizat = bananna.field("hizat", field_i32().opt().build());
        generate(Some("sqlite_gen_param_opt_i32"), vec![(1usize, v.build())], vec![]).unwrap();
    }

    // # (insert) Param: Opt`<i32>`, null
    {
        let v = SqliteVersion::new();
        let bananna = v.table("param_opt_i32_null_bananna");
        let hizat = bananna.field("hizat", field_i32().opt().build());
        generate(Some("sqlite_gen_param_opt_i32_null"), vec![(1usize, v.build())], vec![]).unwrap();
    }

    // # (insert) Param: All custom types
    {
        let v = SqliteVersion::new();
        let my_bool = v.custom_type("MyBool").rust_type("integration_tests::MyBool").base_type(type_bool().build());
        let my_i32 = v.custom_type("MyI32").rust_type("integration_tests::MyI32").base_type(type_i32().build());
        let my_i64 = v.custom_type("MyI64").rust_type("integration_tests::MyI64").base_type(type_i64().build());
        let my_u32 = v.custom_type("MyU32").rust_type("integration_tests::MyU32").base_type(type_u32().build());
        let my_f32 = v.custom_type("MyF32").rust_type("integration_tests::MyF32").base_type(type_f32().build());
        let my_f64 = v.custom_type("MyF64").rust_type("integration_tests::MyF64").base_type(type_f64().build());
        let my_bytes = v.custom_type("MyBytes").rust_type("integration_tests::MyBytes").base_type(type_bytes().build());
        let my_string = v.custom_type("MyString").rust_type("integration_tests::MyString").base_type(type_str().build());
        let my_utctime_chrono = v.custom_type("MyUtctimeChrono").rust_type("integration_tests::MyUtctimeChrono").base_type(type_utctime_s_chrono().build());
        let my_utctime_jiff = v.custom_type("MyUtctimeJiff").rust_type("integration_tests::MyUtctimeJiff").base_type(type_utctime_s_jiff().build());

        let bananna = v.table("param_custom_bananna");
        let mut custom_fields = vec![];
        for (
            i,
            type_,
        ) in [
            my_bool.field_type(),
            my_i32.field_type(),
            my_i64.field_type(),
            my_u32.field_type(),
            my_f32.field_type(),
            my_f64.field_type(),
            my_bytes.field_type(),
            my_string.field_type(),
            my_utctime_chrono.field_type(),
            my_utctime_chrono.field_type(),
            my_utctime_jiff.field_type(),
            my_utctime_jiff.field_type(),
        ]
            .into_iter()
            .enumerate() {
            custom_fields.push(bananna.field(&format!("x_{}", i), type_));
        }
        generate(Some("sqlite_gen_param_custom"), vec![(1usize, v.build())], vec![]).unwrap();
    }

    // # (insert) Param: Opt`<Custom>`
    {
        let v = SqliteVersion::new();
        let my_string = v.custom_type("MyString").rust_type("integration_tests::MyString").base_type(type_str().build());
        let bananna = v.table("param_opt_custom_bananna");
        let hizat =
            bananna.field("hizat", FieldTypeBuilder::new(my_string.field_type().type_).opt().build());
        generate(Some("sqlite_gen_param_opt_custom"), vec![(1usize, v.build())], vec![]).unwrap();
    }

    // # Insert on conflict do nothing
    {
        let v = SqliteVersion::new();
        let bananna = v.table("insert_on_conflict_do_nothing_bananna");
        let hizat = bananna.field("hizat", field_str().build());
        bananna.unique_index("all", &[&hizat]);
        generate(Some("sqlite_gen_insert_on_conflict_do_nothing"),
            vec![(1usize, v.build())],
            vec![
                new_insert(&bananna, vec![(hizat.clone(), Expr::Param {
                    name: "text".into(),
                    type_: get_type(&hizat),
                })])
                    .return_named("one", Expr::LitI32(1))
                    .on_conflict_do_nothing()
                    .build_query("insert_banan", QueryResCount::MaybeOne)
            ],
        ).unwrap();
    }

    // # Insert on conflict update
    {
        let v = SqliteVersion::new();
        let bananna = v.table("insert_on_conflict_update_bananna");
        let hizat = bananna.field("hizat", field_str().build());
        let two = bananna.field("two", field_i32().build());
        bananna.unique_index("all", &[&hizat]);
        generate(Some("sqlite_gen_insert_on_conflict_update"),
            vec![(1usize, v.build())],
            vec![new_insert(&bananna, vec![(hizat.clone(), Expr::Param {
                name: "text".into(),
                type_: get_type(&hizat),
            }), (two.clone(), Expr::Param {
                name: "two".into(),
                type_: get_type(&two),
            })]).return_field(&two).on_conflict_do_update(&[&hizat], vec![(two.clone(), Expr::BinOp {
                left: Box::new(Expr::Field(two.to_ref())),
                op: BinOp::Plus,
                right: Box::new(Expr::LitI32(1)),
            })]).build_query("insert_banan", QueryResCount::One)],
        ).unwrap();
    }

    // # Update
    {
        let v = SqliteVersion::new();
        let bananna = v.table("update_bananna");
        let hizat = bananna.field("hizat", field_str().build());
        generate(Some("sqlite_gen_update"), vec![(1usize, v.build())], vec![]).unwrap();
    }

    // # Update, where
    {
        let v = SqliteVersion::new();
        let bananna = v.table("update_where_ban");
        let hizat = bananna.field("hizat", field_str().build());
        generate(Some("sqlite_gen_update_where"), vec![(1usize, v.build())], vec![]).unwrap();
    }

    // # Update, returning
    {
        let v = SqliteVersion::new();
        let bananna = v.table("update_returning_b");
        let hizat = bananna.field("hizat", field_str().build());
        generate(Some("sqlite_gen_update_returning"), vec![(1usize, v.build())], vec![]).unwrap();
    }

    // # Delete
    {
        let v = SqliteVersion::new();
        let bananna = v.table("delete_b");
        let hizat = bananna.field("hizat", field_str().build());
        generate(Some("sqlite_gen_delete"), vec![(1usize, v.build())], vec![]).unwrap();
    }

    // # Delete, where
    {
        let v = SqliteVersion::new();
        let bananna = v.table("delete_where_ba");
        let hizat = bananna.field("hizat", field_str().build());
        generate(Some("sqlite_gen_delete_where"), vec![(1usize, v.build())], vec![]).unwrap();
    }

    // # Delete, returning
    {
        let v = SqliteVersion::new();
        let bananna = v.table("delete_returning_b");
        let hizat = bananna.field("hizat", field_str().build());
        generate(Some("sqlite_gen_delete_returning"), vec![(1usize, v.build())], vec![]).unwrap();
    }

    // # Select + join
    {
        let v = SqliteVersion::new();
        let bananna = v.table("select_join_b");
        let hizat = bananna.field("hizat", field_str().build());
        let three = bananna.field("three", field_i32().build());
        let one = v.table("select_join_two");
        let hizat1 = one.field("hizat", field_str().build());
        let two = one.field("two", field_str().build());
        generate(Some("sqlite_gen_select_join"), 
            vec![(1usize, v.build())],
            vec![new_select(&bananna).join(Join {
                source: NamedSelectSource {
                    source: JoinSource::Table(one.to_ref()),
                    alias: None,
                },
                type_: JoinType::Left,
                on: Expr::BinOp {
                    left: Box::new(Expr::Cast(Box::new(Expr::Field(hizat.to_ref())), get_type(&hizat).opt())),
                    op: BinOp::Equals,
                    right: Box::new(Expr::Field(hizat1.to_ref())),
                },
            }).return_field(&three).return_field(&two).build_query("get_it", QueryResCount::One)],
        ).unwrap();
    }

    // # Select limit
    {
        let v = SqliteVersion::new();
        let bananna = v.table("select_limit_bannanana");
        let hizat = bananna.field("hizat", field_str().build());
        generate(Some("sqlite_gen_select_limit"), vec![(1usize, v.build())], vec![]).unwrap();
    }

    // # Select order
    {
        let v = SqliteVersion::new();
        let bananna = v.table("select_order_bannanana");
        let hizat = bananna.field("hizat", field_i32().build());
        generate(Some("sqlite_gen_select_order"), vec![(1usize, v.build())], vec![]).unwrap();
    }

    // # Select group
    {
        let v = SqliteVersion::new();
        let bananna = v.table("select_group_by_bannanana");
        let hizat = bananna.field("hizat", field_i32().build());
        let hizat2 = bananna.field("hizat2", field_i32().build());
        generate(Some("sqlite_gen_select_group_by"), vec![(1usize, v.build())], vec![]).unwrap();
    }

    // # Migrate - add field
    {
        let v = SqliteVersion::new();
        let bananna = v.table("migrate_add_field_bannna");
        let hizat = bananna.field("hizat", field_str().build());
        let zomzom =
            bananna.field("zomzom",
                field_bool().migrate_fill(good_ormning::sqlite::query::expr::SerialExpr::LitBool(true)).build(),
            );
        generate(Some("sqlite_gen_migrate_add_field"), vec![
            // Versions (previous)
            (0usize, {
                let v = SqliteVersion::new();
                let bananna = v.table("migrate_add_field_bannna");
                let hizat = bananna.field("hizat", field_str().build());
                let x = v.build();
                x
            }),
            (1usize, v.build())
        ], vec![]).unwrap();
    }

    // # Migrate - rename field
    {
        let v = SqliteVersion::new();
        let bananna = v.table("migrate_rename_field_bannna");
        let hizat = bananna.field("hizat", field_str().build()).renamed_from("hozot");
        generate(Some("sqlite_gen_migrate_rename_field"), vec![
            // Versions (previous)
            (0usize, {
                let v = SqliteVersion::new();
                let bananna = v.table("migrate_rename_field_bannna");
                let _hozot = bananna.field("hozot", field_str().build());
                let x = v.build();
                x
            }),
            (1usize, v.build())
        ], vec![]).unwrap();
    }

    // # Migrate - remove field
    {
        let v = SqliteVersion::new();
        let bananna = v.table("migrate_remove_field_bnanaa");
        let hizat = bananna.field("hizat", field_str().build());
        let _ = generate(Some("sqlite_gen_migrate_remove_field"), vec![
            // Versions (previous)
            (0usize, {
                let v = SqliteVersion::new();
                let bananna = v.table("migrate_remove_field_bnanaa");
                let hizat = bananna.field("hizat", field_str().build());
                let _zomzom = bananna.field("zomzom", field_bool().opt().build());
                let x = v.build();
                x
            }),
            (1usize, v.build())
        ], vec![]);
    }

    // # Migrate - add table
    {
        let v = SqliteVersion::new();
        let bananna = v.table("migrate_add_table_bnanana");
        bananna.field("hizat", field_str().build());
        let two = v.table("migrate_add_table_two");
        let field_two = two.field("two", field_i32().build());
        generate(Some("sqlite_gen_migrate_add_table"), vec![
            // Versions (previous)
            (0usize, {
                let v = SqliteVersion::new();
                let bananna = v.table("migrate_add_table_bnanana");
                let hizat = bananna.field("hizat", field_str().build());
                let x = v.build();
                x
            }),
            (1usize, v.build())
        ], vec![]).unwrap();
    }

    // # Migrate - rename table
    {
        let v = SqliteVersion::new();
        let bananna = v.table("migrate_rename_table_bana").renamed_from("migrate_rename_table_bnanana");
        let hizat = bananna.field("hizat", field_str().build());
        let _ = generate(Some("sqlite_gen_migrate_rename_table"), vec![
            // Versions (previous)
            (0usize, {
                let v = SqliteVersion::new();
                let bananna = v.table("migrate_rename_table_bnanana");
                let hizat = bananna.field("hizat", field_str().build());
                let x = v.build();
                x
            }),
            (1usize, v.build())
        ], vec![]);
    }

    // # Migrate - remove table
    {
        let v = SqliteVersion::new();
        let bananna = v.table("migrate_remove_table_bananana");
        bananna.field("hizat", field_str().build());
        let _ = generate(Some("sqlite_gen_migrate_remove_table"), vec![
            // Versions (previous)
            (0usize, {
                let v = SqliteVersion::new();
                let bananna = v.table("migrate_remove_table_bananana");
                bananna.field("hizat", field_str().build());
                let two = v.table("migrate_remove_table_two");
                two.field("two", field_i32().build());
                let x = v.build();
                x
            }),
            (1usize, v.build())
        ], vec![]);
    }

    // # Junction
    {
        let v = SqliteVersion::new();
        let bananna = v.table("select_junction_bannanana");
        let hizat = bananna.field("hizat", field_i32().build());
        let hizat2 = bananna.field("hizat2", field_i32().build());
        generate(Some("sqlite_gen_select_junction"), vec![(1usize, v.build())], vec![]).unwrap();
    }

    // # Select CTE
    {
        let v = SqliteVersion::new();
        let bananna = v.table("select_cte_bannanana");
        let hizat = bananna.field("hizat", field_i32().build());
        let hizat2 = bananna.field("hizat2", field_i32().build());
        let mut hibbo_builder =
            CteBuilder::new("hibbo", Box::new(new_select_body(&bananna).return_field(&hizat2).build()));
        let (zathi_schema, _zathi_sql, _zathi_type) = hibbo_builder.field("zathi", get_type(&hizat2));
        let hibbo_cte = hibbo_builder.build();
        let hibbo_table = TableHandle {
            version: v.clone(),
            id: hibbo_cte.table_id.clone(),
        };
        let zathi_field = FieldHandle {
            table: hibbo_table.clone(),
            id: zathi_schema,
        };
        generate(Some("sqlite_gen_select_cte"), vec![(1usize, v.build())], vec![]).unwrap();
    }

    // # Window function
    {
        let v = SqliteVersion::new();
        let bananna = v.table("select_window_bannanana");
        let hizat = bananna.field("hizat", field_i32().build());
        let hizat2 = bananna.field("hizat2", field_i32().build());
        generate(Some("sqlite_gen_select_window"), vec![(1usize, v.build())], vec![]).unwrap();
    }

    // # Migrate - pre migration
    {
        let v0 = SqliteVersion::new();
        let _v0_bananna = v0.table("migrate_pre_migration_v0_banana");
        _v0_bananna.field("hizat", field_str().build());
        let v0_two = v0.table("migrate_pre_migration_v0_two");
        let v0_field_two = v0_two.field("two", field_i32().build());
        let v1 = SqliteVersion::new();
        let v1_bananna = v1.table("migrate_pre_migration_v0_banana");
        v1_bananna.field("hizat", field_str().build());
        generate(Some("sqlite_gen_migrate_pre_migration"), vec![
            // Versions (previous)
            (0usize, v0.build()),
            (1usize, v1.build())
        ], vec![]).unwrap();
    }

    // # Param array
    {
        let v = SqliteVersion::new();
        let bananna = v.table("param_arr_i32_bananna");
        let hizat = bananna.field("hizat", field_i32().build());
        generate(Some("sqlite_gen_param_arr_i32"), vec![(1usize, v.build())], vec![]).unwrap();
    }
}
