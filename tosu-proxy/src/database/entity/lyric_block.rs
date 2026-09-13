//! 黑名单规则表（B-04）。
//!
//! 取代旧 `lyric_config` 里 `disable` 与 `offset` 混在一行、且读取时
//! 按 `bid → title → sid` 逐级回退的做法（R7）：
//! 一条规则**只属于一个明确的作用域**，不会再因为标题同名或同谱面集而误伤。
//!
//! 作用域（B-00 契约）：
//! - `bid`   —— 只屏蔽这一张谱面（最精确，优先）
//! - `sid`   —— 屏蔽整个谱面集
//! - `title` —— 屏蔽某个标题（用户显式创建的规则，跨 bid/sid 生效）
//!
//! 同一 `(scope, value)` 只会有一行，重复添加是幂等的。

use crate::database::database;
use sea_orm::entity::prelude::*;
use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveValue, ColumnTrait, QueryFilter, QueryOrder};

// 注意: 不能 `use crate::error::Result` —— 会遮住 prelude 里 derive 宏依赖的
// `Result` 别名, 因此这里全部用 `crate::error::Result` 全限定书写。
type Res<T> = crate::error::Result<T>;

/// 规则作用域
pub const SCOPE_BID: &str = "bid";
pub const SCOPE_SID: &str = "sid";
pub const SCOPE_TITLE: &str = "title";

pub fn is_valid_scope(scope: &str) -> bool {
    matches!(scope, SCOPE_BID | SCOPE_SID | SCOPE_TITLE)
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "lyric_block")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    /// `bid` / `sid` / `title`
    #[sea_orm(indexed)]
    pub scope: String,
    /// 作用域的取值：bid / sid 存数字字符串，title 存标题本身
    #[sea_orm(indexed)]
    pub value: String,
    /// 便于展示的标题（bid/sid 规则也记下当时的标题）
    #[sea_orm(default_value = "")]
    pub title: String,
    /// 创建规则时的谱面集 id（仅用于展示与旧 WS 协议兼容，不参与匹配）
    #[sea_orm(default_value = 0)]
    pub sid: i32,
    /// 备注（B-10）。旧 UI 一直有这个输入框，但从来没被持久化过；
    /// 这里补上真正的存储，而不是让它继续当一次性本地状态。
    #[sea_orm(default_value = "")]
    pub reason: String,
    /// 创建时间（毫秒时间戳）
    #[sea_orm(default_value = 0)]
    pub created_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Entity {
    pub async fn list_all() -> Res<Vec<Model>> {
        Ok(Self::find()
            .order_by_asc(Column::Id)
            .all(database())
            .await?)
    }

    /// 注意：不能叫 `find_by_id` —— 会盖住 `EntityTrait::find_by_id` 造成自递归
    pub async fn get_by_id(id: i32) -> Res<Option<Model>> {
        Ok(Self::find_by_id(id).one(database()).await?)
    }

    /// 幂等新增：同一 (scope, value) 已存在时只更新标题，不报错也不重复插入
    pub async fn upsert(
        scope: &str,
        value: &str,
        title: &str,
        sid: i32,
        reason: &str,
    ) -> Res<Model> {
        if !is_valid_scope(scope) {
            return Err(crate::error::Error::Runtime(format!("未知的屏蔽作用域: {scope}")));
        }
        if value.trim().is_empty() {
            return Err(crate::error::Error::Runtime("屏蔽作用域取值不能为空".into()));
        }

        let now = sea_orm::sqlx::types::chrono::Utc::now().timestamp_millis();
        let model = ActiveModel {
            scope: ActiveValue::Set(scope.to_string()),
            value: ActiveValue::Set(value.to_string()),
            title: ActiveValue::Set(title.to_string()),
            sid: ActiveValue::Set(sid),
            reason: ActiveValue::Set(reason.to_string()),
            created_at: ActiveValue::Set(now),
            ..Default::default()
        };

        let mut on_conflict = OnConflict::columns([Column::Scope, Column::Value]);
        on_conflict
            .update_column(Column::Title)
            .update_column(Column::Sid)
            .update_column(Column::Reason);
        Self::insert(model)
            .on_conflict(on_conflict)
            .exec(database())
            .await?;

        Self::find()
            .filter(Column::Scope.eq(scope))
            .filter(Column::Value.eq(value))
            .one(database())
            .await?
            .ok_or(crate::error::Error::Impossible)
    }

    /// 更新展示用元数据（标题 / 备注）。
    ///
    /// **不改作用域与取值**：规则身份不变，因此不需要重算当前歌曲是否被屏蔽。
    pub async fn update_meta(id: i32, title: Option<&str>, reason: Option<&str>) -> Res<bool> {
        let mut update = Self::update_many();
        if let Some(title) = title {
            update = update.col_expr(
                Column::Title,
                sea_orm::sea_query::Expr::value(title.to_string()),
            );
        }
        if let Some(reason) = reason {
            update = update.col_expr(
                Column::Reason,
                sea_orm::sea_query::Expr::value(reason.to_string()),
            );
        }
        let result = update.filter(Column::Id.eq(id)).exec(database()).await?;
        Ok(result.rows_affected > 0)
    }

    /// 同上：避开 `EntityTrait::delete_by_id`
    pub async fn remove_by_id(id: i32) -> Res<bool> {
        let result = <Self as EntityTrait>::delete_by_id(id)
            .exec(database())
            .await?;
        Ok(result.rows_affected > 0)
    }

    /// 按作用域取值删除（幂等）。只影响这一个作用域，不动其它规则。
    pub async fn delete_by_scope_value(scope: &str, value: &str) -> Res<u64> {
        let result = Self::delete_many()
            .filter(Column::Scope.eq(scope))
            .filter(Column::Value.eq(value))
            .exec(database())
            .await?;
        Ok(result.rows_affected)
    }

    pub async fn delete_all() -> Res<u64> {
        let result = Self::delete_many().exec(database()).await?;
        Ok(result.rows_affected)
    }

    pub async fn count() -> Res<u64> {
        Ok(Self::find().count(database()).await?)
    }

    /// 判断某首歌是否被屏蔽。
    ///
    /// **不做跨作用域回退**：`bid` 命中就是命中，`title` 规则只在用户显式创建过
    /// 同名标题规则时才生效，不会因为"标题恰好相同"而命中别的谱面。
    pub async fn is_blocked(bid: i64, sid: i64, title: &str) -> Res<Option<Model>> {
        let all = Self::list_all().await?;
        let bid_s = bid.to_string();
        let sid_s = sid.to_string();

        // 优先级：bid > sid > title（仅用于返回哪条规则命中，不改变命中判定）
        if let Some(m) = all
            .iter()
            .find(|m| m.scope == SCOPE_BID && m.value == bid_s)
        {
            return Ok(Some(m.clone()));
        }
        if let Some(m) = all
            .iter()
            .find(|m| m.scope == SCOPE_SID && m.value == sid_s)
        {
            return Ok(Some(m.clone()));
        }
        let title_hit = if title.is_empty() {
            None
        } else {
            all.iter()
                .find(|m| m.scope == SCOPE_TITLE && m.value == title)
        };
        if let Some(m) = title_hit {
            return Ok(Some(m.clone()));
        }
        Ok(None)
    }

    /// 旧数据迁移：把 `lyric_config.disable = true` 的行转成 `bid` 作用域规则。
    /// 幂等，可重复调用。
    pub async fn migrate_from_legacy() -> Res<u64> {
        use crate::database::LyricConfigEntity;
        let legacy = LyricConfigEntity::find()
            .filter(crate::database::entity::lyric_config::Column::Disable.eq(true))
            .all(database())
            .await?;

        let mut migrated = 0;
        for row in legacy {
            let value = row.bid.to_string();
            let exists = Self::find()
                .filter(Column::Scope.eq(SCOPE_BID))
                .filter(Column::Value.eq(&value))
                .one(database())
                .await?;
            if exists.is_none() {
                // 旧表没有 reason，迁移时一律为空备注
                Self::upsert(SCOPE_BID, &value, &row.title, row.sid, "").await?;
                migrated += 1;
            }
        }
        Ok(migrated)
    }
}
