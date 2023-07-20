use std::borrow::Cow;
use chrono::{
    DateTime,
    Utc,
};
use good_ormning_runtime::{
    pg,
    sqlite,
};

#[derive(PartialEq, Eq, Debug)]
pub struct MyBool(pub bool);

impl pg::GoodOrmningCustomBool<MyBool> for MyBool {
    fn to_sql(value: &MyBool) -> bool {
        value.0
    }

    fn from_sql(s: bool) -> Result<MyBool, String> {
        Ok(Self(s))
    }
}

impl sqlite::GoodOrmningCustomBool<MyBool> for MyBool {
    fn to_sql<'a>(value: &'a MyBool) -> bool {
        value.0
    }

    fn from_sql(s: bool) -> Result<MyBool, String> {
        Ok(Self(s))
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct MyAuto(pub i64);

impl pg::GoodOrmningCustomAuto<MyAuto> for MyAuto {
    fn to_sql(value: &MyAuto) -> i64 {
        value.0
    }

    fn from_sql(s: i64) -> Result<MyAuto, String> {
        Ok(Self(s))
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct MyI32(pub i32);

impl pg::GoodOrmningCustomI32<MyI32> for MyI32 {
    fn to_sql(value: &MyI32) -> i32 {
        value.0
    }

    fn from_sql(s: i32) -> Result<MyI32, String> {
        Ok(Self(s))
    }
}

impl sqlite::GoodOrmningCustomI32<MyI32> for MyI32 {
    fn to_sql<'a>(value: &'a MyI32) -> i32 {
        value.0
    }

    fn from_sql(s: i32) -> Result<MyI32, String> {
        Ok(Self(s))
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct MyI64(pub i64);

impl pg::GoodOrmningCustomI64<MyI64> for MyI64 {
    fn to_sql(value: &MyI64) -> i64 {
        value.0
    }

    fn from_sql(s: i64) -> Result<MyI64, String> {
        Ok(Self(s))
    }
}

impl sqlite::GoodOrmningCustomI64<MyI64> for MyI64 {
    fn to_sql<'a>(value: &'a MyI64) -> i64 {
        value.0
    }

    fn from_sql(s: i64) -> Result<MyI64, String> {
        Ok(Self(s))
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct MyU32(pub u32);

impl sqlite::GoodOrmningCustomU32<MyU32> for MyU32 {
    fn to_sql<'a>(value: &'a MyU32) -> u32 {
        value.0
    }

    fn from_sql(s: u32) -> Result<MyU32, String> {
        Ok(Self(s))
    }
}

#[derive(PartialEq, Debug)]
pub struct MyF32(pub f32);

impl pg::GoodOrmningCustomF32<MyF32> for MyF32 {
    fn to_sql(value: &MyF32) -> f32 {
        value.0
    }

    fn from_sql(s: f32) -> Result<MyF32, String> {
        Ok(Self(s))
    }
}

impl sqlite::GoodOrmningCustomF32<MyF32> for MyF32 {
    fn to_sql<'a>(value: &'a MyF32) -> f32 {
        value.0
    }

    fn from_sql(s: f32) -> Result<MyF32, String> {
        Ok(Self(s))
    }
}

#[derive(PartialEq, Debug)]
pub struct MyF64(pub f64);

impl pg::GoodOrmningCustomF64<MyF64> for MyF64 {
    fn to_sql(value: &MyF64) -> f64 {
        value.0
    }

    fn from_sql(s: f64) -> Result<MyF64, String> {
        Ok(Self(s))
    }
}

impl sqlite::GoodOrmningCustomF64<MyF64> for MyF64 {
    fn to_sql<'a>(value: &'a MyF64) -> f64 {
        value.0
    }

    fn from_sql(s: f64) -> Result<MyF64, String> {
        Ok(Self(s))
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct MyString(pub String);

impl pg::GoodOrmningCustomString<MyString> for MyString {
    fn to_sql(value: &MyString) -> &str {
        &value.0
    }

    fn from_sql(s: String) -> Result<MyString, String> {
        Ok(Self(s))
    }
}

impl sqlite::GoodOrmningCustomString<MyString> for MyString {
    fn to_sql<'a>(value: &'a MyString) -> Cow<'a, str> {
        Cow::Borrowed(&value.0)
    }

    fn from_sql(s: String) -> Result<MyString, String> {
        Ok(Self(s))
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct MyBytes(pub Vec<u8>);

impl pg::GoodOrmningCustomBytes<MyBytes> for MyBytes {
    fn to_sql<'a>(value: &'a MyBytes) -> Cow<'a, [u8]> {
        Cow::Borrowed(&value.0)
    }

    fn from_sql(s: Vec<u8>) -> Result<MyBytes, String> {
        Ok(Self(s))
    }
}

impl sqlite::GoodOrmningCustomBytes<MyBytes> for MyBytes {
    fn to_sql<'a>(value: &'a MyBytes) -> Cow<'a, [u8]> {
        Cow::Borrowed(&value.0)
    }

    fn from_sql(s: Vec<u8>) -> Result<MyBytes, String> {
        Ok(Self(s))
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct MyUtctime(pub DateTime<Utc>);

impl pg::GoodOrmningCustomUtcTime<MyUtctime> for MyUtctime {
    fn to_sql(value: &MyUtctime) -> DateTime<Utc> {
        value.0
    }

    fn from_sql(s: DateTime<Utc>) -> Result<MyUtctime, String> {
        Ok(Self(s))
    }
}

impl sqlite::GoodOrmningCustomUtcTime<MyUtctime> for MyUtctime {
    fn to_sql<'a>(value: &'a MyUtctime) -> DateTime<Utc> {
        value.0
    }

    fn from_sql(s: DateTime<Utc>) -> Result<MyUtctime, String> {
        Ok(Self(s))
    }
}
