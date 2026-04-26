use std::fmt::Display;

#[cfg(feature = "pg")]
pub mod pg;
#[cfg(feature = "sqlite")]
pub mod sqlite;

#[derive(Debug)]
pub struct GoodError(pub String);

impl std::fmt::Display for GoodError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for GoodError { }

pub trait ToGoodError<T> {
    fn to_good_error<F: FnOnce() -> String>(self, context: F) -> Result<T, GoodError>;
    fn to_good_error_query(self, query: &str) -> Result<T, GoodError>;
}

impl<T, E: Display> ToGoodError<T> for Result<T, E> {
    fn to_good_error<F: FnOnce() -> String>(self, context: F) -> Result<T, GoodError> {
        match self {
            Ok(x) => Ok(x),
            Err(e) => Err(GoodError(format!("{}: {}", context(), e))),
        }
    }

    fn to_good_error_query(self, query: &str) -> Result<T, GoodError> {
        return self.to_good_error(|| format!("In query [{}]", query));
    }
}
