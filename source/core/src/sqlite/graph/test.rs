use {
    crate::{
        graphmigrate::migrate,
        sqlite::{
            Version,
            graph::utils::SqliteMigrateCtx,
            schema::field::{
                field_i32,
                field_str,
            },
        },
        utils::Errs,
    },
    std::collections::HashMap,
};

/// Returns the statements for migrating through each version in order.
fn migrate_versions(versions: Vec<Version>) -> Result<Vec<String>, Vec<String>> {
    let errs = Errs::new();
    let mut statements = vec![];
    let mut prev_version: Option<Version> = None;
    for version in versions {
        let mut table_sql_names = HashMap::new();
        for (table_id, table) in &version.tables {
            table_sql_names.insert(table_id.clone(), table.id.clone());
        }
        let mut state = SqliteMigrateCtx::new(errs.clone(), table_sql_names, version.clone());
        migrate(&mut state, prev_version.take().map(|v| v.to_migrate_nodes()), &version.to_migrate_nodes());
        statements.extend(state.statements);
        prev_version = Some(version);
    }
    errs.raise()?;
    return Ok(statements);
}

fn assert_constraint_err(res: Result<Vec<String>, Vec<String>>) {
    let errs = res.unwrap_err();
    assert!(errs.iter().any(|e| e.contains("constraints on an existing table")), "{:?}", errs);
}

#[test]
fn test_add_constraint_bad() {
    assert_constraint_err(migrate_versions(vec![{
        let v = Version::new();
        v.table("bananna").field("hizat", field_str().build());
        v.build()
    }, {
        let v = Version::new();
        let bananna = v.table("bananna");
        let hizat = bananna.field("hizat", field_str().build());
        bananna.primary_key("hizat_pk", &[&hizat]);
        v.build()
    }]));
}

#[test]
fn test_change_constraint_bad() {
    assert_constraint_err(migrate_versions(vec![{
        let v = Version::new();
        let bananna = v.table("bananna");
        let hizat = bananna.field("hizat", field_str().build());
        bananna.field("zomzom", field_i32().build());
        bananna.primary_key("bananna_pk", &[&hizat]);
        v.build()
    }, {
        let v = Version::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_str().build());
        let zomzom = bananna.field("zomzom", field_i32().build());
        bananna.primary_key("bananna_pk", &[&zomzom]);
        v.build()
    }]));
}

#[test]
fn test_remove_constraint_bad() {
    assert_constraint_err(migrate_versions(vec![{
        let v = Version::new();
        let bananna = v.table("bananna");
        let hizat = bananna.field("hizat", field_str().build());
        bananna.primary_key("hizat_pk", &[&hizat]);
        v.build()
    }, {
        let v = Version::new();
        v.table("bananna").field("hizat", field_str().build());
        v.build()
    }]));
}

#[test]
fn test_rename_constraint_parts_good() {
    let statements = migrate_versions(vec![{
        let v = Version::new();
        let bananna = v.table("bananna");
        let hizat = bananna.field("hizat", field_str().build());
        bananna.primary_key("hizat_pk", &[&hizat]);
        let peel = v.table("peel");
        let bananna_hizat = peel.field("bananna_hizat", field_str().build());
        peel.foreign_key("peel_bananna", &[(&bananna_hizat, &hizat)]);
        v.build()
    }, {
        let v = Version::new();
        let banana = v.table("banana").renamed_from("bananna");
        let hizat2 = banana.field("hizat2", field_str().build()).renamed_from("hizat");
        banana.primary_key("hizat2_pk", &[&hizat2]).renamed_from("hizat_pk");
        let peel = v.table("peel");
        let bananna_hizat = peel.field("bananna_hizat", field_str().build());
        peel.foreign_key("peel_bananna", &[(&bananna_hizat, &hizat2)]);
        v.build()
    }]).unwrap();
    assert_eq!(
        &statements[2..],
        &[
            "alter table \"bananna\" rename to \"banana\"".to_string(),
            "alter table \"banana\" rename column \"hizat\" to \"hizat2\"".to_string(),
        ]
    );
}

#[test]
fn test_rebuild_table_good() {
    let statements = migrate_versions(vec![{
        let v = Version::new();
        let bananna = v.table("bananna");
        let hizat = bananna.field("hizat", field_str().build());
        bananna.field("zomzom", field_i32().build());
        bananna.primary_key("bananna_pk", &[&hizat]);
        v.build()
    }, {
        let v = Version::new();
        let bananna = v.table("bananna");
        let hizat = bananna.field("hizat", field_str().build());
        bananna.field("zomzom", field_i32().build());
        bananna.primary_key("bananna_pk", &[&hizat]);
        let bananna_new = v.table("bananna_new");
        bananna_new.field("hizat", field_str().build());
        let zomzom = bananna_new.field("zomzom", field_i32().build());
        bananna_new.primary_key("bananna_pk", &[&zomzom]);
        v.build()
    }, {
        let v = Version::new();
        let bananna = v.table("bananna").renamed_from("bananna_new");
        bananna.field("hizat", field_str().build());
        let zomzom = bananna.field("zomzom", field_i32().build());
        bananna.primary_key("bananna_pk", &[&zomzom]);
        v.build()
    }]).unwrap();
    assert_eq!(
        &statements[2..],
        &["drop table \"bananna\"".to_string(), "alter table \"bananna_new\" rename to \"bananna\"".to_string()]
    );
}

#[test]
fn test_rename_index_parts_good() {
    let statements = migrate_versions(vec![{
        let v = Version::new();
        let bananna = v.table("bananna");
        let hizat = bananna.field("hizat", field_str().build());
        bananna.index("bananna_hizat", &[&hizat]);
        v.build()
    }, {
        let v = Version::new();
        let banana = v.table("banana").renamed_from("bananna");
        let hizat2 = banana.field("hizat2", field_str().build()).renamed_from("hizat");
        banana.index("bananna_hizat", &[&hizat2]);
        v.build()
    }, {
        let v = Version::new();
        let banana = v.table("banana");
        let hizat2 = banana.field("hizat2", field_str().build());
        banana.index("banana_hizat2", &[&hizat2]).renamed_from("bananna_hizat");
        v.build()
    }]).unwrap();
    assert_eq!(
        &statements[2..],
        &[
            "alter table \"bananna\" rename to \"banana\"".to_string(),
            "alter table \"banana\" rename column \"hizat\" to \"hizat2\"".to_string(),
            "drop index \"bananna_hizat\"".to_string(),
            "create index \"banana_hizat2\" on \"banana\" ( \"hizat2\" )".to_string(),
        ]
    );
}

#[test]
fn test_change_index_unique_good() {
    let statements = migrate_versions(vec![{
        let v = Version::new();
        let bananna = v.table("bananna");
        let hizat = bananna.field("hizat", field_str().build());
        bananna.index("bananna_hizat", &[&hizat]);
        v.build()
    }, {
        let v = Version::new();
        let bananna = v.table("bananna");
        let hizat = bananna.field("hizat", field_str().build());
        bananna.unique_index("bananna_hizat", &[&hizat]);
        v.build()
    }]).unwrap();
    assert_eq!(
        &statements[2..],
        &[
            "drop index \"bananna_hizat\"".to_string(),
            "create unique index \"bananna_hizat\" on \"bananna\" ( \"hizat\" )".to_string(),
        ]
    );
}

#[test]
fn test_rowid_alias_good() {
    let statements = migrate_versions(vec![{
        let v = Version::new();
        let t = v.table("t");
        t.rowid_field(Some("id"));
        t.field("a", field_str().build());
        v.build()
    }]).unwrap();
    assert_eq!(&statements, &["create table \"t\" ( \"a\" text not null , \"id\" integer primary key )".to_string()]);
}

#[test]
fn test_remove_and_change_field_good() {
    use crate::sqlite::query::expr::SerialExpr;

    let statements = migrate_versions(vec![{
        let v = Version::new();
        let t = v.table("t");
        t.field("a", field_str().build());
        t.field("b", field_i32().build());
        v.build()
    }, {
        let v = Version::new();
        let t = v.table("t");
        t.field("a", field_str().build());
        t.field("b", field_str().migrate_fill(SerialExpr::LitString("x".into())).build());
        v.build()
    }, {
        let v = Version::new();
        let t = v.table("t");
        t.field("a", field_str().build());
        v.build()
    }]).unwrap();
    assert_eq!(
        &statements[1..],
        &[
            "alter table \"t\" drop column \"b\"".to_string(),
            "alter table \"t\" add column \"b\" text not null default 'x'".to_string(),
            "alter table \"t\" drop column \"b\"".to_string(),
        ]
    );
}

fn rename_versions(swap: bool, reverse: bool) -> Vec<Version> {
    let (n0, n1, n2) = if reverse {
        ("c", "b", "a")
    } else {
        ("a", "b", "c")
    };
    let v0 = Version::new();
    {
        let t = v0.table("t");
        let a = t.field(n0, field_i32().build());
        let b = t.field(n1, field_i32().build());
        t.index(&format!("i{}", n0), &[&a]);
        t.index(&format!("i{}", n1), &[&b]);
        v0.table(&format!("t{}", n0)).field("x", field_i32().build());
        v0.table(&format!("t{}", n1)).field("x", field_i32().build());
    }
    let v1 = Version::new();
    {
        let t = v1.table("t");
        if swap {
            let a = t.field(n0, field_i32().build()).renamed_from(n1);
            let b = t.field(n1, field_i32().build()).renamed_from(n0);
            t.index(&format!("i{}", n0), &[&a]).renamed_from(&format!("i{}", n1));
            t.index(&format!("i{}", n1), &[&b]).renamed_from(&format!("i{}", n0));
            v1.table(&format!("t{}", n0)).renamed_from(&format!("t{}", n1)).field("x", field_i32().build());
            v1.table(&format!("t{}", n1)).renamed_from(&format!("t{}", n0)).field("x", field_i32().build());
        } else {
            let b = t.field(n1, field_i32().build()).renamed_from(n0);
            let c = t.field(n2, field_i32().build()).renamed_from(n1);
            t.index(&format!("i{}", n1), &[&b]).renamed_from(&format!("i{}", n0));
            t.index(&format!("i{}", n2), &[&c]).renamed_from(&format!("i{}", n1));
            v1.table(&format!("t{}", n1)).renamed_from(&format!("t{}", n0)).field("x", field_i32().build());
            v1.table(&format!("t{}", n2)).renamed_from(&format!("t{}", n1)).field("x", field_i32().build());
        }
    }
    return vec![v0.build(), v1.build()];
}

#[test]
fn test_rename_chain_good() {
    for (
        reverse,
        expected,
    ) in [
        (
            false,
            vec![
                "drop index \"ib\"",
                "drop index \"ia\"",
                "alter table \"tb\" rename to \"tc\"",
                "alter table \"ta\" rename to \"tb\"",
                "alter table \"t\" rename column \"b\" to \"c\"",
                "create index \"ic\" on \"t\" ( \"c\" )",
                "alter table \"t\" rename column \"a\" to \"b\"",
                "create index \"ib\" on \"t\" ( \"b\" )",
            ],
        ),
        (
            true,
            vec![
                "drop index \"ic\"",
                "drop index \"ib\"",
                "alter table \"tb\" rename to \"ta\"",
                "alter table \"tc\" rename to \"tb\"",
                "alter table \"t\" rename column \"b\" to \"a\"",
                "alter table \"t\" rename column \"c\" to \"b\"",
                "create index \"ib\" on \"t\" ( \"b\" )",
                "create index \"ia\" on \"t\" ( \"a\" )",
            ],
        ),
    ] {
        let statements = migrate_versions(rename_versions(false, reverse)).unwrap();
        assert_eq!(&statements[statements.len() - expected.len()..], &expected[..]);
    }
}

#[test]
fn test_rename_swap_bad() {
    let errs = migrate_versions(rename_versions(true, false)).unwrap_err();
    assert_eq!(errs.len(), 2, "{:?}", errs);
    for expected in ["update table tb, update table ta", "update field t.b, update field t.a"] {
        assert!(errs.iter().any(|e| e.ends_with(expected)), "{} not in {:?}", expected, errs);
    }
}

#[test]
fn test_index_swap_good() {
    let v0 = Version::new();
    {
        let t = v0.table("t");
        let a = t.field("a", field_i32().build());
        let b = t.field("b", field_i32().build());
        t.index("i1", &[&a]);
        t.index("i2", &[&b]);
    }
    let v1 = Version::new();
    {
        let t = v1.table("t");
        let a = t.field("a", field_i32().build());
        let b = t.field("b", field_i32().build());
        t.index("i1", &[&b]).renamed_from("i2");
        t.index("i2", &[&a]).renamed_from("i1");
    }
    let statements = migrate_versions(vec![v0.build(), v1.build()]).unwrap();
    assert_eq!(
        &statements[statements.len() - 4..],
        &[
            "drop index \"i2\"",
            "drop index \"i1\"",
            "create index \"i1\" on \"t\" ( \"b\" )",
            "create index \"i2\" on \"t\" ( \"a\" )"
        ]
    );
}
