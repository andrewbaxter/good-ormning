#[cfg(feature = "chrono")]
use chrono::{
    DateTime,
    Utc,
    FixedOffset,
};
use crate::runtime::GoodError;
#[cfg(feature = "jiff")]
use jiff::{
    Zoned,
    Timestamp,
};

pub trait GoodErrorQuery<T> {
    fn to_good_error_query(self, query: &str) -> Result<T, GoodError>;
}

impl<T> GoodErrorQuery<T> for Result<T, GoodError> {
    fn to_good_error_query(self, query: &str) -> Result<T, GoodError> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(GoodError(format!("Error executing query [{}]: {}", query, e))),
        }
    }
}

impl<T> GoodErrorQuery<T> for Result<T, rusqlite::Error> {
    fn to_good_error_query(self, query: &str) -> Result<T, GoodError> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(GoodError(format!("Error executing query [{}]: {}", query, e))),
        }
    }
}

pub trait GoodOrmningCustomAuto<T> {
    fn from_sql(value: i64) -> Result<T, String>;
    fn to_sql(value: &T) -> i64;
}

pub trait GoodOrmningCustomBool<T> {
    fn from_sql(value: bool) -> Result<T, String>;
    fn to_sql(value: &T) -> bool;
}

pub trait GoodOrmningCustomBytes<T> {
    fn from_sql(value: Vec<u8>) -> Result<T, String>;
    fn to_sql<'a>(value: &'a T) -> Vec<u8>;
}

pub trait GoodOrmningCustomF32<T> {
    fn from_sql(value: f32) -> Result<T, String>;
    fn to_sql(value: &T) -> f32;
}

pub trait GoodOrmningCustomF64<T> {
    fn from_sql(value: f64) -> Result<T, String>;
    fn to_sql(value: &T) -> f64;
}

#[cfg(feature = "chrono")]
pub trait GoodOrmningCustomFixedOffsetTimeChrono<T> {
    fn from_sql(value: DateTime<FixedOffset>) -> Result<T, String>;
    fn to_sql(value: &T) -> DateTime<FixedOffset>;
}

#[cfg(feature = "jiff")]
pub trait GoodOrmningCustomFixedOffsetTimeJiff<T> {
    fn from_sql(value: Zoned) -> Result<T, String>;
    fn to_sql(value: &T) -> Zoned;
}

pub trait GoodOrmningCustomI16<T> {
    fn from_sql(value: i16) -> Result<T, String>;
    fn to_sql(value: &T) -> i16;
}

pub trait GoodOrmningCustomI32<T> {
    fn from_sql(value: i32) -> Result<T, String>;
    fn to_sql(value: &T) -> i32;
}

pub trait GoodOrmningCustomI64<T> {
    fn from_sql(value: i64) -> Result<T, String>;
    fn to_sql(value: &T) -> i64;
}

pub trait GoodOrmningCustomString<T> {
    fn from_sql(value: String) -> Result<T, String>;
    fn to_sql<'a>(value: &'a T) -> String;
}

pub trait GoodOrmningCustomU32<T> {
    fn from_sql(value: u32) -> Result<T, String>;
    fn to_sql(value: &T) -> u32;
}

#[cfg(feature = "chrono")]
pub trait GoodOrmningCustomUtcTimeChrono<T> {
    fn from_sql(value: DateTime<Utc>) -> Result<T, String>;
    fn to_sql(value: &T) -> DateTime<Utc>;
}

#[cfg(feature = "jiff")]
pub trait GoodOrmningCustomUtcTimeJiff<T> {
    fn from_sql(value: Timestamp) -> Result<T, String>;
    fn to_sql(value: &T) -> Timestamp;
}

pub trait SqliteConnection {
    fn execute(&mut self, query: &str, params: impl rusqlite::Params) -> Result<usize, GoodError>;
    fn load_array_module(&mut self) -> Result<(), GoodError>;
    fn query<
        T,
        F: FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
    >(&mut self, query: &str, params: impl rusqlite::Params, f: F) -> Result<Vec<T>, GoodError>;
}

impl<T: SqliteConnection + ?Sized> SqliteConnection for &mut T {
    fn execute(&mut self, query: &str, params: impl rusqlite::Params) -> Result<usize, GoodError> {
        (**self).execute(query, params)
    }

    fn load_array_module(&mut self) -> Result<(), GoodError> {
        (**self).load_array_module()
    }

    fn query<
        Res,
        F: FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<Res>,
    >(&mut self, query: &str, params: impl rusqlite::Params, f: F) -> Result<Vec<Res>, GoodError> {
        (**self).query(query, params, f)
    }
}

impl SqliteConnection for rusqlite::Connection {
    fn execute(&mut self, query: &str, params: impl rusqlite::Params) -> Result<usize, GoodError> {
        rusqlite::Connection::execute(self, query, params).map_err(|e| GoodError(e.to_string()))
    }

    fn load_array_module(&mut self) -> Result<(), GoodError> {
        rusqlite::vtab::array::load_module(self).map_err(|e| GoodError(e.to_string()))
    }

    fn query<
        T,
        F: FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
    >(&mut self, query: &str, params: impl rusqlite::Params, mut f: F) -> Result<Vec<T>, GoodError> {
        let mut stmt = self.prepare(query).map_err(|e| GoodError(e.to_string()))?;
        let rows = stmt.query_map(params, |row| f(row)).map_err(|e| GoodError(e.to_string()))?;
        let mut res = vec![];
        for row in rows {
            res.push(row.map_err(|e| GoodError(e.to_string()))?);
        }
        Ok(res)
    }
}

impl SqliteConnection for rusqlite::Transaction<'_> {
    fn execute(&mut self, query: &str, params: impl rusqlite::Params) -> Result<usize, GoodError> {
        rusqlite::Connection::execute(self, query, params).map_err(|e| GoodError(e.to_string()))
    }

    fn load_array_module(&mut self) -> Result<(), GoodError> {
        // Assume loaded on connection
        Ok(())
    }

    fn query<
        T,
        F: FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
    >(&mut self, query: &str, params: impl rusqlite::Params, mut f: F) -> Result<Vec<T>, GoodError> {
        let mut stmt = self.prepare(query).map_err(|e| GoodError(e.to_string()))?;
        let rows = stmt.query_map(params, |row| f(row)).map_err(|e| GoodError(e.to_string()))?;
        let mut res = vec![];
        for row in rows {
            res.push(row.map_err(|e| GoodError(e.to_string()))?);
        }
        Ok(res)
    }
}

pub enum GoodOrmningSqliteTimestamp {
    I64(i64),
    String(String),
}

impl rusqlite::types::FromSql for GoodOrmningSqliteTimestamp {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> Result<Self, rusqlite::types::FromSqlError> {
        match value {
            rusqlite::types::ValueRef::Text(s) => {
                let s = std::str::from_utf8(s).map_err(|e| rusqlite::types::FromSqlError::Other(Box::new(e)))?;
                Ok(GoodOrmningSqliteTimestamp::String(s.to_string()))
            },
            rusqlite::types::ValueRef::Integer(i) => {
                Ok(GoodOrmningSqliteTimestamp::I64(i))
            },
            _ => Err(rusqlite::types::FromSqlError::InvalidType),
        }
    }
}

impl rusqlite::types::ToSql for GoodOrmningSqliteTimestamp {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        match self {
            GoodOrmningSqliteTimestamp::String(s) => Ok(
                rusqlite::types::ToSqlOutput::Owned(rusqlite::types::Value::Text(s.clone())),
            ),
            GoodOrmningSqliteTimestamp::I64(i) => Ok(
                rusqlite::types::ToSqlOutput::Owned(rusqlite::types::Value::Integer(*i)),
            ),
        }
    }
}

impl From<GoodOrmningSqliteTimestamp> for rusqlite::types::Value {
    fn from(val: GoodOrmningSqliteTimestamp) -> Self {
        match val {
            GoodOrmningSqliteTimestamp::String(s) => rusqlite::types::Value::Text(s),
            GoodOrmningSqliteTimestamp::I64(i) => rusqlite::types::Value::Integer(i),
        }
    }
}
