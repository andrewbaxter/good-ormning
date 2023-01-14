use integrationtests::MyString;
use anyhow::Result;

pub mod sqlite_gen_base_insert;
pub mod sqlite_gen_param_i32;
pub mod sqlite_gen_param_opt_i32;
pub mod sqlite_gen_param_opt_i32_null;
pub mod sqlite_gen_param_custom;
pub mod sqlite_gen_param_opt_custom;
pub mod sqlite_gen_migrate_add_field;
pub mod sqlite_gen_migrate_remove_field;
pub mod sqlite_gen_migrate_add_table;
pub mod sqlite_gen_migrate_remove_table;
pub mod sqlite_gen_select_join;
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

#[test]
fn test_select_join() -> Result<()> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    sqlite_gen_select_join::migrate(&mut db)?;
    let res = sqlite_gen_select_join::get_it(&mut db)?;
    assert_eq!(res.three, 33);
    assert_eq!(res.two, Some("no".into()));
    Ok(())
}
