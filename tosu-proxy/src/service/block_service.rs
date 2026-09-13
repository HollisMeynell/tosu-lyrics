//! 黑名单服务（B-04）。
//!
//! 后端是黑名单的**唯一真相来源**：Controller 只能通过 HTTP 读写，
//! 本地不再维护一份可能和后端不一致的副本。
//!
//! 作用域规则（B-00 契约）见 `database::entity::lyric_block`：
//! 一条规则只属于 `bid` / `sid` / `title` 三者之一，不做跨作用域回退，
//! 因此「标题恰好相同的另一首歌」不会被误伤。
//!
//! 副作用（本轮要求）：
//! - 拉黑**当前正在播放**的歌   -> 立即清屏（所有展示端），不等切歌
//! - 解除当前正在播放的歌的黑名单 -> 立即重新加载歌词
//! - 两者都幂等：重复添加 / 重复删除不会重复触发

use crate::database::lyric_block::{is_valid_scope, Model};
use crate::database::{LyricBlockEntity, SCOPE_BID, SCOPE_SID, SCOPE_TITLE};
use crate::error::Error;
use crate::service::lyric_service::{lyric_service, LyricService};
use serde::{Deserialize, Serialize};

pub type Result<T> = std::result::Result<T, BlockError>;

/// 黑名单服务的错误分类。
///
/// 用独立枚举而不是复用 `Error` + 字符串前缀，是为了让 HTTP 层能精确映射
/// 到错误码与状态码，而不是靠解析错误文本猜。
#[derive(Debug)]
pub enum BlockError {
    /// 参数非法（作用域未知 / 取值不是正整数）
    InvalidParam(String),
    /// 目标资源不存在
    NotFound(String),
    /// 其它内部错误（数据库等）
    Internal(Error),
}

impl std::fmt::Display for BlockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidParam(m) | Self::NotFound(m) => write!(f, "{m}"),
            Self::Internal(e) => write!(f, "{e}"),
        }
    }
}

impl From<Error> for BlockError {
    fn from(e: Error) -> Self {
        Self::Internal(e)
    }
}

/// HTTP 黑名单规则（唯一对外形态）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BlockRule {
    pub id: i32,
    /// `bid` / `sid` / `title`
    pub scope: String,
    /// 作用域取值
    pub value: String,
    pub title: String,
    pub sid: i32,
    /// 备注（B-10）
    pub reason: String,
    pub created_at: i64,
}

impl From<Model> for BlockRule {
    fn from(m: Model) -> Self {
        Self {
            id: m.id,
            scope: m.scope,
            value: m.value,
            title: m.title,
            sid: m.sid,
            reason: m.reason,
            created_at: m.created_at,
        }
    }
}

/// 新增规则的请求体
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockRuleInput {
    pub scope: String,
    pub value: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub sid: i32,
    #[serde(default)]
    pub reason: String,
}

/// 修改规则的请求体（只允许改展示用标题）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockRulePatch {
    pub title: Option<String>,
    /// 备注（B-10）：只是元数据，改了不影响命中判定
    pub reason: Option<String>,
}

pub async fn list() -> Result<Vec<BlockRule>> {
    Ok(LyricBlockEntity::list_all()
        .await?
        .into_iter()
        .map(BlockRule::from)
        .collect())
}

pub async fn count() -> Result<u64> {
    Ok(LyricBlockEntity::count().await?)
}

/// 新增（幂等）。同一 `(scope, value)` 重复添加只会更新标题。
pub async fn add(input: BlockRuleInput) -> Result<BlockRule> {
    validate(&input.scope, &input.value)?;
    let model = LyricBlockEntity::upsert(
        &input.scope,
        &input.value,
        &input.title,
        input.sid,
        &input.reason,
    )
    .await?;
    let rule = BlockRule::from(model);

    // 若被拉黑的正是当前播放的歌，立即清屏
    apply_current_song_effects().await;
    Ok(rule)
}

pub async fn update(id: i32, patch: BlockRulePatch) -> Result<BlockRule> {
    let Some(existing) = LyricBlockEntity::get_by_id(id).await? else {
        return Err(BlockError::NotFound(format!("黑名单规则 {id} 不存在")));
    };
    // 标题与备注都只是展示字段，不影响命中判定，因此不需要重算当前歌曲状态
    if patch.title.is_some() || patch.reason.is_some() {
        LyricBlockEntity::update_meta(id, patch.title.as_deref(), patch.reason.as_deref()).await?;
    }
    let title = patch.title.unwrap_or(existing.title);
    let reason = patch.reason.unwrap_or(existing.reason);
    Ok(BlockRule::from(Model {
        title,
        reason,
        ..existing
    }))
}

/// 删除（幂等）。规则本来就不存在时返回 `Ok(None)`，由 HTTP 层决定是否 404。
pub async fn delete(id: i32) -> Result<Option<BlockRule>> {
    let existing = LyricBlockEntity::get_by_id(id).await?;
    LyricBlockEntity::remove_by_id(id).await?;
    if existing.is_some() {
        // 解除屏蔽后当前歌可能要恢复展示
        apply_current_song_effects().await;
    }
    Ok(existing.map(BlockRule::from))
}

/// 清空（幂等），返回删除条数
pub async fn clear_all() -> Result<u64> {
    let removed = LyricBlockEntity::delete_all().await?;
    if removed > 0 {
        apply_current_song_effects().await;
    }
    Ok(removed)
}

/// 当前歌曲是否被屏蔽（返回命中的规则）
pub async fn blocked_rule(bid: i32, sid: i32, title: &str) -> Result<Option<BlockRule>> {
    Ok(LyricBlockEntity::is_blocked(bid as i64, sid as i64, title)
        .await?
        .map(BlockRule::from))
}

fn validate(scope: &str, value: &str) -> Result<()> {
    if !is_valid_scope(scope) {
        return Err(BlockError::InvalidParam(format!(
            "scope 只能是 bid / sid / title, 收到 {scope}"
        )));
    }
    let value = value.trim();
    if value.is_empty() {
        return Err(BlockError::InvalidParam("value 不能为空".into()));
    }
    let numeric_scope = scope == SCOPE_BID || scope == SCOPE_SID;
    if numeric_scope && value.parse::<i64>().map(|v| v <= 0).unwrap_or(true) {
        return Err(BlockError::InvalidParam(format!(
            "{scope} 的 value 必须是正整数, 收到 {value}"
        )));
    }
    Ok(())
}

/// 黑名单变化后，按"当前播放的歌现在应该是什么状态"同步展示端。
///
/// - 现在被屏蔽 -> 清屏
/// - 现在未被屏蔽 -> 重新加载（等价于重新进入这首歌）
///
/// 不依赖调用方传参，直接读当前歌曲身份，因此对「清空黑名单」这类批量操作也成立。
async fn apply_current_song_effects() {
    let Some(ident) = LyricService::now_ident().await else {
        return; // 菜单 / 无歌, 无需处理
    };

    let blocked = LyricBlockEntity::is_blocked(ident.bid as i64, ident.sid as i64, &ident.title)
        .await
        .ok()
        .flatten()
        .is_some();

    if blocked {
        let mut service = lyric_service().await;
        service.clear_display().await;
    } else {
        // 解除屏蔽: 重新载入当前歌的歌词
        LyricService::reload_current().await;
    }
}

/// 默认作用域：从当前歌曲信息推断（UI 未显式指定 scope 时使用）
pub fn default_scope(scope: Option<&str>) -> &'static str {
    match scope.unwrap_or(SCOPE_BID) {
        SCOPE_SID => SCOPE_SID,
        SCOPE_TITLE => SCOPE_TITLE,
        _ => SCOPE_BID,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn scope_validation() {
        assert!(validate(SCOPE_BID, "123").is_ok());
        assert!(validate(SCOPE_SID, "456").is_ok());
        assert!(validate(SCOPE_TITLE, "Lemon").is_ok());

        assert!(validate("playlist", "1").is_err(), "未定义的作用域必须拒绝");
        assert!(validate(SCOPE_BID, "").is_err());
        assert!(validate(SCOPE_BID, "abc").is_err(), "bid 必须是数字");
        assert!(validate(SCOPE_BID, "-1").is_err(), "bid 必须为正");
    }

    #[test]
    fn default_scope_falls_back_to_bid() {
        assert_eq!(default_scope(None), SCOPE_BID);
        assert_eq!(default_scope(Some("sid")), SCOPE_SID);
        assert_eq!(default_scope(Some("title")), SCOPE_TITLE);
        // 未知作用域不作为默认值透传
        assert_eq!(default_scope(Some("nonsense")), SCOPE_BID);
    }

    #[test]
    fn rule_from_model_maps_all_fields() {
        let rule = BlockRule::from(Model {
            id: 7,
            scope: SCOPE_BID.into(),
            value: "3344501".into(),
            title: "Lemon".into(),
            sid: 900001,
            reason: "太吵".into(),
            created_at: 1234567,
        });
        assert_eq!(rule.id, 7);
        assert_eq!(rule.scope, "bid");
        assert_eq!(rule.value, "3344501");
        assert_eq!(rule.sid, 900001);
        assert_eq!(rule.reason, "太吵");
    }
}
