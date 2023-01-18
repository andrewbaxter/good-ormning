use integration_tests::MyString;
use anyhow::Result;

pub mod sqlite_gen_base_insert;
pub mod sqlite_gen_param_i32;
pub mod sqlite_gen_param_utctime_s;
pub mod sqlite_gen_param_utctime_ms;
pub mod sqlite_gen_param_opt_i32;
pub mod sqlite_gen_param_opt_i32_null;
pub mod sqlite_gen_param_custom;
pub mod sqlite_gen_param_opt_custom;
pub mod sqlite_gen_insert_on_conflict_do_nothing;
pub mod sqlite_gen_insert_on_conflict_update;
pub mod sqlite_gen_update;
pub mod sqlite_gen_update_where;
pub mod sqlite_gen_update_returning;
pub mod sqlite_gen_delete;
pub mod sqlite_gen_delete_where;
pub mod sqlite_gen_delete_returning;
pub mod sqlite_gen_select_join;
pub mod sqlite_gen_select_group_by;
pub mod sqlite_gen_select_order;
pub mod sqlite_gen_select_limit;
pub mod sqlite_gen_migrate_add_field;
pub mod sqlite_gen_migrate_detect_1;
pub mod sqlite_gen_migrate_detect_2;
pub mod sqlite_gen_migrate_remove_field;
pub mod sqlite_gen_migrate_add_table;
pub mod sqlite_gen_migrate_remove_table;
pub mod sqlite_gen_hello_world;

#[test]
fn test_hello_world() -> Result<()> {
    use sqlite_gen_hello_world as queries;

    let mut db = rusqlite::Connection::open_in_memory()?;
    queries::migrate(&mut db)?;
    queries::create_user(&mut db, "rust human", 0)?;
    for user_id in queries::list_users(&mut db)? {
        let user = queries::get_user(&mut db, user_id)?;
        println!("User {}: {}", user_id, user.name);
    }
    Ok(())
}

#[test]
fn test_base_insert() -> Result<()> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_base_insert::migrate(&mut db)?;
    sqlite_gen_base_insert::insert_banan(&mut db, "soy")?;
    assert_eq!(sqlite_gen_base_insert::get_banan(&mut db)?, "soy");
    Ok(())
}

#[test]
fn test_param_i32() -> Result<()> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_param_i32::migrate(&mut db)?;
    sqlite_gen_param_i32::insert_banan(&mut db, 22)?;
    assert_eq!(sqlite_gen_param_i32::get_banan(&mut db)?, 22);
    Ok(())
}

#[test]
fn test_param_utctime_s() -> Result<()> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_param_utctime_s::migrate(&mut db)?;
    let ref_date = chrono::TimeZone::with_ymd_and_hms(&chrono::Utc, 1937, 12, 1, 0, 0, 0).unwrap();
    sqlite_gen_param_utctime_s::insert_banan(&mut db, ref_date)?;
    assert_eq!(sqlite_gen_param_utctime_s::get_banan(&mut db)?, ref_date);
    Ok(())
}

#[test]
fn test_param_utctime_ms() -> Result<()> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_param_utctime_ms::migrate(&mut db)?;
    let ref_date = chrono::TimeZone::with_ymd_and_hms(&chrono::Utc, 1937, 12, 1, 0, 0, 0).unwrap();
    sqlite_gen_param_utctime_ms::insert_banan(&mut db, ref_date)?;
    assert_eq!(sqlite_gen_param_utctime_ms::get_banan(&mut db)?, ref_date);
    Ok(())
}

#[test]
fn test_param_opt_i32() -> Result<()> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_param_opt_i32::migrate(&mut db)?;
    sqlite_gen_param_opt_i32::insert_banan(&mut db, Some(47))?;
    assert_eq!(sqlite_gen_param_opt_i32::get_banan(&mut db)?, Some(47));
    Ok(())
}

#[test]
fn test_param_opt_i32_null() -> Result<()> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_param_opt_i32_null::migrate(&mut db)?;
    sqlite_gen_param_opt_i32_null::insert_banan(&mut db)?;
    assert_eq!(sqlite_gen_param_opt_i32_null::get_banan(&mut db)?, None);
    Ok(())
}

#[test]
fn test_param_custom() -> Result<()> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_param_custom::migrate(&mut db)?;
    sqlite_gen_param_custom::insert_banan(&mut db, &MyString("soy".into()))?;
    assert_eq!(sqlite_gen_param_custom::get_banan(&mut db)?, MyString("soy".into()));
    Ok(())
}

#[test]
fn test_param_opt_custom() -> Result<()> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_param_opt_custom::migrate(&mut db)?;
    sqlite_gen_param_opt_custom::insert_banan(&mut db, Some(&MyString("higgins".into())))?;
    assert_eq!(sqlite_gen_param_opt_custom::get_banan(&mut db)?, Some(MyString("higgins".into())));
    Ok(())
}

#[test]
fn test_insert_on_conflict_do_nothing() -> Result<()> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_insert_on_conflict_do_nothing::migrate(&mut db)?;
    assert!(sqlite_gen_insert_on_conflict_do_nothing::insert_banan(&mut db, "soy")?.is_some());
    assert!(sqlite_gen_insert_on_conflict_do_nothing::insert_banan(&mut db, "soy")?.is_none());
    Ok(())
}

#[test]
fn test_insert_on_conflict_update() -> Result<()> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_insert_on_conflict_update::migrate(&mut db)?;
    assert_eq!(sqlite_gen_insert_on_conflict_update::insert_banan(&mut db, "soy", 33)?, 33);
    assert_eq!(sqlite_gen_insert_on_conflict_update::insert_banan(&mut db, "soy", 7)?, 34);
    assert_eq!(sqlite_gen_insert_on_conflict_update::insert_banan(&mut db, "yyyy", 7)?, 7);
    Ok(())
}

#[test]
fn test_update() -> Result<()> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_update::migrate(&mut db)?;
    sqlite_gen_update::insert_banan(&mut db)?;
    assert_eq!(sqlite_gen_update::get_banan(&mut db)?, "yog");
    sqlite_gen_update::update_banan(&mut db)?;
    assert_eq!(sqlite_gen_update::get_banan(&mut db)?, "tep");
    Ok(())
}

#[test]
fn test_update_where() -> Result<()> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_update_where::migrate(&mut db)?;
    sqlite_gen_update_where::insert_banan(&mut db)?;
    assert_eq!(sqlite_gen_update_where::get_banan(&mut db)?, "yog");
    sqlite_gen_update_where::update_banan(&mut db, "tep", "yog2")?;
    assert_eq!(sqlite_gen_update_where::get_banan(&mut db)?, "yog");
    sqlite_gen_update_where::update_banan(&mut db, "tep", "yog")?;
    assert_eq!(sqlite_gen_update_where::get_banan(&mut db)?, "tep");
    Ok(())
}

#[test]
fn test_update_returning() -> Result<()> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_update_returning::migrate(&mut db)?;
    sqlite_gen_update_returning::insert_banan(&mut db)?;
    assert_eq!(sqlite_gen_update_returning::update_banan(&mut db)?, Some("tep".to_string()));
    Ok(())
}

#[test]
fn test_delete() -> Result<()> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_delete::migrate(&mut db)?;
    sqlite_gen_delete::insert_banan(&mut db)?;
    assert_eq!(sqlite_gen_delete::get_banan(&mut db)?, Some("seeon".to_string()));
    sqlite_gen_delete::no_banan(&mut db)?;
    assert_eq!(sqlite_gen_delete::get_banan(&mut db)?, None);
    Ok(())
}

#[test]
fn test_delete_where() -> Result<()> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_delete_where::migrate(&mut db)?;
    sqlite_gen_delete_where::insert_banan(&mut db)?;
    sqlite_gen_delete_where::no_banan(&mut db, "nozo")?;
    assert_eq!(sqlite_gen_delete_where::get_banan(&mut db)?, Some("seeon".to_string()));
    sqlite_gen_delete_where::no_banan(&mut db, "seeon")?;
    assert_eq!(sqlite_gen_delete_where::get_banan(&mut db)?, None);
    Ok(())
}

#[test]
fn test_delete_returning() -> Result<()> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_delete_where::migrate(&mut db)?;
    sqlite_gen_delete_where::insert_banan(&mut db)?;
    assert!(sqlite_gen_delete_where::get_banan(&mut db)?.is_some());
    sqlite_gen_delete_where::no_banan(&mut db, "seeon")?;
    assert!(sqlite_gen_delete_where::get_banan(&mut db)?.is_none());
    Ok(())
}

#[test]
fn test_select_join() -> Result<()> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_select_join::migrate(&mut db)?;
    let res = sqlite_gen_select_join::get_it(&mut db)?;
    assert_eq!(res.three, 33);
    assert_eq!(res.two, Some("no".into()));
    Ok(())
}

#[test]
fn test_select_group_by() -> Result<()> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_select_group_by::migrate(&mut db)?;
    sqlite_gen_select_group_by::insert_banan(&mut db, 1, 7)?;
    sqlite_gen_select_group_by::insert_banan(&mut db, 1, 99)?;
    sqlite_gen_select_group_by::insert_banan(&mut db, 2, 3)?;
    sqlite_gen_select_group_by::insert_banan(&mut db, 2, 10)?;
    let mut res = sqlite_gen_select_group_by::get_banan(&mut db)?;
    res.sort();
    assert_eq!(res, vec![13, 106]);
    Ok(())
}

#[test]
fn test_select_limit() -> Result<()> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_select_limit::migrate(&mut db)?;
    sqlite_gen_select_limit::insert_banan(&mut db, "soy")?;
    sqlite_gen_select_limit::insert_banan(&mut db, "soy")?;
    sqlite_gen_select_limit::insert_banan(&mut db, "soy")?;
    assert_eq!(sqlite_gen_select_limit::get_banan(&mut db)?.len(), 2);
    Ok(())
}

#[test]
fn test_select_order() -> Result<()> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_select_order::migrate(&mut db)?;
    sqlite_gen_select_order::insert_banan(&mut db, 0)?;
    sqlite_gen_select_order::insert_banan(&mut db, 12)?;
    sqlite_gen_select_order::insert_banan(&mut db, 9)?;
    assert_eq!(sqlite_gen_select_order::get_banan(&mut db)?, vec![0, 9, 12]);
    Ok(())
}

#[test]
fn test_migrate_add_field() -> Result<()> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_migrate_add_field::migrate(&mut db)?;
    match sqlite_gen_migrate_add_field::get_banan(&mut db)? {
        Some(x) => {
            assert_eq!(x.zomzom, true);
            assert_eq!(&x.hizat, "nizoot");
        },
        None => assert!(false),
    };
    Ok(())
}

#[test]
fn test_migrate_detect() -> Result<()> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_migrate_detect_2::migrate(&mut db)?;
    assert!(matches!(sqlite_gen_migrate_detect_1::get_banan(&mut db), Err(good_ormning::runtime::Error::BadSchema)));
    Ok(())
}

#[test]
fn test_migrate_remove_field() -> Result<()> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_migrate_remove_field::migrate(&mut db)?;
    sqlite_gen_migrate_remove_field::new_banan(&mut db, "yordol")?;
    Ok(())
}

#[test]
fn test_migrate_add_table() -> Result<()> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_migrate_add_table::migrate(&mut db)?;
    sqlite_gen_migrate_add_table::two(&mut db, 23)?;
    Ok(())
}

#[test]
fn test_migrate_remove_table() -> Result<()> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_migrate_remove_table::migrate(&mut db)?;
    Ok(())
}
