use good_ormning::pg::Version as PgVersion;
use good_ormning::sqlite::Version as SqliteVersion;
use good_ormning::{QueryResCount, pg_type_i32 as type_i32, pg_type_i64 as type_i64, pg_type_u32 as type_u32, pg_type_f32 as type_f32, pg_type_f64 as type_f64, pg_type_bool as type_bool, pg_type_bytes as type_bytes, pg_type_str as type_str, pg_type_utctime_s_chrono as type_utctime_s_chrono, pg_type_utctime_s_jiff as type_utctime_s_jiff, PgType as Type, PgFieldTypeBuilder as FieldTypeBuilder};
use {
    std::path::Path,
    std::rc::Rc,
    good_ormning::{
        pg::{
            VersionHandle,
            FieldHandle,
            TableHandle,
            schema::{
                field::{
                    field_str,
                    field_i32,
                    field_u32,
                    field_bool,
                    field_utctime_s_chrono,
                    field_utctime_s_jiff,
                    field_i64,
                    field_f32,
                    field_f64,
                    field_bytes,
                },
            },
            query::{
                expr::{
                    Expr,
                    BinOp,
                    ComputeType,
                    ExprType,
                    ExprValName,
                },
                select::{
                    Join,
                    NamedSelectSource,
                    JoinSource,
                    JoinType,
                    Order,
                },
                utils::{
                    CteBuilder,
                    With,
                },
                helpers::{
                    set_field,
                    fn_sum,
                },
            },
            generate,
            new_insert,
            new_select,
            new_select_body,
            new_update,
            new_delete,
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
pub fn build(root: &Path) {
    // # Base: create table, insert, select
    {
        let v = PgVersion::new();
        let bananna = v.table("bannanana");
        let hizat = bananna.field("hizat", field_str().build());
        generate(&root.join("tests/pg_gen_base_insert.rs"), vec![(0usize, v.build())], vec![]).unwrap();
    }

    // # (insert) Param: i32
    {
        let v = PgVersion::new();
        let bananna = v.table("bananna_pg_gen_base_insert");
        let hizat = bananna.field("hizat", field_i32().build());
        generate(&root.join("tests/pg_gen_param_i32.rs"), vec![(0usize, v.build())], vec![]).unwrap();
    }

    // # (insert) Param: utctime (chrono)
    {
        let v = PgVersion::new();
        let bananna = v.table("bananna_pg_gen_param_i32");
        let hizat = bananna.field("hizat", field_utctime_s_chrono().build());
        generate(&root.join("tests/pg_gen_param_utctime_chrono.rs"), vec![(0usize, v.build())], vec![]).unwrap();
    }

    // # (insert) Param: utctime (jiff)
    {
        let v = PgVersion::new();
        let bananna = v.table("bananna_pg_gen_param_utctime_chrono");
        let hizat = bananna.field("hizat", field_utctime_s_jiff().build());
        generate(&root.join("tests/pg_gen_param_utctime_jiff.rs"), vec![(0usize, v.build())], vec![]).unwrap();
    }

    // # (insert) Param: Opt`<i32>`
    {
        let v = PgVersion::new();
        let bananna = v.table("bananna_pg_gen_param_utctime_jiff");
        let hizat = bananna.field("hizat", field_i32().opt().build());
        generate(&root.join("tests/pg_gen_param_opt_i32.rs"), vec![(0usize, v.build())], vec![]).unwrap();
    }

    // # (insert) Param: Opt`<i32>`, null
    {
        let v = PgVersion::new();
        let bananna = v.table("bananna_pg_gen_param_opt_i32");
        let hizat = bananna.field("hizat", field_i32().opt().build());
        generate(&root.join("tests/pg_gen_param_opt_i32_null.rs"), vec![(0usize, v.build())], vec![]).unwrap();
    }

    // # (insert) Param: All custom types
    {
        let v = PgVersion::new();
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

        let bananna = v.table("bananna_pg_gen_param_opt_i32_null");
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
        generate(&root.join("tests/pg_gen_param_custom.rs"), vec![(0usize, v.build())], vec![]).unwrap();
    }

    // # (insert) Param: Opt`<Custom>`
    {
        let v = PgVersion::new();
        let my_string = v.custom_type("MyString").rust_type("integration_tests::MyString").base_type(type_str().build());
        let bananna = v.table("bananna_pg_gen_param_custom");
        let hizat =
            bananna.field("hizat", FieldTypeBuilder::new(my_string.field_type().type_).opt().build());
        generate(&root.join("tests/pg_gen_param_opt_custom.rs"), vec![(0usize, v.build())], vec![]).unwrap();
    }

    // # Insert on conflict do nothing
    {
        let v = PgVersion::new();
        let bananna = v.table("bannanana");
        let hizat = bananna.field("hizat", field_str().build());
        bananna.unique_index("all", &[&hizat]);
        generate(
            &root.join("tests/pg_gen_insert_on_conflict_do_nothing.rs"),
            vec![(0usize, v.build())],
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
        let v = PgVersion::new();
        let bananna = v.table("bannanana");
        let hizat = bananna.field("hizat", field_str().build());
        let two = bananna.field("two", field_i32().build());
        bananna.unique_index("all", &[&hizat]);
        generate(
            &root.join("tests/pg_gen_insert_on_conflict_update.rs"),
            vec![(0usize, v.build())],
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
        let v = PgVersion::new();
        let bananna = v.table("bananna_pg_gen_param_opt_custom");
        let hizat = bananna.field("hizat", field_str().build());
        generate(&root.join("tests/pg_gen_update.rs"), vec![(0usize, v.build())], vec![]).unwrap();
    }

    // # Update, where
    {
        let v = PgVersion::new();
        let bananna = v.table("ban");
        let hizat = bananna.field("hizat", field_str().build());
        generate(&root.join("tests/pg_gen_update_where.rs"), vec![(0usize, v.build())], vec![]).unwrap();
    }

    // # Update, returning
    {
        let v = PgVersion::new();
        let bananna = v.table("b");
        let hizat = bananna.field("hizat", field_str().build());
        generate(&root.join("tests/pg_gen_update_returning.rs"), vec![(0usize, v.build())], vec![]).unwrap();
    }

    // # Delete
    {
        let v = PgVersion::new();
        let bananna = v.table("b");
        let hizat = bananna.field("hizat", field_str().build());
        generate(&root.join("tests/pg_gen_delete.rs"), vec![(0usize, v.build())], vec![]).unwrap();
    }

    // # Delete, where
    {
        let v = PgVersion::new();
        let bananna = v.table("ba");
        let hizat = bananna.field("hizat", field_str().build());
        generate(&root.join("tests/pg_gen_delete_where.rs"), vec![(0usize, v.build())], vec![]).unwrap();
    }

    // # Delete, returning
    {
        let v = PgVersion::new();
        let bananna = v.table("b");
        let hizat = bananna.field("hizat", field_str().build());
        generate(&root.join("tests/pg_gen_delete_returning.rs"), vec![(0usize, v.build())], vec![]).unwrap();
    }

    // # Select + join
    {
        let v = PgVersion::new();
        let bananna = v.table("b");
        let hizat = bananna.field("hizat", field_str().build());
        let three = bananna.field("three", field_i32().build());
        let one = v.table("two_pg_gen_delete_returning");
        let hizat1 = one.field("hizat", field_str().build());
        let two = one.field("two", field_str().build());
        v.post_migration(
            new_insert(
                &bananna,
                vec![(hizat.clone(), Expr::LitString("key".into())), (three.clone(), Expr::LitI32(33))],
            ).build_migration(&v),
        );
        v.post_migration(
            new_insert(
                &one,
                vec![(hizat1.clone(), Expr::LitString("key".into())), (two.clone(), Expr::LitString("no".into()))],
            ).build_migration(&v),
        );
        generate(
            &root.join("tests/pg_gen_select_join.rs"),
            vec![(0usize, v.build())],
            vec![new_select(&bananna).join(Join {
                source: Box::new(NamedSelectSource {
                    source: JoinSource::Table(one.to_ref()),
                    alias: None,
                }),
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
        let v = PgVersion::new();
        let bananna = v.table("bannanana");
        let hizat = bananna.field("hizat", field_str().build());
        generate(&root.join("tests/pg_gen_select_limit.rs"), vec![(0usize, v.build())], vec![]).unwrap();
    }

    // # Select order
    {
        let v = PgVersion::new();
        let bananna = v.table("bannanana");
        let hizat = bananna.field("hizat", field_i32().build());
        generate(&root.join("tests/pg_gen_select_order.rs"), vec![(0usize, v.build())], vec![]).unwrap();
    }

    // # Select group
    {
        let v = PgVersion::new();
        let bananna = v.table("bannanana");
        let hizat = bananna.field("hizat", field_i32().build());
        let hizat2 = bananna.field("hizat2", field_i32().build());
        generate(&root.join("tests/pg_gen_select_group_by.rs"), vec![(0usize, v.build())], vec![]).unwrap();
    }

    // # Migrate - add field
    {
        let v = PgVersion::new();
        let bananna = v.table("bannna");
        let hizat = bananna.field("hizat", field_str().build());
        let zomzom =
            bananna.field("zomzom",
                field_bool().migrate_fill(good_ormning::pg::query::expr::SerialExpr::LitBool(true)).build(),
            );
        generate(&root.join("tests/pg_gen_migrate_add_field.rs"), vec![
            // Versions (previous)
            (0usize, {
                let v = PgVersion::new();
                let bananna = v.table("bannna");
                let hizat = bananna.field("hizat", field_str().build());
                v.post_migration(
                    new_insert(&bananna, vec![(hizat.clone(), Expr::LitString("nizoot".into()))]).build_migration(&v),
                );
                let x = v.build();
                x
            }),
            (1usize, v.build())
        ], vec![]).unwrap();
    }

    // # Migrate - rename field
    {
        let v = PgVersion::new();
        let bananna = v.table("bannna");
        let hizat = bananna.field("hizat", field_str().build()).renamed_from("hozot");
        generate(&root.join("tests/pg_gen_migrate_rename_field.rs"), vec![
            // Versions (previous)
            (0usize, {
                let v = PgVersion::new();
                let bananna = v.table("bannna");
                let _hozot = bananna.field("hozot", field_str().build());
                let x = v.build();
                x
            }),
            (1usize, v.build())
        ], vec![]).unwrap();
    }

    // # Migrate - remove field
    {
        let v = PgVersion::new();
        let bananna = v.table("bnanaa");
        let hizat = bananna.field("hizat", field_str().build());
        generate(&root.join("tests/pg_gen_migrate_remove_field.rs"), vec![
            // Versions (previous)
            (0usize, {
                let v = PgVersion::new();
                let bananna = v.table("bnanaa");
                let hizat = bananna.field("hizat", field_str().build());
                bananna.field("zomzom", field_bool().opt().build());
                let x = v.build();
                x
            }),
            (1usize, v.build())
        ], vec![]).unwrap();
    }

    // # Migrate - add table
    {
        let v = PgVersion::new();
        let bananna = v.table("bnanana");
        let hizat = bananna.field("hizat", field_str().build());
        let two = v.table("two_pg_gen_migrate_remove_field");
        let field_two = two.field("two", field_i32().build());
        generate(&root.join("tests/pg_gen_migrate_add_table.rs"), vec![
            // Versions (previous)
            (0usize, {
                let v = PgVersion::new();
                let bananna = v.table("bnanana");
                let hizat = bananna.field("hizat", field_str().build());
                let x = v.build();
                x
            }),
            (1usize, v.build())
        ], vec![]).unwrap();
    }

    // # Migrate - rename table
    {
        let v = PgVersion::new();
        let bananna = v.table("bana").renamed_from("bnanana");
        let hizat = bananna.field("hizat", field_str().build());
        generate(&root.join("tests/pg_gen_migrate_rename_table.rs"), vec![
            // Versions (previous)
            (0usize, {
                let v = PgVersion::new();
                let bananna = v.table("bnanana");
                let hizat = bananna.field("hizat", field_str().build());
                let x = v.build();
                x
            }),
            (1usize, v.build())
        ], vec![]).unwrap();
    }

    // # Migrate - remove table
    {
        let v = PgVersion::new();
        let bananna = v.table("bananana");
        let hizat = bananna.field("hizat", field_str().build());
        generate(&root.join("tests/pg_gen_migrate_remove_table.rs"), vec![
            // Versions (previous)
            (0usize, {
                let v = PgVersion::new();
                let bananna = v.table("bananana");
                let hizat = bananna.field("hizat", field_str().build());
                let two = v.table("two_pg_gen_migrate_remove_table");
                two.field("two", field_i32().build());
                let x = v.build();
                x
            }),
            (1usize, v.build())
        ], vec![]).unwrap();
    }

    // # Select CTE
    {
        let v = PgVersion::new();
        let bananna = v.table("bannanana");
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
        generate(&root.join("tests/pg_gen_select_cte.rs"), vec![(0usize, v.build())], vec![]).unwrap();
    }

    // # Window function
    {
        let v = PgVersion::new();
        let bananna = v.table("bannanana");
        let hizat = bananna.field("hizat", field_i32().build());
        let hizat2 = bananna.field("hizat2", field_i32().build());
        generate(&root.join("tests/pg_gen_select_window.rs"), vec![(0usize, v.build())], vec![]).unwrap();
    }

    // # Migrate - pre migration
    {
        let v0 = PgVersion::new();
        let _v0_bananna = v0.table("v0_banana");
        _v0_bananna.field("hizat", field_str().build());
        let v0_two = v0.table("v0_two");
        let v0_field_two = v0_two.field("two", field_i32().build());
        let v1 = PgVersion::new();
        let v1_bananna = v1.table("v0_banana");
        v1_bananna.field("hizat", field_str().build());
        v1.pre_migration(new_insert(&v0_two, vec![(v0_field_two.clone(), Expr::LitI32(7))]).build_migration(&v0));
        generate(&root.join("tests/pg_gen_migrate_pre_migration.rs"), vec![
            // Versions (previous)
            (0usize, v0.build()),
            (1usize, v1.build())
        ], vec![]).unwrap();
    }

    // # Migrate - make field optional
    {
        let v = PgVersion::new();
        let bananna = v.table("bannna");
        let hizat = bananna.field("hizat", field_str().opt().build());
        generate(&root.join("tests/pg_gen_migrate_make_field_optional.rs"), vec![
            // Versions (previous)
            (0usize, {
                let v = PgVersion::new();
                let bananna = v.table("bannna");
                let hizat = bananna.field("hizat", field_str().build());
                v.build()
            }),
            (1usize, v.build())
        ], vec![]).unwrap();
    }
}
