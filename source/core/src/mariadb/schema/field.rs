use {
    crate::mariadb::{
        query::expr::SerialExpr,
        types::Type,
    },
    serde::{
        Deserialize,
        Serialize,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    pub id: String,
    pub renamed_from: Option<String>,
    pub type_: FieldType,
}

pub fn field_binary() -> FieldTypeBuilder {
    return FieldTypeBuilder::new(crate::mariadb::types::type_bytes().build()).variant(TypeVariant::Binary);
}

pub fn field_blob() -> FieldTypeBuilder {
    return FieldTypeBuilder::new(crate::mariadb::types::type_bytes().build()).variant(TypeVariant::Blob);
}

pub fn field_bool() -> FieldTypeBuilder {
    return FieldTypeBuilder::new(crate::mariadb::types::type_bool().build());
}

pub fn field_bytes() -> FieldTypeBuilder {
    return FieldTypeBuilder::new(crate::mariadb::types::type_bytes().build());
}

pub fn field_char() -> FieldTypeBuilder {
    return FieldTypeBuilder::new(crate::mariadb::types::type_str().build()).variant(TypeVariant::Char);
}

#[cfg(feature = "chrono")]
pub fn field_date_chrono() -> FieldTypeBuilder {
    return FieldTypeBuilder::new(crate::mariadb::types::type_date_chrono().build());
}

#[cfg(feature = "jiff")]
pub fn field_date_jiff() -> FieldTypeBuilder {
    return FieldTypeBuilder::new(crate::mariadb::types::type_date_jiff().build());
}

#[cfg(feature = "chrono")]
pub fn field_datetime_chrono() -> FieldTypeBuilder {
    return FieldTypeBuilder::new(crate::mariadb::types::type_datetime_chrono().build());
}

#[cfg(feature = "jiff")]
pub fn field_datetime_jiff() -> FieldTypeBuilder {
    return FieldTypeBuilder::new(crate::mariadb::types::type_datetime_jiff().build());
}

pub fn field_decimal() -> FieldTypeBuilder {
    return FieldTypeBuilder::new(crate::mariadb::types::type_decimal().build());
}

pub fn field_f32() -> FieldTypeBuilder {
    return FieldTypeBuilder::new(crate::mariadb::types::type_f32().build());
}

pub fn field_f64() -> FieldTypeBuilder {
    return FieldTypeBuilder::new(crate::mariadb::types::type_f64().build());
}

pub fn field_i8() -> FieldTypeBuilder {
    return FieldTypeBuilder::new(crate::mariadb::types::type_i8().build());
}

pub fn field_i16() -> FieldTypeBuilder {
    return FieldTypeBuilder::new(crate::mariadb::types::type_i16().build());
}

pub fn field_i32() -> FieldTypeBuilder {
    return FieldTypeBuilder::new(crate::mariadb::types::type_i32().build());
}

pub fn field_i64() -> FieldTypeBuilder {
    return FieldTypeBuilder::new(crate::mariadb::types::type_i64().build());
}

pub fn field_mediumblob() -> FieldTypeBuilder {
    return FieldTypeBuilder::new(crate::mariadb::types::type_bytes().build()).variant(TypeVariant::MediumBlob);
}

pub fn field_mediumint() -> FieldTypeBuilder {
    return FieldTypeBuilder::new(crate::mariadb::types::type_i32().build()).variant(TypeVariant::Mediumint);
}

pub fn field_mediumint_unsigned() -> FieldTypeBuilder {
    return FieldTypeBuilder::new(crate::mariadb::types::type_u32().build()).variant(TypeVariant::Mediumint);
}

pub fn field_mediumtext() -> FieldTypeBuilder {
    return FieldTypeBuilder::new(crate::mariadb::types::type_str().build()).variant(TypeVariant::MediumText);
}

pub fn field_str() -> FieldTypeBuilder {
    return FieldTypeBuilder::new(crate::mariadb::types::type_str().build());
}

pub fn field_text() -> FieldTypeBuilder {
    return FieldTypeBuilder::new(crate::mariadb::types::type_str().build()).variant(TypeVariant::Text);
}

#[cfg(feature = "chrono")]
pub fn field_time_chrono() -> FieldTypeBuilder {
    return FieldTypeBuilder::new(crate::mariadb::types::type_time_chrono().build());
}

#[cfg(feature = "jiff")]
pub fn field_time_jiff() -> FieldTypeBuilder {
    return FieldTypeBuilder::new(crate::mariadb::types::type_time_jiff().build());
}

#[cfg(feature = "chrono")]
pub fn field_timestamp_chrono() -> FieldTypeBuilder {
    return FieldTypeBuilder::new(
        crate::mariadb::types::type_datetime_chrono().build(),
    ).variant(TypeVariant::Timestamp);
}

#[cfg(feature = "jiff")]
pub fn field_timestamp_jiff() -> FieldTypeBuilder {
    return FieldTypeBuilder::new(
        crate::mariadb::types::type_datetime_jiff().build(),
    ).variant(TypeVariant::Timestamp);
}

pub fn field_tinyblob() -> FieldTypeBuilder {
    return FieldTypeBuilder::new(crate::mariadb::types::type_bytes().build()).variant(TypeVariant::TinyBlob);
}

pub fn field_tinytext() -> FieldTypeBuilder {
    return FieldTypeBuilder::new(crate::mariadb::types::type_str().build()).variant(TypeVariant::TinyText);
}

pub fn field_u8() -> FieldTypeBuilder {
    return FieldTypeBuilder::new(crate::mariadb::types::type_u8().build());
}

pub fn field_u16() -> FieldTypeBuilder {
    return FieldTypeBuilder::new(crate::mariadb::types::type_u16().build());
}

pub fn field_u32() -> FieldTypeBuilder {
    return FieldTypeBuilder::new(crate::mariadb::types::type_u32().build());
}

pub fn field_u64() -> FieldTypeBuilder {
    return FieldTypeBuilder::new(crate::mariadb::types::type_u64().build());
}

pub fn field_year() -> FieldTypeBuilder {
    return FieldTypeBuilder::new(crate::mariadb::types::type_u16().build()).variant(TypeVariant::Year);
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FieldRef {
    pub field_id: String,
    pub table_id: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct FieldType {
    pub auto_increment: bool,
    pub default: Option<SerialExpr>,
    pub max_len: Option<u32>,
    pub precision: Option<u8>,
    pub scale: Option<u8>,
    pub type_: Type,
    pub variant: Option<TypeVariant>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum TypeVariant {
    Binary,
    Blob,
    Char,
    MediumBlob,
    MediumText,
    Mediumint,
    Text,
    Timestamp,
    TinyBlob,
    TinyText,
    Year,
}

pub struct FieldTypeBuilder(pub FieldType);

impl FieldTypeBuilder {
    pub fn auto_increment(mut self) -> Self {
        self.0.auto_increment = true;
        return self;
    }

    pub fn build(self) -> FieldType {
        return self.0;
    }

    pub fn default(mut self, e: SerialExpr) -> Self {
        self.0.default = Some(e);
        return self;
    }

    pub fn max_len(mut self, len: u32) -> Self {
        self.0.max_len = Some(len);
        return self;
    }

    pub fn new(t: Type) -> Self {
        return Self(FieldType {
            type_: t,
            auto_increment: false,
            default: None,
            max_len: None,
            precision: None,
            scale: None,
            variant: None,
        });
    }

    pub fn precision(mut self, precision: u8) -> Self {
        self.0.precision = Some(precision);
        return self;
    }

    pub fn scale(mut self, scale: u8) -> Self {
        self.0.scale = Some(scale);
        return self;
    }

    fn variant(mut self, variant: TypeVariant) -> Self {
        self.0.variant = Some(variant);
        return self;
    }
}

impl FieldTypeBuilder {
    pub fn opt(mut self) -> Self {
        self.0.type_.opt = true;
        return self;
    }
}
