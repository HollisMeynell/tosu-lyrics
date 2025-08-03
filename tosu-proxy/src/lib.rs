pub mod error;
pub mod util;

#[cfg(feature = "new")]
pub mod config;
#[cfg(feature = "new")]
pub mod database;
#[cfg(feature = "new")]
pub mod lyric;
#[cfg(feature = "new")]
pub mod model;
#[cfg(feature = "new")]
pub mod osu_source;
#[cfg(feature = "new")]
pub mod server;
#[cfg(feature = "new")]
pub mod service;
#[cfg(feature = "new")]
pub mod setting;

#[cfg(feature = "old")]
pub mod old;
