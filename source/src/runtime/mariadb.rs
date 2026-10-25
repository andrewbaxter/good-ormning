use {
    async_trait::async_trait,
    crate::runtime::GoodError,
    mysql_async::{
        Params,
        Row,
        Value,
        prelude::Queryable,
    },
    std::borrow::Cow,
};
pub use mysql_async;

pub trait GoodOrmningCustomBool<T> {
    fn from_sql(value: bool) -> Result<T, String>;
    fn to_sql(value: &T) -> bool;
}

pub trait GoodOrmningCustomBytes<T> {
    fn from_sql(value: Vec<u8>) -> Result<T, String>;
    fn to_sql<'a>(value: &'a T) -> Cow<'a, [u8]>;
}

pub trait GoodOrmningCustomDecimal<T> {
    fn from_sql(value: String) -> Result<T, String>;
    fn to_sql(value: &T) -> &str;
}

pub trait GoodOrmningCustomF32<T> {
    fn from_sql(value: f32) -> Result<T, String>;
    fn to_sql(value: &T) -> f32;
}

pub trait GoodOrmningCustomF64<T> {
    fn from_sql(value: f64) -> Result<T, String>;
    fn to_sql(value: &T) -> f64;
}

pub trait GoodOrmningCustomI8<T> {
    fn from_sql(value: i8) -> Result<T, String>;
    fn to_sql(value: &T) -> i8;
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

pub trait GoodOrmningCustomU8<T> {
    fn from_sql(value: u8) -> Result<T, String>;
    fn to_sql(value: &T) -> u8;
}

pub trait GoodOrmningCustomU16<T> {
    fn from_sql(value: u16) -> Result<T, String>;
    fn to_sql(value: &T) -> u16;
}

pub trait GoodOrmningCustomU32<T> {
    fn from_sql(value: u32) -> Result<T, String>;
    fn to_sql(value: &T) -> u32;
}

pub trait GoodOrmningCustomU64<T> {
    fn from_sql(value: u64) -> Result<T, String>;
    fn to_sql(value: &T) -> u64;
}

#[cfg(feature = "chrono")]
pub trait GoodOrmningCustomDatetimeChrono<T> {
    fn from_sql(value: chrono::NaiveDateTime) -> Result<T, String>;
    fn to_sql(value: &T) -> chrono::NaiveDateTime;
}

#[cfg(feature = "jiff")]
pub trait GoodOrmningCustomDatetimeJiff<T> {
    fn from_sql(value: jiff::civil::DateTime) -> Result<T, String>;
    fn to_sql(value: &T) -> jiff::civil::DateTime;
}

#[cfg(feature = "chrono")]
pub trait GoodOrmningCustomDateChrono<T> {
    fn from_sql(value: chrono::NaiveDate) -> Result<T, String>;
    fn to_sql(value: &T) -> chrono::NaiveDate;
}

#[cfg(feature = "jiff")]
pub trait GoodOrmningCustomDateJiff<T> {
    fn from_sql(value: jiff::civil::Date) -> Result<T, String>;
    fn to_sql(value: &T) -> jiff::civil::Date;
}

#[cfg(feature = "chrono")]
pub trait GoodOrmningCustomTimeChrono<T> {
    fn from_sql(value: chrono::TimeDelta) -> Result<T, String>;
    fn to_sql(value: &T) -> chrono::TimeDelta;
}

#[cfg(feature = "jiff")]
pub trait GoodOrmningCustomTimeJiff<T> {
    fn from_sql(value: jiff::SignedDuration) -> Result<T, String>;
    fn to_sql(value: &T) -> jiff::SignedDuration;
}

pub trait ToMariadbValue {
    fn to_mariadb_value(self) -> Value;
}

macro_rules! to_value_via_from{
    ($($t: ty), *) => {
        $(impl ToMariadbValue for $t {
            fn to_mariadb_value(self) -> Value {
                return Value::from(self);
            }
        }) *
    };
}

to_value_via_from!(i8, i16, i32, i64, u8, u16, u32, u64, f32, f64, bool, &str, String, &[u8], Vec < u8 >);

impl ToMariadbValue for &String {
    fn to_mariadb_value(self) -> Value {
        return Value::from(self.as_str());
    }
}

impl ToMariadbValue for Cow<'_, [u8]> {
    fn to_mariadb_value(self) -> Value {
        return Value::Bytes(self.into_owned());
    }
}

#[cfg(feature = "chrono")]
impl ToMariadbValue for chrono::NaiveDateTime {
    fn to_mariadb_value(self) -> Value {
        use chrono::{
            Datelike,
            Timelike,
        };

        return Value::Date(
            self.year() as u16,
            self.month() as u8,
            self.day() as u8,
            self.hour() as u8,
            self.minute() as u8,
            self.second() as u8,
            self.nanosecond() / 1000,
        );
    }
}

#[cfg(feature = "jiff")]
impl ToMariadbValue for jiff::civil::DateTime {
    fn to_mariadb_value(self) -> Value {
        return Value::Date(
            self.year() as u16,
            self.month() as u8,
            self.day() as u8,
            self.hour() as u8,
            self.minute() as u8,
            self.second() as u8,
            (self.subsec_nanosecond() / 1000) as u32,
        );
    }
}

#[cfg(feature = "chrono")]
impl ToMariadbValue for chrono::NaiveDate {
    fn to_mariadb_value(self) -> Value {
        use chrono::Datelike;

        return Value::Date(self.year() as u16, self.month() as u8, self.day() as u8, 0, 0, 0, 0);
    }
}

#[cfg(feature = "jiff")]
impl ToMariadbValue for jiff::civil::Date {
    fn to_mariadb_value(self) -> Value {
        return Value::Date(self.year() as u16, self.month() as u8, self.day() as u8, 0, 0, 0, 0);
    }
}

#[cfg(any(feature = "chrono", feature = "jiff"))]
fn time_value(negative: bool, seconds: u64, micros: u32) -> Value {
    return Value::Time(
        negative,
        (seconds / 86400).min(u32::MAX as u64) as u32,
        (seconds / 3600 % 24) as u8,
        (seconds / 60 % 60) as u8,
        (seconds % 60) as u8,
        micros,
    );
}

#[cfg(feature = "chrono")]
impl ToMariadbValue for chrono::TimeDelta {
    fn to_mariadb_value(self) -> Value {
        let abs = self.abs();
        return time_value(
            self < chrono::TimeDelta::zero(),
            abs.num_seconds() as u64,
            abs.subsec_nanos() as u32 / 1000,
        );
    }
}

#[cfg(feature = "jiff")]
impl ToMariadbValue for jiff::SignedDuration {
    fn to_mariadb_value(self) -> Value {
        let abs = self.abs();
        return time_value(self.is_negative(), abs.as_secs() as u64, abs.subsec_micros() as u32);
    }
}

impl<T: ToMariadbValue> ToMariadbValue for Option<T> {
    fn to_mariadb_value(self) -> Value {
        match self {
            Some(v) => return v.to_mariadb_value(),
            None => return Value::NULL,
        }
    }
}

pub fn to_value<T: ToMariadbValue>(v: T) -> Value {
    return v.to_mariadb_value();
}

pub trait FromMariadbValue: Sized {
    fn from_mariadb_value(v: Value) -> Result<Self, String>;
}

macro_rules! from_value_via_from_value{
    ($($t: ty), *) => {
        $(impl FromMariadbValue for $t {
            fn from_mariadb_value(v: Value) -> Result < Self,
            String > {
                return mysql_async:: from_value_opt::<$t >(v).map_err(|e| e.to_string());
            }
        }) *
    };
}

from_value_via_from_value!(i8, i16, i32, i64, u8, u16, u32, u64, f32, f64, Vec < u8 >);

impl FromMariadbValue for String {
    fn from_mariadb_value(v: Value) -> Result<Self, String> {
        match v {
            Value::Int(v) => return Ok(v.to_string()),
            Value::UInt(v) => return Ok(v.to_string()),
            v => return mysql_async::from_value_opt::<String>(v).map_err(|e| e.to_string()),
        }
    }
}

impl FromMariadbValue for bool {
    fn from_mariadb_value(v: Value) -> Result<Self, String> {
        return Ok(mysql_async::from_value_opt::<i64>(v).map_err(|e| e.to_string())? != 0);
    }
}

#[cfg(feature = "chrono")]
impl FromMariadbValue for chrono::NaiveDateTime {
    fn from_mariadb_value(v: Value) -> Result<Self, String> {
        match v {
            Value::Date(year, month, day, hour, minute, second, micros) => {
                return chrono::NaiveDate::from_ymd_opt(year as i32, month as u32, day as u32)
                    .and_then(|d| d.and_hms_micro_opt(hour as u32, minute as u32, second as u32, micros))
                    .ok_or_else(|| format!("Invalid datetime {:?}", v));
            },
            v => return Err(format!("Expected datetime but got {:?}", v)),
        }
    }
}

#[cfg(feature = "jiff")]
impl FromMariadbValue for jiff::civil::DateTime {
    fn from_mariadb_value(v: Value) -> Result<Self, String> {
        match v {
            Value::Date(year, month, day, hour, minute, second, micros) => {
                return jiff::civil::DateTime::new(
                    year as i16,
                    month as i8,
                    day as i8,
                    hour as i8,
                    minute as i8,
                    second as i8,
                    micros as i32 * 1000,
                ).map_err(|e| format!("Invalid datetime {:?}: {}", v, e));
            },
            v => return Err(format!("Expected datetime but got {:?}", v)),
        }
    }
}

#[cfg(feature = "chrono")]
impl FromMariadbValue for chrono::NaiveDate {
    fn from_mariadb_value(v: Value) -> Result<Self, String> {
        match v {
            Value::Date(year, month, day, 0, 0, 0, 0) => {
                return chrono::NaiveDate::from_ymd_opt(
                    year as i32,
                    month as u32,
                    day as u32,
                ).ok_or_else(|| format!("Invalid date {:?}", v));
            },
            v => return Err(format!("Expected date but got {:?}", v)),
        }
    }
}

#[cfg(feature = "jiff")]
impl FromMariadbValue for jiff::civil::Date {
    fn from_mariadb_value(v: Value) -> Result<Self, String> {
        match v {
            Value::Date(year, month, day, 0, 0, 0, 0) => {
                return jiff::civil::Date::new(
                    year as i16,
                    month as i8,
                    day as i8,
                ).map_err(|e| format!("Invalid date {:?}: {}", v, e));
            },
            v => return Err(format!("Expected date but got {:?}", v)),
        }
    }
}

#[cfg(any(feature = "chrono", feature = "jiff"))]
fn time_parts(v: Value) -> Result<(bool, i64, i32), String> {
    match v {
        Value::Time(negative, days, hours, minutes, seconds, micros) => {
            return Ok(
                (
                    negative,
                    ((days as i64 * 24 + hours as i64) * 60 + minutes as i64) * 60 + seconds as i64,
                    micros as i32 * 1000,
                ),
            );
        },
        v => return Err(format!("Expected time but got {:?}", v)),
    }
}

#[cfg(feature = "chrono")]
impl FromMariadbValue for chrono::TimeDelta {
    fn from_mariadb_value(v: Value) -> Result<Self, String> {
        let (negative, seconds, nanos) = time_parts(v)?;
        let out =
            chrono::TimeDelta::new(
                seconds,
                nanos as u32,
            ).ok_or_else(|| format!("Time {}s is out of range", seconds))?;
        if negative {
            return Ok(-out);
        } else {
            return Ok(out);
        }
    }
}

#[cfg(feature = "jiff")]
impl FromMariadbValue for jiff::SignedDuration {
    fn from_mariadb_value(v: Value) -> Result<Self, String> {
        let (negative, seconds, nanos) = time_parts(v)?;
        let out = jiff::SignedDuration::new(seconds, nanos);
        if negative {
            return Ok(-out);
        } else {
            return Ok(out);
        }
    }
}

impl<T: FromMariadbValue> FromMariadbValue for Option<T> {
    fn from_mariadb_value(v: Value) -> Result<Self, String> {
        match v {
            Value::NULL => return Ok(None),
            v => return Ok(Some(T::from_mariadb_value(v)?)),
        }
    }
}

pub fn get<T: FromMariadbValue>(row: &mut Row, i: usize) -> Result<T, GoodError> {
    let Some(v) = row.take::<Value, usize>(i) else {
        return Err(GoodError(format!("Result is missing column {}", i)));
    };
    return T::from_mariadb_value(v).map_err(|e| GoodError(format!("Reading result {}: {}", i, e)));
}

pub fn push_in_list(query: &mut String, params: &mut Vec<Value>, values: Vec<Value>, element: &str, width: usize) {
    let rows = values.len() / width;
    if rows == 0 {
        query.push_str("(select ");
        for i in 0 .. width {
            if i > 0 {
                query.push_str(", ");
            }
            query.push('1');
        }
        query.push_str(" from dual where false)");
        return;
    }
    let count = rows.next_power_of_two();
    let last = values[values.len() - width..].to_vec();
    query.push('(');
    for i in 0 .. count {
        if i > 0 {
            query.push_str(", ");
        }
        query.push_str(element);
    }
    query.push(')');
    params.extend(values);
    for _ in rows .. count {
        params.extend(last.iter().cloned());
    }
}

pub fn push_table_list(
    query: &mut String,
    params: &mut Vec<Value>,
    values: Vec<Value>,
    templates: &[&str],
    names: &[&str],
    with_ordinality: bool,
) {
    let push_name = |query: &mut String, name: &str| {
        query.push_str(" as `");
        query.push_str(&name.replace('`', "``"));
        query.push('`');
    };
    let width = templates.len();
    let rows = values.len() / width;
    query.push('(');
    if rows == 0 {
        query.push_str("select ");
        for (i, name) in names.iter().enumerate() {
            if i > 0 {
                query.push_str(", ");
            }
            query.push_str("null");
            push_name(query, name);
        }
        query.push_str(" from dual where false");
    } else {
        for row in 0 .. rows {
            if row > 0 {
                query.push_str(" union all ");
            }
            query.push_str("select ");
            for (i, template) in templates.iter().enumerate() {
                if i > 0 {
                    query.push_str(", ");
                }
                query.push_str(template);
                if row == 0 {
                    push_name(query, names[i]);
                }
            }
            if with_ordinality {
                query.push_str(", ");
                query.push_str(&(row + 1).to_string());
                if row == 0 {
                    push_name(query, names[width]);
                }
            }
        }
        params.extend(values);
    }
    query.push(')');
}

fn to_params(params: Vec<Value>) -> Params {
    if params.is_empty() {
        return Params::Empty;
    } else {
        return Params::Positional(params);
    }
}

#[async_trait]
pub trait MariadbConnection: Send {
    async fn execute(&mut self, query: &str, params: Vec<Value>) -> Result<u64, GoodError>;
    async fn query(&mut self, query: &str, params: Vec<Value>) -> Result<Vec<Row>, GoodError>;
}

#[async_trait]
impl<T: MariadbConnection + ?Sized> MariadbConnection for &mut T {
    async fn execute(&mut self, query: &str, params: Vec<Value>) -> Result<u64, GoodError> {
        return (**self).execute(query, params).await;
    }

    async fn query(&mut self, query: &str, params: Vec<Value>) -> Result<Vec<Row>, GoodError> {
        return (**self).query(query, params).await;
    }
}

#[async_trait]
impl MariadbConnection for mysql_async::Conn {
    async fn execute(&mut self, query: &str, params: Vec<Value>) -> Result<u64, GoodError> {
        self.exec_drop(query, to_params(params)).await.map_err(|e| GoodError(e.to_string()))?;
        return Ok(self.affected_rows());
    }

    async fn query(&mut self, query: &str, params: Vec<Value>) -> Result<Vec<Row>, GoodError> {
        return self.exec(query, to_params(params)).await.map_err(|e| GoodError(e.to_string()));
    }
}

#[async_trait]
impl MariadbConnection for mysql_async::Transaction<'_> {
    async fn execute(&mut self, query: &str, params: Vec<Value>) -> Result<u64, GoodError> {
        self.exec_drop(query, to_params(params)).await.map_err(|e| GoodError(e.to_string()))?;
        return Ok(self.affected_rows());
    }

    async fn query(&mut self, query: &str, params: Vec<Value>) -> Result<Vec<Row>, GoodError> {
        return self.exec(query, to_params(params)).await.map_err(|e| GoodError(e.to_string()));
    }
}

#[async_trait]
impl MariadbConnection for mysql_async::Pool {
    async fn execute(&mut self, query: &str, params: Vec<Value>) -> Result<u64, GoodError> {
        let mut conn = self.get_conn().await.map_err(|e| GoodError(e.to_string()))?;
        return conn.execute(query, params).await;
    }

    async fn query(&mut self, query: &str, params: Vec<Value>) -> Result<Vec<Row>, GoodError> {
        let mut conn = self.get_conn().await.map_err(|e| GoodError(e.to_string()))?;
        return MariadbConnection::query(&mut conn, query, params).await;
    }
}
