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
    assert_eq!(&statements[2..], &[
        "alter table \"bananna\" rename to \"banana\"".to_string(),
        "alter table \"banana\" rename column \"hizat\" to \"hizat2\"".to_string(),
        "drop index \"bananna_hizat\"".to_string(),
        "create index \"banana_hizat2\" on \"banana\" ( \"hizat2\" )".to_string(),
    ]);
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
    assert_eq!(&statements[2..], &[
        "drop index \"bananna_hizat\"".to_string(),
        "create unique index \"bananna_hizat\" on \"bananna\" ( \"hizat\" )".to_string(),
    ]);
}
