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
                    field_bool,
                    field_utctime_chrono,
                    field_utctime_jiff,
                    field_auto,
                    field_i64,
                    field_f32,
                    field_f64,
                    field_bytes,
                },
                table::{
                    SchemaTableId,
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
                helpers::set_field,
            },
            generate,
            new_insert,
            QueryResCount,
            new_select,
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
    f.table.version.0.borrow().tables.get(&f.table.schema_id).unwrap().fields.get(&f.schema_id).unwrap().type_.type_.clone()
}

pub fn build(root: &Path) {
    // # Base: create table, insert, select
    {
        let v = VersionHandle::new();
        let bananna = v.table("zEOIWAACJ", "bannanana");
        let hizat = 
        generate(&root.join("tests/pg_gen_base_insert.rs"), vec![(0usize, v.0.borrow().clone())], vec![
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
        let v = VersionHandle::new();
        let bananna = v.table("zJCPRHK37", "bananna");
        let hizat = bananna.field("z437INV6D", "hizat", field_i32().build());
        generate(&root.join("tests/pg_gen_param_i32.rs"), vec![(0usize, v.0.borrow().clone())], vec![
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
        let v = VersionHandle::new();
        let bananna = v.table("zJCPRHK37", "bananna");
        let hizat = bananna.field("z437INV6D", "hizat", field_utctime_chrono().build());
        generate(&root.join("tests/pg_gen_param_utctime_chrono.rs"), vec![(0usize, v.0.borrow().clone())], vec![
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
        let v = VersionHandle::new();
        let bananna = v.table("zJCPRHK37", "bananna");
        let hizat = bananna.field("z437INV6D", "hizat", field_utctime_jiff().build());
        generate(&root.join("tests/pg_gen_param_utctime_jiff.rs"), vec![(0usize, v.0.borrow().clone())], vec![
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
        let v = VersionHandle::new();
        let bananna = v.table("z8JI0I1E4", "bananna");
        let hizat = bananna.field("z437INV6D", "hizat", field_i32().opt().build());
        generate(&root.join("tests/pg_gen_param_opt_i32.rs"), vec![(0usize, v.0.borrow().clone())], vec![
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
        let v = VersionHandle::new();
        let bananna = v.table("zT7F4746C", "bananna");
        let hizat = bananna.field("z437INV6D", "hizat", field_i32().opt().build());
        generate(&root.join("tests/pg_gen_param_opt_i32_null.rs"), vec![(0usize, v.0.borrow().clone())], vec![
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
        let v = VersionHandle::new();
        let bananna = v.table("zH2Q9TOLG", "bananna");
        let mut custom_fields = vec![];
        for (
            i,
            (schema_id, type_),
        ) in [
            ("zPZS1I5WW", field_bool().custom("integration_tests::MyBool").build()),
            ("zC06X4BAF", field_i32().custom("integration_tests::MyI32").build()),
            ("z9JQDQ8ZB", field_i64().custom("integration_tests::MyI64").build()),
            ("zMSGIBKUC", field_f32().custom("integration_tests::MyF32").build()),
            ("zQ23DTVF3", field_f64().custom("integration_tests::MyF64").build()),
            ("zV3TUIVTU", field_bytes().custom("integration_tests::MyBytes").build()),
            ("z7AJMBYHP", field_str().custom("integration_tests::MyString").build()),
            ("zCKQAR1KC", field_utctime_chrono().custom("integration_tests::MyUtctimeChrono").build()),
            ("zNDD21YUS", field_utctime_jiff().custom("integration_tests::MyUtctimeJiff").build()),
        ]
            .into_iter()
            .enumerate() {
            custom_fields.push(bananna.field(schema_id, &format!("x_{}", i), type_));
        }
        generate(&root.join("tests/pg_gen_param_custom.rs"), vec![(0usize, v.0.borrow().clone())], vec![
            // Queries
            new_insert(
                &bananna,
                custom_fields.iter().map(|f| set_field(&f.table.version.0.borrow().tables.get(&f.table.schema_id).unwrap().fields.get(&f.schema_id).unwrap().id.clone(), f)).collect(),
            ).build_query("insert_banan", QueryResCount::None),
            new_select(&bananna)
                .return_fields(&custom_fields.iter().collect::<Vec<&FieldHandle>>())
                .build_query("get_banan", QueryResCount::One)
        ]).unwrap();
    }

    // # (insert) Param: Opt`<Custom>`
    {
        let v = VersionHandle::new();
        let bananna = v.table("z202QTVDB", "bananna");
        let hizat =
            bananna.field(
                "z437INV6D",
                "hizat",
                field_str().custom("integration_tests::MyString").opt().build(),
            );
        generate(&root.join("tests/pg_gen_param_opt_custom.rs"), vec![(0usize, v.0.borrow().clone())], vec![
            // Queries
            new_insert(&bananna, vec![(hizat.clone(), Expr::Param {
                name: "text".into(),
                type_: get_type(&hizat),
            })]).build_query("insert_banan", QueryResCount::None),
            new_select(&bananna).return_field(&hizat).build_query("get_banan", QueryResCount::One)
        ]).unwrap();
    }

    // # Insert on conflict update
    {
        let v = VersionHandle::new();
        let bananna = v.table("zEOIWAACJ", "bannanana");
        let hizat = 
        bananna.field("z3AL5J609", "two", field_i32().build()); let two = TableHandle { version: v.clone(), schema_id: bananna.schema_id.clone() }.field("z3AL5J609", "two", field_i32().build());
        bananna.unique_index("zPRVXKY6D", "all", &[&hizat]);
        generate(&root.join("tests/pg_gen_insert_on_conflict_update.rs"), vec![(0usize, v.0.borrow().clone())], vec![
            new_insert(&bananna, vec![(hizat.clone(), Expr::Param {
                name: "text".into(),
                type_: get_type(&hizat),
            }), (two.clone(), Expr::Param {
                name: "two".into(),
                type_: get_type(&two),
            })]).return_field(&two).on_conflict_do_update(&[&hizat], vec![(two.clone(), Expr::BinOp {
                left: Box::new(Expr::Field(two.to_ref())),
                op: BinOp::Plus,
                right: Box::new(Expr::LitI32(1)),
            })]).build_query("insert_banan", QueryResCount::One)
        ]).unwrap();
    }

    // # Update
    {
        let v = VersionHandle::new();
        let bananna = v.table("zSPEZNHA8", "bananna");
        let hizat = 
        generate(&root.join("tests/pg_gen_update.rs"), vec![(0usize, v.0.borrow().clone())], vec![
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
        let v = VersionHandle::new();
        let bananna = v.table("zSPEZNHA8", "ban");
        let hizat = 
        generate(&root.join("tests/pg_gen_update_where.rs"), vec![(0usize, v.0.borrow().clone())], vec![
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
        let v = VersionHandle::new();
        let bananna = v.table("zSPEZNHA8", "b");
        let hizat = 
        generate(&root.join("tests/pg_gen_update_returning.rs"), vec![(0usize, v.0.borrow().clone())], vec![
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
        let v = VersionHandle::new();
        let bananna = v.table("zLBDEHGRB", "b");
        let hizat = 
        generate(&root.join("tests/pg_gen_delete.rs"), vec![(0usize, v.0.borrow().clone())], vec![
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
        let v = VersionHandle::new();
        let bananna = v.table("zLBDEHGRB", "ba");
        let hizat = 
        generate(&root.join("tests/pg_gen_delete_where.rs"), vec![(0usize, v.0.borrow().clone())], vec![
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
        let v = VersionHandle::new();
        let bananna = v.table("zLBDEHGRB", "b");
        let hizat = 
        generate(&root.join("tests/pg_gen_delete_returning.rs"), vec![(0usize, v.0.borrow().clone())], vec![
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
        let v = VersionHandle::new();
        let bananna = v.table("zT6D0LWI8", "b");
        let hizat = 
        let three = bananna.field("zVXQUXEXT", "three", field_i32().build());
        let one = v.table("zQ8SFVHEV", "two");
        let hizat1 = one.field("zDZA6FVSS", "hizat", field_str().build());
        let two = one.field("z7KU525LW", "two", field_str().build());
        generate(
            &root.join("tests/pg_gen_select_join.rs"),
            vec![(0usize, v.0.borrow().clone())],
            vec![new_select(&bananna).join(Join {
                source: Box::new(NamedSelectSource {
                    source: JoinSource::Table(one.to_ref()),
                    alias: None,
                }),
                type_: JoinType::Left,
                on: Expr::BinOp {
                    left: Box::new(Expr::Field(hizat.to_ref())),
                    op: BinOp::Equals,
                    right: Box::new(Expr::Field(hizat1.to_ref())),
                },
            }).return_field(&three).return_field(&two).build_query("get_it", QueryResCount::One)],
        ).unwrap();
    }

    // # Select limit
    {
        let v = VersionHandle::new();
        let bananna = v.table("zEOIWAACJ", "bannanana");
        let hizat = 
        generate(&root.join("tests/pg_gen_select_limit.rs"), vec![(0usize, v.0.borrow().clone())], vec![
            // Queries
            new_insert(&bananna, vec![(hizat.clone(), Expr::Param {
                name: "text".into(),
                type_: get_type(&hizat),
            })]).build_query("insert_banan", QueryResCount::None),
            new_select(&bananna)
                .return_field(&hizat)
                .limit(Expr::LitI32(2))
                .build_query("get_banan", QueryResCount::Many)
        ]).unwrap();
    }

    // # Select order
    {
        let v = VersionHandle::new();
        let bananna = v.table("zEOIWAACJ", "bannanana");
        let hizat = 
        generate(&root.join("tests/pg_gen_select_order.rs"), vec![(0usize, v.0.borrow().clone())], vec![
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
        let v = VersionHandle::new();
        let bananna = v.table("zEOIWAACJ", "bannanana");
        let hizat = bananna.field("z437INV6D", "hizat", field_i32().build());
        let hizat2 = bananna.field("z3CRAVV3M", "hizat2", field_i32().build());
        generate(&root.join("tests/pg_gen_select_group_by.rs"), vec![(0usize, v.0.borrow().clone())], vec![
            // Queries
            new_insert(&bananna, vec![(hizat.clone(), Expr::Param {
                name: "v".into(),
                type_: get_type(&hizat),
            }), (hizat2.clone(), Expr::Param {
                name: "v2".into(),
                type_: get_type(&hizat2),
            })]).build_query("insert_banan", QueryResCount::None),
            new_select(&bananna).return_named("hizat2", Expr::Call {
                func: "sum".into(),
                args: vec![Expr::Field(hizat2.to_ref())],
                compute_type: ComputeType(Rc::new(|ctx, path, args| {
                    shed!{
                        if args.len() != 1 {
                            ctx.errs.err(path, format!("Sum needs exactly one arg, got {}", args.len()));
                        }
                        let Some(arg) = args.iter().next() else {
                            break;
                        };
                        let Some(type_) = arg.assert_scalar(&mut ctx.errs, path) else {
                            break;
                        };
                    };
                    return ExprType(vec![(ExprValName::empty(), type_i32().build())]);
                })),
            }).group(vec![Expr::Field(hizat.to_ref())]).build_query("get_banan", QueryResCount::Many)
        ]).unwrap();
    }

    // # Migrate - add field
    {
        let v = VersionHandle::new();
        let bananna = v.table("zTWA93SX0", "bannna");
        let hizat = 
        let zomzom =
            bananna.field("zPREUVAOD", "zomzom", field_bool().migrate_fill(Expr::LitBool(true)).build());
        generate(&root.join("tests/pg_gen_migrate_add_field.rs"), vec![
            // Versions (previous)
            (0usize, {
                let v = VersionHandle::new();
                let bananna = v.table("zTWA93SX0", "bannna");
                let _hizat = 
                v.0.borrow().clone()
            }),
            (1usize, v.0.borrow().clone())
        ], vec![
            // Queries
            new_select(&bananna).return_fields(&[&hizat, &zomzom]).build_query("get_banan", QueryResCount::MaybeOne)
        ]).unwrap();
    }

    // # Migrate - rename field
    {
        let v = VersionHandle::new();
        let bananna = v.table("zTWA93SX0", "bannna");
        let hizat = 
        generate(&root.join("tests/pg_gen_migrate_rename_field.rs"), vec![
            // Versions (previous)
            (0usize, {
                let v = VersionHandle::new();
                let bananna = v.table("zTWA93SX0", "bannna");
                let _hozot = bananna.field("z437INV6D", "hozot", field_str().build());
                v.0.borrow().clone()
            }),
            (1usize, v.0.borrow().clone())
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
        let v = VersionHandle::new();
        let bananna = v.table("z1MD8L1CZ", "bnanaa");
        let hizat = 
        generate(&root.join("tests/pg_gen_migrate_remove_field.rs"), vec![
            // Versions (previous)
            (0usize, {
                let v = VersionHandle::new();
                let bananna = v.table("z1MD8L1CZ", "bnanaa");
                
                bananna.field("zPREUVAOD", "zomzom", field_bool().build());
                v.0.borrow().clone()
            }),
            (1usize, v.0.borrow().clone())
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
        let v = VersionHandle::new();
        let bananna = v.table("z4RGW742J", "bnanana");
        bananna.field("z437INV6D", "hizat", field_str().build());
        
        let two = v.table("zHXF3YVGQ", "two");
        let field_two = two.field("z156A4Q8W", "two", field_i32().build());
        generate(&root.join("tests/pg_gen_migrate_add_table.rs"), vec![
            // Versions (previous)
            (0usize, {
                let v = VersionHandle::new();
                let bananna = v.table("z4RGW742J", "bnanana");
        bananna.field("z437INV6D", "hizat", field_str().build());
                
                v.0.borrow().clone()
            }),
            (1usize, v.0.borrow().clone())
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
        let v = VersionHandle::new();
        let bananna = v.table("z4RGW742J", "bana");
        let hizat = 
        generate(&root.join("tests/pg_gen_migrate_rename_table.rs"), vec![
            // Versions (previous)
            (0usize, {
                let v = VersionHandle::new();
                let bananna = v.table("z4RGW742J", "bnanana");
        bananna.field("z437INV6D", "hizat", field_str().build());
                
                v.0.borrow().clone()
            }),
            (1usize, v.0.borrow().clone())
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
        let v = VersionHandle::new();
        let bananna = v.table("zX7CEK8JC", "bananana");
        
        generate(&root.join("tests/pg_gen_migrate_remove_table.rs"), vec![
            // Versions (previous)
            (0usize, {
                let v = VersionHandle::new();
                let bananna = v.table("zX7CEK8JC", "bananana");
                
                let two = v.table("z45HT1YW2", "two");
                two.field("z156A4Q8W", "two", field_i32().build());
                v.0.borrow().clone()
            }),
            (1usize, v.0.borrow().clone())
        ], vec![]).unwrap();
    }
}
