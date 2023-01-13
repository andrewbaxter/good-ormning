use std::fmt::Display;

#[derive(PartialEq, Eq, Debug)]
pub struct MyString(pub String);
#[derive(Debug)]
pub struct MyErr;

impl Display for MyErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "".fmt(f)
    }
}

impl std::error::Error for MyErr { }

impl MyString {
    pub fn to_sql(&self) -> &str {
        &self.0
    }

    pub fn from_sql(s: String) -> Result<Self, MyErr> {
        Ok(Self(s))
    }
}
