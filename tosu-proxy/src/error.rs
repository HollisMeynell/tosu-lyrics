use salvo::http::StatusCode;
use salvo::{async_trait, Depot, Request, Response};
use std::fmt::Debug;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Runtime(String),

    #[error("parse lyric error: {0}")]
    LyricParse(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    WebSB1(#[from] salvo::http::StatusError),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error("this error can not be throw...")]
    Impossible,
}
/*
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
*/
#[async_trait]
impl salvo::Writer for Error {
    async fn write(self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        res.status_code = Some(StatusCode::BAD_REQUEST);
        res.render(format!("{:?}", &self));
    }
}
