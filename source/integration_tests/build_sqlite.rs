use good_ormning::{
    SqliteFieldTypeBuilder as FieldTypeBuilder,
    SqliteType as Type,
    sqlite::{
        FieldHandle,
        Version as SqliteVersion,
        generate,
        GenerateArgs,
        new_select_body,
        query::utils::CteBuilder,
        schema::field::{
            field_bool,
            field_i32,
            field_i64,
            field_str,
            field_utctime_ms_chrono,
            field_utctime_ms_jiff,
            field_utctime_s_chrono,
            field_utctime_s_jiff,
        },
    },
    sqlite_type_bool as type_bool,
    sqlite_type_bytes as type_bytes,
    sqlite_type_f32 as type_f32,
    sqlite_type_f64 as type_f64,
    sqlite_type_i32 as type_i32,
    sqlite_type_i64 as type_i64,
    sqlite_type_str as type_str,
    sqlite_type_u32 as type_u32,
    sqlite_type_utctime_s_chrono as type_utctime_s_chrono,
    sqlite_type_utctime_s_jiff as type_utctime_s_jiff,
};

fn get_type(f: &FieldHandle) -> Type {
    let version = f.table.version.0.borrow();
    version.as_ref().unwrap().tables.get(&f.table.id).unwrap().fields.get(&f.id).unwrap().type_.type_.clone()
}

pub fn build() {
    // # Hello world example
    {
        let v = SqliteVersion::new();
        let users = v.table("hello_world_users");
        users.rowid_field(None);
        users.field("name", field_str().build());
        users.field("points", field_i64().build());
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_hello_world".to_string()),
            versions: vec![
                // Versions
                (1usize, v.build())
            ],
            ..Default::default()
        }).unwrap();
    }

    // # Base: create table, insert, select
    {
        let v = SqliteVersion::new();
        let bananna = v.table("bannanana");
        bananna.field("hizat", field_str().build());
        bananna.field("hizat2", field_i32().opt().build());
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_base_insert".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # Primary key
    {
        let v = SqliteVersion::new();
        let bananna = v.table("bannanana");
        let hizat = bananna.field("hizat", field_str().build());
        bananna.primary_key("hizat_pk", &[&hizat]);
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_constraint".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # (insert) Param: i32
    {
        let v = SqliteVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_i32().build());
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_param_i32".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # (insert) Param: datetime (seconds) (chrono)
    {
        let v = SqliteVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_utctime_s_chrono().build());
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_param_utctime_s_chrono".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # (insert) Param: datetime (ms) (chrono)
    {
        let v = SqliteVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_utctime_ms_chrono().build());
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_param_utctime_ms_chrono".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # (insert) Param: datetime (seconds) (jiff)
    {
        let v = SqliteVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_utctime_s_jiff().build());
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_param_utctime_s_jiff".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # (insert) Param: datetime (ms) (jiff)
    {
        let v = SqliteVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_utctime_ms_jiff().build());
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_param_utctime_ms_jiff".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = SqliteVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_query_like".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = SqliteVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_query_is_null".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = SqliteVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_query_concat".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = SqliteVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_query_row_number".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # (insert) Param: Opt`<i32>`
    {
        let v = SqliteVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_i32().opt().build());
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_param_opt_i32".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # (insert) Param: Opt`<i32>`, null
    {
        let v = SqliteVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_i32().opt().build());
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_param_opt_i32_null".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
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
        let my_bytes =
            v.custom_type("MyBytes").rust_type("integration_tests::MyBytes").base_type(type_bytes().build());
        let my_string =
            v.custom_type("MyString").rust_type("integration_tests::MyString").base_type(type_str().build());
        let my_utctime_chrono =
            v
                .custom_type("MyUtctimeChrono")
                .rust_type("integration_tests::MyUtctimeChrono")
                .base_type(type_utctime_s_chrono().build());
        let my_utctime_jiff =
            v
                .custom_type("MyUtctimeJiff")
                .rust_type("integration_tests::MyUtctimeJiff")
                .base_type(type_utctime_s_jiff().build());
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
            db_name: Some("sqlite_gen_param_custom".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # (insert) Param: Opt`<Custom>`
    {
        let v = SqliteVersion::new();
        let my_string =
            v.custom_type("MyString").rust_type("integration_tests::MyString").base_type(type_str().build());
        let bananna = v.table("bananna");
        bananna.field("hizat", FieldTypeBuilder::new(my_string.field_type().type_).opt().build());
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_param_opt_custom".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # Insert on conflict do nothing
    {
        let v = SqliteVersion::new();
        let bananna = v.table("bannanana");
        let hizat = bananna.field("hizat", field_str().build());
        bananna.unique_index("all", &[&hizat]);
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_insert_on_conflict_do_nothing".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # Insert on conflict update
    {
        let v = SqliteVersion::new();
        let bananna = v.table("bannanana");
        let hizat = bananna.field("hizat", field_str().build());
        bananna.field("two", field_i32().build());
        bananna.unique_index("all", &[&hizat]);
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_insert_on_conflict_update".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # Update
    {
        let v = SqliteVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_update".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # Update, where
    {
        let v = SqliteVersion::new();
        let bananna = v.table("ban");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_update_where".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # Update, returning
    {
        let v = SqliteVersion::new();
        let bananna = v.table("b");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_update_returning".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # Delete
    {
        let v = SqliteVersion::new();
        let bananna = v.table("b");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_delete".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # Delete CTE
    {
        let v = SqliteVersion::new();
        let bananna = v.table("b");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_delete_cte".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # Correlated Subquery
    {
        let v = SqliteVersion::new();
        let bananna = v.table("b");
        bananna.field("hizat", field_str().build());
        let snapshot = v.table("snap");
        snapshot.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_query_correlated_subquery".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # Delete, where
    {
        let v = SqliteVersion::new();
        let bananna = v.table("ba");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_delete_where".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # Delete, returning
    {
        let v = SqliteVersion::new();
        let bananna = v.table("b");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_delete_returning".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # Select + join
    {
        let v = SqliteVersion::new();
        let bananna = v.table("b");
        bananna.field("hizat", field_str().build());
        bananna.field("three", field_i32().build());
        let one = v.table("select_join_two");
        one.field("hizat", field_str().build());
        one.field("two", field_str().build());
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_select_join".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # Select limit
    {
        let v = SqliteVersion::new();
        let bananna = v.table("bannanana");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_select_limit".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # Select order
    {
        let v = SqliteVersion::new();
        let bananna = v.table("bannanana");
        bananna.field("hizat", field_i32().build());
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_select_order".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # Select group
    {
        let v = SqliteVersion::new();
        let bananna = v.table("bannanana");
        bananna.field("hizat", field_i32().build());
        bananna.field("hizat2", field_i32().build());
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_select_group_by".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # Migrate - add field
    {
        let v = SqliteVersion::new();
        let bananna = v.table("bannna");
        bananna.field("hizat", field_str().build());
        bananna.field(
            "zomzom",
            field_bool().migrate_fill(good_ormning::sqlite::query::expr::SerialExpr::LitBool(true)).build(),
        );
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_migrate_add_field".to_string()),
            versions: vec![
                // Versions (previous)
                (0usize, {
                    let v = SqliteVersion::new();
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
        let v = SqliteVersion::new();
        let bananna = v.table("bannna");
        bananna.field("hizat", field_str().build()).renamed_from("hozot");
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_migrate_rename_field".to_string()),
            versions: vec![
                // Versions (previous)
                (0usize, {
                    let v = SqliteVersion::new();
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
        let v = SqliteVersion::new();
        let bananna = v.table("bnanaa");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_migrate_remove_field".to_string()),
            versions: vec![
                // Versions (previous)
                (0usize, {
                    let v = SqliteVersion::new();
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
        let v = SqliteVersion::new();
        let bananna = v.table("bnanana");
        bananna.field("hizat", field_str().build());
        let two = v.table("migrate_add_table_two");
        two.field("two", field_i32().build());
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_migrate_add_table".to_string()),
            versions: vec![
                // Versions (previous)
                (0usize, {
                    let v = SqliteVersion::new();
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
        let v = SqliteVersion::new();
        let bananna = v.table("bana").renamed_from("migrate_rename_table_bnanana");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_migrate_rename_table".to_string()),
            versions: vec![
                // Versions (previous)
                (0usize, {
                    let v = SqliteVersion::new();
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
        let v = SqliteVersion::new();
        let bananna = v.table("bananana");
        bananna.field("hizat", field_str().build());
        let two = v.table("migrate_remove_table_two");
        two.field("two", field_i32().build());
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_migrate_remove_table".to_string()),
            versions: vec![
                // Versions (previous)
                (0usize, {
                    let v = SqliteVersion::new();
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

    // # Junction
    {
        let v = SqliteVersion::new();
        let bananna = v.table("bannanana");
        bananna.field("hizat", field_i32().build());
        bananna.field("hizat2", field_i32().build());
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_select_junction".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # Select CTE
    {
        let v = SqliteVersion::new();
        let bananna = v.table("bannanana");
        bananna.field("hizat", field_i32().build());
        let hizat2 = bananna.field("hizat2", field_i32().build());
        let mut hibbo_builder =
            CteBuilder::new("hibbo", Box::new(new_select_body(&bananna).return_field(&hizat2).build()));
        hibbo_builder.field("zathi", get_type(&hizat2));
        hibbo_builder.build();
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_select_cte".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # Window function
    {
        let v = SqliteVersion::new();
        let bananna = v.table("bannanana");
        bananna.field("hizat", field_i32().build());
        bananna.field("hizat2", field_i32().build());
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_select_window".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # Migrate - pre migration
    {
        let v0 = SqliteVersion::new();
        let v0_bananna = v0.table("migrate_pre_migration_v0_banana");
        v0_bananna.field("hizat", field_str().build());
        let v0_two = v0.table("migrate_pre_migration_v0_two");
        v0_two.field("two", field_i32().build());
        let v1 = SqliteVersion::new();
        let v1_bananna = v1.table("migrate_pre_migration_v0_banana");
        v1_bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_migrate_pre_migration".to_string()),
            versions: vec![
                // Versions (previous)
                (0usize, v0.build()),
                (1usize, v1.build())
            ],
            ..Default::default()
        }).unwrap();
    }

    // # Param array
    {
        let v = SqliteVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_i32().build());
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_param_arr_i32".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = SqliteVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_str().build());
        bananna.field("two", field_i32().build());
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_query_filter".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = SqliteVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_str().build());
        bananna.field("two", field_i32().build());
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_query_window_frame".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = SqliteVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_query_collate".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = SqliteVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_str().opt().build());
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_query_is_distinct_from".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = SqliteVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_str().build());
        bananna.field("two", field_i32().build());
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_query_having".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = SqliteVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_query_glob".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = SqliteVersion::new();
        let bananna = v.table("bananna");
        let hizat = bananna.field("hizat", field_str().build());
        let _ = bananna.index("bananna_hizat", &[&hizat]);
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_query_indexed_by".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = SqliteVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_query_cte_subquery".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = SqliteVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_str().build());
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_query_like_escape".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = SqliteVersion::new();
        let genrerank = v.table("genrerank");
        genrerank.field("date", field_i32().build());
        let genre = genrerank.field("genre", field_str().build());
        let secondary = genrerank.field("secondary", field_str().build());
        let sort = genrerank.field("sort", field_i32().build());
        genrerank.field("rank", field_i32().build());
        let track = genrerank.field("track", field_str().build());
        genrerank.unique_index("conflict_idx", &[&genre, &secondary, &sort, &track]);
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_repeated_param".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = SqliteVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_i32().build());
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_inline_param_i32".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }
    {
        let v = SqliteVersion::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_i32().build());
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_inline_param_i32_common".to_string()),
            versions: vec![(1usize, v.build())],
            ..Default::default()
        }).unwrap();
    }

    // # Generated query functions compile test
    // This ensures query functions generated in build.rs use valid generic syntax.
    {
        let v = SqliteVersion::new();
        let bananna = v.table("bananna");
        let hizat = bananna.field("hizat", field_str().build());
        let body = new_select_body(&bananna).return_field(&hizat).build();
        let query = good_ormning::sqlite::Query {
            name: "hist_list_all".to_string(),
            body: Box::new(body),
            res_count: good_ormning::QueryResCount::Many,
            res_name: None,
        };
        generate(GenerateArgs {
            db_name: Some("sqlite_gen_query".to_string()),
            versions: vec![(1usize, v.build())],
            queries: vec![query],
        }).unwrap();
    }
}
