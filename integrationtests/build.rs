use std::{
    path::PathBuf,
    env,
    str::FromStr,
};
use goodormning::pg::{
    Version,
    schema::field::{
        field_str,
        field_i32,
        field_bool,
    },
    queries::{
        expr::Expr,
    },
    new_insert,
    QueryResCount,
    new_select,
    new_update,
    new_delete,
};

pub fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let root = PathBuf::from_str(&env::var("CARGO_MANIFEST_DIR").unwrap()).unwrap();

    // # Base: create table, insert, select
    {
        let mut v = Version::default();
        let bananna = v.table("bananna");
        let hizat = bananna.field(&mut v, "z437INV6D", "hizat", field_str().build());
        goodormning::pg::generate(&root.join("tests/pg_gen_base_insert.rs"), vec![(0usize, v)], vec![
            // Queries
            new_insert(&bananna, vec![(hizat.id.clone(), Expr::Param {
                name: "text".into(),
                type_: hizat.def.type_.type_.clone(),
            })]).build_query("insert_banan", QueryResCount::None),
            new_select(&bananna).output_field(&hizat).build_query("get_banan", QueryResCount::One)
        ]).unwrap();
    }

    // # (insert) Param: i32
    {
        let mut v = Version::default();
        let bananna = v.table("bananna");
        let hizat = bananna.field(&mut v, "z437INV6D", "hizat", field_i32().build());
        goodormning::pg::generate(&root.join("tests/pg_gen_param_i32.rs"), vec![(0usize, v)], vec![
            // Queries
            new_insert(&bananna, vec![(hizat.id.clone(), Expr::Param {
                name: "val".into(),
                type_: hizat.def.type_.type_.clone(),
            })]).build_query("insert_banan", QueryResCount::None),
            new_select(&bananna).output_field(&hizat).build_query("get_banan", QueryResCount::One)
        ]).unwrap();
    }

    // # (insert) Param: Opt`<i32>`
    {
        let mut v = Version::default();
        let bananna = v.table("bananna");
        let hizat = bananna.field(&mut v, "z437INV6D", "hizat", field_i32().opt().build());
        goodormning::pg::generate(&root.join("tests/pg_gen_param_opt_i32.rs"), vec![(0usize, v)], vec![
            // Queries
            new_insert(&bananna, vec![(hizat.id.clone(), Expr::Param {
                name: "val".into(),
                type_: hizat.def.type_.type_.clone(),
            })]).build_query("insert_banan", QueryResCount::None),
            new_select(&bananna).output_field(&hizat).build_query("get_banan", QueryResCount::One)
        ]).unwrap();
    }

    // # (insert) Param: Opt`<i32>`, null
    {
        let mut v = Version::default();
        let bananna = v.table("bananna");
        let hizat = bananna.field(&mut v, "z437INV6D", "hizat", field_i32().opt().build());
        goodormning::pg::generate(&root.join("tests/pg_gen_param_opt_i32_null.rs"), vec![(0usize, v)], vec![
            // Queries
            new_insert(
                &bananna,
                vec![(hizat.id.clone(), Expr::LitNull(hizat.def.type_.type_.type_.clone()))],
            ).build_query("insert_banan", QueryResCount::None),
            new_select(&bananna).output_field(&hizat).build_query("get_banan", QueryResCount::One)
        ]).unwrap();
    }

    // # (insert) Param: Custom
    {
        let mut v = Version::default();
        let bananna = v.table("bananna");
        let hizat =
            bananna.field(&mut v, "z437INV6D", "hizat", field_str().custom("integrationtests::MyString").build());
        goodormning::pg::generate(&root.join("tests/pg_gen_param_custom.rs"), vec![(0usize, v)], vec![
            // Queries
            new_insert(&bananna, vec![(hizat.id.clone(), Expr::Param {
                name: "val".into(),
                type_: hizat.def.type_.type_.clone(),
            })]).build_query("insert_banan", QueryResCount::None),
            new_select(&bananna).output_field(&hizat).build_query("get_banan", QueryResCount::One)
        ]).unwrap();
    }

    // # (insert) Param: Opt`<Custom>`
    {
        let mut v = Version::default();
        let bananna = v.table("bananna");
        let hizat =
            bananna.field(
                &mut v,
                "z437INV6D",
                "hizat",
                field_str().custom("integrationtests::MyString").opt().build(),
            );
        goodormning::pg::generate(&root.join("tests/pg_gen_param_opt_custom.rs"), vec![(0usize, v)], vec![
            // Queries
            new_insert(&bananna, vec![(hizat.id.clone(), Expr::Param {
                name: "text".into(),
                type_: hizat.def.type_.type_.clone(),
            })]).build_query("insert_banan", QueryResCount::None),
            new_select(&bananna).output_field(&hizat).build_query("get_banan", QueryResCount::One)
        ]).unwrap();
    }

    // # Insert, returning
    //
    // # Insert on conflict do nothing
    //
    // # Insert on conflict update
    //
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
        let bananna = v.table("bananna");
        let hizat = bananna.field(&mut v, "z437INV6D", "hizat", field_str().build());
        goodormning::pg::generate(&root.join("tests/pg_gen_update.rs"), vec![(0usize, v)], vec![
            // Queries
            new_insert(
                &bananna,
                vec![(hizat.id.clone(), Expr::LitString("yog".into()))],
            ).build_query("insert_banan", QueryResCount::None),
            new_select(&bananna).output_field(&hizat).build_query("get_banan", QueryResCount::One),
            new_update(
                &bananna,
                vec![(hizat.id.clone(), Expr::LitString("tep".into()))],
            ).build_query("update_banan", QueryResCount::None)
        ]).unwrap();
    }

    // # Update, returning
    //
    // # Delete
    {
        let mut v = Version::default();
        let bananna = v.table("bananna");
        let hizat = bananna.field(&mut v, "z437INV6D", "hizat", field_str().build());
        goodormning::pg::generate(&root.join("tests/pg_gen_delete.rs"), vec![(0usize, v)], vec![
            // Queries
            new_insert(
                &bananna,
                vec![(hizat.id.clone(), Expr::LitString("seeon".into()))],
            ).build_query("insert_banan", QueryResCount::None),
            new_select(&bananna).output_field(&hizat).build_query("get_banan", QueryResCount::MaybeOne),
            new_delete(&bananna).build_query("no_banan", QueryResCount::None)
        ]).unwrap();
    }

    // # Delete, returning
    //
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
    // # Select + return nothing is err
    //
    // # Select + join
    //
    // # Select limit
    //
    // # Select order
    //
    // # Select group
    //
    // # (select) Where, field equals
    //
    // # (select) Where, is null
    //
    // # (select) Where, is not null
    //
    // # Migrate - add field
    {
        let mut v = Version::default();
        let bananna = v.table("bananna");
        let hizat = bananna.field(&mut v, "z437INV6D", "hizat", field_str().build());
        let zomzom =
            bananna.field(&mut v, "zPREUVAOD", "zomzom", field_bool().migrate_fill(Expr::LitBool(true)).build());
        goodormning::pg::generate(&root.join("tests/pg_gen_migrate_add_field.rs"), vec![
            // Versions (previous)
            (0usize, {
                let mut v = Version::default();
                let bananna = v.table("bananna");
                let hizat = bananna.field(&mut v, "z437INV6D", "hizat", field_str().build());
                v.post_migration(
                    new_insert(
                        &bananna,
                        vec![(hizat.id.clone(), Expr::LitString("nizoot".into()))],
                    ).build_migration(),
                );
                v
            }),
            (1usize, v)
        ], vec![
            // Queries
            new_select(&bananna).output_fields(&[&hizat, &zomzom]).build_query("get_banan", QueryResCount::MaybeOne)
        ]).unwrap();
    }

    // # Migrate - remove field
    {
        let mut v = Version::default();
        let bananna = v.table("bananna");
        let hizat = bananna.field(&mut v, "z437INV6D", "hizat", field_str().build());
        goodormning::pg::generate(&root.join("tests/pg_gen_migrate_remove_field.rs"), vec![
            // Versions (previous)
            (0usize, {
                let mut v = Version::default();
                let bananna = v.table("bananna");
                bananna.field(&mut v, "z437INV6D", "hizat", field_str().build());
                bananna.field(&mut v, "zPREUVAOD", "zomzom", field_bool().build());
                v
            }),
            (1usize, v)
        ], vec![
            // Queries
            new_insert(&bananna, vec![(hizat.id.clone(), Expr::Param {
                name: "okolor".into(),
                type_: hizat.def.type_.type_.clone(),
            })]).build_query("new_banan", QueryResCount::None)
        ]).unwrap();
    }
    // # Migrate - add table
    //
    // # Migrate - remove table
    //
    // # Migrate - add index
    //
    // # Migrate - remove index
    //
    // # Migrate - add unique index
    //
    // # Migrate - add primary constraint
    //
    // # Migrate - remove primary constraint
    //
    // # Migrate - add fk constraint
    //
    // # Migrate - disallow new serial fields
    //
    // # Migrate - disallow duplicate tables
    //
    // # Migrate - disallow duplicate fields
    //
    // # Migrate - disallow duplicate indexes
    //
    // # Migrate - disallow duplicate constraints
    //
    // # Migrate - pre migration
    //
    // # Migrate - post migration
}
