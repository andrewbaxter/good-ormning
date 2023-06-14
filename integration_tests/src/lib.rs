use std::borrow::Cow;
use good_ormning_runtime::{
    pg,
    sqlite,
};

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
