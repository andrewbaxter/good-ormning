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
        expr::{
            Expr,
            BinOp,
        },
        select::{
            Join,
            NamedSelectSource,
            JoinSource,
            JoinType,
        },
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
        let bananna = v.table("zEOIWAACJ");
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
        let bananna = v.table("zJCPRHK37");
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
        let bananna = v.table("z8JI0I1E4");
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
        let bananna = v.table("zT7F4746C");
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
        let bananna = v.table("zH2Q9TOLG");
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
        let bananna = v.table("z202QTVDB");
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
        let bananna = v.table("zSPEZNHA8");
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
        let bananna = v.table("zLBDEHGRB");
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
    // # Select + join
    {
        let mut v = Version::default();
        let bananna = v.table("zT6D0LWI8");
        let hizat = bananna.field(&mut v, "z437INV6D", "hizat", field_str().build());
        let three = bananna.field(&mut v, "zVXQUXEXT", "three", field_i32().build());
        let one = v.table("zQ8SFVHEV");
        let hizat1 = one.field(&mut v, "zDZA6FVSS", "hizat", field_str().build());
        let two = one.field(&mut v, "z7KU525LW", "two", field_str().build());
        v.post_migration(
            new_insert(
                &bananna,
                vec![(hizat.id.clone(), Expr::LitString("key".into())), (three.id.clone(), Expr::LitI32(33))],
            ).build_migration(),
        );
        v.post_migration(
            new_insert(
                &one,
                vec![
                    (hizat1.id.clone(), Expr::LitString("key".into())),
                    (two.id.clone(), Expr::LitString("no".into()))
                ],
            ).build_migration(),
        );
        goodormning::pg::generate(
            &root.join("tests/pg_gen_select_join.rs"),
            vec![(0usize, v)],
            vec![new_select(&bananna).join(Join {
                source: Box::new(NamedSelectSource {
                    source: JoinSource::Table(one.0.clone()),
                    alias: None,
                }),
                type_: JoinType::Left,
                on: Expr::BinOp {
                    left: Box::new(Expr::Field(hizat.id.clone())),
                    op: BinOp::Equals,
                    right: Box::new(Expr::Field(hizat1.id.clone())),
                },
            }).output_field(&three).output_field(&two).build_query("get_it", QueryResCount::One)],
        ).unwrap();
    }

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
        let bananna = v.table("zTWA93SX0");
        let hizat = bananna.field(&mut v, "z437INV6D", "hizat", field_str().build());
        let zomzom =
            bananna.field(&mut v, "zPREUVAOD", "zomzom", field_bool().migrate_fill(Expr::LitBool(true)).build());
        goodormning::pg::generate(&root.join("tests/pg_gen_migrate_add_field.rs"), vec![
            // Versions (previous)
            (0usize, {
                let mut v = Version::default();
                let bananna = v.table("zTWA93SX0");
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
        let bananna = v.table("z1MD8L1CZ");
        let hizat = bananna.field(&mut v, "z437INV6D", "hizat", field_str().build());
        goodormning::pg::generate(&root.join("tests/pg_gen_migrate_remove_field.rs"), vec![
            // Versions (previous)
            (0usize, {
                let mut v = Version::default();
                let bananna = v.table("z1MD8L1CZ");
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
    {
        let mut v = Version::default();
        let bananna = v.table("z4RGW742J");
        bananna.field(&mut v, "z437INV6D", "hizat", field_str().build());
        let two = v.table("zHXF3YVGQ");
        let field_two = two.field(&mut v, "z156A4Q8W", "two", field_i32().build());
        goodormning::pg::generate(&root.join("tests/pg_gen_migrate_add_table.rs"), vec![
            // Versions (previous)
            (0usize, {
                let mut v = Version::default();
                let bananna = v.table("z4RGW742J");
                bananna.field(&mut v, "z437INV6D", "hizat", field_str().build());
                v
            }),
            (1usize, v)
        ], vec![
            // Queries
            new_insert(&two, vec![(field_two.id.clone(), Expr::Param {
                name: "two".into(),
                type_: field_two.def.type_.type_.clone(),
            })]).build_query("two", QueryResCount::None)
        ]).unwrap();
    }

    // # Migrate - remove table
    {
        let mut v = Version::default();
        let bananna = v.table("zX7CEK8JC");
        bananna.field(&mut v, "z437INV6D", "hizat", field_str().build());
        goodormning::pg::generate(&root.join("tests/pg_gen_migrate_remove_table.rs"), vec![
            // Versions (previous)
            (0usize, {
                let mut v = Version::default();
                let bananna = v.table("zX7CEK8JC");
                bananna.field(&mut v, "z437INV6D", "hizat", field_str().build());
                let two = v.table("z45HT1YW2");
                two.field(&mut v, "z156A4Q8W", "two", field_i32().build());
                v
            }),
            (1usize, v)
        ], vec![]).unwrap();
    }

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
    // # Migrate - disallow duplicate indexes
    //
    // # Migrate - disallow duplicate constraints
    //
    // # Migrate - pre migration
    {
        let mut v0 = Version::default();
        let v0_bananna = v0.table("zMI5V9F2V");
        v0_bananna.field(&mut v0, "z437INV6D", "hizat", field_str().build());
        let v0_two = v0.table("z450WBJCO");
        let v0_field_two = v0_two.field(&mut v0, "z156A4Q8W", "two", field_i32().build());
        let mut v1 = Version::default();
        v1.pre_migration(new_insert(&v0_two, vec![(v0_field_two.id.clone(), Expr::LitI32(7))]).build_migration());
        let v1_bananna = v1.table("zMI5V9F2V");
        v1_bananna.field(&mut v1, "z437INV6D", "hizat", field_str().build());
        goodormning::pg::generate(&root.join("tests/pg_gen_migrate_pre_migration.rs"), vec![
            // Versions (previous)
            (0usize, v0),
            (1usize, v1)
        ], vec![]).unwrap();
    }
}
