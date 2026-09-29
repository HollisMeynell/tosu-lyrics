use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NumberName {
    #[serde(default)]
    pub number: i64,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BeatmapTime {
    pub live: i32,

    #[serde(rename = "firstObject")]
    pub first_object: i64,

    #[serde(rename = "lastObject")]
    pub last_object: i64,

    #[serde(rename = "mp3Length", default)]
    pub mp3_length: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Beatmap {
    #[serde(rename = "isConvert")]
    pub is_convert: bool,

    pub time: BeatmapTime,

    pub status: NumberName,

    #[serde(default)]
    pub checksum: String,

    #[serde(default)]
    pub id: i64,

    #[serde(default)]
    pub set: i64,

    pub mode: NumberName,

    pub artist: Option<String>,

    #[serde(rename = "artistUnicode")]
    pub artist_unicode: Option<String>,

    pub title: Option<String>,

    #[serde(rename = "titleUnicode")]
    pub title_unicode: Option<String>,

    #[serde(default)]
    pub mapper: String,

    #[serde(default)]
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Files {
    #[serde(default)]
    pub beatmap: String,

    #[serde(default)]
    pub background: String,

    #[serde(default)]
    pub audio: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Folders {
    #[serde(default)]
    pub game: String,

    #[serde(default)]
    pub skin: String,

    #[serde(default)]
    pub songs: String,

    #[serde(default)]
    pub beatmap: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TosuApi {
    pub beatmap: Beatmap,

    pub files: Files,

    pub folders: Folders,
}

impl TosuApi {
    pub fn audio_path(&self) -> PathBuf {
        Path::new(&self.folders.songs)
            .join(&self.folders.beatmap)
            .join(&self.files.audio)
    }
}

impl TryFrom<&str> for TosuApi {
    type Error = crate::error::Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        crate::util::to_json(value)
    }
}

#[cfg(test)]
mod test {
    use super::TosuApi;

    /// 后端不读、但真实 tosu 一定会发的字段，验证"未知字段被忽略"
    const EXTRA_PAYLOAD: &str = r#"
        "game": { "focused": true, "paused": false },
        "client": "stable",
        "server": "ppy.sh",
        "state": { "number": 2, "name": "play" },
        "session": { "playTime": 39122, "playCount": 0 },
        "settings": { "interfaceVisible": true, "bassDensity": 0, "skin": { "name": "default" } },
        "profile": { "id": 1234, "name": "probe" },
        "play": { "failed": false, "score": 123456, "combo": { "current": 12, "max": 30 } },
        "leaderboard": [],
        "performance": { "accuracy": { "95": 1 }, "graph": { "series": [], "xaxis": [] } },
        "resultsScreen": { "scoreId": 0, "maxCombo": 0 },
        "directPath": { "beatmapFile": "a.osu", "beatmapAudio": "audio.mp3" }
    "#;

    fn menu_beatmap() -> String {
        r#"{
            "isKiai": false, "isBreak": false, "isConvert": false,
            "time": { "live": 0, "firstObject": 0, "lastObject": 0, "mp3Length": 0 },
            "status": { "number": 0, "name": "" },
            "checksum": "", "id": 0, "set": 0,
            "mode": { "number": 0, "name": "osu" },
            "artist": "", "artistUnicode": "", "title": "", "titleUnicode": "",
            "mapper": "", "version": "", "source": "", "tags": "", "stats": {}
        }"#
        .to_string()
    }

    fn playing_beatmap(live: i64) -> String {
        format!(
            r#"{{
            "isKiai": false, "isBreak": false, "isConvert": false,
            "time": {{ "live": {live}, "firstObject": 21000, "lastObject": 251000, "mp3Length": 256122 }},
            "status": {{ "number": 1, "name": "Ranked" }},
            "checksum": "0f1a2b3c", "id": 3344501, "set": 900001,
            "mode": {{ "number": 0, "name": "osu" }},
            "artist": "Kenshi Yonezu", "artistUnicode": "米津玄師",
            "title": "Lemon", "titleUnicode": "Lemon",
            "mapper": "probe", "version": "Normal", "source": "", "tags": "", "stats": {{}}
        }}"#
        )
    }

    fn files_folders() -> &'static str {
        r#""folders": { "game": "C:/osu!", "skin": "default", "songs": "C:/osu!/Songs", "beatmap": "3344501 Kenshi Yonezu - Lemon" },
           "files": { "beatmap": "Lemon.osu", "background": "bg.jpg", "audio": "audio.mp3" }"#
    }

    fn payload_with(beatmap: &str, extra_tail: &str) -> String {
        format!(
            "{{ {EXTRA_PAYLOAD}, \"beatmap\": {beatmap}, {}, {extra_tail} }}",
            files_folders()
        )
    }

    fn real_playing_payload(live: i64) -> String {
        payload_with(&playing_beatmap(live), r#""tourney": { "scoreVisible": false, "clients": [] }"#)
    }

    #[test]
    fn parse_real_playing_payload() {
        let api = TosuApi::try_from(real_playing_payload(39122).as_str())
            .expect("真实 tosu v2 播放报文必须能解析");

        assert_eq!(api.beatmap.id, 3344501);
        assert_eq!(api.beatmap.set, 900001);
        assert_eq!(api.beatmap.time.live, 39122);
        assert_eq!(api.beatmap.time.first_object, 21000);
        assert_eq!(api.beatmap.time.last_object, 251000);
        assert_eq!(api.beatmap.time.mp3_length, 256122);
        assert_eq!(api.beatmap.title_unicode.as_deref(), Some("Lemon"));
        assert_eq!(api.beatmap.artist_unicode.as_deref(), Some("米津玄師"));
        assert_eq!(api.folders.songs, "C:/osu!/Songs");
        assert_eq!(api.folders.beatmap, "3344501 Kenshi Yonezu - Lemon");
        assert_eq!(api.files.audio, "audio.mp3");

        assert_eq!(
            api.audio_path().to_string_lossy().replace('\\', "/"),
            "C:/osu!/Songs/3344501 Kenshi Yonezu - Lemon/audio.mp3"
        );
    }

    #[test]
    fn parse_real_menu_payload() {
        let api = TosuApi::try_from(payload_with(&menu_beatmap(), r#""tourney": {}"#).as_str())
            .expect("真实 tosu v2 菜单报文必须能解析");
        assert_eq!(api.beatmap.id, 0);
        assert_eq!(api.beatmap.set, 0);
        assert_eq!(api.beatmap.time.live, 0);
        assert_eq!(api.beatmap.title_unicode.as_deref(), Some(""));
        assert_eq!(api.folders.beatmap, "3344501 Kenshi Yonezu - Lemon");
    }

    /// `rankedPlay` 在 tosu 里可能是 `undefined`, JSON.stringify 会**直接省略该 key**
    #[test]
    fn parse_payload_without_ranked_play() {
        let payload = real_playing_payload(1000).replace(r#""tourney": { "scoreVisible": false, "clients": [] }"#, r#""tourney": {}"#);
        assert!(!payload.contains("rankedPlay"));
        TosuApi::try_from(payload.as_str()).expect("缺少 rankedPlay 不应导致解析失败");
    }

    /// 回归: serde 对 `Option<T>` 字段在 **key 缺失**(不是 null)时的行为。
    /// 真实 tosu 永远会发这些 key, 但一旦某次状态未就绪就可能缺失,
    /// 该用例用于固定"缺失时是否会整条报文被丢弃"的结论。
    #[test]
    fn missing_optional_field_behaviour() {
        let full = real_playing_payload(1000);
        let without_title = full.replace(r#""titleUnicode": "Lemon","#, "");
        assert!(!without_title.contains("titleUnicode"));

        let result = TosuApi::try_from(without_title.as_str());
        assert!(
            result.is_ok(),
            "缺少 titleUnicode 会导致整条报文被丢弃, 需要给该字段加 serde(default): {:?}",
            result.err()
        );
    }
}
