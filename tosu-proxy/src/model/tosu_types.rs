use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NumberName {
    #[serde(default)]
    pub number: i64,
    pub name: String,
}

/// 谱面的时间信息
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BeatmapTime {
    /// 实时位置
    pub live: i32,

    /// 第一个物件的时间
    #[serde(rename = "firstObject")]
    pub first_object: i64,

    /// 最后一个物件的时间
    #[serde(rename = "lastObject")]
    pub last_object: i64,

    /// MP3 文件长度
    #[serde(rename = "mp3Length", default)]
    pub mp3_length: i64,
}

/// 谱面信息
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Beatmap {
    /// 是否为转换谱面
    #[serde(rename = "isConvert")]
    pub is_convert: bool,

    /// 时间
    pub time: BeatmapTime,

    /// Rank 状态
    pub status: NumberName,

    /// 谱面校验和
    #[serde(default)]
    pub checksum: String,

    /// 谱面ID
    #[serde(default)]
    pub id: i64,

    /// 谱面集ID
    #[serde(default)]
    pub set: i64,

    /// 游戏模式
    pub mode: NumberName,

    /// 艺术家名
    pub artist: Option<String>,

    /// Unicode格式的艺术家名
    #[serde(rename = "artistUnicode")]
    pub artist_unicode: Option<String>,

    /// 歌曲标题
    pub title: Option<String>,

    /// Unicode格式的歌曲标题
    #[serde(rename = "titleUnicode")]
    pub title_unicode: Option<String>,

    /// 谱面作者
    #[serde(default)]
    pub mapper: String,

    /// 难度名称
    #[serde(default)]
    pub version: String,
}

/// 谱面相关文件的路径
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Files {
    /// 谱面文件相对路径
    #[serde(default)]
    pub beatmap: String,

    /// bg相对路径
    #[serde(default)]
    pub background: String,

    /// 音频相对路径
    #[serde(default)]
    pub audio: String,
}

/// 相关文件夹的路径
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Folders {
    /// 主目录
    #[serde(default)]
    pub game: String,

    /// 皮肤目录
    #[serde(default)]
    pub skin: String,

    /// 歌曲目录
    #[serde(default)]
    pub songs: String,

    /// 当前谱面目录
    #[serde(default)]
    pub beatmap: String,
}

/// Tosu API 响应的主结构
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TosuApi {
    /// 谱面信息
    pub beatmap: Beatmap,

    /// 文件路径
    pub files: Files,

    /// 文件夹路径
    pub folders: Folders,
}

impl TosuApi {
    /// 获取音频文件的完整路径
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

    /// `/websocket/v2` 里后端**不读**、但真实 tosu 一定会发的字段。
    /// 全部由 `buildResultV2.ts` 生成, 这里保留真实 key 名以验证"未知字段被忽略"。
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

    /// 真实 tosu 在菜单里(未选图)时的 beatmap 形状:
    /// `menu.mapID` / `menu.setID` 为 0, 标题歌手为空串
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

    /// 播放中的完整 v2 报文(含 rankedPlay 被 JSON.stringify 省略的情况)
    fn real_playing_payload(live: i64) -> String {
        payload_with(&playing_beatmap(live), r#""tourney": { "scoreVisible": false, "clients": [] }"#)
    }

    #[test]
    fn parse_real_playing_payload() {
        let api = TosuApi::try_from(real_playing_payload(39122).as_str())
            .expect("真实 tosu v2 播放报文必须能解析");

        assert_eq!(api.beatmap.id, 3344501);
        assert_eq!(api.beatmap.set, 900001);
        // live 是**毫秒**, 直接就是当前播放位置
        assert_eq!(api.beatmap.time.live, 39122);
        assert_eq!(api.beatmap.time.first_object, 21000);
        assert_eq!(api.beatmap.time.last_object, 251000);
        assert_eq!(api.beatmap.time.mp3_length, 256122);
        assert_eq!(api.beatmap.title_unicode.as_deref(), Some("Lemon"));
        assert_eq!(api.beatmap.artist_unicode.as_deref(), Some("米津玄師"));
        assert_eq!(api.folders.songs, "C:/osu!/Songs");
        assert_eq!(api.folders.beatmap, "3344501 Kenshi Yonezu - Lemon");
        assert_eq!(api.files.audio, "audio.mp3");

        // 音频完整路径 = songs / beatmap 文件夹 / 文件名, 与 directPath.beatmapAudio 一致
        assert_eq!(
            api.audio_path().to_string_lossy().replace('\\', "/"),
            "C:/osu!/Songs/3344501 Kenshi Yonezu - Lemon/audio.mp3"
        );
    }

    /// 菜单/未选图: mapID=0 —— 后端据此判定"清空"
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
