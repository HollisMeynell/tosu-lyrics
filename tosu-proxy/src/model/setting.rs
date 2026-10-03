use crate::database::SettingEntity;
use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};

pub const SETTINGS_DB_KEY: &str = "settings";

pub const ALIGNMENTS: [&str; 3] = ["left", "center", "right"];
pub const MIN_FONT_SIZE: f32 = 0.5;
pub const MAX_FONT_SIZE: f32 = 12.0;
pub const MAX_FONT_NAME_LEN: usize = 128;
/// 歌词行数（可见窗口）的取值范围：只允许奇数
pub const MIN_LYRIC_LINES: i32 = 1;
pub const MAX_LYRIC_LINES: i32 = 15;
pub const DEFAULT_LYRIC_LINES: i32 = 3;

/// 歌词行数归一化：任何来源（旧数据 / 手改 / 越界 / 偶数）都收敛到合法奇数。
///
/// - 小于下限 → 1，大于上限 → 15
/// - 偶数 → 相邻合法奇数（向上优先，15 封顶时向下）
///
/// 渲染层只会拿到 1 / 3 / 5 / 7 / 9 / 11 / 13 / 15（前端
/// `utils/lyricLines.ts` 的 `normalizeLyricLines` 是同一套规则）。
pub fn normalize_lyric_lines(value: i32) -> i32 {
    let clamped = value.clamp(MIN_LYRIC_LINES, MAX_LYRIC_LINES);
    if clamped % 2 == 1 {
        clamped
    } else if clamped + 1 <= MAX_LYRIC_LINES {
        clamped + 1
    } else {
        clamped - 1
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pair<T> {
    pub first: T,
    pub second: T,
}

impl<T: Clone> Pair<T> {
    pub fn new(first: T, second: T) -> Self {
        Self { first, second }
    }

    /// 主副同值（用于只有单一取值的设置，如对齐方式）
    pub fn same(value: T) -> Self {
        Self {
            first: value.clone(),
            second: value,
        }
    }

    pub fn map<U: Clone>(&self, f: impl Fn(&T) -> U) -> Pair<U> {
        Pair {
            first: f(&self.first),
            second: f(&self.second),
        }
    }
}

impl Pair<f32> {
    /// 保持 2:1 的 active/inactive 比例，与 LyricsBox / utils/lyricScroll.ts 一致
    pub fn inactive(&self) -> Pair<f32> {
        self.map(|size| size / 2.0)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ShadowSettings {
    pub enable: bool,
    pub color: String,
    pub blur: f32,
    pub offset_x: f32,
    pub offset_y: f32,
}

impl Default for ShadowSettings {
    /// 必须与前端 DEFAULT_SHADOW（stores/settingsStore.ts）保持一致，
    /// 否则新装与"恢复默认样式"会给出不同结果。
    /// 默认偏移与模糊都是 3px；已保存的自定义值不受影响（不会回落到这里）。
    fn default() -> Self {
        Self {
            enable: true,
            color: "#000000".to_string(),
            blur: 3.0,
            offset_x: 3.0,
            offset_y: 3.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LyricSettings {
    pub text_color: Pair<String>,
    /// em，指 active 时的字号
    pub font_size: Pair<f32>,
    /// 空串表示用默认字体
    pub font: Pair<String>,
    /// 整体对齐，不支持主副分别设置
    pub alignment: String,
    pub translation_main: bool,
    pub second_show: bool,
    /// 歌词行数（可见窗口）：合法值只有 1/3/5/…/15，当前歌词始终居中，缺失时为 3
    pub lyric_lines: i32,
    pub shadow: Pair<ShadowSettings>,
}

impl Default for LyricSettings {
    fn default() -> Self {
        Self {
            text_color: Pair::new("#ffffff".to_string(), "#e0e0e0".to_string()),
            font_size: Pair::new(3.0, 2.0),
            font: Pair::same(String::new()),
            alignment: "center".to_string(),
            translation_main: true,
            second_show: true,
            lyric_lines: DEFAULT_LYRIC_LINES,
            shadow: Pair::same(ShadowSettings::default()),
        }
    }
}

/// 只处理出现过的字段，未出现的保持原值
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LyricSettingsPatch {
    pub text_color: Option<Pair<String>>,
    pub font_size: Option<Pair<f32>>,
    pub font: Option<Pair<String>>,
    pub alignment: Option<String>,
    pub translation_main: Option<bool>,
    pub second_show: Option<bool>,
    pub lyric_lines: Option<i32>,
    pub shadow: Option<Pair<ShadowSettings>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingKey {
    TextColor,
    FontSize,
    Font,
    Alignment,
    TranslationMain,
    SecondShow,
    LyricLines,
    Shadow,
}

impl SettingKey {
    /// 与前端 handleSettingBroadcast 一致
    pub fn ws_key(&self) -> &'static str {
        match self {
            SettingKey::TextColor => "setColor",
            SettingKey::FontSize => "setFontSize",
            SettingKey::Font => "setFont",
            SettingKey::Alignment => "setAlignment",
            SettingKey::TranslationMain => "setTranslationMain",
            SettingKey::SecondShow => "setSecondShow",
            SettingKey::LyricLines => "setLyricLines",
            SettingKey::Shadow => "setShadow",
        }
    }
}

pub const ALL_SETTING_KEYS: [SettingKey; 8] = [
    SettingKey::TextColor,
    SettingKey::FontSize,
    SettingKey::Font,
    SettingKey::Alignment,
    SettingKey::TranslationMain,
    SettingKey::SecondShow,
    SettingKey::LyricLines,
    SettingKey::Shadow,
];

fn invalid(message: impl Into<String>) -> Error {
    Error::Runtime(message.into())
}

fn validate_color(value: &str) -> Result<()> {
    let hex = value
        .strip_prefix('#')
        .ok_or_else(|| invalid(format!("颜色必须以 # 开头: {value}")))?;
    let len_ok = matches!(hex.len(), 3 | 4 | 6 | 8);
    if !len_ok || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(invalid(format!("颜色格式非法: {value}")));
    }
    Ok(())
}

fn validate_font_size(value: f32) -> Result<()> {
    if !value.is_finite() || !(MIN_FONT_SIZE..=MAX_FONT_SIZE).contains(&value) {
        return Err(invalid(format!(
            "字号必须在 {MIN_FONT_SIZE}~{MAX_FONT_SIZE} 之间: {value}"
        )));
    }
    Ok(())
}

pub const MIN_SHADOW_OFFSET: f32 = -20.0;
pub const MAX_SHADOW_OFFSET: f32 = 20.0;
pub const MAX_SHADOW_BLUR: f32 = 30.0;

fn validate_shadow(shadow: &ShadowSettings) -> Result<()> {
    validate_color(&shadow.color)?;
    for (name, value) in [("offsetX", shadow.offset_x), ("offsetY", shadow.offset_y)] {
        if !value.is_finite() || !(MIN_SHADOW_OFFSET..=MAX_SHADOW_OFFSET).contains(&value) {
            return Err(invalid(format!(
                "阴影 {name} 必须在 {MIN_SHADOW_OFFSET}~{MAX_SHADOW_OFFSET} px 之间: {value}"
            )));
        }
    }
    if !shadow.blur.is_finite() || !(0.0..=MAX_SHADOW_BLUR).contains(&shadow.blur) {
        return Err(invalid(format!(
            "阴影模糊必须在 0~{MAX_SHADOW_BLUR} px 之间: {}",
            shadow.blur
        )));
    }
    Ok(())
}

fn validate_font_name(value: &str) -> Result<()> {
    if value.chars().count() > MAX_FONT_NAME_LEN {
        return Err(invalid(format!(
            "字体名过长（最多 {MAX_FONT_NAME_LEN} 字符）"
        )));
    }
    Ok(())
}

fn validate_lyric_lines(value: i32) -> Result<()> {
    if !(MIN_LYRIC_LINES..=MAX_LYRIC_LINES).contains(&value) || value % 2 == 0 {
        return Err(invalid(format!(
            "歌词行数必须是 {MIN_LYRIC_LINES}~{MAX_LYRIC_LINES} 之间的奇数: {value}"
        )));
    }
    Ok(())
}

impl LyricSettings {
    pub async fn load() -> Self {
        let Some(raw) = SettingEntity::get_config(SETTINGS_DB_KEY)
            .await
            .ok()
            .flatten()
        else {
            return Self::default();
        };
        match crate::util::to_json::<Self>(&raw) {
            Ok(mut settings) => {
                // 旧数据 / 手改过的值也要收敛到合法行数（缺失时 serde 已给默认 3）
                settings.lyric_lines = normalize_lyric_lines(settings.lyric_lines);
                settings
            }
            Err(err) => {
                tracing::error!("设置解析失败，回退默认值: {err}");
                Self::default()
            }
        }
    }

    /// 失败时调用方不得广播，必须把错误原样返回
    pub async fn save(&self) -> Result<()> {
        let raw = serde_json::to_string(self)?;
        SettingEntity::save_config(SETTINGS_DB_KEY.to_string(), raw).await
    }

    pub fn validate(&self) -> Result<()> {
        validate_color(&self.text_color.first)?;
        validate_color(&self.text_color.second)?;
        validate_font_size(self.font_size.first)?;
        validate_font_size(self.font_size.second)?;
        validate_font_name(&self.font.first)?;
        validate_font_name(&self.font.second)?;
        validate_shadow(&self.shadow.first)?;
        validate_shadow(&self.shadow.second)?;
        validate_lyric_lines(self.lyric_lines)?;
        if !ALIGNMENTS.contains(&self.alignment.as_str()) {
            return Err(invalid(format!(
                "对齐方式必须是 {ALIGNMENTS:?} 之一: {}",
                self.alignment
            )));
        }
        Ok(())
    }

    /// 不落库、不改内存，只返回新值
    pub fn with_patch(&self, patch: LyricSettingsPatch) -> Result<Self> {
        let mut next = self.clone();
        if let Some(value) = patch.text_color {
            next.text_color = value;
        }
        if let Some(value) = patch.font_size {
            next.font_size = value;
        }
        if let Some(value) = patch.font {
            next.font = value;
        }
        if let Some(value) = patch.alignment {
            next.alignment = value;
        }
        if let Some(value) = patch.translation_main {
            next.translation_main = value;
        }
        if let Some(value) = patch.second_show {
            next.second_show = value;
        }
        if let Some(value) = patch.lyric_lines {
            // 归一化后才落库 / 广播：渲染层拿到的永远是合法奇数
            next.lyric_lines = normalize_lyric_lines(value);
        }
        if let Some(value) = patch.shadow {
            next.shadow = value;
        }
        next.validate()?;
        Ok(next)
    }

    pub fn changed_keys(&self, before: &Self) -> Vec<SettingKey> {
        let mut changed = Vec::new();
        if self.text_color != before.text_color {
            changed.push(SettingKey::TextColor);
        }
        if self.font_size != before.font_size {
            changed.push(SettingKey::FontSize);
        }
        if self.font != before.font {
            changed.push(SettingKey::Font);
        }
        if self.alignment != before.alignment {
            changed.push(SettingKey::Alignment);
        }
        if self.translation_main != before.translation_main {
            changed.push(SettingKey::TranslationMain);
        }
        if self.second_show != before.second_show {
            changed.push(SettingKey::SecondShow);
        }
        if self.lyric_lines != before.lyric_lines {
            changed.push(SettingKey::LyricLines);
        }
        if self.shadow != before.shadow {
            changed.push(SettingKey::Shadow);
        }
        changed
    }

    pub fn key_value(&self, key: SettingKey) -> serde_json::Value {
        match key {
            SettingKey::TextColor => serde_json::json!(self.text_color),
            SettingKey::FontSize => serde_json::json!(self.font_size),
            SettingKey::Font => serde_json::json!(self.font),
            SettingKey::Alignment => serde_json::json!(Pair::same(&self.alignment)),
            SettingKey::TranslationMain => serde_json::json!(self.translation_main),
            SettingKey::SecondShow => serde_json::json!(self.second_show),
            SettingKey::LyricLines => serde_json::json!(self.lyric_lines),
            SettingKey::Shadow => serde_json::json!(self.shadow),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn default_is_valid() {
        LyricSettings::default().validate().expect("默认值必须合法");
    }

    #[test]
    fn both_shadow_defaults_enabled_and_black() {
        let d = LyricSettings::default();
        assert!(d.shadow.first.enable, "主歌词阴影默认应开启");
        assert!(d.shadow.second.enable, "副歌词阴影默认应开启");
        assert_eq!(d.shadow.first.color, "#000000");
        assert_eq!(d.shadow.second.color, "#000000");
    }

    #[test]
    fn default_values_match_frontend() {
        let d = LyricSettings::default();
        assert_eq!(d.text_color.first, "#ffffff");
        assert_eq!(d.text_color.second, "#e0e0e0");
        assert_eq!(d.font_size.first, 3.0);
        assert_eq!(d.font_size.second, 2.0);
        assert_eq!(d.alignment, "center");
        assert!(d.translation_main);
        assert!(d.second_show);
        assert_eq!(d.lyric_lines, DEFAULT_LYRIC_LINES);
    }

    /// 歌词行数：缺失 → 3，越界收敛到 1 / 15，偶数收敛到相邻奇数，禁止偶数
    #[test]
    fn lyric_lines_normalization() {
        // 合法奇数原样保留
        for value in [1, 3, 5, 7, 9, 11, 13, 15] {
            assert_eq!(normalize_lyric_lines(value), value);
        }
        // 越界
        assert_eq!(normalize_lyric_lines(0), 1);
        assert_eq!(normalize_lyric_lines(-7), 1);
        assert_eq!(normalize_lyric_lines(16), 15);
        assert_eq!(normalize_lyric_lines(99), 15);
        // 偶数 → 相邻合法奇数（向上优先，15 封顶时向下）
        assert_eq!(normalize_lyric_lines(2), 3);
        assert_eq!(normalize_lyric_lines(4), 5);
        assert_eq!(normalize_lyric_lines(14), 15);
        for value in -20..=40 {
            let normalized = normalize_lyric_lines(value);
            assert!(
                (MIN_LYRIC_LINES..=MAX_LYRIC_LINES).contains(&normalized) && normalized % 2 == 1,
                "归一化结果必须是合法奇数: {value} -> {normalized}"
            );
        }
    }

    /// 行数经过 PATCH 落库 / 广播的完整链路，且只影响这一个设置
    #[test]
    fn lyric_lines_patch_roundtrip() {
        let before = LyricSettings::default();
        let patch: LyricSettingsPatch =
            serde_json::from_str(r#"{"lyricLines":7}"#).expect("patch 解析");
        let after = before.with_patch(patch).expect("patch 应用");
        assert_eq!(after.lyric_lines, 7);
        assert_eq!(after.changed_keys(&before), vec![SettingKey::LyricLines]);
        assert_eq!(SettingKey::LyricLines.ws_key(), "setLyricLines");
        assert_eq!(
            after.key_value(SettingKey::LyricLines),
            serde_json::json!(7)
        );
        // 其他设置必须原样保留
        assert_eq!(after.text_color, before.text_color);
        assert_eq!(after.font_size, before.font_size);
        assert_eq!(after.alignment, before.alignment);
        assert_eq!(after.second_show, before.second_show);
        assert_eq!(after.shadow, before.shadow);
        // 偶数会被归一化，而不会报错
        let even: LyricSettingsPatch =
            serde_json::from_str(r#"{"lyricLines":6}"#).expect("patch 解析");
        assert_eq!(before.with_patch(even).expect("patch 应用").lyric_lines, 7);
    }

    #[test]
    fn shadow_defaults() {
        let d = ShadowSettings::default();
        // 主 / 副默认都开启、默认黑色、偏移与模糊都是 3px
        // —— 与前端 DEFAULT_SHADOW 必须一致
        assert!(d.enable, "默认应开启阴影");
        assert_eq!(d.color, "#000000");
        assert_eq!(d.blur, 3.0);
        assert_eq!(d.offset_x, 3.0);
        assert_eq!(d.offset_y, 3.0);
    }

    #[test]
    fn shadow_range_validation() {
        let ok = ShadowSettings { enable: true, color: "#000000".into(), blur: 4.0, offset_x: -5.0, offset_y: 5.0 };
        assert!(validate_shadow(&ok).is_ok());
        // 超出范围必须拒绝，避免用户把歌词推到看不见的地方
        let too_far = ShadowSettings { offset_x: 999.0, ..ok.clone() };
        assert!(validate_shadow(&too_far).is_err());
        let too_blurry = ShadowSettings { blur: 999.0, ..ok.clone() };
        assert!(validate_shadow(&too_blurry).is_err());
        let negative_blur = ShadowSettings { blur: -1.0, ..ok.clone() };
        assert!(validate_shadow(&negative_blur).is_err());
    }

    #[test]
    fn shadow_patch_roundtrip() {
        let before = LyricSettings::default();
        // 注意用 r##" 分隔符：颜色里的 "# 会提前终止 r#"..."#
        let patch: LyricSettingsPatch = serde_json::from_str(
            r##"{"shadow":{"first":{"enable":true,"color":"#ff0000","blur":3,"offsetX":1,"offsetY":1},"second":{"enable":false,"color":"#00ff00","blur":0,"offsetX":0,"offsetY":0}}}"##,
        )
        .expect("patch 解析");
        let after = before.with_patch(patch).expect("patch 应用");
        assert!(after.shadow.first.enable);
        assert_eq!(after.shadow.first.color, "#ff0000");
        assert_eq!(after.shadow.first.offset_x, 1.0);
        assert_eq!(after.shadow.first.blur, 3.0);
        assert_eq!(after.changed_keys(&before), vec![SettingKey::Shadow]);
    }

    #[test]
    fn invalid_shadow_rejected() {
        let before = LyricSettings::default();
        let patch: LyricSettingsPatch = serde_json::from_str(
            r##"{"shadow":{"first":{"color":"not-a-color","blur":3,"offsetX":1,"offsetY":1},"second":{"color":"#000","blur":0,"offsetX":0,"offsetY":0}}}"##,
        )
        .expect("patch 解析");
        assert!(before.with_patch(patch).is_err(), "非法颜色必须被拒绝");
    }

    #[test]
    fn patch_only_touches_given_fields() {
        let before = LyricSettings::default();
        let patch: LyricSettingsPatch =
            serde_json::from_str(r#"{"alignment":"left"}"#).expect("patch 解析");
        let after = before.with_patch(patch).expect("patch 应用");
        assert_eq!(after.alignment, "left");
        // 其他字段必须原样保留
        assert_eq!(after.text_color, before.text_color);
        assert_eq!(after.font_size, before.font_size);
        assert_eq!(after.font, before.font);
        assert_eq!(after.translation_main, before.translation_main);
        assert_eq!(after.second_show, before.second_show);
        assert_eq!(after.lyric_lines, before.lyric_lines);
        assert_eq!(after.changed_keys(&before), vec![SettingKey::Alignment]);
    }

    #[test]
    fn rejects_invalid_values() {
        let cases = [
            r#"{"alignment":"middle"}"#,
            r##"{"textColor":{"first":"red","second":"#fff"}}"##,
            r#"{"fontSize":{"first":0,"second":2}}"#,
            r#"{"fontSize":{"first":99,"second":2}}"#,
        ];
        for raw in cases {
            let patch: LyricSettingsPatch = serde_json::from_str(raw).expect("patch 解析");
            assert!(
                LyricSettings::default().with_patch(patch).is_err(),
                "应当拒绝: {raw}"
            );
        }
    }

    #[test]
    fn serializes_with_camel_case() {
        let json = serde_json::to_value(LyricSettings::default()).expect("序列化");
        assert!(json.get("textColor").is_some());
        assert!(json.get("fontSize").is_some());
        assert!(json.get("translationMain").is_some());
        assert!(json.get("secondShow").is_some());
        assert_eq!(json["textColor"]["first"], "#ffffff");
        assert_eq!(json["lyricLines"], DEFAULT_LYRIC_LINES);
    }

    /// 旧数据只写了部分字段时, 缺失字段必须回落到默认值而不是解析失败
    #[test]
    fn tolerates_partial_legacy_json() {
        let settings: LyricSettings =
            serde_json::from_str(r#"{"alignment":"right"}"#).expect("部分字段应能解析");
        assert_eq!(settings.alignment, "right");
        assert_eq!(settings.font_size.first, 3.0);
        assert!(settings.second_show);
        // 老配置文件没有 lyricLines → 默认 3 行
        assert_eq!(settings.lyric_lines, DEFAULT_LYRIC_LINES);
    }

    #[test]
    fn inactive_font_size_keeps_half() {
        let sizes = Pair::new(4.0, 3.0);
        let inactive = sizes.inactive();
        assert_eq!((inactive.first, inactive.second), (2.0, 1.5));
    }
}
