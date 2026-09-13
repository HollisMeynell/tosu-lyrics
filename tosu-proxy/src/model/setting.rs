//! 展示设置的**唯一**数据模型。
//!
//! 取代原先两套 key（`LyricSetting` 宏用 `trans_main` / `align` / `show_second`，
//! WS 管理命令用 `translation-main` / `alignment` / `second-show`）——这就是 R8。
//! 现在整份设置以**一条**数据库记录（key = [`SETTINGS_DB_KEY`]）保存，
//! WS 管理命令与新的 HTTP 接口都读写这一份内存状态。

use crate::database::SettingEntity;
use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};

/// 整份设置在数据库里的 key
pub const SETTINGS_DB_KEY: &str = "settings";

/// 允许的对齐方式
pub const ALIGNMENTS: [&str; 3] = ["left", "center", "right"];
/// 允许的字号范围（em）
pub const MIN_FONT_SIZE: f32 = 0.5;
pub const MAX_FONT_SIZE: f32 = 12.0;
/// 字体名最大长度
pub const MAX_FONT_NAME_LEN: usize = 128;

/// 主 / 副一对值，序列化后就是前端消费的 `{ first, second }`
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
    /// 非 active（未唱到）时的字号：保持 2:1 的 active/inactive 比例，
    /// 与 `LyricsBox` / `utils/lyricScroll.ts` 的假设一致
    pub fn inactive(&self) -> Pair<f32> {
        self.map(|size| size / 2.0)
    }
}

/// 歌词展示设置
/// 阴影设置。
///
/// 简化为 **开关 + 颜色 + 模糊 + X/Y 偏移**：
/// 去掉了 `inset`（内阴影在歌词上几乎用不到）和"保存按钮"（改为即时生效），
/// 偏移也从"CSS 字符串"改成两个数值，UI 才能用滑块/数字框调。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ShadowSettings {
    /// 是否启用阴影（UI 上是"关闭阴影"的反面）
    pub enable: bool,
    /// 阴影颜色，`#rrggbb` / `#rrggbbaa`
    pub color: String,
    /// 模糊半径（px）
    pub blur: f32,
    /// X 偏移（px）
    pub offset_x: f32,
    /// Y 偏移（px）
    pub offset_y: f32,
}

impl Default for ShadowSettings {
    /// 默认**开启**阴影，颜色纯黑。
    ///
    /// 这四处必须保持一致，否则会出现"新装开启、恢复默认后又关掉"这类矛盾：
    /// 1. 这里（后端默认值）
    /// 2. 前端 `DEFAULT_SHADOW`（stores/settingsStore.ts）
    /// 3. 「恢复默认样式」按钮（它直接用前端那份常量）
    /// 4. 展示端 `LyricsBox` 的 filter 生成（由 `enable` 驱动）
    fn default() -> Self {
        Self {
            enable: true,
            color: "#000000".to_string(),
            blur: 3.0,
            offset_x: 2.0,
            offset_y: 2.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LyricSettings {
    /// 主 / 副歌词颜色，`#rrggbb`
    pub text_color: Pair<String>,
    /// 主 / 副歌词字号（em，指 active 时的字号）
    pub font_size: Pair<f32>,
    /// 主 / 副歌词字体名，空串表示用默认字体
    pub font: Pair<String>,
    /// 对齐方式（整体，不支持主副分别设置）
    pub alignment: String,
    /// 是否以翻译为主
    pub translation_main: bool,
    /// 是否显示副歌词
    pub second_show: bool,
    /// 主 / 副歌词阴影
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
            shadow: Pair::same(ShadowSettings::default()),
        }
    }
}

/// 局部更新载荷：只处理出现过的字段，未出现的保持原值
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LyricSettingsPatch {
    pub text_color: Option<Pair<String>>,
    pub font_size: Option<Pair<f32>>,
    pub font: Option<Pair<String>>,
    pub alignment: Option<String>,
    pub translation_main: Option<bool>,
    pub second_show: Option<bool>,
    pub shadow: Option<Pair<ShadowSettings>>,
}

/// 会触发 WS 广播的设置项
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingKey {
    TextColor,
    FontSize,
    Font,
    Alignment,
    TranslationMain,
    SecondShow,
    Shadow,
}

impl SettingKey {
    /// 展示端消费的 WS 广播 key（与前端 `handleSettingBroadcast` 一致）
    pub fn ws_key(&self) -> &'static str {
        match self {
            SettingKey::TextColor => "setColor",
            SettingKey::FontSize => "setFontSize",
            SettingKey::Font => "setFont",
            SettingKey::Alignment => "setAlignment",
            SettingKey::TranslationMain => "setTranslationMain",
            SettingKey::SecondShow => "setSecondShow",
            SettingKey::Shadow => "setShadow",
        }
    }
}

/// 所有设置项，按展示端关心的顺序
pub const ALL_SETTING_KEYS: [SettingKey; 7] = [
    SettingKey::TextColor,
    SettingKey::FontSize,
    SettingKey::Font,
    SettingKey::Alignment,
    SettingKey::TranslationMain,
    SettingKey::SecondShow,
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

/// 阴影数值范围。
///
/// 按"相对字号的一个合理比例"来定：字号范围是 0.5~12em，
/// 阴影偏移 ±20px、模糊 0~30px 足够覆盖正常观感，又不至于让用户把
/// 歌词推到看不见的地方。
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

impl LyricSettings {
    /// 从数据库读取；没有记录时返回默认值
    pub async fn load() -> Self {
        let Some(raw) = SettingEntity::get_config(SETTINGS_DB_KEY)
            .await
            .ok()
            .flatten()
        else {
            return Self::default();
        };
        match crate::util::to_json::<Self>(&raw) {
            Ok(settings) => settings,
            Err(err) => {
                tracing::error!("设置解析失败，回退默认值: {err}");
                Self::default()
            }
        }
    }

    /// 写入数据库；失败会把错误原样返回（调用方**不得**在失败时广播）
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
        if !ALIGNMENTS.contains(&self.alignment.as_str()) {
            return Err(invalid(format!(
                "对齐方式必须是 {ALIGNMENTS:?} 之一: {}",
                self.alignment
            )));
        }
        Ok(())
    }

    /// 应用局部更新并校验；不落库、不改内存
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
        if let Some(value) = patch.shadow {
            next.shadow = value;
        }
        next.validate()?;
        Ok(next)
    }

    /// 相对 `before` 发生了变化的设置项
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
        if self.shadow != before.shadow {
            changed.push(SettingKey::Shadow);
        }
        changed
    }

    /// 单个设置项对应的 WS 广播值
    pub fn key_value(&self, key: SettingKey) -> serde_json::Value {
        match key {
            SettingKey::TextColor => serde_json::json!(self.text_color),
            SettingKey::FontSize => serde_json::json!(self.font_size),
            SettingKey::Font => serde_json::json!(self.font),
            // 对齐只有单一取值，主副填同一个，兼容前端读 first ?? second
            SettingKey::Alignment => serde_json::json!(Pair::same(&self.alignment)),
            SettingKey::TranslationMain => serde_json::json!(self.translation_main),
            SettingKey::SecondShow => serde_json::json!(self.second_show),
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
    }

    #[test]
    fn shadow_defaults() {
        let d = ShadowSettings::default();
        // 主 / 副默认都开启，且默认黑色 —— 与前端 DEFAULT_SHADOW 必须一致
        assert!(d.enable, "默认应开启阴影");
        assert_eq!(d.color, "#000000");
        assert_eq!(d.blur, 3.0);
        assert_eq!(d.offset_x, 2.0);
        assert_eq!(d.offset_y, 2.0);
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
    }

    /// 旧数据只写了部分字段时, 缺失字段必须回落到默认值而不是解析失败
    #[test]
    fn tolerates_partial_legacy_json() {
        let settings: LyricSettings =
            serde_json::from_str(r#"{"alignment":"right"}"#).expect("部分字段应能解析");
        assert_eq!(settings.alignment, "right");
        assert_eq!(settings.font_size.first, 3.0);
        assert!(settings.second_show);
    }

    #[test]
    fn inactive_font_size_keeps_half() {
        let sizes = Pair::new(4.0, 3.0);
        let inactive = sizes.inactive();
        assert_eq!((inactive.first, inactive.second), (2.0, 1.5));
    }
}
