use tokio_postgres::error::SqlState;

#[derive(Debug)]
pub enum Error {
    BadSchema,
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::BadSchema => "Current DB schema doesn't match query schema".fmt(f),
            Error::Other(s) => s.fmt(f),
        }
    }
}

impl std::error::Error for Error { }

#[cfg(feature = "sqlite")]
impl From<rusqlite::Error> for Error {
    fn from(value: rusqlite::Error) -> Self {
        match &value {
            rusqlite::Error::SqliteFailure(_, Some(m)) => {
                if m.starts_with("no such table") {
                    return Self::BadSchema
                }
                return Self::Other(value.to_string())
            },
            _ => return Self::Other(value.to_string()),
        }
    }
}

#[cfg(feature = "pg")]
impl From<tokio_postgres::Error> for Error {
    fn from(value: tokio_postgres::Error) -> Self {
        if value.code() == Some(&SqlState::UNDEFINED_TABLE) {
            return Self::BadSchema
        }
        return Self::Other(value.to_string())
    }
}
