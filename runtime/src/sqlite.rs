#[cfg(feature = "chrono")]
use chrono::{
    DateTime,
    Utc,
    FixedOffset,
};
#[cfg(feature = "jiff")]
use jiff::{
    Zoned,
    Timestamp,
};

pub trait GoodOrmningCustomBool<T> {
    fn to_sql(value: &T) -> bool;
    fn from_sql(value: bool) -> Result<T, String>;
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
    fn to_sql(value: &T) -> u32;
    fn from_sql(value: u32) -> Result<T, String>;
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
    fn to_sql<'a>(value: &'a T) -> String;
    fn from_sql(value: String) -> Result<T, String>;
}

pub trait GoodOrmningCustomBytes<T> {
    fn to_sql<'a>(value: &'a T) -> Vec<u8>;
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
