use {
    crate::{
        graphmigrate::migrate,
        mariadb::{
            Version,
            graph::utils::MariadbMigrateCtx,
            schema::field::{
                field_i32,
                field_str,
            },
        },
        utils::Errs,
    },
    std::collections::HashMap,
};

fn migrate_versions_result(versions: Vec<Version>) -> Result<Vec<String>, Vec<String>> {
    let errs = Errs::new();
    let mut statements = vec![];
    let mut prev_version: Option<Version> = None;
    for version in versions {
        let mut table_sql_names = HashMap::new();
        for (table_id, table) in &version.tables {
            table_sql_names.insert(table_id.clone(), table.id.clone());
        }
        let mut state = MariadbMigrateCtx::new(errs.clone(), table_sql_names, version.clone());
        let first = prev_version.is_none();
        migrate(&mut state, prev_version.take().map(|v| v.to_migrate_nodes()), &version.to_migrate_nodes());
        if !first {
            statements.extend(state.statements);
        }
        prev_version = Some(version);
    }
    errs.raise()?;
    return Ok(statements);
}

fn migrate_versions(versions: Vec<Version>) -> Vec<String> {
    return migrate_versions_result(versions).unwrap();
}

#[test]
fn test_rename_constraint_parts_good() {
    let statements = migrate_versions(vec![{
        let v = Version::new();
        let bananna = v.table("bananna");
        let hizat = bananna.field("hizat", field_str().max_len(100).build());
        bananna.primary_key(&[&hizat]);
        let peel = v.table("peel");
        let bananna_hizat = peel.field("bananna_hizat", field_str().max_len(100).build());
        peel.foreign_key("peel_bananna", &[(&bananna_hizat, &hizat)]);
        v.build()
    }, {
        let v = Version::new();
        let banana = v.table("banana").renamed_from("bananna");
        let hizat2 = banana.field("hizat2", field_str().max_len(100).build()).renamed_from("hizat");
        banana.primary_key(&[&hizat2]);
        let peel = v.table("peel");
        let bananna_hizat = peel.field("bananna_hizat", field_str().max_len(100).build());
        peel.foreign_key("peel_bananna", &[(&bananna_hizat, &hizat2)]);
        v.build()
    }]);
    assert_eq!(
        statements,
        vec![
            "rename table if exists `bananna` to `banana`".to_string(),
            "alter table `banana` rename column if exists `hizat` to `hizat2`".to_string(),
        ]
    );
}

#[test]
fn test_change_referenced_pk_good() {
    let statements = migrate_versions(vec![{
        let v = Version::new();
        let bananna = v.table("bananna");
        let hizat = bananna.field("hizat", field_str().max_len(100).build());
        bananna.field("zomzom", field_i32().build());
        bananna.primary_key(&[&hizat]);
        let peel = v.table("peel");
        let bananna_hizat = peel.field("bananna_hizat", field_str().max_len(100).build());
        peel.field("bananna_zomzom", field_i32().build());
        peel.foreign_key("peel_bananna", &[(&bananna_hizat, &hizat)]);
        v.build()
    }, {
        let v = Version::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_str().max_len(100).build());
        let zomzom = bananna.field("zomzom", field_i32().build());
        bananna.primary_key(&[&zomzom]);
        let peel = v.table("peel");
        peel.field("bananna_hizat", field_str().max_len(100).build());
        let bananna_zomzom = peel.field("bananna_zomzom", field_i32().build());
        peel.foreign_key("peel_bananna", &[(&bananna_zomzom, &zomzom)]);
        v.build()
    }]);
    assert_eq!(
        statements,
        vec![
            "alter table `peel` drop foreign key if exists `peel_bananna` , drop index if exists `peel_bananna`".to_string(),
            "alter table `bananna` drop index if exists `PRIMARY`".to_string(),
            "alter table `bananna` drop index if exists `PRIMARY` , add primary key ( `zomzom` )".to_string(),
            "alter table `peel` add constraint `peel_bananna` foreign key if not exists ( `bananna_zomzom` ) references `bananna` ( `zomzom` )".to_string(),
        ]
    );
}

#[test]
fn test_new_tables_fk_good() {
    let errs = Errs::new();
    let v = Version::new();
    let bananna = v.table("bananna");
    let hizat = bananna.field("hizat", field_str().max_len(100).build());
    bananna.primary_key(&[&hizat]);
    let peel = v.table("peel");
    let bananna_hizat = peel.field("bananna_hizat", field_str().max_len(100).build());
    peel.foreign_key("peel_bananna", &[(&bananna_hizat, &hizat)]);
    let version = v.build();
    let mut table_sql_names = HashMap::new();
    for (table_id, table) in &version.tables {
        table_sql_names.insert(table_id.clone(), table.id.clone());
    }
    let mut state = MariadbMigrateCtx::new(errs.clone(), table_sql_names, version.clone());
    migrate(&mut state, None, &version.to_migrate_nodes());
    assert_eq!(
        state.statements,
        vec![
            "create table if not exists `peel` ( `bananna_hizat` varchar(100) not null )".to_string(),
            "create table if not exists `bananna` ( `hizat` varchar(100) not null , primary key ( `hizat` ) )".to_string(),
            "alter table `peel` add constraint `peel_bananna` foreign key if not exists ( `bananna_hizat` ) references `bananna` ( `hizat` )".to_string(),
        ]
    );
}

#[test]
fn test_add_pk_and_fk_good() {
    let statements = migrate_versions(vec![{
        let v = Version::new();
        let peel = v.table("aapeel");
        peel.field("bananna_hizat", field_str().max_len(100).build());
        let bananna = v.table("bananna");
        bananna.field("hizat", field_str().max_len(100).build());
        v.build()
    }, {
        let v = Version::new();
        let peel = v.table("aapeel");
        let bananna_hizat = peel.field("bananna_hizat", field_str().max_len(100).build());
        let bananna = v.table("bananna");
        let hizat = bananna.field("hizat", field_str().max_len(100).build());
        bananna.primary_key(&[&hizat]);
        peel.foreign_key("peel_bananna", &[(&bananna_hizat, &hizat)]);
        v.build()
    }]);
    assert_eq!(
        statements,
        vec![
            "alter table `bananna` drop index if exists `PRIMARY` , add primary key ( `hizat` )".to_string(),
            "alter table `aapeel` add constraint `peel_bananna` foreign key if not exists ( `bananna_hizat` ) references `bananna` ( `hizat` )".to_string(),
        ]
    );
}

#[test]
fn test_rename_index_parts_good() {
    let statements = migrate_versions(vec![{
        let v = Version::new();
        let bananna = v.table("bananna");
        let hizat = bananna.field("hizat", field_str().max_len(100).build());
        bananna.index("bananna_hizat", &[&hizat]);
        v.build()
    }, {
        let v = Version::new();
        let banana = v.table("banana").renamed_from("bananna");
        let hizat2 = banana.field("hizat2", field_str().max_len(100).build()).renamed_from("hizat");
        banana.index("banana_hizat2", &[&hizat2]).renamed_from("bananna_hizat");
        v.build()
    }]);
    assert_eq!(
        statements,
        vec![
            "rename table if exists `bananna` to `banana`".to_string(),
            "alter table `banana` rename column if exists `hizat` to `hizat2`".to_string(),
            "alter table `banana` rename index if exists `bananna_hizat` to `banana_hizat2`".to_string(),
        ]
    );
}

#[test]
fn test_change_index_unique_good() {
    let statements = migrate_versions(vec![{
        let v = Version::new();
        let bananna = v.table("bananna");
        let hizat = bananna.field("hizat", field_str().max_len(100).build());
        bananna.index("bananna_hizat", &[&hizat]);
        v.build()
    }, {
        let v = Version::new();
        let bananna = v.table("bananna");
        let hizat = bananna.field("hizat", field_str().max_len(100).build());
        bananna.unique_index("bananna_hizat", &[&hizat]);
        v.build()
    }]);
    assert_eq!(
        statements,
        vec![
            "alter table `bananna` drop index if exists `bananna_hizat`".to_string(),
            "alter table `bananna` add unique index if not exists `bananna_hizat` ( `hizat` )".to_string(),
        ]
    );
}

#[test]
fn test_auto_increment_key_changes_good() {
    use crate::mariadb::schema::field::field_i64;

    let table = |f: &dyn Fn(&crate::mariadb::TableHandle)| {
        let v = Version::new();
        let t = v.table("t");
        f(&t);
        return v.build();
    };
    let cases: Vec<(&str, Version, Version)> = vec![("index_to_pk", table(&|t| {
        let id = t.field("id", field_i64().auto_increment().build());
        let other = t.field("other", field_i32().build());
        t.primary_key(&[&other]);
        t.index("t_id", &[&id]);
    }), table(&|t| {
        let id = t.field("id", field_i64().auto_increment().build());
        t.field("other", field_i32().build());
        t.primary_key(&[&id]);
    })), ("same_name", table(&|t| {
        let id = t.field("id", field_i64().auto_increment().build());
        t.field("other", field_i32().build());
        t.index("t_key", &[&id]);
    }), table(&|t| {
        let id = t.field("id", field_i64().auto_increment().build());
        let other = t.field("other", field_i32().build());
        t.index("t_key", &[&id, &other]);
    })), ("drop_auto", table(&|t| {
        let id = t.field("id", field_i64().auto_increment().build());
        t.field("other", field_i32().build());
        t.index("t_id", &[&id]);
    }), table(&|t| {
        t.field("id", field_i64().build());
        t.field("other", field_i32().build());
    })), ("drop_column", table(&|t| {
        let id = t.field("id", field_i64().auto_increment().build());
        t.field("other", field_i32().build());
        t.index("t_id", &[&id]);
    }), table(&|t| {
        t.field("other", field_i32().build());
    })), ("new_column", table(&|t| {
        t.field("other", field_i32().build());
    }), table(&|t| {
        let id = t.field("id", field_i64().auto_increment().build());
        t.field("other", field_i32().build());
        t.primary_key(&[&id]);
    })), ("add_auto_new_key", table(&|t| {
        t.field("id", field_i64().build());
        t.field("other", field_i32().build());
    }), table(&|t| {
        let id = t.field("id", field_i64().auto_increment().build());
        t.field("other", field_i32().build());
        t.unique_index("t_id", &[&id]);
    }))];
    let expected: Vec<Vec<&str>> =
        vec![
            vec![
                "alter table `t` drop index if exists `PRIMARY`",
                "alter table `t` drop index if exists `PRIMARY` , add primary key ( `id` )",
                "alter table `t` drop index if exists `t_id`",
            ],
            vec!["alter table `t` drop index if exists `t_key` , add index `t_key` ( `id` , `other` )"],
            vec!["alter table `t` drop index if exists `t_id` , modify column if exists `id` bigint not null"],
            vec!["alter table `t` drop index if exists `t_id` , drop column if exists `id`"],
            vec![
                "alter table `t` add column if not exists `id` bigint auto_increment not null , drop index if exists `PRIMARY` , add primary key ( `id` )",
            ],
            vec![
                "alter table `t` modify column if exists `id` bigint auto_increment not null , drop index if exists `t_id` , add unique index `t_id` ( `id` )",
            ],
        ];
    for ((name, a, b), expected) in cases.into_iter().zip(expected) {
        assert_eq!(migrate_versions(vec![a, b]), expected, "{}", name);
    }
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
                "rename table if exists `tb` to `tc`",
                "rename table if exists `ta` to `tb`",
                "alter table `t` rename column if exists `b` to `c`",
                "alter table `t` rename index if exists `ib` to `ic`",
                "alter table `t` rename column if exists `a` to `b`",
                "alter table `t` rename index if exists `ia` to `ib`",
            ],
        ),
        (
            true,
            vec![
                "rename table if exists `tb` to `ta`",
                "rename table if exists `tc` to `tb`",
                "alter table `t` rename column if exists `b` to `a`",
                "alter table `t` rename column if exists `c` to `b`",
                "alter table `t` rename index if exists `ib` to `ia`",
                "alter table `t` rename index if exists `ic` to `ib`",
            ],
        ),
    ] {
        let statements = migrate_versions_result(rename_versions(false, reverse)).unwrap();
        assert_eq!(&statements[statements.len() - expected.len()..], &expected[..]);
    }
}

#[test]
fn test_rename_swap_bad() {
    let errs = migrate_versions_result(rename_versions(true, false)).unwrap_err();
    assert_eq!(errs.len(), 3, "{:?}", errs);
    for expected in [
        "table ta to table tb, table tb to table ta",
        "column t.a to column t.b, column t.b to column t.a",
        "index t.ia to index t.ib, index t.ib to index t.ia",
    ] {
        assert!(errs.iter().any(|e| e.ends_with(expected)), "{} not in {:?}", expected, errs);
    }
}
