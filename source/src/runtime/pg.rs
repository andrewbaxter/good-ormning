use {
    async_trait::async_trait,
    std::borrow::Cow,
    crate::runtime::GoodError,
};
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

pub trait GoodErrorQuery<T> {
    fn to_good_error_query(self, query: &str) -> Result<T, GoodError>;
}

impl<T> GoodErrorQuery<T> for Result<T, tokio_postgres::Error> {
    fn to_good_error_query(self, query: &str) -> Result<T, GoodError> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(GoodError(format!("Error executing query [{}]: {}", query, e))),
        }
    }
}

impl<T> GoodErrorQuery<T> for Result<T, GoodError> {
    fn to_good_error_query(self, query: &str) -> Result<T, GoodError> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(GoodError(format!("Error executing query [{}]: {}", query, e))),
        }
    }
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

#[cfg(feature = "deadpool")]
#[async_trait]
impl PgConnection for deadpool_postgres::Object {
    async fn execute(
        &mut self,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<u64, GoodError> {
        tokio_postgres::GenericClient::execute(self, query, params).await.map_err(|e| GoodError(e.to_string()))
    }

    async fn query(
        &mut self,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<Vec<tokio_postgres::Row>, GoodError> {
        tokio_postgres::GenericClient::query(self, query, params).await.map_err(|e| GoodError(e.to_string()))
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
        self.get().await.map_err(|e| GoodError(e.to_string()))?.execute(query, params).await.map_err(|e| GoodError(e.to_string()))
    }

    async fn query(
        &mut self,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<Vec<tokio_postgres::Row>, GoodError> {
        self.get().await.map_err(|e| GoodError(e.to_string()))?.query(query, params).await.map_err(|e| GoodError(e.to_string()))
    }
}

pub trait GoodOrmningCustomAuto<T> {
    fn to_sql(value: &T) -> i64;
    fn from_sql(value: i64) -> Result<T, String>;
}

pub trait GoodOrmningCustomBool<T> {
    fn to_sql(value: &T) -> bool;
    fn from_sql(value: bool) -> Result<T, String>;
}

pub trait GoodOrmningCustomI16<T> {
    fn to_sql(value: &T) -> i16;
    fn from_sql(value: i16) -> Result<T, String>;
}

pub trait GoodOrmningCustomI32<T> {
    fn to_sql(value: &T) -> i32;
    fn from_sql(value: i32) -> Result<T, String>;
}

pub trait GoodOrmningCustomI64<T> {
    fn to_sql(value: &T) -> i64;
    fn from_sql(value: i64) -> Result<T, String>;
}

pub trait GoodOrmningCustomU32<T> {
    fn to_sql(value: &T) -> i64;
    fn from_sql(value: i64) -> Result<T, String>;
}

pub trait GoodOrmningCustomF32<T> {
    fn to_sql(value: &T) -> f32;
    fn from_sql(value: f32) -> Result<T, String>;
}

pub trait GoodOrmningCustomF64<T> {
    fn to_sql(value: &T) -> f64;
    fn from_sql(value: f64) -> Result<T, String>;
}

pub trait GoodOrmningCustomString<T> {
    fn to_sql(value: &T) -> &str;
    fn from_sql(value: String) -> Result<T, String>;
}

pub trait GoodOrmningCustomBytes<T> {
    fn to_sql<'a>(value: &'a T) -> Cow<'a, [u8]>;
    fn from_sql(value: Vec<u8>) -> Result<T, String>;
}

#[cfg(feature = "chrono")]
pub trait GoodOrmningCustomUtcTimeChrono<T> {
    fn to_sql(value: &T) -> DateTime<Utc>;
    fn from_sql(value: DateTime<Utc>) -> Result<T, String>;
}

#[cfg(feature = "chrono")]
pub trait GoodOrmningCustomFixedOffsetTimeChrono<T> {
    fn to_sql(value: &T) -> DateTime<FixedOffset>;
    fn from_sql(value: DateTime<FixedOffset>) -> Result<T, String>;
}

#[cfg(feature = "jiff")]
pub trait GoodOrmningCustomUtcTimeJiff<T> {
    fn to_sql(value: &T) -> Timestamp;
    fn from_sql(value: Timestamp) -> Result<T, String>;
}

#[cfg(feature = "jiff")]
pub trait GoodOrmningCustomFixedOffsetTimeJiff<T> {
    fn to_sql(value: &T) -> Zoned;
    fn from_sql(value: Zoned) -> Result<T, String>;
}
