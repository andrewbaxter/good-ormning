use std::borrow::Cow;
#[cfg(feature = "chrono")]
use chrono::{
    DateTime,
    Utc,
};

pub trait GoodOrmningCustomAuto<T> {
    fn to_sql(value: &T) -> i64;
    fn from_sql(value: i64) -> Result<T, String>;
}

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
pub trait GoodOrmningCustomUtcTime<T> {
    fn to_sql(value: &T) -> DateTime<Utc>;
    fn from_sql(value: DateTime<Utc>) -> Result<T, String>;
}
