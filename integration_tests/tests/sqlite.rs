use chrono::{
    Utc,
    TimeZone,
};
use integration_tests::MyString;

pub mod sqlite_gen_base_insert;
pub mod sqlite_gen_constraint;
pub mod sqlite_gen_param_i32;
pub mod sqlite_gen_param_utctime_s;
pub mod sqlite_gen_param_utctime_ms;
pub mod sqlite_gen_param_opt_i32;
pub mod sqlite_gen_param_opt_i32_null;
pub mod sqlite_gen_param_arr_i32;
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
pub mod sqlite_gen_migrate_rename_field;
pub mod sqlite_gen_migrate_remove_field;
pub mod sqlite_gen_migrate_add_table;
pub mod sqlite_gen_migrate_rename_table;
pub mod sqlite_gen_migrate_remove_table;
pub mod sqlite_gen_select_cte;
pub mod sqlite_gen_select_window;
pub mod sqlite_gen_select_junction;
pub mod sqlite_gen_hello_world;

#[test]
fn test_hello_world() -> Result<(), loga::Error> {
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
fn test_base_insert() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_base_insert::migrate(&mut db)?;
    sqlite_gen_base_insert::insert_banan(&mut db, "soy")?;
    assert_eq!(sqlite_gen_base_insert::get_banan(&mut db)?, "soy");
    Ok(())
}

#[test]
fn test_constraint() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_base_insert::migrate(&mut db)?;
    Ok(())
}

#[test]
fn test_param_i32() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_param_i32::migrate(&mut db)?;
    sqlite_gen_param_i32::insert_banan(&mut db, 22)?;
    assert_eq!(sqlite_gen_param_i32::get_banan(&mut db)?, 22);
    Ok(())
}

#[test]
fn test_param_utctime_s() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_param_utctime_s::migrate(&mut db)?;
    let ref_date = chrono::TimeZone::with_ymd_and_hms(&chrono::Utc, 1937, 12, 1, 0, 0, 0).unwrap();
    sqlite_gen_param_utctime_s::insert_banan(&mut db, ref_date)?;
    assert_eq!(sqlite_gen_param_utctime_s::get_banan(&mut db)?, ref_date);
    Ok(())
}

#[test]
fn test_param_utctime_ms() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_param_utctime_ms::migrate(&mut db)?;
    let ref_date = chrono::TimeZone::with_ymd_and_hms(&chrono::Utc, 1937, 12, 1, 0, 0, 0).unwrap();
    sqlite_gen_param_utctime_ms::insert_banan(&mut db, ref_date)?;
    assert_eq!(sqlite_gen_param_utctime_ms::get_banan(&mut db)?, ref_date);
    Ok(())
}

#[test]
fn test_param_opt_i32() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_param_opt_i32::migrate(&mut db)?;
    sqlite_gen_param_opt_i32::insert_banan(&mut db, Some(47))?;
    assert_eq!(sqlite_gen_param_opt_i32::get_banan(&mut db)?, Some(47));
    Ok(())
}

#[test]
fn test_param_opt_i32_null() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_param_opt_i32_null::migrate(&mut db)?;
    sqlite_gen_param_opt_i32_null::insert_banan(&mut db)?;
    assert_eq!(sqlite_gen_param_opt_i32_null::get_banan(&mut db)?, None);
    Ok(())
}

#[test]
fn test_param_arr_i32() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_param_arr_i32::migrate(&mut db)?;
    sqlite_gen_param_arr_i32::insert_banan(&mut db, 7)?;
    assert_eq!(sqlite_gen_param_arr_i32::get_banan(&mut db, vec![7])?, Some(7));
    Ok(())
}

#[test]
fn test_param_custom() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_param_custom::migrate(&mut db)?;
    let x_0 = integration_tests::MyBool(true);
    let x_1 = integration_tests::MyI32(13);
    let x_2 = integration_tests::MyI64(-22);
    let x_3 = integration_tests::MyU32(14);
    let x_4 = integration_tests::MyF32(12.);
    let x_5 = integration_tests::MyF64(99.);
    let x_6 = integration_tests::MyBytes("hi".as_bytes().to_vec());
    let x_7 = integration_tests::MyString("hogo".to_string());
    let x_8 = integration_tests::MyUtctime(Utc.with_ymd_and_hms(1999, 11, 14, 1, 2, 13).unwrap());
    let x_9 = integration_tests::MyUtctime(Utc.with_ymd_and_hms(1999, 6, 14, 10, 13, 57).unwrap());
    sqlite_gen_param_custom::insert_banan(&mut db, &x_0, &x_1, &x_2, &x_3, &x_4, &x_5, &x_6, &x_7, &x_8, &x_9)?;
    let res = sqlite_gen_param_custom::get_banan(&mut db)?;
    assert_eq!(x_0, res.x_0);
    assert_eq!(x_1, res.x_1);
    assert_eq!(x_2, res.x_2);
    assert_eq!(x_3, res.x_3);
    assert_eq!(x_4, res.x_4);
    assert_eq!(x_5, res.x_5);
    assert_eq!(x_6, res.x_6);
    assert_eq!(x_7, res.x_7);
    assert_eq!(x_8, res.x_8);
    assert_eq!(x_9, res.x_9);
    Ok(())
}

#[test]
fn test_param_opt_custom() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_param_opt_custom::migrate(&mut db)?;
    sqlite_gen_param_opt_custom::insert_banan(&mut db, Some(&MyString("higgins".into())))?;
    assert_eq!(sqlite_gen_param_opt_custom::get_banan(&mut db)?, Some(MyString("higgins".into())));
    Ok(())
}

#[test]
fn test_insert_on_conflict_do_nothing() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_insert_on_conflict_do_nothing::migrate(&mut db)?;
    assert!(sqlite_gen_insert_on_conflict_do_nothing::insert_banan(&mut db, "soy")?.is_some());
    assert!(sqlite_gen_insert_on_conflict_do_nothing::insert_banan(&mut db, "soy")?.is_none());
    Ok(())
}

#[test]
fn test_insert_on_conflict_update() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_insert_on_conflict_update::migrate(&mut db)?;
    assert_eq!(sqlite_gen_insert_on_conflict_update::insert_banan(&mut db, "soy", 33)?, 33);
    assert_eq!(sqlite_gen_insert_on_conflict_update::insert_banan(&mut db, "soy", 7)?, 34);
    assert_eq!(sqlite_gen_insert_on_conflict_update::insert_banan(&mut db, "yyyy", 7)?, 7);
    Ok(())
}

#[test]
fn test_update() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_update::migrate(&mut db)?;
    sqlite_gen_update::insert_banan(&mut db)?;
    assert_eq!(sqlite_gen_update::get_banan(&mut db)?, "yog");
    sqlite_gen_update::update_banan(&mut db)?;
    assert_eq!(sqlite_gen_update::get_banan(&mut db)?, "tep");
    Ok(())
}

#[test]
fn test_update_where() -> Result<(), loga::Error> {
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
fn test_update_returning() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_update_returning::migrate(&mut db)?;
    sqlite_gen_update_returning::insert_banan(&mut db)?;
    assert_eq!(sqlite_gen_update_returning::update_banan(&mut db)?, Some("tep".to_string()));
    Ok(())
}

#[test]
fn test_delete() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_delete::migrate(&mut db)?;
    sqlite_gen_delete::insert_banan(&mut db)?;
    assert_eq!(sqlite_gen_delete::get_banan(&mut db)?, Some("seeon".to_string()));
    sqlite_gen_delete::no_banan(&mut db)?;
    assert_eq!(sqlite_gen_delete::get_banan(&mut db)?, None);
    Ok(())
}

#[test]
fn test_delete_where() -> Result<(), loga::Error> {
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
fn test_delete_returning() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_delete_where::migrate(&mut db)?;
    sqlite_gen_delete_where::insert_banan(&mut db)?;
    assert!(sqlite_gen_delete_where::get_banan(&mut db)?.is_some());
    sqlite_gen_delete_where::no_banan(&mut db, "seeon")?;
    assert!(sqlite_gen_delete_where::get_banan(&mut db)?.is_none());
    Ok(())
}

#[test]
fn test_select_join() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_select_join::migrate(&mut db)?;
    let res = sqlite_gen_select_join::get_it(&mut db)?;
    assert_eq!(res.three, 33);
    assert_eq!(res.two, Some("no".into()));
    Ok(())
}

#[test]
fn test_select_group_by() -> Result<(), loga::Error> {
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
fn test_select_limit() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_select_limit::migrate(&mut db)?;
    sqlite_gen_select_limit::insert_banan(&mut db, "soy")?;
    sqlite_gen_select_limit::insert_banan(&mut db, "soy")?;
    sqlite_gen_select_limit::insert_banan(&mut db, "soy")?;
    assert_eq!(sqlite_gen_select_limit::get_banan(&mut db)?.len(), 2);
    Ok(())
}

#[test]
fn test_select_order() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_select_order::migrate(&mut db)?;
    sqlite_gen_select_order::insert_banan(&mut db, 0)?;
    sqlite_gen_select_order::insert_banan(&mut db, 12)?;
    sqlite_gen_select_order::insert_banan(&mut db, 9)?;
    assert_eq!(sqlite_gen_select_order::get_banan(&mut db)?, vec![0, 9, 12]);
    Ok(())
}

#[test]
fn test_migrate_add_field() -> Result<(), loga::Error> {
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
fn test_migrate_rename_field() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_migrate_rename_field::migrate(&mut db)?;
    sqlite_gen_migrate_rename_field::ins(&mut db)?;
    Ok(())
}

#[test]
fn test_migrate_remove_field() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_migrate_remove_field::migrate(&mut db)?;
    sqlite_gen_migrate_remove_field::new_banan(&mut db, "yordol")?;
    Ok(())
}

#[test]
fn test_migrate_add_table() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_migrate_add_table::migrate(&mut db)?;
    sqlite_gen_migrate_add_table::two(&mut db, 23)?;
    Ok(())
}

#[test]
fn test_migrate_rename_table() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_migrate_rename_table::migrate(&mut db)?;
    sqlite_gen_migrate_rename_table::two(&mut db, "inset")?;
    Ok(())
}

#[test]
fn test_migrate_remove_table() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_migrate_remove_table::migrate(&mut db)?;
    Ok(())
}

#[test]
fn test_select_cte() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_select_cte::migrate(&mut db)?;
    sqlite_gen_select_cte::insert_banan(&mut db, 1, 7)?;
    sqlite_gen_select_cte::insert_banan(&mut db, 1, 99)?;
    let mut res = sqlite_gen_select_cte::get_banan(&mut db)?;
    res.sort();
    assert_eq!(res, vec![7, 99]);
    Ok(())
}

#[test]
fn test_select_window() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_select_window::migrate(&mut db)?;
    sqlite_gen_select_window::insert_banan(&mut db, 1, 7)?;
    sqlite_gen_select_window::insert_banan(&mut db, 1, 99)?;
    sqlite_gen_select_window::insert_banan(&mut db, 2, 3)?;
    sqlite_gen_select_window::insert_banan(&mut db, 2, 10)?;
    let mut res =
        sqlite_gen_select_window::get_banan(&mut db)?
            .into_iter()
            .map(|x| (x.hizat, x.hizat2, x.zombo))
            .collect::<Vec<_>>();
    res.sort();
    assert_eq!(res, vec![(1, 7, 99), (1, 99, 99), (2, 3, 99), (2, 10, 99)]);
    Ok(())
}

#[test]
fn test_select_junction() -> Result<(), loga::Error> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_select_junction::migrate(&mut db)?;
    sqlite_gen_select_junction::insert_banan(&mut db, 1, 7)?;
    sqlite_gen_select_junction::insert_banan(&mut db, 2, 3)?;
    let mut res = sqlite_gen_select_junction::get_banan(&mut db)?;
    res.sort();
    assert_eq!(res, vec![1, 2, 3, 7]);
    Ok(())
}
