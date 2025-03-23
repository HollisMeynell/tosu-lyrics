use std::fmt::Debug;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, LyricError>;

#[derive(Error, Debug)]
pub enum LyricError {
    #[error("{0}")]
    Runtime(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Web(#[from] salvo::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

impl<T: Debug> From<T> for LyricError {
    fn from(value: T) -> Self {
        Self::Runtime(format!("this struct error: {value:?}"))
    }
}

impl<T: ToString> From<T> for LyricError {
    fn from(value: T) -> Self {
        Self::Runtime(value.to_string())
    }
}
