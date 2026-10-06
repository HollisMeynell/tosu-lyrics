use crate::database::lyric_block::{is_valid_scope, Model};
use crate::database::{LyricBlockEntity, SCOPE_BID, SCOPE_SID, SCOPE_TITLE};
use crate::error::Error;
use crate::service::lyric_service::{lyric_service, LyricService};
use serde::{Deserialize, Serialize};

pub type Result<T> = std::result::Result<T, BlockError>;

#[derive(Debug)]
pub enum BlockError {
    InvalidParam(String),
    NotFound(String),
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BlockRule {
    pub id: i32,
    pub scope: String,
    pub value: String,
    pub title: String,
    pub sid: i32,
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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockRulePatch {
    pub title: Option<String>,
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

/// 幂等：同一 (scope, value) 重复添加只会更新标题
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

    apply_current_song_effects().await;
    Ok(rule)
}

pub async fn update(id: i32, patch: BlockRulePatch) -> Result<BlockRule> {
    let Some(existing) = LyricBlockEntity::get_by_id(id).await? else {
        return Err(BlockError::NotFound(format!("黑名单规则 {id} 不存在")));
    };
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

/// 幂等删除。规则不存在时返回 Ok(None)，由 HTTP 层决定是否 404。
pub async fn delete(id: i32) -> Result<Option<BlockRule>> {
    let existing = LyricBlockEntity::get_by_id(id).await?;
    LyricBlockEntity::remove_by_id(id).await?;
    if existing.is_some() {
        apply_current_song_effects().await;
    }
    Ok(existing.map(BlockRule::from))
}

pub async fn clear_all() -> Result<u64> {
    let removed = LyricBlockEntity::delete_all().await?;
    if removed > 0 {
        apply_current_song_effects().await;
    }
    Ok(removed)
}

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
        crate::service::LYRIC_SERVICE.call(move |service| Box::pin(async move {
            if service.get_now_song().is_some_and(|song| song.bid as i32 == ident.bid) {
                service.clear_display().await;
            }
        })).await;
    } else {
        // 解除屏蔽: 重新载入当前歌的歌词
        LyricService::reload_current().await;
    }
}

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
