pub mod util;
pub mod error;


#[cfg(feature = "new")]
pub mod database;
#[cfg(feature = "new")]
pub mod lyric;
#[cfg(feature = "new")]
pub mod server;
#[cfg(feature = "new")]
pub mod config;

#[cfg(feature = "old")]
pub mod old;
