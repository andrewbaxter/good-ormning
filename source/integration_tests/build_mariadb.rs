use good_ormning::{
    MariadbFieldTypeBuilder as FieldTypeBuilder,
    mariadb::{
        Version as MariadbVersion,
        generate,
        GenerateArgs,
        new_select_body,
        query::utils::CteBuilder,
        schema::field::{
            field_bool,
            field_f64,
            field_i32,
            field_i64,
            field_str,
            field_u32,
            field_datetime_chrono,
            field_datetime_jiff,
            field_date_chrono,
            field_date_jiff,
            field_time_chrono,
            field_time_jiff,
            field_timestamp_jiff,
            field_year,
            field_decimal,
            field_char,
            field_mediumint,
            field_text,
            field_u64,
            field_i8,
            field_f32,
            field_bytes,
        },
    },
    mariadb_type_bool as type_bool,
    mariadb_type_bytes as type_bytes,
    mariadb_type_f32 as type_f32,
    mariadb_type_f64 as type_f64,
    mariadb_type_i32 as type_i32,
    mariadb_type_i64 as type_i64,
    mariadb_type_str as type_str,
    mariadb_type_u32 as type_u32,
    mariadb_type_datetime_chrono as type_datetime_chrono,
    mariadb_type_datetime_jiff as type_datetime_jiff,
};

pub fn build() {
    {
        use good_ormning::{
            QueryResCount,
            mariadb::{
                new_select,
                query::{
                    expr::Expr,
                    helpers::in_field,
                    select::Order,
                },
            },
        };

        let v = MariadbVersion::new();
        let t = v.table("v");
        t.field("ti", field_i8().opt().build());
        t.field("iu", field_u32().opt().build());
        t.field("bi", field_i64().opt().build());
        t.field("bu", field_u64().opt().build());
        t.field("f", field_f32().opt().build());
        t.field("de", field_decimal().precision(30).scale(20).opt().build());
        t.field("b", field_bytes().opt().build());
        let s = t.field("s", field_str().opt().build());
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_safety".to_string()),
            versions: vec![(1usize, v.build())],
            queries: vec![
                new_select(&t)
                    .return_named("f64", Expr::LitF64(1.0))
                    .return_named("f32", Expr::LitF32(1.5))
                    .build_query("float_literals", QueryResCount::One),
                new_select(&t)
                    .return_field(&s)
                    .where_(in_field("names", &s))
                    .order(Expr::Field(s.to_ref()), Order::Asc)
                    .build_query("by_names", QueryResCount::Many),
            ],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let bananna = v.table("bannanana");
        bananna.field("hizat", field_str().build());
        bananna.field("hizat2", field_i32().opt().build());
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_base_insert".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_i32().build());
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_param_i32".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_datetime_chrono().build());
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_param_datetime_chrono".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_datetime_jiff().build());
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_param_datetime_jiff".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_query_like".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_str().opt().build());
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_query_is_null".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_query_concat".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_query_row_number".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_i32().opt().build());
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_param_opt_i32".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_i32().opt().build());
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_param_opt_i32_null".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let my_bool = v.custom_type("MyBool").rust_type("integration_tests::MyBool").base_type(type_bool().build());
        let my_i32 = v.custom_type("MyI32").rust_type("integration_tests::MyI32").base_type(type_i32().build());
        let my_i64 = v.custom_type("MyI64").rust_type("integration_tests::MyI64").base_type(type_i64().build());
        let my_u32 = v.custom_type("MyU32").rust_type("integration_tests::MyU32").base_type(type_u32().build());
        let my_f32 = v.custom_type("MyF32").rust_type("integration_tests::MyF32").base_type(type_f32().build());
        let my_f64 = v.custom_type("MyF64").rust_type("integration_tests::MyF64").base_type(type_f64().build());
        let my_bytes =
            v.custom_type("MyBytes").rust_type("integration_tests::MyBytes").base_type(type_bytes().build());
        let my_string =
            v.custom_type("MyString").rust_type("integration_tests::MyString").base_type(type_str().build());
        let my_datetime_chrono =
            v
                .custom_type("MyDatetimeChrono")
                .rust_type("integration_tests::MyDatetimeChrono")
                .base_type(type_datetime_chrono().build());
        let my_datetime_jiff =
            v
                .custom_type("MyDatetimeJiff")
                .rust_type("integration_tests::MyDatetimeJiff")
                .base_type(type_datetime_jiff().build());
        let bananna = v.table("bananna");
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
            my_datetime_chrono.field_type(),
            my_datetime_chrono.field_type(),
            my_datetime_jiff.field_type(),
            my_datetime_jiff.field_type(),
        ]
            .into_iter()
            .enumerate() {
            custom_fields.push(bananna.field(&format!("x_{}", i), type_));
        }
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_param_custom".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let my_string =
            v.custom_type("MyString").rust_type("integration_tests::MyString").base_type(type_str().build());
        let bananna = v.table("bananna");
        bananna.field("hizat", FieldTypeBuilder::new(my_string.field_type().type_).opt().build());
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_param_opt_custom".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let bananna = v.table("bannanana");
        let hizat = bananna.field("hizat", field_str().build());
        bananna.unique_index("all", &[&hizat]);
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_insert_on_conflict_do_nothing".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let bananna = v.table("bannanana");
        let hizat = bananna.field("hizat", field_str().build());
        bananna.field("two", field_i32().build());
        bananna.unique_index("all", &[&hizat]);
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_insert_on_conflict_update".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_update".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let bananna = v.table("ban");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_update_where".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let bananna = v.table("b");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_update_returning".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let bananna = v.table("b");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_delete".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let bananna = v.table("b");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_delete_cte".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let bananna = v.table("b");
        bananna.field("hizat", field_str().build());
        let snapshot = v.table("snap");
        snapshot.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_query_correlated_subquery".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let bananna = v.table("ba");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_delete_where".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let bananna = v.table("b");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_delete_returning".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let bananna = v.table("b");
        bananna.field("hizat", field_str().build());
        bananna.field("three", field_i32().build());
        let one = v.table("select_join_two");
        one.field("hizat", field_str().build());
        one.field("two", field_str().build());
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_select_join".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let bananna = v.table("bannanana");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_select_limit".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let bananna = v.table("bannanana");
        bananna.field("hizat", field_i32().build());
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_select_offset".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let bananna = v.table("bannanana");
        bananna.field("hizat", field_i32().build());
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_select_order".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let bananna = v.table("bannanana");
        bananna.field("hizat", field_i32().build());
        bananna.field("hizat2", field_i32().build());
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_select_group_by".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let bananna = v.table("bannna");
        bananna.field("hizat", field_str().build());
        bananna.field(
            "zomzom",
            field_bool().default(good_ormning::mariadb::query::expr::SerialExpr::LitBool(true)).build(),
        );
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_migrate_add_field".to_string()),
            versions: vec![(0usize, {
                let v = MariadbVersion::new();
                let bananna = v.table("bannna");
                bananna.field("hizat", field_str().build());
                let x = v.build();
                x
            }), (1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let bananna = v.table("bannna");
        bananna.field("hizat", field_str().build()).renamed_from("hozot");
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_migrate_rename_field".to_string()),
            versions: vec![(0usize, {
                let v = MariadbVersion::new();
                let bananna = v.table("bannna");
                bananna.field("hozot", field_str().build());
                let x = v.build();
                x
            }), (1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let bananna = v.table("bnanaa");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_migrate_remove_field".to_string()),
            versions: vec![(0usize, {
                let v = MariadbVersion::new();
                let bananna = v.table("bnanaa");
                bananna.field("hizat", field_str().build());
                bananna.field("zomzom", field_bool().opt().build());
                let x = v.build();
                x
            }), (1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v0 = MariadbVersion::new();
        {
            let aapeel = v0.table("aapeel");
            let aapeel_hizat = aapeel.field("bananna_hizat", field_str().max_len(100).build());
            let bananna = v0.table("bananna");
            let hizat = bananna.field("hizat", field_str().max_len(100).build());
            bananna.field("zomzom", field_i32().build());
            bananna.primary_key(&[&hizat]);
            aapeel.foreign_key("aapeel_bananna", &[(&aapeel_hizat, &hizat)]);
            let peel = v0.table("peel");
            let peel_hizat = peel.field("bananna_hizat", field_str().max_len(100).build());
            peel.field("bananna_zomzom", field_i32().build());
            peel.foreign_key("peel_bananna", &[(&peel_hizat, &hizat)]);
        }
        let v1 = MariadbVersion::new();
        {
            let banana = v1.table("banana").renamed_from("bananna");
            let hizat2 = banana.field("hizat2", field_str().max_len(100).build()).renamed_from("hizat");
            banana.field("zomzom", field_i32().build());
            banana.primary_key(&[&hizat2]);
            let peel = v1.table("peel");
            let peel_hizat = peel.field("bananna_hizat", field_str().max_len(100).build());
            peel.field("bananna_zomzom", field_i32().build());
            peel.foreign_key("peel_bananna", &[(&peel_hizat, &hizat2)]);
        }
        let v2 = MariadbVersion::new();
        {
            let banana = v2.table("banana");
            banana.field("hizat2", field_str().max_len(100).build());
            let zomzom = banana.field("zomzom", field_i32().build());
            banana.primary_key(&[&zomzom]);
            let peel = v2.table("peel");
            peel.field("bananna_hizat", field_str().max_len(100).build());
            let peel_zomzom = peel.field("bananna_zomzom", field_i32().build());
            peel.foreign_key("peel_bananna", &[(&peel_zomzom, &zomzom)]);
        }
        let v3 = MariadbVersion::new();
        {
            v3.table("other").field("other", field_i32().build());
        }
        let v4 = MariadbVersion::new();
        {
            let banana = v4.table("banana");
            let zomzom = banana.field("zomzom", field_i32().build());
            banana.primary_key(&[&zomzom]);
            let peel = v4.table("peel");
            let peel_zomzom = peel.field("bananna_zomzom", field_i32().build());
            peel.foreign_key("peel_bananna", &[(&peel_zomzom, &zomzom)]);
        }
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_migrate_constraints".to_string()),
            versions: vec![
                (0usize, v0.build()),
                (1usize, v1.build()),
                (2usize, v2.build()),
                (3usize, v3.build()),
                (4usize, v4.build()),
            ],
            ..Default::default()
        }).unwrap();
    }
    {
        let v0 = MariadbVersion::new();
        {
            let bananna = v0.table("bananna");
            let hizat = bananna.field("hizat", field_str().max_len(100).build());
            bananna.index("bananna_hizat", &[&hizat]);
        }
        let v1 = MariadbVersion::new();
        {
            let banana = v1.table("banana").renamed_from("bananna");
            let hizat2 = banana.field("hizat2", field_str().max_len(100).build()).renamed_from("hizat");
            banana.index("banana_hizat2", &[&hizat2]).renamed_from("bananna_hizat");
        }
        let v2 = MariadbVersion::new();
        {
            let banana = v2.table("banana");
            let hizat2 = banana.field("hizat2", field_str().max_len(100).build());
            banana.unique_index("banana_hizat2", &[&hizat2]);
        }
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_migrate_indices".to_string()),
            versions: vec![(0usize, v0.build()), (1usize, v1.build()), (2usize, v2.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let bananna = v.table("bnanana");
        bananna.field("hizat", field_str().build());
        let two = v.table("migrate_add_table_two");
        two.field("two", field_i32().build());
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_migrate_add_table".to_string()),
            versions: vec![(0usize, {
                let v = MariadbVersion::new();
                let bananna = v.table("bnanana");
                bananna.field("hizat", field_str().build());
                let x = v.build();
                x
            }), (1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let bananna = v.table("bana").renamed_from("migrate_rename_table_bnanana");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_migrate_rename_table".to_string()),
            versions: vec![(0usize, {
                let v = MariadbVersion::new();
                let bananna = v.table("migrate_rename_table_bnanana");
                bananna.field("hizat", field_str().build());
                let x = v.build();
                x
            }), (1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let bananna = v.table("bananana");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_migrate_remove_table".to_string()),
            versions: vec![(0usize, {
                let v = MariadbVersion::new();
                let bananna = v.table("bananana");
                bananna.field("hizat", field_str().build());
                let two = v.table("migrate_remove_table_two");
                two.field("two", field_i32().build());
                let x = v.build();
                x
            }), (1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let bananna = v.table("select_cte_bannanana");
        bananna.field("hizat", field_i32().build());
        let hizat2 = bananna.field("hizat2", field_i32().build());
        let mut hibbo_builder =
            CteBuilder::new("hibbo", Box::new(new_select_body(&bananna).return_field(&hizat2).build()));
        let zathi_type =
            hizat2
                .table
                .version
                .0
                .borrow()
                .as_ref()
                .unwrap()
                .tables
                .get(&hizat2.table.id)
                .unwrap()
                .fields
                .get(&hizat2.id)
                .unwrap()
                .type_
                .type_
                .clone();
        hibbo_builder.field("zathi", zathi_type);
        hibbo_builder.build();
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_select_cte".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let bananna = v.table("select_window_bannanana");
        bananna.field("hizat", field_i32().build());
        bananna.field("hizat2", field_i32().build());
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_select_window".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v0 = MariadbVersion::new();
        let v0_bananna = v0.table("migrate_pre_migration_v0_banana");
        v0_bananna.field("hizat", field_str().build());
        let v0_two = v0.table("migrate_pre_migration_v0_two");
        v0_two.field("two", field_i32().build());
        let v1 = MariadbVersion::new();
        let v1_bananna = v1.table("migrate_pre_migration_v0_banana");
        v1_bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_migrate_pre_migration".to_string()),
            versions: vec![(0usize, v0.build()), (1usize, v1.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let bananna = v.table("migrate_make_field_optional_bannna");
        bananna.field("hizat", field_str().opt().build());
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_migrate_make_field_optional".to_string()),
            versions: vec![(0usize, {
                let v = MariadbVersion::new();
                let bananna = v.table("migrate_make_field_optional_bannna");
                bananna.field("hizat", field_str().build());
                v.build()
            }), (1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_str().build());
        bananna.field("two", field_i32().build());
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_query_filter".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_str().build());
        bananna.field("two", field_i32().build());
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_query_window_frame".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_query_collate".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_str().opt().build());
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_query_is_distinct_from".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_str().build());
        bananna.field("two", field_i32().build());
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_query_having".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_query_cte_subquery".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_query_like_escape".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let genrerank = v.table("genrerank");
        genrerank.field("date", field_i32().build());
        let genre = genrerank.field("genre", field_str().build());
        let secondary = genrerank.field("secondary", field_str().build());
        let sort = genrerank.field("sort", field_i32().build());
        genrerank.field("rank", field_i32().build());
        let track = genrerank.field("track", field_str().build());
        genrerank.unique_index("conflict_idx", &[&genre, &secondary, &sort, &track]);
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_repeated_param".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_i32().build());
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_inline_param_i32".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_i32().build());
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_inline_param_i32_common".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let bananna = v.table("b");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_set_ops_all".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let t = v.table("t");
        t.field("val", field_i32().build());
        t.field("a", field_bool().build());
        t.field("b", field_bool().build());
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_nested_paren".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        let t = v.table("t");
        let id = t.field("id", field_i64().auto_increment().build());
        t.primary_key(&[&id]);
        let name = t.field("name", field_str().build());
        t.field("n", field_i32().build());
        t.field("f", field_f64().build());
        t.field("x", field_i64().opt().build());
        t.field("u", field_u32().build());
        t.unique_index("t_name", &[&name]);
        let u = v.table("u");
        u.field("uname", field_str().build());
        u.field("t_id", field_i64().opt().build());
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_fixes".to_string()),
            default_time_library: Some(good_ormning::utils::TimeLibrary::Jiff),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v0 = MariadbVersion::new();
        {
            let t = v0.table("t");
            let id = t.field("id", field_i64().auto_increment().build());
            t.primary_key(&[&id]);
            t.field("name", field_str().max_len(20).build());
            t.field("n", field_i32().build());
            t.field("f", field_f64().build());
            let u = v0.table("u");
            u.field("name", field_str().max_len(20).build());
            u.field("v", field_i32().build());
            v0.table("w").field("x", field_i32().build());
        }
        let v1 = MariadbVersion::new();
        {
            let t = v1.table("t");
            let id = t.field("id", field_i64().auto_increment().build());
            t.primary_key(&[&id]);
            t.field("name", field_str().max_len(20).build());
            t.field("n", field_i32().build());
            t.field("f", field_f64().build());
            t.field("d", field_decimal().precision(10).scale(2).build());
            t.field("at", field_datetime_chrono().precision(3).opt().build());
            let u = v1.table("u");
            u.field("name", field_str().max_len(20).build());
            u.field("v", field_i32().build());
            let w = v1.table("w");
            w.field("x", field_i32().build());
            let id = w.field("id", field_u64().auto_increment().build());
            w.primary_key(&[&id]);
            w.field(
                "label",
                field_text()
                    .default(good_ormning::mariadb::query::expr::SerialExpr::LitString("none".to_string()))
                    .build(),
            );
            w.field(
                "code",
                field_char()
                    .max_len(3)
                    .default(good_ormning::mariadb::query::expr::SerialExpr::LitString("abc".to_string()))
                    .build(),
            );
            w.field(
                "m",
                field_mediumint().default(good_ormning::mariadb::query::expr::SerialExpr::LitAnyInt(-5)).build(),
            );
            w.field("flag", field_bool().opt().build());
            let parent = v1.table("parent");
            let parent_id = parent.field("id", field_i64().build());
            parent.primary_key(&[&parent_id]);
            let child = v1.table("child");
            let child_parent = child.field("pid", field_i64().opt().build());
            child
                .foreign_key("child_parent", &[(&child_parent, &parent_id)])
                .on_delete(good_ormning::mariadb::schema::constraint::ForeignKeyAction::Cascade);
        }
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_native".to_string()),
            default_time_library: Some(good_ormning::utils::TimeLibrary::Chrono),
            versions: vec![(0usize, v0.build()), (1usize, v1.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = MariadbVersion::new();
        {
            let e = v.table("e");
            let id = e.field("id", field_i64().auto_increment().build());
            e.primary_key(&[&id]);
            e.field("d", field_date_jiff().opt().build());
            e.field("dc", field_date_chrono().opt().build());
            e.field("tm", field_time_jiff().precision(2).opt().build());
            e.field("tc", field_time_chrono().opt().build());
            e.field("y", field_year().opt().build());
            e.field("ts", field_timestamp_jiff().precision(6).opt().build());
            e.field("s", field_str().opt().build());
            e.field(
                "dd",
                field_date_jiff()
                    .default(good_ormning::mariadb::query::expr::SerialExpr::LitDate("2020-01-02".to_string()))
                    .build(),
            );
            e.field(
                "td",
                field_time_jiff()
                    .default(good_ormning::mariadb::query::expr::SerialExpr::LitTime("-01:02:03".to_string()))
                    .build(),
            );
            e.field(
                "sd",
                field_str()
                    .max_len(10)
                    .default(good_ormning::mariadb::query::expr::SerialExpr::LitString("a\\b'c".to_string()))
                    .build(),
            );
        }
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_temporal".to_string()),
            versions: vec![(0usize, v.build())],
            default_time_library: Some(good_ormning::utils::TimeLibrary::Chrono),
            ..Default::default()
        }).unwrap();
    }
    {
        let v0 = MariadbVersion::new();
        v0.table("resume_a").field("x", field_i32().build());
        let v1 = MariadbVersion::new();
        {
            let a = v1.table("resume_a");
            let x = a.field("x", field_i32().build());
            a.field("y", field_i32().opt().build());
            a.unique_index("resume_a_x", &[&x]);
            v1.table("resume_b").field("z", field_i32().build());
        }
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_migrate_resume".to_string()),
            versions: vec![(0usize, v0.build()), (1usize, v1.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        use good_ormning::{
            QueryResCount,
            mariadb::{
                new_delete,
                new_insert,
                new_select,
                new_update,
                query::{
                    expr::{
                        BinOp,
                        Expr,
                    },
                    helpers::{
                        eq_field,
                        fn_values,
                        set_field,
                    },
                    select::{
                        Join,
                        JoinCondition,
                        JoinSource,
                        JoinType,
                        NamedSelectSource,
                        Order,
                    },
                },
            },
        };

        let v = MariadbVersion::new();
        let t = v.table("t");
        let id = t.field("id", field_i64().auto_increment().build());
        t.primary_key(&[&id]);
        let name = t.field("name", field_str().max_len(20).build());
        let n = t.field("n", field_i32().build());
        t.unique_index("t_name", &[&name]);
        let u = v.table("u");
        let t_name = u.field("t_name", field_str().max_len(20).build());
        let bonus = u.field("bonus", field_i32().build());
        let join_u = || Join {
            source: Box::new(NamedSelectSource {
                source: JoinSource::Table(u.to_ref()),
                alias: Some("u".to_string()),
            }),
            type_: JoinType::Inner,
            condition: Some(JoinCondition::On(Expr::BinOp {
                left: Box::new(Expr::Field(name.to_ref())),
                op: BinOp::Equals,
                right: Box::new(Expr::Field(t_name.to_ref())),
            })),
        };
        let queries =
            vec![
                new_insert(&t, vec![set_field("name", &name), set_field("n", &n)])
                    .on_duplicate_key_update(vec![(n.clone(), Expr::BinOp {
                        left: Box::new(Expr::Field(n.to_ref())),
                        op: BinOp::Plus,
                        right: Box::new(fn_values(Expr::Field(n.to_ref()))),
                    })])
                    .return_field(&id)
                    .build_query("upsert", QueryResCount::One),
                new_insert(&u, vec![set_field("t_name", &t_name), set_field("bonus", &bonus)])
                    .ignore()
                    .build_query("insert_u", QueryResCount::None),
                new_update(&t, vec![(n.clone(), Expr::LitI32(0))])
                    .order(Expr::Field(name.to_ref()), Order::Desc)
                    .limit(Expr::LitAnyInt(1))
                    .build_query("zero_last"),
                new_update(&t, vec![(n.clone(), Expr::BinOp {
                    left: Box::new(Expr::Field(n.to_ref())),
                    op: BinOp::Plus,
                    right: Box::new(Expr::Field(bonus.to_ref())),
                })]).join(join_u()).build_query("add_bonus"),
                new_delete(&t)
                    .join(join_u())
                    .where_(eq_field("bonus", &bonus))
                    .build_query("delete_by_bonus", QueryResCount::None),
                new_select(&t)
                    .return_fields(&[&name, &n])
                    .order(Expr::Field(name.to_ref()), Order::Asc)
                    .build_query_named_res("list", QueryResCount::Many, "ListRow"),
            ];
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_builder".to_string()),
            versions: vec![(0usize, v.build())],
            queries: queries,
            default_time_library: None,
        }).unwrap();
    }
    {
        use good_ormning::mariadb::schema::field::{
            field_bytes,
            field_i16,
            field_i8,
            field_u16,
            field_u64,
            field_u8,
        };

        let v = MariadbVersion::new();
        let t = v.table("t");
        t.field("i8", field_i8().build());
        t.field("i16", field_i16().build());
        t.field("u8", field_u8().build());
        t.field("u16", field_u16().build());
        t.field("u64", field_u64().build());
        let short = t.field("short", field_str().max_len(3).build());
        t.field("b", field_bytes().max_len(4).opt().build());
        t.primary_key(&[&short]);
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_types".to_string()),
            versions: vec![(0usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v0 = MariadbVersion::new();
        v0.table("default_table").field("id", field_i32().build());
        let v1 = MariadbVersion::new();
        v1.table("default_table").field("id", field_i32().build());
        v1.table("default_table_v1").field("id", field_i32().build());
        generate(GenerateArgs {
            db_name: Some("mariadb_gen_default".to_string()),
            versions: vec![(0usize, v0.build()), (1usize, v1.build())],
            ..Default::default()
        }).unwrap();
    }
}
