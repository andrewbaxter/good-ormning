#[derive(Debug)]
pub struct GoodError(pub String);

impl std::fmt::Display for GoodError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for GoodError { }

impl From<rusqlite::Error> for GoodError {
    fn from(value: rusqlite::Error) -> Self {
        GoodError(value.to_string())
    }
}

fn lock_version(txn: &rusqlite::Transaction) -> Result<Option<i64>, GoodError> {
    let mut stmt = txn.prepare("update __good_version set lock = 1 where rid = 0 and lock = 0 returning version")?;
    let mut rows = stmt.query(())?;
    let version = match rows.next()? {
        Some(r) => {
            let ver: i64 = r.get(0usize)?;
            ver
        },
        None => return Ok(None),
    };
    drop(rows);
    stmt.finalize()?;
    Ok(Some(version))
}

fn unlock_version(txn: &rusqlite::Transaction, v: i64) -> Result<(), GoodError> {
    txn.execute("update __good_version set version = $1, lock = 0", rusqlite::params![v])?;
    Ok(())
}

fn prep_metadata_table(db: &mut rusqlite::Connection) -> Result<(), GoodError> {
    db.execute(
        "create table if not exists __good_version (rid int primary key, version bigint not null, lock int not null);",
        (),
    )?;
    db.execute("insert into __good_version (rid, version, lock) values (0, -1, 0) on conflict do nothing;", ())?;
    Ok(())
}

#[doc= "Sets up an uninitialized database, otherwise does nothing. Safe to call every start."]
pub fn initialize(db: &mut rusqlite::Connection) -> Result<(), GoodError> {
    prep_metadata_table(db)?;
    loop {
        let txn = db.transaction()?;
        match (|| {
            let version = match lock_version(&txn)? {
                Some(v) => v,
                None => {
                    return Ok(false);
                },
            };
            if version > 0i64 {
                return Err(
                    GoodError(
                        format!(
                            "The latest known version is {}, but the schema is at unknown version {}",
                            0i64,
                            version
                        ),
                    ),
                );
            }
            if version < 0i64 {
                txn.execute("create table \"users\" ( \"points\" integer not null , \"name\" text not null )", ())?;
            }
            unlock_version(&txn, 0i64)?;
            let out: Result<bool, GoodError> = Ok(true);
            out
        })() {
            Err(e) => {
                match txn.rollback() {
                    Err(e1) => {
                        return Err(
                            GoodError(
                                format!(
                                    "{}\n\nIn addition to above query error during the transaction, rollback failed: {}",
                                    e,
                                    e1
                                ),
                            ),
                        );
                    },
                    Ok(_) => {
                        return Err(e);
                    },
                };
            },
            Ok(migrated) => {
                match txn.commit() {
                    Err(e) => {
                        return Err(GoodError(format!("Error committing the migration transaction: {}", e)));
                    },
                    Ok(_) => {
                        if migrated {
                            return Ok(())
                        } else {
                            std::thread::sleep(std::time::Duration::from_millis(5 * 1000));
                        }
                    },
                };
            },
        }
    }
}

#[doc= "Does incremental migrations from the current migration to the latest version. In a single-server environment you can run this every startup, but otherwise you may wish to trigger this after old hosts are shut down during a backwards-compatible migration."]
pub fn migrate(db: &mut rusqlite::Connection) -> Result<(), GoodError> {
    prep_metadata_table(db)?;
    loop {
        let txn = db.transaction()?;
        match (|| {
            let version = match lock_version(&txn)? {
                Some(v) => v,
                None => {
                    return Ok(false);
                },
            };
            if version > 0i64 {
                return Err(
                    GoodError(
                        format!(
                            "The latest known version is {}, but the schema is at unknown version {}",
                            0i64,
                            version
                        ),
                    ),
                );
            }
            if version < 0i64 {
                txn.execute("create table \"users\" ( \"points\" integer not null , \"name\" text not null )", ())?;
            }
            unlock_version(&txn, 0i64)?;
            let out: Result<bool, GoodError> = Ok(true);
            out
        })() {
            Err(e) => {
                match txn.rollback() {
                    Err(e1) => {
                        return Err(
                            GoodError(
                                format!(
                                    "{}\n\nIn addition to above query error during the transaction, rollback failed: {}",
                                    e,
                                    e1
                                ),
                            ),
                        );
                    },
                    Ok(_) => {
                        return Err(e);
                    },
                };
            },
            Ok(migrated) => {
                match txn.commit() {
                    Err(e) => {
                        return Err(GoodError(format!("Error committing the migration transaction: {}", e)));
                    },
                    Ok(_) => {
                        if migrated {
                            return Ok(())
                        } else {
                            std::thread::sleep(std::time::Duration::from_millis(5 * 1000));
                        }
                    },
                };
            },
        }
    }
}

pub fn create_user(db: &mut rusqlite::Connection, name: &str, points: i64) -> Result<(), GoodError> {
    db
        .execute("insert into \"users\" ( \"name\" , \"points\" ) values ( $1 , $2 )", rusqlite::params![name, points])
        .map_err(|e| GoodError(e.to_string()))?;
    Ok(())
}

pub struct DbRes1 {
    pub name: String,
    pub points: i64,
}

pub fn get_user(db: &mut rusqlite::Connection, id: i64) -> Result<DbRes1, GoodError> {
    let mut stmt =
        db.prepare(
            "select \"users\" . \"name\" , \"users\" . \"points\" from \"users\" where ( \"users\" . \"rowid\" = $1 )",
        )?;
    let mut rows = stmt.query(rusqlite::params![id]).map_err(|e| GoodError(e.to_string()))?;
    let r = rows.next()?.ok_or_else(|| GoodError("Query expected to return one row but returned no rows".into()))?;
    Ok(DbRes1 {
        name: {
            let x: String = r.get(0usize)?;
            x
        },
        points: {
            let x: i64 = r.get(1usize)?;
            x
        },
    })
}

pub fn list_users(db: &mut rusqlite::Connection) -> Result<Vec<i64>, GoodError> {
    let mut out = vec![];
    let mut stmt = db.prepare("select \"users\" . \"rowid\" from \"users\"")?;
    let mut rows = stmt.query(rusqlite::params![]).map_err(|e| GoodError(e.to_string()))?;
    while let Some(r) = rows.next()? {
        out.push({
            let x: i64 = r.get(0usize)?;
            x
        });
    }
    Ok(out)
}
