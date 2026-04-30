use good_ormning::{
    PgFieldTypeBuilder as FieldTypeBuilder,
    PgType as Type,
    pg::{
        FieldHandle,
        Version as PgVersion,
        generate,
        GenerateArgs,
        new_select_body,
        query::utils::CteBuilder,
        schema::field::{
            field_bool,
            field_i32,
            field_str,
            field_utctime_s_chrono,
            field_utctime_s_jiff,
        },
    },
    pg_type_bool as type_bool,
    pg_type_bytes as type_bytes,
    pg_type_f32 as type_f32,
    pg_type_f64 as type_f64,
    pg_type_i32 as type_i32,
    pg_type_i64 as type_i64,
    pg_type_str as type_str,
    pg_type_u32 as type_u32,
    pg_type_utctime_s_chrono as type_utctime_s_chrono,
    pg_type_utctime_s_jiff as type_utctime_s_jiff,
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
    // # Base: create table, insert, select
    {
        let v = PgVersion::new();
        let bananna = v.table("bannanana");
        bananna.field("hizat", field_str().build());
        bananna.field("hizat2", field_i32().opt().build());
        generate(GenerateArgs {
            db_name: Some("pg_gen_base_insert".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # (insert) Param: i32
    {
        let v = PgVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_i32().build());
        generate(GenerateArgs {
            db_name: Some("pg_gen_param_i32".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # (insert) Param: utctime (chrono)
    {
        let v = PgVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_utctime_s_chrono().build());
        generate(GenerateArgs {
            db_name: Some("pg_gen_param_utctime_chrono".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # (insert) Param: utctime (jiff)
    {
        let v = PgVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_utctime_s_jiff().build());
        generate(GenerateArgs {
            db_name: Some("pg_gen_param_utctime_jiff".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    {
        let v = PgVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("pg_gen_query_like".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    {
        let v = PgVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("pg_gen_query_is_null".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    {
        let v = PgVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("pg_gen_query_concat".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    {
        let v = PgVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("pg_gen_query_row_number".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # (insert) Param: Opt`<i32>`
    {
        let v = PgVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_i32().opt().build());
        generate(GenerateArgs {
            db_name: Some("pg_gen_param_opt_i32".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # (insert) Param: Opt`<i32>`, null
    {
        let v = PgVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_i32().opt().build());
        generate(GenerateArgs {
            db_name: Some("pg_gen_param_opt_i32_null".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
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
            my_utctime_chrono.field_type(),
            my_utctime_chrono.field_type(),
            my_utctime_jiff.field_type(),
            my_utctime_jiff.field_type(),
        ]
            .into_iter()
            .enumerate() {
            custom_fields.push(bananna.field(&format!("x_{}", i), type_));
        }
        generate(GenerateArgs {
            db_name: Some("pg_gen_param_custom".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # (insert) Param: Opt`<Custom>`
    {
        let v = PgVersion::new();
        let my_string = v.custom_type("MyString").rust_type("integration_tests::MyString").base_type(type_str().build());
        let bananna = v.table("bananna");
        bananna.field("hizat", FieldTypeBuilder::new(my_string.field_type().type_).opt().build());
        generate(GenerateArgs {
            db_name: Some("pg_gen_param_opt_custom".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # Insert on conflict do nothing
    {
        let v = PgVersion::new();
        let bananna = v.table("bannanana");
        let hizat = bananna.field("hizat", field_str().build());
        bananna.unique_index("all", &[&hizat]);
        generate(GenerateArgs {
            db_name: Some("pg_gen_insert_on_conflict_do_nothing".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # Insert on conflict update
    {
        let v = PgVersion::new();
        let bananna = v.table("bannanana");
        let hizat = bananna.field("hizat", field_str().build());
        bananna.field("two", field_i32().build());
        bananna.unique_index("all", &[&hizat]);
        generate(GenerateArgs {
            db_name: Some("pg_gen_insert_on_conflict_update".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # Update
    {
        let v = PgVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("pg_gen_update".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # Update, where
    {
        let v = PgVersion::new();
        let bananna = v.table("ban");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("pg_gen_update_where".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # Update, returning
    {
        let v = PgVersion::new();
        let bananna = v.table("b");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("pg_gen_update_returning".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # Delete
    {
        let v = PgVersion::new();
        let bananna = v.table("b");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("pg_gen_delete".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # Delete, where
    {
        let v = PgVersion::new();
        let bananna = v.table("ba");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("pg_gen_delete_where".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # Delete, returning
    {
        let v = PgVersion::new();
        let bananna = v.table("b");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("pg_gen_delete_returning".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # Select + join
    {
        let v = PgVersion::new();
        let bananna = v.table("b");
        bananna.field("hizat", field_str().build());
        bananna.field("three", field_i32().build());
        let one = v.table("select_join_two");
        one.field("hizat", field_str().build());
        one.field("two", field_str().build());
        generate(GenerateArgs {
            db_name: Some("pg_gen_select_join".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # Select limit
    {
        let v = PgVersion::new();
        let bananna = v.table("bannanana");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("pg_gen_select_limit".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # Select order
    {
        let v = PgVersion::new();
        let bananna = v.table("bannanana");
        bananna.field("hizat", field_i32().build());
        generate(GenerateArgs {
            db_name: Some("pg_gen_select_order".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # Select group
    {
        let v = PgVersion::new();
        let bananna = v.table("bannanana");
        bananna.field("hizat", field_i32().build());
        bananna.field("hizat2", field_i32().build());
        generate(GenerateArgs {
            db_name: Some("pg_gen_select_group_by".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # Migrate - add field
    {
        let v = PgVersion::new();
        let bananna = v.table("bannna");
        bananna.field("hizat", field_str().build());
        bananna.field("zomzom",
            field_bool().migrate_fill(good_ormning::pg::query::expr::SerialExpr::LitBool(true)).build(),
        );
        generate(GenerateArgs {
            db_name: Some("pg_gen_migrate_add_field".to_string()),
            versions: vec![
            // Versions (previous)
            (0usize, {
                let v = PgVersion::new();
                let bananna = v.table("bannna");
                bananna.field("hizat", field_str().build());
                let x = v.build();
                x
            }),
            (1usize, v.build())
        ],
            ..Default::default()
        }).unwrap();
    }

    // # Migrate - rename field
    {
        let v = PgVersion::new();
        let bananna = v.table("bannna");
        bananna.field("hizat", field_str().build()).renamed_from("hozot");
        generate(GenerateArgs {
            db_name: Some("pg_gen_migrate_rename_field".to_string()),
            versions: vec![
            // Versions (previous)
            (0usize, {
                let v = PgVersion::new();
                let bananna = v.table("bannna");
                bananna.field("hozot", field_str().build());
                let x = v.build();
                x
            }),
            (1usize, v.build())
        ],
            ..Default::default()
        }).unwrap();
    }

    // # Migrate - remove field
    {
        let v = PgVersion::new();
        let bananna = v.table("bnanaa");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("pg_gen_migrate_remove_field".to_string()),
            versions: vec![
            // Versions (previous)
            (0usize, {
                let v = PgVersion::new();
                let bananna = v.table("bnanaa");
                bananna.field("hizat", field_str().build());
                bananna.field("zomzom", field_bool().opt().build());
                let x = v.build();
                x
            }),
            (1usize, v.build())
        ],
            ..Default::default()
        }).unwrap();
    }

    // # Migrate - add table
    {
        let v = PgVersion::new();
        let bananna = v.table("bnanana");
        bananna.field("hizat", field_str().build());
        let two = v.table("migrate_add_table_two");
        two.field("two", field_i32().build());
        generate(GenerateArgs {
            db_name: Some("pg_gen_migrate_add_table".to_string()),
            versions: vec![
            // Versions (previous)
            (0usize, {
                let v = PgVersion::new();
                let bananna = v.table("bnanana");
                bananna.field("hizat", field_str().build());
                let x = v.build();
                x
            }),
            (1usize, v.build())
        ],
            ..Default::default()
        }).unwrap();
    }

    // # Migrate - rename table
    {
        let v = PgVersion::new();
        let bananna = v.table("bana").renamed_from("migrate_rename_table_bnanana");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("pg_gen_migrate_rename_table".to_string()),
            versions: vec![
            // Versions (previous)
            (0usize, {
                let v = PgVersion::new();
                let bananna = v.table("migrate_rename_table_bnanana");
                bananna.field("hizat", field_str().build());
                let x = v.build();
                x
            }),
            (1usize, v.build())
        ],
            ..Default::default()
        }).unwrap();
    }

    // # Migrate - remove table
    {
        let v = PgVersion::new();
        let bananna = v.table("bananana");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("pg_gen_migrate_remove_table".to_string()),
            versions: vec![
            // Versions (previous)
            (0usize, {
                let v = PgVersion::new();
                let bananna = v.table("bananana");
                bananna.field("hizat", field_str().build());
                let two = v.table("migrate_remove_table_two");
                two.field("two", field_i32().build());
                let x = v.build();
                x
            }),
            (1usize, v.build())
        ],
            ..Default::default()
        }).unwrap();
    }

    // # Select CTE
    {
        let v = PgVersion::new();
        let bananna = v.table("select_cte_bannanana");
        bananna.field("hizat", field_i32().build());
        let hizat2 = bananna.field("hizat2", field_i32().build());
        let mut hibbo_builder =
            CteBuilder::new("hibbo", Box::new(new_select_body(&bananna).return_field(&hizat2).build()));
        hibbo_builder.field("zathi", get_type(&hizat2));
        hibbo_builder.build();
        generate(GenerateArgs {
            db_name: Some("pg_gen_select_cte".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # Window function
    {
        let v = PgVersion::new();
        let bananna = v.table("select_window_bannanana");
        bananna.field("hizat", field_i32().build());
        bananna.field("hizat2", field_i32().build());
        generate(GenerateArgs {
            db_name: Some("pg_gen_select_window".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # Migrate - pre migration
    {
        let v0 = PgVersion::new();
        let v0_bananna = v0.table("migrate_pre_migration_v0_banana");
        v0_bananna.field("hizat", field_str().build());
        let v0_two = v0.table("migrate_pre_migration_v0_two");
        v0_two.field("two", field_i32().build());
        let v1 = PgVersion::new();
        let v1_bananna = v1.table("migrate_pre_migration_v0_banana");
        v1_bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("pg_gen_migrate_pre_migration".to_string()),
            versions: vec![
            // Versions (previous)
            (0usize, v0.build()),
            (1usize, v1.build())
        ],
            ..Default::default()
        }).unwrap();
    }

    // # Migrate - make field optional
    {
        let v = PgVersion::new();
        let bananna = v.table("migrate_make_field_optional_bannna");
        bananna.field("hizat", field_str().opt().build());
        generate(GenerateArgs {
            db_name: Some("pg_gen_migrate_make_field_optional".to_string()),
            versions: vec![
            // Versions (previous)
            (0usize, {
                let v = PgVersion::new();
                let bananna = v.table("migrate_make_field_optional_bannna");
                bananna.field("hizat", field_str().build());
                v.build()
            }),
            (1usize, v.build())
        ],
            ..Default::default()
        }).unwrap();
    }

    {
        let v = PgVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_str().build());
        bananna.field("two", field_i32().build());
        generate(GenerateArgs {
            db_name: Some("pg_gen_query_filter".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    {
        let v = PgVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_str().build());
        bananna.field("two", field_i32().build());
        generate(GenerateArgs {
            db_name: Some("pg_gen_query_window_frame".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    {
        let v = PgVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("pg_gen_query_collate".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    {
        let v = PgVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_str().opt().build());
        generate(GenerateArgs {
            db_name: Some("pg_gen_query_is_distinct_from".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    {
        let v = PgVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_str().build());
        bananna.field("two", field_i32().build());
        generate(GenerateArgs {
            db_name: Some("pg_gen_query_having".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    {
        let v = PgVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("pg_gen_query_cte_subquery".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    {
        let v = PgVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("pg_gen_query_like_escape".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
}
