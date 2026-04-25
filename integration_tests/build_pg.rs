use good_ormning::pg::Version as PgVersion;
use good_ormning::sqlite::Version as SqliteVersion;
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
            QueryResCount,
            new_select,
            new_select_body,
            new_update,
            new_delete,
            types::{
                Type,
                type_i32,
            },
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
        generate(&root.join("tests/pg_gen_base_insert.rs"), vec![(0usize, v.build())], vec![
            // Queries
            new_insert(&bananna, vec![(hizat.clone(), Expr::Param {
                name: "text".into(),
                type_: get_type(&hizat),
            })]).build_query("insert_banan", QueryResCount::None),
            new_select(&bananna).return_field(&hizat).build_query("get_banan", QueryResCount::One)
        ]).unwrap();
    }

    // # (insert) Param: i32
    {
        let v = PgVersion::new();
        let bananna = v.table("bananna");
        let hizat = bananna.field("hizat", field_i32().build());
        generate(&root.join("tests/pg_gen_param_i32.rs"), vec![(0usize, v.build())], vec![
            // Queries
            new_insert(&bananna, vec![(hizat.clone(), Expr::Param {
                name: "val".into(),
                type_: get_type(&hizat),
            })]).build_query("insert_banan", QueryResCount::None),
            new_select(&bananna).return_field(&hizat).build_query("get_banan", QueryResCount::One)
        ]).unwrap();
    }

    // # (insert) Param: utctime (chrono)
    {
        let v = PgVersion::new();
        let bananna = v.table("bananna");
        let hizat = bananna.field("hizat", field_utctime_s_chrono().build());
        generate(&root.join("tests/pg_gen_param_utctime_chrono.rs"), vec![(0usize, v.build())], vec![
            // Queries
            new_insert(&bananna, vec![(hizat.clone(), Expr::Param {
                name: "val".into(),
                type_: get_type(&hizat),
            })]).build_query("insert_banan", QueryResCount::None),
            new_select(&bananna).return_field(&hizat).build_query("get_banan", QueryResCount::One)
        ]).unwrap();
    }

    // # (insert) Param: utctime (jiff)
    {
        let v = PgVersion::new();
        let bananna = v.table("bananna");
        let hizat = bananna.field("hizat", field_utctime_s_jiff().build());
        generate(&root.join("tests/pg_gen_param_utctime_jiff.rs"), vec![(0usize, v.build())], vec![
            // Queries
            new_insert(&bananna, vec![(hizat.clone(), Expr::Param {
                name: "val".into(),
                type_: get_type(&hizat),
            })]).build_query("insert_banan", QueryResCount::None),
            new_select(&bananna).return_field(&hizat).build_query("get_banan", QueryResCount::One)
        ]).unwrap();
    }

    // # (insert) Param: Opt`<i32>`
    {
        let v = PgVersion::new();
        let bananna = v.table("bananna");
        let hizat = bananna.field("hizat", field_i32().opt().build());
        generate(&root.join("tests/pg_gen_param_opt_i32.rs"), vec![(0usize, v.build())], vec![
            // Queries
            new_insert(&bananna, vec![(hizat.clone(), Expr::Param {
                name: "val".into(),
                type_: get_type(&hizat),
            })]).build_query("insert_banan", QueryResCount::None),
            new_select(&bananna).return_field(&hizat).build_query("get_banan", QueryResCount::One)
        ]).unwrap();
    }

    // # (insert) Param: Opt`<i32>`, null
    {
        let v = PgVersion::new();
        let bananna = v.table("bananna");
        let hizat = bananna.field("hizat", field_i32().opt().build());
        generate(&root.join("tests/pg_gen_param_opt_i32_null.rs"), vec![(0usize, v.build())], vec![
            // Queries
            new_insert(
                &bananna,
                vec![(hizat.clone(), Expr::LitNull(get_type(&hizat).type_))],
            ).build_query("insert_banan", QueryResCount::None),
            new_select(&bananna).return_field(&hizat).build_query("get_banan", QueryResCount::One)
        ]).unwrap();
    }

    // # (insert) Param: All custom types
    {
        let v = PgVersion::new();
        let bananna = v.table("bananna");
        let mut custom_fields = vec![];
        for (
            i,
            (schema_id, type_),
        ) in [
            ("zPZS1I5WW", field_bool().custom("integration_tests::MyBool").build()),
            ("zC06X4BAF", field_i32().custom("integration_tests::MyI32").build()),
            ("z9JQDQ8ZB", field_i64().custom("integration_tests::MyI64").build()),
            ("zU32S1I5W", field_u32().custom("integration_tests::MyU32").build()),
            ("zMSGIBKUC", field_f32().custom("integration_tests::MyF32").build()),
            ("zQ23DTVF3", field_f64().custom("integration_tests::MyF64").build()),
            ("zV3TUIVTU", field_bytes().custom("integration_tests::MyBytes").build()),
            ("z7AJMBYHP", field_str().custom("integration_tests::MyString").build()),
            ("zCKQAR1KC", field_utctime_s_chrono().custom("integration_tests::MyUtctimeChrono").build()),
            ("zNDD21YUS", field_utctime_s_chrono().custom("integration_tests::MyUtctimeChrono").build()),
            ("zNDD21YUT", field_utctime_s_jiff().custom("integration_tests::MyUtctimeJiff").build()),
            ("zNDD21YUU", field_utctime_s_jiff().custom("integration_tests::MyUtctimeJiff").build()),
        ]
            .into_iter()
            .enumerate() {
            custom_fields.push(bananna.field(&format!("x_{}", i), type_));
        }
        generate(&root.join("tests/pg_gen_param_custom.rs"), vec![(0usize, v.build())], vec![
            // Queries
            new_insert(
                &bananna,
                custom_fields
                    .iter()
                    .map(
                        |f| set_field(
                            &f
                                .table
                                .version
                                .0
                                .borrow()
                                .as_ref()
                                .unwrap()
                                .tables
                                .get(&f.table.id)
                                .unwrap()
                                .fields
                                .get(&f.id)
                                .unwrap()
                                .id
                                .clone(),
                            f
                        )
                    )
                    .collect(),
            ).build_query("insert_banan", QueryResCount::None),
            new_select(&bananna)
                .returns_from_iter(
                    custom_fields
                        .iter()
                        .enumerate()
                        .map(|(i, f): (usize, &_)| good_ormning::pg::query::utils::Returning {
                            e: Expr::Field(f.to_ref()),
                            rename: Some(format!("x_{}", i)),
                        })
                )
                .build_query("get_banan", QueryResCount::One)
        ]).unwrap();
    }

    // # (insert) Param: Opt`<Custom>`
    {
        let v = PgVersion::new();
        let bananna = v.table("bananna");
        let hizat =
            bananna.field("hizat", field_str().custom("integration_tests::MyString").opt().build());
        generate(&root.join("tests/pg_gen_param_opt_custom.rs"), vec![(0usize, v.build())], vec![
            // Queries
            new_insert(&bananna, vec![(hizat.clone(), Expr::Param {
                name: "text".into(),
                type_: get_type(&hizat),
            })]).build_query("insert_banan", QueryResCount::None),
            new_select(&bananna).return_field(&hizat).build_query("get_banan", QueryResCount::One)
        ]).unwrap();
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
        let bananna = v.table("bananna");
        let hizat = bananna.field("hizat", field_str().build());
        generate(&root.join("tests/pg_gen_update.rs"), vec![(0usize, v.build())], vec![
            // Queries
            new_insert(
                &bananna,
                vec![(hizat.clone(), Expr::LitString("yog".into()))],
            ).build_query("insert_banan", QueryResCount::None),
            new_select(&bananna).return_field(&hizat).build_query("get_banan", QueryResCount::One),
            new_update(
                &bananna,
                vec![(hizat.clone(), Expr::LitString("tep".into()))],
            ).build_query("update_banan", QueryResCount::None)
        ]).unwrap();
    }

    // # Update, where
    {
        let v = PgVersion::new();
        let bananna = v.table("ban");
        let hizat = bananna.field("hizat", field_str().build());
        generate(&root.join("tests/pg_gen_update_where.rs"), vec![(0usize, v.build())], vec![
            // Queries
            new_insert(
                &bananna,
                vec![(hizat.clone(), Expr::LitString("yog".into()))],
            ).build_query("insert_banan", QueryResCount::None),
            new_select(&bananna).return_field(&hizat).build_query("get_banan", QueryResCount::One),
            new_update(&bananna, vec![(hizat.clone(), Expr::Param {
                name: "val".into(),
                type_: get_type(&hizat),
            })]).where_(Expr::BinOp {
                left: Box::new(Expr::Field(hizat.to_ref())),
                op: BinOp::Equals,
                right: Box::new(Expr::Param {
                    name: "cond".into(),
                    type_: get_type(&hizat),
                }),
            }).build_query("update_banan", QueryResCount::None)
        ]).unwrap();
    }

    // # Update, returning
    {
        let v = PgVersion::new();
        let bananna = v.table("b");
        let hizat = bananna.field("hizat", field_str().build());
        generate(&root.join("tests/pg_gen_update_returning.rs"), vec![(0usize, v.build())], vec![
            // Queries
            new_insert(
                &bananna,
                vec![(hizat.clone(), Expr::LitString("yog".into()))],
            ).build_query("insert_banan", QueryResCount::None),
            new_update(&bananna, vec![(hizat.clone(), Expr::LitString("tep".into()))])
                .return_field(&hizat)
                .build_query("update_banan", QueryResCount::MaybeOne)
        ]).unwrap();
    }

    // # Delete
    {
        let v = PgVersion::new();
        let bananna = v.table("b");
        let hizat = bananna.field("hizat", field_str().build());
        generate(&root.join("tests/pg_gen_delete.rs"), vec![(0usize, v.build())], vec![
            // Queries
            new_insert(
                &bananna,
                vec![(hizat.clone(), Expr::LitString("seeon".into()))],
            ).build_query("insert_banan", QueryResCount::None),
            new_select(&bananna).return_field(&hizat).build_query("get_banan", QueryResCount::MaybeOne),
            new_delete(&bananna).build_query("no_banan", QueryResCount::None)
        ]).unwrap();
    }

    // # Delete, where
    {
        let v = PgVersion::new();
        let bananna = v.table("ba");
        let hizat = bananna.field("hizat", field_str().build());
        generate(&root.join("tests/pg_gen_delete_where.rs"), vec![(0usize, v.build())], vec![
            // Queries
            new_insert(
                &bananna,
                vec![(hizat.clone(), Expr::LitString("seeon".into()))],
            ).build_query("insert_banan", QueryResCount::None),
            new_select(&bananna).return_field(&hizat).build_query("get_banan", QueryResCount::MaybeOne),
            new_delete(&bananna).where_(Expr::BinOp {
                left: Box::new(Expr::Field(hizat.to_ref())),
                op: BinOp::Equals,
                right: Box::new(Expr::Param {
                    name: "hiz".into(),
                    type_: get_type(&hizat),
                }),
            }).build_query("no_banan", QueryResCount::None)
        ]).unwrap();
    }

    // # Delete, returning
    {
        let v = PgVersion::new();
        let bananna = v.table("b");
        let hizat = bananna.field("hizat", field_str().build());
        generate(&root.join("tests/pg_gen_delete_returning.rs"), vec![(0usize, v.build())], vec![
            // Queries
            new_insert(
                &bananna,
                vec![(hizat.clone(), Expr::LitString("seeon".into()))],
            ).build_query("insert_banan", QueryResCount::None),
            new_delete(&bananna).return_field(&hizat).build_query("no_banan", QueryResCount::One)
        ]).unwrap();
    }

    // # Select + join
    {
        let v = PgVersion::new();
        let bananna = v.table("b");
        let hizat = bananna.field("hizat", field_str().build());
        let three = bananna.field("three", field_i32().build());
        let one = v.table("two");
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
        generate(&root.join("tests/pg_gen_select_limit.rs"), vec![(0usize, v.build())], vec![
            // Queries
            new_insert(&bananna, vec![(hizat.clone(), Expr::Param {
                name: "text".into(),
                type_: get_type(&hizat),
            })]).build_query("insert_banan", QueryResCount::None),
            new_select(&bananna)
                .return_field(&hizat)
                .limit(Expr::LitI64(2))
                .build_query("get_banan", QueryResCount::Many)
        ]).unwrap();
    }

    // # Select order
    {
        let v = PgVersion::new();
        let bananna = v.table("bannanana");
        let hizat = bananna.field("hizat", field_i32().build());
        generate(&root.join("tests/pg_gen_select_order.rs"), vec![(0usize, v.build())], vec![
            // Queries
            new_insert(&bananna, vec![(hizat.clone(), Expr::Param {
                name: "v".into(),
                type_: get_type(&hizat),
            })]).build_query("insert_banan", QueryResCount::None),
            new_select(&bananna)
                .return_field(&hizat)
                .order(Expr::Field(hizat.to_ref()), Order::Asc)
                .build_query("get_banan", QueryResCount::Many)
        ]).unwrap();
    }

    // # Select group
    {
        let v = PgVersion::new();
        let bananna = v.table("bannanana");
        let hizat = bananna.field("hizat", field_i32().build());
        let hizat2 = bananna.field("hizat2", field_i32().build());
        generate(&root.join("tests/pg_gen_select_group_by.rs"), vec![(0usize, v.build())], vec![
            // Queries
            new_insert(&bananna, vec![(hizat.clone(), Expr::Param {
                name: "v".into(),
                type_: get_type(&hizat),
            }), (hizat2.clone(), Expr::Param {
                name: "v2".into(),
                type_: get_type(&hizat2),
            })]).build_query("insert_banan", QueryResCount::None),
            new_select(&bananna)
                .return_named("hizat2", fn_sum(Expr::Field(hizat2.to_ref())))
                .group(vec![Expr::Field(hizat.to_ref())])
                .build_query("get_banan", QueryResCount::Many)
        ]).unwrap();
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
        ], vec![
            // Queries
            new_select(&bananna).return_field(&hizat).return_field(&zomzom).build_query("get_banan", QueryResCount::MaybeOne)
        ]).unwrap();
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
        ], vec![
            // Queries
            new_insert(&bananna, vec![(hizat.clone(), Expr::LitString("nizoot".into()))],).build_query("ins", QueryResCount::None)
        ]).unwrap();
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
        ], vec![
            // Queries
            new_insert(&bananna, vec![(hizat.clone(), Expr::Param {
                name: "okolor".into(),
                type_: get_type(&hizat),
            })]).build_query("new_banan", QueryResCount::None)
        ]).unwrap();
    }

    // # Migrate - add table
    {
        let v = PgVersion::new();
        let bananna = v.table("bnanana");
        let hizat = bananna.field("hizat", field_str().build());
        let two = v.table("two");
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
        ], vec![
            // Queries
            new_insert(&two, vec![(field_two.clone(), Expr::Param {
                name: "two".into(),
                type_: get_type(&field_two),
            })]).build_query("two", QueryResCount::None)
        ]).unwrap();
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
        ], vec![
            // Queries
            new_insert(&bananna, vec![(hizat.clone(), Expr::Param {
                name: "two".into(),
                type_: get_type(&hizat),
            })]).build_query("two", QueryResCount::None)
        ]).unwrap();
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
                let two = v.table("two");
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
        generate(&root.join("tests/pg_gen_select_cte.rs"), vec![(0usize, v.build())], vec![
            // Queries
            new_insert(
                &bananna,
                vec![set_field("v", &hizat), set_field("v2", &hizat2)],
            ).build_query("insert_banan", QueryResCount::None),
            new_select(&hibbo_table).with(With {
                recursive: false,
                ctes: vec![hibbo_cte],
            }).return_named("zathi", Expr::Field(zathi_field.to_ref())).build_query("get_banan", QueryResCount::Many)
        ]).unwrap();
    }

    // # Window function
    {
        let v = PgVersion::new();
        let bananna = v.table("bannanana");
        let hizat = bananna.field("hizat", field_i32().build());
        let hizat2 = bananna.field("hizat2", field_i32().build());
        generate(&root.join("tests/pg_gen_select_window.rs"), vec![(0usize, v.build())], vec![
            // Queries
            new_insert(
                &bananna,
                vec![set_field("v", &hizat), set_field("v2", &hizat2)],
            ).build_query("insert_banan", QueryResCount::None),
            new_select(&bananna).return_named("hizat2", Expr::Window {
                expr: Box::new(fn_sum(Expr::Field(hizat2.to_ref()))),
                partition_by: vec![Expr::Field(hizat.to_ref())],
                order_by: vec![],
            }).build_query("get_banan", QueryResCount::Many)
        ]).unwrap();
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
