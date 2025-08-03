mod tosu;

pub use tosu::TosuWebsocketClient;

#[derive(Debug)]
pub struct OsuSongInfo {
    pub bid: i64,
    pub sid: i64,
    /// 毫秒
    pub length: i32,
    /// 毫秒
    pub now: i32,

    pub artist: String,
    pub artist_unicode: String,
    pub title: String,
    pub title_unicode: String,
}

pub enum OsuState {
    Time(i32),
    Song(OsuSongInfo),
    Clean,
}

pub trait OsuSource {
    fn start(self) -> impl Future<Output = ()> + Send;
    fn on_osu_state_change(&self, state: OsuState) -> impl Future<Output = ()> + Send {
        use crate::service::on_osu_state_change;
        on_osu_state_change(state)
    }
}
