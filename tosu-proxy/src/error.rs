#[cfg(feature = "new")]
use salvo::http::StatusCode;
#[cfg(feature = "new")]
use salvo::{Depot, Request, Response, async_trait};
use std::fmt::{Debug, Display};
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
    LyricParse(&'static str),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Request(#[from] reqwest::Error),

    #[cfg(feature = "new")]
    #[error(transparent)]
    WebSB1(#[from] salvo::http::StatusError),

    #[error(transparent)]
    ToString(#[from] std::string::FromUtf8Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[cfg(feature = "new")]
    #[error(transparent)]
    Database(#[from] sea_orm::error::DbErr),

    #[error("this error can not be throw...")]
    Impossible,

    #[error("{0:?}")]
    Box(Box<BoxError>),
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

pub enum BoxError {
    Err(Error),
    Trace(Box<Self>, String),
}

impl BoxError {
    fn format(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BoxError::Err(e) => writeln!(f, "{e}"),
            BoxError::Trace(e, s) => {
                e.format(f)?;
                writeln!(f, "\t:{s}")
            }
        }
    }

    fn new<T: ToString>(self: Box<Self>, context: T) -> Box<Self> {
        let result = Self::Trace(self, context.to_string());
        Box::new(result)
    }
}

impl Debug for BoxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format(f)
    }
}

impl Display for BoxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format(f)
    }
}

impl From<Error> for BoxError {
    fn from(e: Error) -> Self {
        Self::Err(e)
    }
}

pub trait Context {
    fn with_context<T: ToString>(self, context: T) -> Self;
}

impl<R> Context for Result<R> {
    fn with_context<T: ToString>(self, context: T) -> Self {
        let err = match self {
            Ok(r) => return Ok(r),
            Err(e) => e,
        };
        let err = err.with_context(context);
        Err(err)
    }
}

impl Error {
    pub(crate) fn with_context<T: ToString>(self, context: T) -> Self {
        match self {
            Error::Box(box_err) => Error::Box(box_err.new(context)),
            other => Error::Box(BoxError::new(Box::new(other.into()), context)),
        }
    }
}
