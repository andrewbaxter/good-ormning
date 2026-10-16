use {
    crate::{
        mariadb::schema::field::{
            FieldType,
            TypeVariant,
        },
        utils::{
            Errs,
            RustTypes,
            TimeLibrary,
            time_library,
        },
    },
    quote::quote,
    serde::{
        Deserialize,
        Serialize,
    },
};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum SimpleSimpleType {
    AnyDate,
    AnyDatetime,
    AnyDecimal,
    AnyFloat,
    AnyInt,
    AnyNull,
    AnyTime,
    Bool,
    Bytes,
    #[cfg(feature = "chrono")]
    DateChrono,
    #[cfg(feature = "jiff")]
    DateJiff,
    #[cfg(feature = "chrono")]
    DatetimeChrono,
    #[cfg(feature = "jiff")]
    DatetimeJiff,
    Decimal,
    F32,
    F64,
    I16,
    I32,
    I64,
    I8,
    String,
    #[cfg(feature = "chrono")]
    TimeChrono,
    #[cfg(feature = "jiff")]
    TimeJiff,
    U16,
    U32,
    U64,
    U8,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Temporal {
    Date,
    Datetime,
    Time,
}

pub fn temporal(t: &SimpleSimpleType) -> Option<Temporal> {
    return match t {
        SimpleSimpleType::AnyDate => Some(Temporal::Date),
        SimpleSimpleType::AnyDatetime => Some(Temporal::Datetime),
        SimpleSimpleType::AnyTime => Some(Temporal::Time),
        #[cfg(feature = "chrono")]
        SimpleSimpleType::DateChrono => Some(Temporal::Date),
        #[cfg(feature = "jiff")]
        SimpleSimpleType::DateJiff => Some(Temporal::Date),
        #[cfg(feature = "chrono")]
        SimpleSimpleType::DatetimeChrono => Some(Temporal::Datetime),
        #[cfg(feature = "jiff")]
        SimpleSimpleType::DatetimeJiff => Some(Temporal::Datetime),
        #[cfg(feature = "chrono")]
        SimpleSimpleType::TimeChrono => Some(Temporal::Time),
        #[cfg(feature = "jiff")]
        SimpleSimpleType::TimeJiff => Some(Temporal::Time),
        _ => None,
    };
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct SimpleType {
    pub custom: Option<String>,
    pub type_: SimpleSimpleType,
}

pub fn to_rust_types(
    errs: &Errs,
    path: &rpds::Vector<String>,
    t: &SimpleSimpleType,
    time: Option<TimeLibrary>,
) -> RustTypes {
    return match t {
        SimpleSimpleType::I8 => RustTypes {
            custom_trait: quote!(good_ormning::runtime::mariadb::GoodOrmningCustomI8),
            ret_type: quote!(i8),
            arg_type: quote!(i8),
        },
        SimpleSimpleType::I16 => RustTypes {
            custom_trait: quote!(good_ormning::runtime::mariadb::GoodOrmningCustomI16),
            ret_type: quote!(i16),
            arg_type: quote!(i16),
        },
        SimpleSimpleType::AnyNull | SimpleSimpleType::AnyInt | SimpleSimpleType::I32 => RustTypes {
            custom_trait: quote!(good_ormning::runtime::mariadb::GoodOrmningCustomI32),
            ret_type: quote!(i32),
            arg_type: quote!(i32),
        },
        SimpleSimpleType::I64 => RustTypes {
            custom_trait: quote!(good_ormning::runtime::mariadb::GoodOrmningCustomI64),
            ret_type: quote!(i64),
            arg_type: quote!(i64),
        },
        SimpleSimpleType::AnyDate | SimpleSimpleType::AnyDatetime | SimpleSimpleType::AnyTime => {
            let library = match time_library(time) {
                Ok(l) => l,
                Err(e) => {
                    errs.err(path, e);
                    None
                },
            };
            match library {
                None => {
                    return RustTypes {
                        custom_trait: quote!(good_ormning::runtime::mariadb::GoodOrmningCustomString),
                        ret_type: quote!(String),
                        arg_type: quote!(&str),
                    };
                },
                #[cfg(any(feature = "chrono", feature = "jiff"))]
                Some(library) => {
                    let concrete = match (library, t) {
                        #[cfg(feature = "chrono")]
                        (TimeLibrary::Chrono, SimpleSimpleType::AnyDate) => SimpleSimpleType::DateChrono,
                        #[cfg(feature = "chrono")]
                        (TimeLibrary::Chrono, SimpleSimpleType::AnyTime) => SimpleSimpleType::TimeChrono,
                        #[cfg(feature = "chrono")]
                        (TimeLibrary::Chrono, _) => SimpleSimpleType::DatetimeChrono,
                        #[cfg(feature = "jiff")]
                        (TimeLibrary::Jiff, SimpleSimpleType::AnyDate) => SimpleSimpleType::DateJiff,
                        #[cfg(feature = "jiff")]
                        (TimeLibrary::Jiff, SimpleSimpleType::AnyTime) => SimpleSimpleType::TimeJiff,
                        #[cfg(feature = "jiff")]
                        (TimeLibrary::Jiff, _) => SimpleSimpleType::DatetimeJiff,
                    };
                    return to_rust_types(errs, path, &concrete, time);
                },
            }
        },
        SimpleSimpleType::U8 => RustTypes {
            custom_trait: quote!(good_ormning::runtime::mariadb::GoodOrmningCustomU8),
            ret_type: quote!(u8),
            arg_type: quote!(u8),
        },
        SimpleSimpleType::U16 => RustTypes {
            custom_trait: quote!(good_ormning::runtime::mariadb::GoodOrmningCustomU16),
            ret_type: quote!(u16),
            arg_type: quote!(u16),
        },
        SimpleSimpleType::U32 => RustTypes {
            custom_trait: quote!(good_ormning::runtime::mariadb::GoodOrmningCustomU32),
            ret_type: quote!(u32),
            arg_type: quote!(u32),
        },
        SimpleSimpleType::U64 => RustTypes {
            custom_trait: quote!(good_ormning::runtime::mariadb::GoodOrmningCustomU64),
            ret_type: quote!(u64),
            arg_type: quote!(u64),
        },
        SimpleSimpleType::F32 => RustTypes {
            custom_trait: quote!(good_ormning::runtime::mariadb::GoodOrmningCustomF32),
            ret_type: quote!(f32),
            arg_type: quote!(f32),
        },
        SimpleSimpleType::AnyFloat | SimpleSimpleType::F64 => RustTypes {
            custom_trait: quote!(good_ormning::runtime::mariadb::GoodOrmningCustomF64),
            ret_type: quote!(f64),
            arg_type: quote!(f64),
        },
        SimpleSimpleType::Bool => RustTypes {
            custom_trait: quote!(good_ormning::runtime::mariadb::GoodOrmningCustomBool),
            ret_type: quote!(bool),
            arg_type: quote!(bool),
        },
        SimpleSimpleType::String => RustTypes {
            custom_trait: quote!(good_ormning::runtime::mariadb::GoodOrmningCustomString),
            ret_type: quote!(String),
            arg_type: quote!(&str),
        },
        SimpleSimpleType::AnyDecimal | SimpleSimpleType::Decimal => RustTypes {
            custom_trait: quote!(good_ormning::runtime::mariadb::GoodOrmningCustomDecimal),
            ret_type: quote!(String),
            arg_type: quote!(&str),
        },
        SimpleSimpleType::Bytes => RustTypes {
            custom_trait: quote!(good_ormning::runtime::mariadb::GoodOrmningCustomBytes),
            ret_type: quote!(Vec < u8 >),
            arg_type: quote!(&[u8]),
        },
        #[cfg(feature = "chrono")]
        SimpleSimpleType::DatetimeChrono => RustTypes {
            custom_trait: quote!(good_ormning::runtime::mariadb::GoodOrmningCustomDatetimeChrono),
            ret_type: quote!(chrono::NaiveDateTime),
            arg_type: quote!(chrono::NaiveDateTime),
        },
        #[cfg(feature = "jiff")]
        SimpleSimpleType::DatetimeJiff => RustTypes {
            custom_trait: quote!(good_ormning::runtime::mariadb::GoodOrmningCustomDatetimeJiff),
            ret_type: quote!(jiff::civil::DateTime),
            arg_type: quote!(jiff::civil::DateTime),
        },
        #[cfg(feature = "chrono")]
        SimpleSimpleType::DateChrono => RustTypes {
            custom_trait: quote!(good_ormning::runtime::mariadb::GoodOrmningCustomDateChrono),
            ret_type: quote!(chrono::NaiveDate),
            arg_type: quote!(chrono::NaiveDate),
        },
        #[cfg(feature = "jiff")]
        SimpleSimpleType::DateJiff => RustTypes {
            custom_trait: quote!(good_ormning::runtime::mariadb::GoodOrmningCustomDateJiff),
            ret_type: quote!(jiff::civil::Date),
            arg_type: quote!(jiff::civil::Date),
        },
        #[cfg(feature = "chrono")]
        SimpleSimpleType::TimeChrono => RustTypes {
            custom_trait: quote!(good_ormning::runtime::mariadb::GoodOrmningCustomTimeChrono),
            ret_type: quote!(chrono::TimeDelta),
            arg_type: quote!(chrono::TimeDelta),
        },
        #[cfg(feature = "jiff")]
        SimpleSimpleType::TimeJiff => RustTypes {
            custom_trait: quote!(good_ormning::runtime::mariadb::GoodOrmningCustomTimeJiff),
            ret_type: quote!(jiff::SignedDuration),
            arg_type: quote!(jiff::SignedDuration),
        },
    };
}

pub fn to_sql_type(f: &FieldType) -> String {
    let max_len = f.max_len;
    let precision = f.precision;
    let sized = |name: &str| match max_len {
        Some(l) => format!("{}({})", name, l),
        None => name.to_string(),
    };
    let mut out = match (&f.variant, &f.type_.type_.type_) {
        (Some(TypeVariant::Mediumint), SimpleSimpleType::U32) => "mediumint unsigned".to_string(),
        (Some(TypeVariant::Mediumint), _) => "mediumint".to_string(),
        (Some(TypeVariant::Char), _) => sized("char"),
        (Some(TypeVariant::TinyText), _) => "tinytext".to_string(),
        (Some(TypeVariant::Text), _) => "text".to_string(),
        (Some(TypeVariant::MediumText), _) => "mediumtext".to_string(),
        (Some(TypeVariant::Binary), _) => sized("binary"),
        (Some(TypeVariant::TinyBlob), _) => "tinyblob".to_string(),
        (Some(TypeVariant::Blob), _) => "blob".to_string(),
        (Some(TypeVariant::MediumBlob), _) => "mediumblob".to_string(),
        (Some(TypeVariant::Year), _) => "year".to_string(),
        (Some(TypeVariant::Timestamp), _) => match precision {
            Some(p) => format!("timestamp({})", p),
            None => "timestamp".to_string(),
        },
        (None, SimpleSimpleType::I8) => "tinyint".to_string(),
        (None, SimpleSimpleType::I16) => "smallint".to_string(),
        (None, SimpleSimpleType::AnyNull | SimpleSimpleType::AnyInt | SimpleSimpleType::I32) => "int".to_string(),
        (None, SimpleSimpleType::I64) => "bigint".to_string(),
        (None, SimpleSimpleType::U8) => "tinyint unsigned".to_string(),
        (None, SimpleSimpleType::U16) => "smallint unsigned".to_string(),
        (None, SimpleSimpleType::U32) => "int unsigned".to_string(),
        (None, SimpleSimpleType::U64) => "bigint unsigned".to_string(),
        (None, SimpleSimpleType::F32) => "float".to_string(),
        (None, SimpleSimpleType::AnyFloat | SimpleSimpleType::F64) => "double".to_string(),
        (None, SimpleSimpleType::AnyDecimal | SimpleSimpleType::Decimal) => match (precision, f.scale) {
            (Some(p), Some(s)) => format!("decimal({}, {})", p, s),
            (Some(p), None) => format!("decimal({})", p),
            (None, _) => "decimal".to_string(),
        },
        (None, SimpleSimpleType::Bool) => "boolean".to_string(),
        (None, SimpleSimpleType::String) => match max_len {
            Some(l) => format!("varchar({})", l),
            None => "longtext".to_string(),
        },
        (None, SimpleSimpleType::Bytes) => match max_len {
            Some(l) => format!("varbinary({})", l),
            None => "longblob".to_string(),
        },
        (None, t) => match (temporal(t), precision) {
            (Some(Temporal::Date), _) => "date".to_string(),
            (Some(Temporal::Time), Some(p)) => format!("time({})", p),
            (Some(Temporal::Time), None) => "time".to_string(),
            (_, Some(p)) => format!("datetime({})", p),
            (_, None) => "datetime".to_string(),
        },
    };
    if f.auto_increment {
        out.push_str(" auto_increment");
    }
    return out;
}

pub fn to_sql_cast_type(t: &SimpleSimpleType) -> &'static str {
    return match t {
        SimpleSimpleType::AnyNull |
        SimpleSimpleType::AnyInt |
        SimpleSimpleType::Bool |
        SimpleSimpleType::I8 |
        SimpleSimpleType::I16 |
        SimpleSimpleType::I32 |
        SimpleSimpleType::I64 => "signed",
        SimpleSimpleType::U8 | SimpleSimpleType::U16 | SimpleSimpleType::U32 | SimpleSimpleType::U64 => "unsigned",
        SimpleSimpleType::F32 => "float",
        SimpleSimpleType::AnyFloat | SimpleSimpleType::F64 => "double",
        SimpleSimpleType::AnyDecimal | SimpleSimpleType::Decimal => "decimal",
        SimpleSimpleType::String => "char",
        SimpleSimpleType::Bytes => "binary",
        SimpleSimpleType::AnyDatetime => "datetime",
        #[cfg(feature = "chrono")]
        SimpleSimpleType::DatetimeChrono => "datetime",
        #[cfg(feature = "jiff")]
        SimpleSimpleType::DatetimeJiff => "datetime",
        SimpleSimpleType::AnyDate => "date",
        #[cfg(feature = "chrono")]
        SimpleSimpleType::DateChrono => "date",
        #[cfg(feature = "jiff")]
        SimpleSimpleType::DateJiff => "date",
        SimpleSimpleType::AnyTime => "time",
        #[cfg(feature = "chrono")]
        SimpleSimpleType::TimeChrono => "time",
        #[cfg(feature = "jiff")]
        SimpleSimpleType::TimeJiff => "time",
    };
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Type {
    pub opt: bool,
    pub type_: SimpleType,
}

impl Type {
    pub fn opt(mut self) -> Self {
        self.opt = true;
        return self;
    }
}

pub fn type_bool() -> TypeBuilder {
    return TypeBuilder::new(SimpleSimpleType::Bool);
}

pub fn type_bytes() -> TypeBuilder {
    return TypeBuilder::new(SimpleSimpleType::Bytes);
}

#[cfg(feature = "chrono")]
pub fn type_date_chrono() -> TypeBuilder {
    return TypeBuilder::new(SimpleSimpleType::DateChrono);
}

#[cfg(feature = "jiff")]
pub fn type_date_jiff() -> TypeBuilder {
    return TypeBuilder::new(SimpleSimpleType::DateJiff);
}

#[cfg(feature = "chrono")]
pub fn type_datetime_chrono() -> TypeBuilder {
    return TypeBuilder::new(SimpleSimpleType::DatetimeChrono);
}

#[cfg(feature = "jiff")]
pub fn type_datetime_jiff() -> TypeBuilder {
    return TypeBuilder::new(SimpleSimpleType::DatetimeJiff);
}

pub fn type_decimal() -> TypeBuilder {
    return TypeBuilder::new(SimpleSimpleType::Decimal);
}

pub fn type_f32() -> TypeBuilder {
    return TypeBuilder::new(SimpleSimpleType::F32);
}

pub fn type_f64() -> TypeBuilder {
    return TypeBuilder::new(SimpleSimpleType::F64);
}

pub fn type_i8() -> TypeBuilder {
    return TypeBuilder::new(SimpleSimpleType::I8);
}

pub fn type_i16() -> TypeBuilder {
    return TypeBuilder::new(SimpleSimpleType::I16);
}

pub fn type_i32() -> TypeBuilder {
    return TypeBuilder::new(SimpleSimpleType::I32);
}

pub fn type_i64() -> TypeBuilder {
    return TypeBuilder::new(SimpleSimpleType::I64);
}

pub fn type_str() -> TypeBuilder {
    return TypeBuilder::new(SimpleSimpleType::String);
}

#[cfg(feature = "chrono")]
pub fn type_time_chrono() -> TypeBuilder {
    return TypeBuilder::new(SimpleSimpleType::TimeChrono);
}

#[cfg(feature = "jiff")]
pub fn type_time_jiff() -> TypeBuilder {
    return TypeBuilder::new(SimpleSimpleType::TimeJiff);
}

pub fn type_u8() -> TypeBuilder {
    return TypeBuilder::new(SimpleSimpleType::U8);
}

pub fn type_u16() -> TypeBuilder {
    return TypeBuilder::new(SimpleSimpleType::U16);
}

pub fn type_u32() -> TypeBuilder {
    return TypeBuilder::new(SimpleSimpleType::U32);
}

pub fn type_u64() -> TypeBuilder {
    return TypeBuilder::new(SimpleSimpleType::U64);
}

pub struct TypeBuilder {
    custom: Option<String>,
    opt: bool,
    t: SimpleSimpleType,
}

impl TypeBuilder {
    pub fn build(self) -> Type {
        return Type {
            type_: SimpleType {
                custom: self.custom,
                type_: self.t,
            },
            opt: self.opt,
        };
    }

    fn new(t: SimpleSimpleType) -> TypeBuilder {
        return TypeBuilder {
            t: t,
            opt: false,
            custom: None,
        };
    }

    pub fn opt(mut self) -> TypeBuilder {
        self.opt = true;
        return self;
    }
}

pub fn unify_types(left: &SimpleSimpleType, right: &SimpleSimpleType) -> Option<SimpleSimpleType> {
    let is_int =
        |t: &SimpleSimpleType| matches!(
            t,
            SimpleSimpleType::AnyInt | SimpleSimpleType::I8 | SimpleSimpleType::I16 | SimpleSimpleType::I32 |
                SimpleSimpleType::I64 |
                SimpleSimpleType::U8 |
                SimpleSimpleType::U16 |
                SimpleSimpleType::U32 |
                SimpleSimpleType::U64
        );
    if left == right {
        return Some(left.clone());
    }
    if matches!(
        (left, right),
        (SimpleSimpleType::Bool, SimpleSimpleType::AnyInt) | (SimpleSimpleType::AnyInt, SimpleSimpleType::Bool)
    ) {
        return Some(SimpleSimpleType::Bool);
    }
    if *left == SimpleSimpleType::AnyNull {
        return Some(right.clone());
    }
    if *right == SimpleSimpleType::AnyNull {
        return Some(left.clone());
    }
    let is_float =
        |t: &SimpleSimpleType| matches!(
            t,
            SimpleSimpleType::AnyFloat | SimpleSimpleType::F32 | SimpleSimpleType::F64
        );
    let is_decimal = |t: &SimpleSimpleType| matches!(t, SimpleSimpleType::AnyDecimal | SimpleSimpleType::Decimal);
    if *left == SimpleSimpleType::Decimal && (is_int(right) || *right == SimpleSimpleType::AnyDecimal) {
        return Some(left.clone());
    }
    if *right == SimpleSimpleType::Decimal && (is_int(left) || *left == SimpleSimpleType::AnyDecimal) {
        return Some(right.clone());
    }
    if *left == SimpleSimpleType::F32 && *right != SimpleSimpleType::F32 &&
        (is_float(right) || is_decimal(right) || is_int(right)) {
        return Some(SimpleSimpleType::F64);
    }
    if *right == SimpleSimpleType::F32 && *left != SimpleSimpleType::F32 &&
        (is_float(left) || is_decimal(left) || is_int(left)) {
        return Some(SimpleSimpleType::F64);
    }
    if *left == SimpleSimpleType::AnyDecimal && (is_float(right) || is_decimal(right)) {
        return Some(right.clone());
    }
    if *right == SimpleSimpleType::AnyDecimal && (is_float(left) || is_decimal(left)) {
        return Some(left.clone());
    }
    if *left == SimpleSimpleType::AnyDecimal && is_int(right) {
        return Some(left.clone());
    }
    if *right == SimpleSimpleType::AnyDecimal && is_int(left) {
        return Some(right.clone());
    }
    match (temporal(left), temporal(right)) {
        (Some(Temporal::Date), Some(Temporal::Datetime)) => return Some(right.clone()),
        (Some(Temporal::Datetime), Some(Temporal::Date)) => return Some(left.clone()),
        (Some(l), Some(r)) if
            l == r && *left != SimpleSimpleType::AnyDate && *left != SimpleSimpleType::AnyDatetime &&
                *left != SimpleSimpleType::AnyTime => return Some(
            left.clone(),
        ),
        _ => { },
    }
    for (
        any,
        kind,
    ) in [
        (SimpleSimpleType::AnyDate, Temporal::Date),
        (SimpleSimpleType::AnyDatetime, Temporal::Datetime),
        (SimpleSimpleType::AnyTime, Temporal::Time),
    ] {
        if *left == any && temporal(right) == Some(kind) {
            return Some(right.clone());
        }
        if *right == any && temporal(left) == Some(kind) {
            return Some(left.clone());
        }
    }
    if *left == SimpleSimpleType::AnyFloat && is_float(right) {
        return Some(right.clone());
    }
    if *right == SimpleSimpleType::AnyFloat && is_float(left) {
        return Some(left.clone());
    }
    if *left == SimpleSimpleType::AnyInt && is_float(right) {
        return Some(right.clone());
    }
    if *right == SimpleSimpleType::AnyInt && is_float(left) {
        return Some(left.clone());
    }
    let widen_for_literal = |t: &SimpleSimpleType| match t {
        SimpleSimpleType::I8 | SimpleSimpleType::I16 => SimpleSimpleType::I32,
        SimpleSimpleType::U8 | SimpleSimpleType::U16 => SimpleSimpleType::U32,
        t => t.clone(),
    };
    if *left == SimpleSimpleType::AnyInt && is_int(right) {
        return Some(widen_for_literal(right));
    }
    if *right == SimpleSimpleType::AnyInt && is_int(left) {
        return Some(widen_for_literal(left));
    }
    return None;
}
