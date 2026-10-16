pub mod mariadb_server;

use {
    chrono::{
        DateTime,
        Utc,
    },
    good_ormning::runtime::{
        mariadb,
        pg,
        sqlite,
    },
    jiff::Timestamp,
    std::borrow::Cow,
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct MyAuto(pub i64);

impl pg::GoodOrmningCustomAuto<MyAuto> for MyAuto {
    fn from_sql(s: i64) -> Result<MyAuto, String> {
        return Ok(Self(s));
    }

    fn to_sql(value: &MyAuto) -> i64 {
        return value.0;
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct MyBool(pub bool);

impl pg::GoodOrmningCustomBool<MyBool> for MyBool {
    fn from_sql(s: bool) -> Result<MyBool, String> {
        return Ok(Self(s));
    }

    fn to_sql(value: &MyBool) -> bool {
        return value.0;
    }
}

impl sqlite::GoodOrmningCustomBool<MyBool> for MyBool {
    fn from_sql(s: bool) -> Result<MyBool, String> {
        return Ok(Self(s));
    }

    fn to_sql<'a>(value: &'a MyBool) -> bool {
        return value.0;
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct MyBytes(pub Vec<u8>);

impl pg::GoodOrmningCustomBytes<MyBytes> for MyBytes {
    fn from_sql(s: Vec<u8>) -> Result<MyBytes, String> {
        return Ok(Self(s));
    }

    fn to_sql<'a>(value: &'a MyBytes) -> Cow<'a, [u8]> {
        return Cow::Borrowed(&value.0);
    }
}

impl sqlite::GoodOrmningCustomBytes<MyBytes> for MyBytes {
    fn from_sql(s: Vec<u8>) -> Result<MyBytes, String> {
        return Ok(Self(s));
    }

    fn to_sql<'a>(value: &'a MyBytes) -> Vec<u8> {
        return value.0.clone();
    }
}

#[derive(PartialEq, PartialOrd, Debug)]
pub struct MyF32(pub f32);

impl Eq for MyF32 { }

impl Ord for MyF32 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        return self.0.partial_cmp(&other.0).unwrap_or(std::cmp::Ordering::Equal);
    }
}

impl pg::GoodOrmningCustomF32<MyF32> for MyF32 {
    fn from_sql(s: f32) -> Result<MyF32, String> {
        return Ok(Self(s));
    }

    fn to_sql(value: &MyF32) -> f32 {
        return value.0;
    }
}

impl sqlite::GoodOrmningCustomF32<MyF32> for MyF32 {
    fn from_sql(s: f32) -> Result<MyF32, String> {
        return Ok(Self(s));
    }

    fn to_sql<'a>(value: &'a MyF32) -> f32 {
        return value.0;
    }
}

#[derive(PartialEq, PartialOrd, Debug)]
pub struct MyF64(pub f64);

impl Eq for MyF64 { }

impl Ord for MyF64 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        return self.0.partial_cmp(&other.0).unwrap_or(std::cmp::Ordering::Equal);
    }
}

impl pg::GoodOrmningCustomF64<MyF64> for MyF64 {
    fn from_sql(s: f64) -> Result<MyF64, String> {
        return Ok(Self(s));
    }

    fn to_sql(value: &MyF64) -> f64 {
        return value.0;
    }
}

impl sqlite::GoodOrmningCustomF64<MyF64> for MyF64 {
    fn from_sql(s: f64) -> Result<MyF64, String> {
        return Ok(Self(s));
    }

    fn to_sql<'a>(value: &'a MyF64) -> f64 {
        return value.0;
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct MyI32(pub i32);

impl pg::GoodOrmningCustomI32<MyI32> for MyI32 {
    fn from_sql(s: i32) -> Result<MyI32, String> {
        return Ok(Self(s));
    }

    fn to_sql(value: &MyI32) -> i32 {
        return value.0;
    }
}

impl sqlite::GoodOrmningCustomI32<MyI32> for MyI32 {
    fn from_sql(s: i32) -> Result<MyI32, String> {
        return Ok(Self(s));
    }

    fn to_sql<'a>(value: &'a MyI32) -> i32 {
        return value.0;
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct MyI64(pub i64);

impl pg::GoodOrmningCustomI64<MyI64> for MyI64 {
    fn from_sql(s: i64) -> Result<MyI64, String> {
        return Ok(Self(s));
    }

    fn to_sql(value: &MyI64) -> i64 {
        return value.0;
    }
}

impl sqlite::GoodOrmningCustomI64<MyI64> for MyI64 {
    fn from_sql(s: i64) -> Result<MyI64, String> {
        return Ok(Self(s));
    }

    fn to_sql<'a>(value: &'a MyI64) -> i64 {
        return value.0;
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct MyString(pub String);

impl pg::GoodOrmningCustomString<MyString> for MyString {
    fn from_sql(s: String) -> Result<MyString, String> {
        return Ok(Self(s));
    }

    fn to_sql(value: &MyString) -> &str {
        return &value.0;
    }
}

impl sqlite::GoodOrmningCustomString<MyString> for MyString {
    fn from_sql(s: String) -> Result<MyString, String> {
        return Ok(Self(s));
    }

    fn to_sql<'a>(value: &'a MyString) -> String {
        return value.0.clone();
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct MyU32(pub u32);

impl pg::GoodOrmningCustomU32<MyU32> for MyU32 {
    fn from_sql(s: u32) -> Result<MyU32, String> {
        return Ok(Self(s));
    }

    fn to_sql(value: &MyU32) -> u32 {
        return value.0;
    }
}

impl sqlite::GoodOrmningCustomU32<MyU32> for MyU32 {
    fn from_sql(s: u32) -> Result<MyU32, String> {
        return Ok(Self(s));
    }

    fn to_sql<'a>(value: &'a MyU32) -> u32 {
        return value.0;
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct MyUtctimeChrono(pub DateTime<Utc>);

impl pg::GoodOrmningCustomUtcTimeChrono<MyUtctimeChrono> for MyUtctimeChrono {
    fn from_sql(s: DateTime<Utc>) -> Result<MyUtctimeChrono, String> {
        return Ok(Self(s));
    }

    fn to_sql(value: &MyUtctimeChrono) -> DateTime<Utc> {
        return value.0;
    }
}

impl sqlite::GoodOrmningCustomUtcTimeChrono<MyUtctimeChrono> for MyUtctimeChrono {
    fn from_sql(s: DateTime<Utc>) -> Result<MyUtctimeChrono, String> {
        return Ok(Self(s));
    }

    fn to_sql<'a>(value: &'a MyUtctimeChrono) -> DateTime<Utc> {
        return value.0;
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct MyUtctimeJiff(pub Timestamp);

impl pg::GoodOrmningCustomUtcTimeJiff<MyUtctimeJiff> for MyUtctimeJiff {
    fn from_sql(s: Timestamp) -> Result<MyUtctimeJiff, String> {
        return Ok(Self(s));
    }

    fn to_sql(value: &MyUtctimeJiff) -> Timestamp {
        return value.0;
    }
}

impl sqlite::GoodOrmningCustomUtcTimeJiff<MyUtctimeJiff> for MyUtctimeJiff {
    fn from_sql(s: Timestamp) -> Result<MyUtctimeJiff, String> {
        return Ok(Self(s));
    }

    fn to_sql<'a>(value: &'a MyUtctimeJiff) -> Timestamp {
        return value.0;
    }
}

impl mariadb::GoodOrmningCustomBool<MyBool> for MyBool {
    fn from_sql(s: bool) -> Result<MyBool, String> {
        return Ok(Self(s));
    }

    fn to_sql(value: &MyBool) -> bool {
        return value.0;
    }
}

impl mariadb::GoodOrmningCustomBytes<MyBytes> for MyBytes {
    fn from_sql(s: Vec<u8>) -> Result<MyBytes, String> {
        return Ok(Self(s));
    }

    fn to_sql<'a>(value: &'a MyBytes) -> Cow<'a, [u8]> {
        return Cow::Borrowed(&value.0);
    }
}

impl mariadb::GoodOrmningCustomF32<MyF32> for MyF32 {
    fn from_sql(s: f32) -> Result<MyF32, String> {
        return Ok(Self(s));
    }

    fn to_sql(value: &MyF32) -> f32 {
        return value.0;
    }
}

impl mariadb::GoodOrmningCustomF64<MyF64> for MyF64 {
    fn from_sql(s: f64) -> Result<MyF64, String> {
        return Ok(Self(s));
    }

    fn to_sql(value: &MyF64) -> f64 {
        return value.0;
    }
}

impl mariadb::GoodOrmningCustomI32<MyI32> for MyI32 {
    fn from_sql(s: i32) -> Result<MyI32, String> {
        return Ok(Self(s));
    }

    fn to_sql(value: &MyI32) -> i32 {
        return value.0;
    }
}

impl mariadb::GoodOrmningCustomI64<MyI64> for MyI64 {
    fn from_sql(s: i64) -> Result<MyI64, String> {
        return Ok(Self(s));
    }

    fn to_sql(value: &MyI64) -> i64 {
        return value.0;
    }
}

impl mariadb::GoodOrmningCustomString<MyString> for MyString {
    fn from_sql(s: String) -> Result<MyString, String> {
        return Ok(Self(s));
    }

    fn to_sql(value: &MyString) -> &str {
        return &value.0;
    }
}

impl mariadb::GoodOrmningCustomU32<MyU32> for MyU32 {
    fn from_sql(s: u32) -> Result<MyU32, String> {
        return Ok(Self(s));
    }

    fn to_sql(value: &MyU32) -> u32 {
        return value.0;
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct MyDatetimeChrono(pub chrono::NaiveDateTime);

impl mariadb::GoodOrmningCustomDatetimeChrono<MyDatetimeChrono> for MyDatetimeChrono {
    fn from_sql(s: chrono::NaiveDateTime) -> Result<MyDatetimeChrono, String> {
        return Ok(Self(s));
    }

    fn to_sql(value: &MyDatetimeChrono) -> chrono::NaiveDateTime {
        return value.0;
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct MyDatetimeJiff(pub jiff::civil::DateTime);

impl mariadb::GoodOrmningCustomDatetimeJiff<MyDatetimeJiff> for MyDatetimeJiff {
    fn from_sql(s: jiff::civil::DateTime) -> Result<MyDatetimeJiff, String> {
        return Ok(Self(s));
    }

    fn to_sql(value: &MyDatetimeJiff) -> jiff::civil::DateTime {
        return value.0;
    }
}
