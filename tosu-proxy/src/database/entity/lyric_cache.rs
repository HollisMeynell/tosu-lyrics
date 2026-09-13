use crate::database::database;
use crate::database::entity::DB_ERROR_MESSAGE;
use crate::lyric::Lyric;
use sea_orm::ActiveValue;
use sea_orm::entity::prelude::*;
use sea_orm::sea_query::OnConflict;
use sea_orm::{PaginatorTrait, QueryOrder, QuerySelect};

/// 当前时间（毫秒时间戳）
pub fn now_ms() -> i64 {
    sea_orm::sqlx::types::chrono::Utc::now().timestamp_millis()
}

/// 条目是否仍在 TTL 内。`ttl_ms <= 0` 表示不启用 TTL。
///
/// 迁移前写入的旧行 `updated_at == 0`，迁移时会回填，因此这里不做特判。
pub fn is_fresh(model: &Model, ttl_ms: i64) -> bool {
    ttl_ms <= 0 || now_ms().saturating_sub(model.updated_at) < ttl_ms
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "lyric_cache")]
pub struct Model {
    pub sid: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub bid: i32,
    #[sea_orm(column_type = "Blob")]
    pub cache: Vec<u8>,
    pub title: String,
    /// ms
    pub audio_length: i32,
    /// 写入 / 刷新时间（毫秒时间戳）。用于 TTL 过期判定。
    /// 旧库没有这一列，迁移时补上并回填为迁移时刻。
    #[sea_orm(default_value = 0)]
    pub updated_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Entity {
    /// 按 bid 精确命中，**且未过期**。`ttl_ms <= 0` 表示不启用 TTL。
    pub async fn find_by_bid(bid: i32, ttl_ms: i64) -> crate::error::Result<Option<Model>> {
        Ok(Self::find_by_id(bid)
            .one(database())
            .await?
            .filter(|m| is_fresh(m, ttl_ms)))
    }

    /// 按 sid 回退命中（歌曲级归属），**且未过期**。
    pub async fn find_by_sid(sid: i32, ttl_ms: i64) -> crate::error::Result<Option<Model>> {
        Ok(Self::find()
            .filter(Column::Sid.eq(sid))
            .order_by_desc(Column::UpdatedAt)
            .one(database())
            .await?
            .filter(|m| is_fresh(m, ttl_ms)))
    }

    pub async fn find_by_title_like(title: &str) -> crate::error::Result<Vec<Model>> {
        Ok(Self::find()
            .filter(Column::Title.contains(title))
            .order_by_desc(Column::UpdatedAt)
            .all(database())
            .await?)
    }

    /// 分页列表。`query` 非空时按标题模糊过滤。
    pub async fn page(query: Option<&str>, offset: u64, limit: u64) -> crate::error::Result<Vec<Model>> {
        let mut select = Self::find();
        if let Some(q) = query.filter(|q| !q.is_empty()) {
            select = select.filter(Column::Title.contains(q));
        }
        Ok(select
            .order_by_desc(Column::UpdatedAt)
            .offset(offset)
            .limit(limit)
            .all(database())
            .await?)
    }

    pub async fn count_filtered(query: Option<&str>) -> crate::error::Result<u64> {
        let mut select = Self::find();
        if let Some(q) = query.filter(|q| !q.is_empty()) {
            select = select.filter(Column::Title.contains(q));
        }
        Ok(select.count(database()).await?)
    }

    pub async fn all_count() -> crate::error::Result<u64> {
        Ok(Self::find().count(database()).await?)
    }

    /// 删除单条。**必须 await** —— R9 的原缺陷就是构造了 statement 却没执行。
    pub async fn delete_by_bid(bid: i32) -> crate::error::Result<u64> {
        let result = <Self as EntityTrait>::delete_by_id(bid)
            .exec(database())
            .await?;
        Ok(result.rows_affected)
    }

    /// 按标题（模糊）删除，返回删除条数
    pub async fn delete_by_title_like(title: &str) -> crate::error::Result<u64> {
        let result = Self::delete_many()
            .filter(Column::Title.contains(title))
            .exec(database())
            .await?;
        Ok(result.rows_affected)
    }

    /// 删除全部，返回删除条数
    pub async fn delete_all() -> crate::error::Result<u64> {
        let result = Self::delete_many().exec(database()).await?;
        Ok(result.rows_affected)
    }

    /// 清理过期条目，返回删除条数。`ttl_ms <= 0` 时什么都不做。
    pub async fn purge_expired(ttl_ms: i64) -> crate::error::Result<u64> {
        if ttl_ms <= 0 {
            return Ok(0);
        }
        let cutoff = now_ms().saturating_sub(ttl_ms);
        let result = Self::delete_many()
            .filter(Column::UpdatedAt.lt(cutoff))
            .exec(database())
            .await?;
        Ok(result.rows_affected)
    }

    /// - `sid`：sid
    /// - `bid`：bid
    /// - `title`：title
    /// - `audio_length`：毫秒
    /// - `lyric`：歌词
    pub async fn save(
        sid: i32,
        bid: i32,
        title: &str,
        audio_length: i32,
        lyric: &Lyric,
    ) -> crate::error::Result<()> {
        let model = ActiveModel {
            sid: ActiveValue::Set(sid),
            bid: ActiveValue::Set(bid),
            title: ActiveValue::Set(title.to_string()),
            audio_length: ActiveValue::Set(audio_length),
            cache: ActiveValue::Set(lyric.to_json_cache()?),
            updated_at: ActiveValue::Set(now_ms()),
        };

        Self::save_model(model).await
    }

    pub async fn save_model(model: ActiveModel) -> crate::error::Result<()> {
        // 如果存在相同的 bid，则更新记录
        let mut on_conflict = OnConflict::column(Column::Bid);
        on_conflict
            .update_column(Column::Sid)
            .update_column(Column::Cache)
            .update_column(Column::Title)
            .update_column(Column::AudioLength)
            .update_column(Column::UpdatedAt);

        Self::insert(model)
            .on_conflict(on_conflict)
            .exec(database())
            .await?;
        Ok(())
    }
}

impl TryInto<Lyric> for &Model {
    type Error = crate::error::Error;

    fn try_into(self) -> crate::error::Result<Lyric> {
        Lyric::from_json_cache(self.cache.as_slice())
    }
}
