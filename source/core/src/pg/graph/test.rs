use {
    crate::{
        graphmigrate::migrate,
        pg::{
            Version,
            graph::utils::PgMigrateCtx,
            schema::field::{
                field_i32,
                field_str,
            },
        },
        utils::Errs,
    },
    std::collections::HashMap,
};

/// Returns the statements for migrating from each version to the next (excluding
/// creating the first version).
fn migrate_versions(versions: Vec<Version>) -> Vec<String> {
    let errs = Errs::new();
    let mut statements = vec![];
    let mut prev_version: Option<Version> = None;
    for version in versions {
        let mut table_sql_names = HashMap::new();
        for (table_id, table) in &version.tables {
            table_sql_names.insert(table_id.clone(), table.id.clone());
        }
        let mut state = PgMigrateCtx::new(errs.clone(), table_sql_names, version.clone());
        let first = prev_version.is_none();
        migrate(&mut state, prev_version.take().map(|v| v.to_migrate_nodes()), &version.to_migrate_nodes());
        if !first {
            statements.extend(state.statements);
        }
        prev_version = Some(version);
    }
    errs.raise().unwrap();
    return statements;
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
    }]);
    assert_eq!(
        statements,
        vec![
            "alter table \"bananna\" rename to \"banana\"".to_string(),
            "alter table \"banana\" rename column \"hizat\" to \"hizat2\"".to_string(),
            "alter table \"banana\" rename constraint \"hizat_pk\" to \"hizat2_pk\"".to_string(),
        ]
    );
}

#[test]
fn test_change_referenced_pk_good() {
    let statements = migrate_versions(vec![{
        let v = Version::new();
        let bananna = v.table("bananna");
        let hizat = bananna.field("hizat", field_str().build());
        bananna.field("zomzom", field_i32().build());
        bananna.primary_key("bananna_pk", &[&hizat]);
        let peel = v.table("peel");
        let bananna_hizat = peel.field("bananna_hizat", field_str().build());
        peel.field("bananna_zomzom", field_i32().build());
        peel.foreign_key("peel_bananna", &[(&bananna_hizat, &hizat)]);
        v.build()
    }, {
        let v = Version::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_str().build());
        let zomzom = bananna.field("zomzom", field_i32().build());
        bananna.primary_key("bananna_pk", &[&zomzom]);
        let peel = v.table("peel");
        peel.field("bananna_hizat", field_str().build());
        let bananna_zomzom = peel.field("bananna_zomzom", field_i32().build());
        peel.foreign_key("peel_bananna", &[(&bananna_zomzom, &zomzom)]);
        v.build()
    }]);
    assert_eq!(
        statements,
        vec![
            "alter table \"peel\" drop constraint \"peel_bananna\"".to_string(),
            "alter table \"bananna\" drop constraint \"bananna_pk\"".to_string(),
            "alter table \"bananna\" add constraint \"bananna_pk\" primary key ( \"zomzom\" )".to_string(),
            "alter table \"peel\" add constraint \"peel_bananna\" foreign key ( \"bananna_zomzom\" ) references \"bananna\" ( \"zomzom\" )".to_string(),
        ]
    );
}

#[test]
fn test_new_tables_fk_good() {
    let errs = Errs::new();
    let v = Version::new();
    let bananna = v.table("bananna");
    let hizat = bananna.field("hizat", field_str().build());
    bananna.primary_key("bananna_pk", &[&hizat]);
    let peel = v.table("peel");
    let bananna_hizat = peel.field("bananna_hizat", field_str().build());
    peel.foreign_key("peel_bananna", &[(&bananna_hizat, &hizat)]);
    let version = v.build();
    let mut table_sql_names = HashMap::new();
    for (table_id, table) in &version.tables {
        table_sql_names.insert(table_id.clone(), table.id.clone());
    }
    let mut state = PgMigrateCtx::new(errs.clone(), table_sql_names, version.clone());
    migrate(&mut state, None, &version.to_migrate_nodes());
    assert_eq!(
        state.statements,
        vec![
            "create table \"peel\" ( \"bananna_hizat\" text not null )".to_string(),
            "create table \"bananna\" ( \"hizat\" text not null , constraint \"bananna_pk\" primary key ( \"hizat\" ) )".to_string(),
            "alter table \"peel\" add constraint \"peel_bananna\" foreign key ( \"bananna_hizat\" ) references \"bananna\" ( \"hizat\" )".to_string(),
        ]
    );
}

#[test]
fn test_add_pk_and_fk_good() {
    let statements = migrate_versions(vec![{
        let v = Version::new();
        let peel = v.table("aapeel");
        peel.field("bananna_hizat", field_str().build());
        let bananna = v.table("bananna");
        bananna.field("hizat", field_str().build());
        v.build()
    }, {
        let v = Version::new();
        let peel = v.table("aapeel");
        let bananna_hizat = peel.field("bananna_hizat", field_str().build());
        let bananna = v.table("bananna");
        let hizat = bananna.field("hizat", field_str().build());
        bananna.primary_key("bananna_pk", &[&hizat]);
        peel.foreign_key("peel_bananna", &[(&bananna_hizat, &hizat)]);
        v.build()
    }]);
    assert_eq!(
        statements,
        vec![
            "alter table \"bananna\" add constraint \"bananna_pk\" primary key ( \"hizat\" )".to_string(),
            "alter table \"aapeel\" add constraint \"peel_bananna\" foreign key ( \"bananna_hizat\" ) references \"bananna\" ( \"hizat\" )".to_string(),
        ]
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
        banana.index("banana_hizat2", &[&hizat2]).renamed_from("bananna_hizat");
        v.build()
    }]);
    assert_eq!(
        statements,
        vec![
            "alter table \"bananna\" rename to \"banana\"".to_string(),
            "alter table \"banana\" rename column \"hizat\" to \"hizat2\"".to_string(),
            "alter index \"bananna_hizat\" rename to \"banana_hizat2\"".to_string(),
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
    }]);
    assert_eq!(
        statements,
        vec![
            "drop index \"bananna_hizat\"".to_string(),
            "create unique index \"bananna_hizat\" on \"bananna\" ( \"hizat\" )".to_string(),
        ]
    );
}
