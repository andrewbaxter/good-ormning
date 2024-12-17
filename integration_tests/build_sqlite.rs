use {
    std::path::Path,
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
                },
                helpers::{
                    fn_max,
                    set_field,
                },
                insert::InsertConflict,
                select_body::{
                    Join,
                    JoinSource,
                    JoinType,
                    NamedSelectSource,
                    Order,
                    SelectJunction,
                },
                utils::{
                    CteBuilder,
                    With,
                },
            },
            schema::{
                constraint::{
                    ConstraintType,
                    PrimaryKeyDef,
                },
                field::{
                    field_bool,
                    field_bytes,
                    field_f32,
                    field_f64,
                    field_i32,
                    field_i64,
                    field_str,
                    field_u32,
                    field_utctime_ms,
                    field_utctime_s,
                    Field,
                },
            },
            types::type_i32,
            QueryResCount,
            Version,
        },
    },
    flowcontrol::shed,
};

pub fn build(root: &Path) {
    // # Hello world example
    {
        let mut latest_version = Version::default();
        let users = latest_version.table("zQLEK3CT0", "users");
        let id = users.rowid_field(&mut latest_version, None);
        let name = users.field(&mut latest_version, "zLQI9HQUQ", "name", field_str().build());
        let points = users.field(&mut latest_version, "zLAPH3H29", "points", field_i64().build());
        generate(&root.join("tests/sqlite_gen_hello_world.rs"), vec![
            // Versions
            (0usize, latest_version)
        ], vec![
            // Queries
            new_insert(&users, vec![(name.clone(), Expr::Param {
                name: "name".into(),
                type_: name.type_.type_.clone(),
            }), (points.clone(), Expr::Param {
                name: "points".into(),
                type_: points.type_.type_.clone(),
            })]).build_query("create_user", QueryResCount::None),
            new_select(&users).where_(Expr::BinOp {
                left: Box::new(Expr::Binding(Binding::field(&id))),
                op: BinOp::Equals,
                right: Box::new(Expr::Param {
                    name: "id".into(),
                    type_: id.type_.type_.clone(),
                }),
            }).return_fields(&[&name, &points]).build_query("get_user", QueryResCount::One),
            new_select(&users).return_field(&id).build_query("list_users", QueryResCount::Many)
        ]).unwrap();
    }

    // # Base: create table, insert, select
    {
        let mut v = Version::default();
        let bananna = v.table("zEOIWAACJ", "bannanana");
        let hizat = bananna.field(&mut v, "z437INV6D", "hizat", field_str().build());
        generate(&root.join("tests/sqlite_gen_base_insert.rs"), vec![(0usize, v)], vec![
            // Queries
            new_insert(&bananna, vec![(hizat.clone(), Expr::Param {
                name: "text".into(),
                type_: hizat.type_.type_.clone(),
            })]).build_query("insert_banan", QueryResCount::None),
            new_select(&bananna).return_field(&hizat).build_query("get_banan", QueryResCount::One)
        ]).unwrap();
    }

    // # Primary key
    {
        let mut v = Version::default();
        let bananna = v.table("zEOIWAACJ", "bannanana");
        let hizat = bananna.field(&mut v, "z437INV6D", "hizat", field_str().build());
        bananna.constraint(
            &mut v,
            "z2KEN3UL1",
            "hizat_pk",
            ConstraintType::PrimaryKey(PrimaryKeyDef { fields: vec![hizat.clone()] }),
        );
        generate(&root.join("tests/sqlite_gen_constraint.rs"), vec![(0usize, v)], vec![]).unwrap();
    }

    // # (insert) Param: i32
    {
        let mut v = Version::default();
        let bananna = v.table("zJCPRHK37", "bananna");
        let hizat = bananna.field(&mut v, "z437INV6D", "hizat", field_i32().build());
        generate(&root.join("tests/sqlite_gen_param_i32.rs"), vec![(0usize, v)], vec![
            // Queries
            new_insert(&bananna, vec![(hizat.clone(), Expr::Param {
                name: "val".into(),
                type_: hizat.type_.type_.clone(),
            })]).build_query("insert_banan", QueryResCount::None),
            new_select(&bananna).return_field(&hizat).build_query("get_banan", QueryResCount::One)
        ]).unwrap();
    }

    // # (insert) Param: datetime (seconds)
    {
        let mut v = Version::default();
        let bananna = v.table("zJCPRHK37", "bananna");
        let hizat = bananna.field(&mut v, "z437INV6D", "hizat", field_utctime_s().build());
        generate(&root.join("tests/sqlite_gen_param_utctime_s.rs"), vec![(0usize, v)], vec![
            // Queries
            new_insert(&bananna, vec![(hizat.clone(), Expr::Param {
                name: "val".into(),
                type_: hizat.type_.type_.clone(),
            })]).build_query("insert_banan", QueryResCount::None),
            new_select(&bananna).return_field(&hizat).build_query("get_banan", QueryResCount::One)
        ]).unwrap();
    }

    // # (insert) Param: datetime (seconds)
    {
        let mut v = Version::default();
        let bananna = v.table("zJCPRHK37", "bananna");
        let hizat = bananna.field(&mut v, "z437INV6D", "hizat", field_utctime_ms().build());
        generate(&root.join("tests/sqlite_gen_param_utctime_ms.rs"), vec![(0usize, v)], vec![
            // Queries
            new_insert(&bananna, vec![(hizat.clone(), Expr::Param {
                name: "val".into(),
                type_: hizat.type_.type_.clone(),
            })]).build_query("insert_banan", QueryResCount::None),
            new_select(&bananna).return_field(&hizat).build_query("get_banan", QueryResCount::One)
        ]).unwrap();
    }

    // # (insert) Param: Opt`<i32>`
    {
        let mut v = Version::default();
        let bananna = v.table("z8JI0I1E4", "bananna");
        let hizat = bananna.field(&mut v, "z437INV6D", "hizat", field_i32().opt().build());
        generate(&root.join("tests/sqlite_gen_param_opt_i32.rs"), vec![(0usize, v)], vec![
            // Queries
            new_insert(&bananna, vec![(hizat.clone(), Expr::Param {
                name: "val".into(),
                type_: hizat.type_.type_.clone(),
            })]).build_query("insert_banan", QueryResCount::None),
            new_select(&bananna).return_field(&hizat).build_query("get_banan", QueryResCount::One)
        ]).unwrap();
    }

    // # (insert) Param: Opt`<i32>`, null
    {
        let mut v = Version::default();
        let bananna = v.table("zT7F4746C", "bananna");
        let hizat = bananna.field(&mut v, "z437INV6D", "hizat", field_i32().opt().build());
        generate(&root.join("tests/sqlite_gen_param_opt_i32_null.rs"), vec![(0usize, v)], vec![
            // Queries
            new_insert(
                &bananna,
                vec![(hizat.clone(), Expr::LitNull(hizat.type_.type_.type_.clone()))],
            ).build_query("insert_banan", QueryResCount::None),
            new_select(&bananna).return_field(&hizat).build_query("get_banan", QueryResCount::One)
        ]).unwrap();
    }

    // # (select) Param: Array `<i32>`
    {
        let mut v = Version::default();
        let bananna = v.table("zT7F4746C", "bananna");
        let hizat = bananna.field(&mut v, "z437INV6D", "hizat", field_i32().build());
        generate(&root.join("tests/sqlite_gen_param_arr_i32.rs"), vec![(0usize, v)], vec![
            // Queries
            new_insert(&bananna, vec![set_field("hizat", &hizat)]).build_query("insert_banan", QueryResCount::None),
            new_select(&bananna).where_(Expr::BinOp {
                left: Box::new(Expr::field(&hizat)),
                op: BinOp::In,
                right: Box::new(Expr::Param {
                    name: "hizats".to_string(),
                    type_: type_i32().array().build(),
                }),
            }).return_field(&hizat).build_query("get_banan", QueryResCount::MaybeOne)
        ]).unwrap();
    }

    // # (insert) Param: All custom types
    {
        let mut v = Version::default();
        let bananna = v.table("zH2Q9TOLG", "bananna");
        let mut custom_fields = vec![];
        for (
            i,
            (schema_id, type_),
        ) in [
            ("zPZS1I5WW", field_bool().custom("integration_tests::MyBool").build()),
            ("zC06X4BAF", field_i32().custom("integration_tests::MyI32").build()),
            ("z9JQDQ8ZB", field_i64().custom("integration_tests::MyI64").build()),
            ("zM6UFN0J7", field_u32().custom("integration_tests::MyU32").build()),
            ("zMSGIBKUC", field_f32().custom("integration_tests::MyF32").build()),
            ("zQ23DTVF3", field_f64().custom("integration_tests::MyF64").build()),
            ("zV3TUIVTU", field_bytes().custom("integration_tests::MyBytes").build()),
            ("z7AJMBYHP", field_str().custom("integration_tests::MyString").build()),
            ("zCKQAR1KC", field_utctime_s().custom("integration_tests::MyUtctime").build()),
            ("z6BUG6P8R", field_utctime_ms().custom("integration_tests::MyUtctime").build()),
        ]
            .into_iter()
            .enumerate() {
            custom_fields.push(bananna.field(&mut v, schema_id, format!("x_{}", i), type_));
        }
        generate(&root.join("tests/sqlite_gen_param_custom.rs"), vec![(0usize, v)], vec![
            // Queries
            new_insert(
                &bananna,
                custom_fields.iter().map(|f| set_field(&f.id, f)).collect(),
            ).build_query("insert_banan", QueryResCount::None),
            new_select(&bananna)
                .return_fields(&custom_fields.iter().map(|f| f).collect::<Vec<&Field>>())
                .build_query("get_banan", QueryResCount::One)
        ]).unwrap();
    }

    // # (insert) Param: Opt`<Custom>`
    {
        let mut v = Version::default();
        let bananna = v.table("z202QTVDB", "bananna");
        let hizat =
            bananna.field(
                &mut v,
                "z437INV6D",
                "hizat",
                field_str().custom("integration_tests::MyString").opt().build(),
            );
        generate(&root.join("tests/sqlite_gen_param_opt_custom.rs"), vec![(0usize, v)], vec![
            // Queries
            new_insert(&bananna, vec![(hizat.clone(), Expr::Param {
                name: "text".into(),
                type_: hizat.type_.type_.clone(),
            })]).build_query("insert_banan", QueryResCount::None),
            new_select(&bananna).return_field(&hizat).build_query("get_banan", QueryResCount::One)
        ]).unwrap();
    }

    // # Insert on conflict do nothing
    {
        let mut v = Version::default();
        let bananna = v.table("zEOIWAACJ", "bannanana");
        let hizat = bananna.field(&mut v, "z437INV6D", "hizat", field_str().build());
        bananna.index("zPRVXKY6D", "all", &[&hizat]).unique().build(&mut v);
        generate(
            &root.join("tests/sqlite_gen_insert_on_conflict_do_nothing.rs"),
            vec![(0usize, v)],
            vec![
                new_insert(&bananna, vec![(hizat.clone(), Expr::Param {
                    name: "text".into(),
                    type_: hizat.type_.type_.clone(),
                })])
                    .return_named("one", Expr::LitI32(1))
                    .on_conflict(InsertConflict::DoNothing)
                    .build_query("insert_banan", QueryResCount::MaybeOne)
            ],
        ).unwrap();
    }

    // # Insert on conflict update
    {
        let mut v = Version::default();
        let bananna = v.table("zEOIWAACJ", "bannanana");
        let hizat = bananna.field(&mut v, "z437INV6D", "hizat", field_str().build());
        let two = bananna.field(&mut v, "z3AL5J609", "two", field_i32().build());
        bananna.index("zPRVXKY6D", "all", &[&hizat]).unique().build(&mut v);
        generate(
            &root.join("tests/sqlite_gen_insert_on_conflict_update.rs"),
            vec![(0usize, v)],
            vec![new_insert(&bananna, vec![(hizat.clone(), Expr::Param {
                name: "text".into(),
                type_: hizat.type_.type_.clone(),
            }), (two.clone(), Expr::Param {
                name: "two".into(),
                type_: two.type_.type_.clone(),
            })]).return_field(&two).on_conflict(InsertConflict::DoUpdate(vec![(two.clone(), Expr::BinOp {
                left: Box::new(Expr::Binding(Binding::field(&two))),
                op: BinOp::Plus,
                right: Box::new(Expr::LitI32(1)),
            })])).build_query("insert_banan", QueryResCount::One)],
        ).unwrap();
    }

    // # Insert pass return 1
    //
    // # Insert fail return 1
    //
    // # Insert pass return maybe 1
    //
    // # Insert fail return maybe 1
    //
    // # Insert pass return none
    //
    // # Insert fail return none
    //
    // # Update
    {
        let mut v = Version::default();
        let bananna = v.table("zSPEZNHA8", "bananna");
        let hizat = bananna.field(&mut v, "z437INV6D", "hizat", field_str().build());
        generate(&root.join("tests/sqlite_gen_update.rs"), vec![(0usize, v)], vec![
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
        let mut v = Version::default();
        let bananna = v.table("zSPEZNHA8", "ban");
        let hizat = bananna.field(&mut v, "z437INV6D", "hizat", field_str().build());
        generate(&root.join("tests/sqlite_gen_update_where.rs"), vec![(0usize, v)], vec![
            // Queries
            new_insert(
                &bananna,
                vec![(hizat.clone(), Expr::LitString("yog".into()))],
            ).build_query("insert_banan", QueryResCount::None),
            new_select(&bananna).return_field(&hizat).build_query("get_banan", QueryResCount::One),
            new_update(&bananna, vec![(hizat.clone(), Expr::Param {
                name: "val".into(),
                type_: hizat.type_.type_.clone(),
            })]).where_(Expr::BinOp {
                left: Box::new(Expr::Binding(Binding::field(&hizat))),
                op: BinOp::Equals,
                right: Box::new(Expr::Param {
                    name: "cond".into(),
                    type_: hizat.type_.type_.clone(),
                }),
            }).build_query("update_banan", QueryResCount::None)
        ]).unwrap();
    }

    // # Update, returning
    {
        let mut v = Version::default();
        let bananna = v.table("zSPEZNHA8", "b");
        let hizat = bananna.field(&mut v, "z437INV6D", "hizat", field_str().build());
        generate(&root.join("tests/sqlite_gen_update_returning.rs"), vec![(0usize, v)], vec![
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
        let mut v = Version::default();
        let bananna = v.table("zLBDEHGRB", "b");
        let hizat = bananna.field(&mut v, "z437INV6D", "hizat", field_str().build());
        generate(&root.join("tests/sqlite_gen_delete.rs"), vec![(0usize, v)], vec![
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
        let mut v = Version::default();
        let bananna = v.table("zLBDEHGRB", "ba");
        let hizat = bananna.field(&mut v, "z437INV6D", "hizat", field_str().build());
        generate(&root.join("tests/sqlite_gen_delete_where.rs"), vec![(0usize, v)], vec![
            // Queries
            new_insert(
                &bananna,
                vec![(hizat.clone(), Expr::LitString("seeon".into()))],
            ).build_query("insert_banan", QueryResCount::None),
            new_select(&bananna).return_field(&hizat).build_query("get_banan", QueryResCount::MaybeOne),
            new_delete(&bananna).where_(Expr::BinOp {
                left: Box::new(Expr::Binding(Binding::field(&hizat))),
                op: BinOp::Equals,
                right: Box::new(Expr::Param {
                    name: "hiz".into(),
                    type_: hizat.type_.type_.clone(),
                }),
            }).build_query("no_banan", QueryResCount::None)
        ]).unwrap();
    }

    // # Delete, returning
    {
        let mut v = Version::default();
        let bananna = v.table("zLBDEHGRB", "b");
        let hizat = bananna.field(&mut v, "z437INV6D", "hizat", field_str().build());
        generate(&root.join("tests/sqlite_gen_delete_returning.rs"), vec![(0usize, v)], vec![
            // Queries
            new_insert(
                &bananna,
                vec![(hizat.clone(), Expr::LitString("seeon".into()))],
            ).build_query("insert_banan", QueryResCount::None),
            new_delete(&bananna).return_field(&hizat).build_query("no_banan", QueryResCount::One)
        ]).unwrap();
    }

    // # (select) Return: record
    //
    // # (select) Return: one
    //
    // # (select) Return: maybe one (non-opt)
    //
    // # (select) Return: maybe one (opt)
    //
    // # (select) Return: many
    //
    // # (select) Return: rename
    //
    // # (select) Return: rename (err, not record)
    //
    // # Select + join
    {
        let mut v = Version::default();
        let bananna = v.table("zT6D0LWI8", "b");
        let hizat = bananna.field(&mut v, "z437INV6D", "hizat", field_str().build());
        let three = bananna.field(&mut v, "zVXQUXEXT", "three", field_i32().build());
        let one = v.table("zQ8SFVHEV", "two");
        let hizat1 = one.field(&mut v, "zDZA6FVSS", "hizat", field_str().build());
        let two = one.field(&mut v, "z7KU525LW", "two", field_str().build());
        v.post_migration(
            new_insert(
                &bananna,
                vec![(hizat.clone(), Expr::LitString("key".into())), (three.clone(), Expr::LitI32(33))],
            ).build_migration(),
        );
        v.post_migration(
            new_insert(
                &one,
                vec![(hizat1.clone(), Expr::LitString("key".into())), (two.clone(), Expr::LitString("no".into()))],
            ).build_migration(),
        );
        generate(
            &root.join("tests/sqlite_gen_select_join.rs"),
            vec![(0usize, v)],
            vec![new_select(&bananna).join(Join {
                source: Box::new(NamedSelectSource {
                    source: JoinSource::Table(one.clone()),
                    alias: None,
                }),
                type_: JoinType::Left,
                on: Expr::BinOp {
                    left: Box::new(Expr::Binding(Binding::field(&hizat))),
                    op: BinOp::Equals,
                    right: Box::new(Expr::Binding(Binding::field(&hizat1))),
                },
            }).return_field(&three).return_field(&two).build_query("get_it", QueryResCount::One)],
        ).unwrap();
    }

    // # Select limit
    {
        let mut v = Version::default();
        let bananna = v.table("zEOIWAACJ", "bannanana");
        let hizat = bananna.field(&mut v, "z437INV6D", "hizat", field_str().build());
        generate(&root.join("tests/sqlite_gen_select_limit.rs"), vec![(0usize, v)], vec![
            // Queries
            new_insert(&bananna, vec![(hizat.clone(), Expr::Param {
                name: "text".into(),
                type_: hizat.type_.type_.clone(),
            })]).build_query("insert_banan", QueryResCount::None),
            new_select(&bananna)
                .return_field(&hizat)
                .limit(Expr::LitI32(2))
                .build_query("get_banan", QueryResCount::Many)
        ]).unwrap();
    }

    // # Select order
    {
        let mut v = Version::default();
        let bananna = v.table("zEOIWAACJ", "bannanana");
        let hizat = bananna.field(&mut v, "z437INV6D", "hizat", field_i32().build());
        generate(&root.join("tests/sqlite_gen_select_order.rs"), vec![(0usize, v)], vec![
            // Queries
            new_insert(&bananna, vec![(hizat.clone(), Expr::Param {
                name: "v".into(),
                type_: hizat.type_.type_.clone(),
            })]).build_query("insert_banan", QueryResCount::None),
            new_select(&bananna)
                .return_field(&hizat)
                .order(Expr::Binding(Binding::field(&hizat)), Order::Asc)
                .build_query("get_banan", QueryResCount::Many)
        ]).unwrap();
    }

    // # Select group
    {
        let mut v = Version::default();
        let bananna = v.table("zEOIWAACJ", "bannanana");
        let hizat = bananna.field(&mut v, "z437INV6D", "hizat", field_i32().build());
        let hizat2 = bananna.field(&mut v, "z3CRAVV3M", "hizat2", field_i32().build());
        generate(&root.join("tests/sqlite_gen_select_group_by.rs"), vec![(0usize, v)], vec![
            // Queries
            new_insert(&bananna, vec![(hizat.clone(), Expr::Param {
                name: "v".into(),
                type_: hizat.type_.type_.clone(),
            }), (hizat2.clone(), Expr::Param {
                name: "v2".into(),
                type_: hizat2.type_.type_.clone(),
            })]).build_query("insert_banan", QueryResCount::None),
            new_select(&bananna).return_named("hizat2", Expr::Call {
                func: "sum".into(),
                args: vec![Expr::Binding(Binding::field(&hizat2))],
                compute_type: ComputeType::new(|ctx, path, args| {
                    shed!{
                        if args.len() != 1 {
                            ctx.errs.err(path, format!("Sum needs exactly one arg, got {}", args.len()));
                        }
                        let Some(arg) = args.iter().next() else {
                            break;
                        };
                        if arg.0.len() != 1 {
                            ctx
                                .errs
                                .err(
                                    path,
                                    format!(
                                        "Sum argument must be a primitive, but got a record with {} elements",
                                        arg.0.len()
                                    ),
                                );
                        }
                        let Some(type_) = arg.0.iter().next() else {
                            break;
                        };
                        if !match type_.1.type_.type_ {
                            good_ormning::sqlite::types::SimpleSimpleType::U32 |
                            good_ormning::sqlite::types::SimpleSimpleType::I32 |
                            good_ormning::sqlite::types::SimpleSimpleType::I64 |
                            good_ormning::sqlite::types::SimpleSimpleType::F32 |
                            good_ormning::sqlite::types::SimpleSimpleType::F64 => true,
                            _ => false,
                        } {
                            ctx
                                .errs
                                .err(
                                    path,
                                    format!("Sum argument must be a numeric type, but fot {:?}", type_.1.type_.type_),
                                );
                        }
                    };
                    return Some(type_i32().build());
                }),
            }).group(vec![Expr::Binding(Binding::field(&hizat))]).build_query("get_banan", QueryResCount::Many)
        ]).unwrap();
    }

    // # Migrate - add field
    {
        let mut v = Version::default();
        let bananna = v.table("zTWA93SX0", "bannna");
        let hizat = bananna.field(&mut v, "z437INV6D", "hizat", field_str().build());
        let zomzom =
            bananna.field(&mut v, "zPREUVAOD", "zomzom", field_bool().migrate_fill(Expr::LitBool(true)).build());
        generate(&root.join("tests/sqlite_gen_migrate_add_field.rs"), vec![
            // Versions (previous)
            (0usize, {
                let mut v = Version::default();
                let bananna = v.table("zTWA93SX0", "bannna");
                let hizat = bananna.field(&mut v, "z437INV6D", "hizat", field_str().build());
                v.post_migration(
                    new_insert(&bananna, vec![(hizat.clone(), Expr::LitString("nizoot".into()))]).build_migration(),
                );
                v
            }),
            (1usize, v)
        ], vec![
            // Queries
            new_select(&bananna).return_fields(&[&hizat, &zomzom]).build_query("get_banan", QueryResCount::MaybeOne)
        ]).unwrap();
    }

    // # Migrate - rename field
    {
        let mut v = Version::default();
        let bananna = v.table("zTWA93SX0", "bannna");
        let hizat = bananna.field(&mut v, "z437INV6D", "hizat", field_str().build());
        generate(&root.join("tests/sqlite_gen_migrate_rename_field.rs"), vec![
            // Versions (previous)
            (0usize, {
                let mut v = Version::default();
                let bananna = v.table("zTWA93SX0", "bannna");
                bananna.field(&mut v, "z437INV6D", "hozot", field_str().build());
                v
            }),
            (1usize, v)
        ], vec![
            // Queries
            new_insert(
                &bananna,
                vec![(hizat.clone(), Expr::LitString("nizoot".into()))],
            ).build_query("ins", QueryResCount::None)
        ]).unwrap();
    }

    // # Migrate - remove field
    {
        let mut v = Version::default();
        let bananna = v.table("z1MD8L1CZ", "bnanaa");
        let hizat = bananna.field(&mut v, "z437INV6D", "hizat", field_str().build());
        generate(&root.join("tests/sqlite_gen_migrate_remove_field.rs"), vec![
            // Versions (previous)
            (0usize, {
                let mut v = Version::default();
                let bananna = v.table("z1MD8L1CZ", "bnanaa");
                bananna.field(&mut v, "z437INV6D", "hizat", field_str().build());
                bananna.field(&mut v, "zPREUVAOD", "zomzom", field_bool().build());
                v
            }),
            (1usize, v)
        ], vec![
            // Queries
            new_insert(&bananna, vec![(hizat.clone(), Expr::Param {
                name: "okolor".into(),
                type_: hizat.type_.type_.clone(),
            })]).build_query("new_banan", QueryResCount::None)
        ]).unwrap();
    }

    // # Migrate - add table
    {
        let mut v = Version::default();
        let bananna = v.table("z4RGW742J", "bnanana");
        bananna.field(&mut v, "z437INV6D", "hizat", field_str().build());
        let two = v.table("zHXF3YVGQ", "two");
        let field_two = two.field(&mut v, "z156A4Q8W", "two", field_i32().build());
        generate(&root.join("tests/sqlite_gen_migrate_add_table.rs"), vec![
            // Versions (previous)
            (0usize, {
                let mut v = Version::default();
                let bananna = v.table("z4RGW742J", "bnanana");
                bananna.field(&mut v, "z437INV6D", "hizat", field_str().build());
                v
            }),
            (1usize, v)
        ], vec![
            // Queries
            new_insert(&two, vec![(field_two.clone(), Expr::Param {
                name: "two".into(),
                type_: field_two.type_.type_.clone(),
            })]).build_query("two", QueryResCount::None)
        ]).unwrap();
    }

    // # Migrate - rename table
    {
        let mut v = Version::default();
        let bananna = v.table("z4RGW742J", "bana");
        let hizat = bananna.field(&mut v, "z437INV6D", "hizat", field_str().build());
        generate(&root.join("tests/sqlite_gen_migrate_rename_table.rs"), vec![
            // Versions (previous)
            (0usize, {
                let mut v = Version::default();
                let bananna = v.table("z4RGW742J", "bnanana");
                bananna.field(&mut v, "z437INV6D", "hizat", field_str().build());
                v
            }),
            (1usize, v)
        ], vec![
            // Queries
            new_insert(&bananna, vec![(hizat.clone(), Expr::Param {
                name: "two".into(),
                type_: hizat.type_.type_.clone(),
            })]).build_query("two", QueryResCount::None)
        ]).unwrap();
    }

    // # Migrate - remove table
    {
        let mut v = Version::default();
        let bananna = v.table("zX7CEK8JC", "bananana");
        bananna.field(&mut v, "z437INV6D", "hizat", field_str().build());
        generate(&root.join("tests/sqlite_gen_migrate_remove_table.rs"), vec![
            // Versions (previous)
            (0usize, {
                let mut v = Version::default();
                let bananna = v.table("zX7CEK8JC", "bananana");
                bananna.field(&mut v, "z437INV6D", "hizat", field_str().build());
                let two = v.table("z45HT1YW2", "two");
                two.field(&mut v, "z156A4Q8W", "two", field_i32().build());
                v
            }),
            (1usize, v)
        ], vec![]).unwrap();
    }

    // # Migrate - remove index
    //
    // # Migrate - add primary constraint
    //
    // # Migrate - remove primary constraint
    //
    // # Migrate - add fk constraint
    //
    // # Migrate - pre migration
    {
        let mut v0 = Version::default();
        let v0_bananna = v0.table("zMI5V9F2V", "v0_banana");
        v0_bananna.field(&mut v0, "z437INV6D", "hizat", field_str().build());
        let v0_two = v0.table("z450WBJCO", "v0_two");
        let v0_field_two = v0_two.field(&mut v0, "z156A4Q8W", "two", field_i32().build());
        let mut v1 = Version::default();
        v1.pre_migration(new_insert(&v0_two, vec![(v0_field_two.clone(), Expr::LitI32(7))]).build_migration());
        let v1_bananna = v1.table("zMI5V9F2V", "v0_banana");
        v1_bananna.field(&mut v1, "z437INV6D", "hizat", field_str().build());
        generate(&root.join("tests/sqlite_gen_migrate_pre_migration.rs"), vec![
            // Versions (previous)
            (0usize, v0),
            (1usize, v1)
        ], vec![]).unwrap();
    }

    // # CTE
    {
        let mut v = Version::default();
        let bananna = v.table("zEOIWAACJ", "bannanana");
        let hizat = bananna.field(&mut v, "z437INV6D", "hizat", field_i32().build());
        let hizat2 = bananna.field(&mut v, "z3CRAVV3M", "hizat2", field_i32().build());
        let mut hibbo = CteBuilder::new("hibbo", new_select_body(&bananna).return_field(&hizat2).build());
        let zathi = hibbo.field("zathi", hizat2.0.type_.type_.clone());
        let (hibbo, hibbo_cte) = hibbo.build();
        generate(&root.join("tests/sqlite_gen_select_cte.rs"), vec![(0usize, v)], vec![
            // Queries
            new_insert(
                &bananna,
                vec![set_field("v", &hizat), set_field("v2", &hizat2)],
            ).build_query("insert_banan", QueryResCount::None),
            new_select(&hibbo).with(With {
                recursive: false,
                ctes: vec![hibbo_cte],
            }).return_(Expr::Binding(Binding::field(&zathi))).build_query("get_banan", QueryResCount::Many)
        ]).unwrap();
    }

    // # Window function
    {
        let mut v = Version::default();
        let bananna = v.table("zEOIWAACJ", "bannanana");
        let hizat = bananna.field(&mut v, "z437INV6D", "hizat", field_i32().build());
        let hizat2 = bananna.field(&mut v, "z3CRAVV3M", "hizat2", field_i32().build());
        generate(&root.join("tests/sqlite_gen_select_window.rs"), vec![(0usize, v)], vec![
            // Queries
            new_insert(
                &bananna,
                vec![set_field("v", &hizat), set_field("v2", &hizat2)],
            ).build_query("insert_banan", QueryResCount::None),
            new_select(&bananna).return_field(&hizat).return_field(&hizat2).return_named("zombo", Expr::Window {
                expr: Box::new(fn_max(Expr::Binding(Binding::field(&hizat2)))),
                partition_by: vec![],
                order_by: vec![],
            }).build_query("get_banan", QueryResCount::Many)
        ]).unwrap();
    }

    // # Junction
    {
        let mut v = Version::default();
        let bananna = v.table("zEOIWAACJ", "bannanana");
        let hizat = bananna.field(&mut v, "z437INV6D", "hizat", field_i32().build());
        let hizat2 = bananna.field(&mut v, "z3CRAVV3M", "hizat2", field_i32().build());
        generate(&root.join("tests/sqlite_gen_select_junction.rs"), vec![(0usize, v)], vec![
            // Queries
            new_insert(
                &bananna,
                vec![set_field("v", &hizat), set_field("v2", &hizat2)],
            ).build_query("insert_banan", QueryResCount::None),
            new_select(&bananna).return_field(&hizat).junction(SelectJunction {
                op: good_ormning::sqlite::query::select_body::SelectJunctionOperator::Union,
                body: new_select_body(&bananna).return_field(&hizat2).build(),
            }).build_query("get_banan", QueryResCount::Many)
        ]).unwrap();
    }
}
