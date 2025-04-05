#[cfg(feature = "new")]
use salvo::http::StatusCode;
#[cfg(feature = "new")]
use salvo::{Depot, Request, Response, async_trait};
use std::fmt::Debug;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Static(&'static str),

    #[error("{0}")]
    Runtime(String),
    #[cfg(feature = "new")]
    #[error("parse lyric error: {0}")]
    LyricParse(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Request(#[from] reqwest::Error),

    #[cfg(feature = "new")]
    #[error(transparent)]
    WebSB1(#[from] salvo::http::StatusError),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[cfg(feature = "new")]
    #[error(transparent)]
    Database(#[from] sea_orm::error::DbErr),

    #[error("this error can not be throw...")]
    Impossible,
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::Runtime(value)
    }
}

impl From<&'static str> for Error {
    fn from(value: &'static str) -> Self {
        Self::Static(value)
    }
}

#[cfg(feature = "new")]
#[async_trait]
impl salvo::Writer for Error {
    async fn write(self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        res.status_code = Some(StatusCode::BAD_REQUEST);
        res.render(format!("{:?}", &self));
    }
}
