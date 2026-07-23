#[cfg(feature = "chrono")]
use chrono::{
    DateTime,
    FixedOffset,
    Utc,
};
#[cfg(feature = "jiff")]
use jiff::{
    Timestamp,
    Zoned,
};
use {
    async_trait::async_trait,
    crate::runtime::GoodError,
    std::borrow::Cow,
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

impl<T> GoodErrorQuery<T> for Result<T, tokio_postgres::Error> {
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
    fn to_sql<'a>(value: &'a T) -> Cow<'a, [u8]>;
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
    fn to_sql(value: &T) -> &str;
}

pub trait GoodOrmningCustomU32<T> {
    fn from_sql(value: i64) -> Result<T, String>;
    fn to_sql(value: &T) -> i64;
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

#[async_trait]
pub trait PgConnection: Send {
    async fn execute(
        &mut self,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<u64, GoodError>;
    async fn query(
        &mut self,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<Vec<tokio_postgres::Row>, GoodError>;
}

#[cfg(feature = "deadpool")]
#[async_trait]
impl PgConnection for deadpool_postgres::Object {
    async fn execute(
        &mut self,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<u64, GoodError> {
        use std::ops::DerefMut;

        tokio_postgres::Client::execute(self.deref_mut().deref_mut(), query, params)
            .await
            .map_err(|e| GoodError(e.to_string()))
    }

    async fn query(
        &mut self,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<Vec<tokio_postgres::Row>, GoodError> {
        use std::ops::DerefMut;

        tokio_postgres::Client::query(self.deref_mut().deref_mut(), query, params)
            .await
            .map_err(|e| GoodError(e.to_string()))
    }
}

#[cfg(feature = "deadpool")]
#[async_trait]
impl PgConnection for deadpool_postgres::Pool {
    async fn execute(
        &mut self,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<u64, GoodError> {
        self
            .get()
            .await
            .map_err(|e| GoodError(e.to_string()))?
            .execute(query, params)
            .await
            .map_err(|e| GoodError(e.to_string()))
    }

    async fn query(
        &mut self,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<Vec<tokio_postgres::Row>, GoodError> {
        self
            .get()
            .await
            .map_err(|e| GoodError(e.to_string()))?
            .query(query, params)
            .await
            .map_err(|e| GoodError(e.to_string()))
    }
}

#[async_trait]
impl PgConnection for tokio_postgres::Client {
    async fn execute(
        &mut self,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<u64, GoodError> {
        tokio_postgres::Client::execute(self, query, params).await.map_err(|e| GoodError(e.to_string()))
    }

    async fn query(
        &mut self,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<Vec<tokio_postgres::Row>, GoodError> {
        tokio_postgres::Client::query(self, query, params).await.map_err(|e| GoodError(e.to_string()))
    }
}

#[async_trait]
impl PgConnection for tokio_postgres::Transaction<'_> {
    async fn execute(
        &mut self,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<u64, GoodError> {
        tokio_postgres::Transaction::execute(self, query, params).await.map_err(|e| GoodError(e.to_string()))
    }

    async fn query(
        &mut self,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<Vec<tokio_postgres::Row>, GoodError> {
        tokio_postgres::Transaction::query(self, query, params).await.map_err(|e| GoodError(e.to_string()))
    }
}
