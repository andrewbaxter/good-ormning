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

#[cfg(feature = "sqlite")]
impl From<rusqlite::Error> for GoodError {
    fn from(value: rusqlite::Error) -> Self {
        GoodError(value.to_string())
    }
}

#[cfg(feature = "pg")]
impl From<tokio_postgres::Error> for GoodError {
    fn from(value: tokio_postgres::Error) -> Self {
        GoodError(value.to_string())
    }
}
