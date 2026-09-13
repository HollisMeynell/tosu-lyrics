//! 歌曲级偏移表（B-00 契约 · 收敛后）。
//!
//! **只存 offset**。黑名单已迁到 `lyric_block`（见 `lyric_block.rs`）。
//!
//! 收敛掉的行为（R7）：旧版 `find_first` 会按 `bid → title → sid` 逐级回退，
//! 于是一首**只是标题相同**的歌会继承别人的屏蔽与偏移。
//! 现在偏移严格按 `bid` 精确匹配，不跨作用域回退。
//!
//! `disable` 列保留仅为兼容旧库结构（迁移后恒为 false），不再参与任何判断。

use crate::database::database;
use sea_orm::entity::prelude::*;
use sea_orm::sea_query::OnConflict;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "lyric_config")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub bid: i32,
    #[sea_orm(indexed)]
    pub sid: i32,
    #[sea_orm(indexed)]
    pub title: String,
    /// 已废弃：黑名单不再使用此列，仅保留列以兼容旧库
    pub disable: bool,
    pub offset: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Entity {
    /// 读取某首歌的偏移。**严格按 bid 精确匹配**，找不到就是 0。
    pub async fn get_offset(bid: i32) -> crate::error::Result<i32> {
        Ok(Self::find_by_id(bid)
            .one(database())
            .await?
            .map(|m| m.offset)
            .unwrap_or(0))
    }

    /// 写入偏移。`offset == 0` 视为"无自定义偏移"，直接删除该行。
    ///
    /// 全程不触碰黑名单：黑名单在 `lyric_block`，两者互不影响。
    pub async fn save_offset(
        bid: i32,
        sid: i32,
        title: &str,
        offset: i32,
    ) -> crate::error::Result<()> {
        if offset == 0 {
            return Self::delete_by_bid(bid).await;
        }

        let model = ActiveModel::from(Model {
            bid,
            sid,
            title: title.to_string(),
            disable: false,
            offset,
        });

        let mut on_conflict = OnConflict::column(Column::Bid);
        on_conflict
            .update_column(Column::Sid)
            .update_column(Column::Title)
            .update_column(Column::Offset);

        Self::insert(model)
            .on_conflict(on_conflict)
            .exec(database())
            .await?;
        Ok(())
    }

    pub async fn delete_by_bid(bid: i32) -> crate::error::Result<()> {
        Self::delete_by_id(bid).exec(database()).await?;
        Ok(())
    }

    /// 兼容旧调用点的只读访问（仅取 offset，忽略 disable）
    pub async fn get_by_bid(bid: i32) -> crate::error::Result<Option<i32>> {
        Ok(Self::find_by_id(bid).one(database()).await?.map(|m| m.offset))
    }

    /// 迁移辅助：取出所有 `disable = true` 的旧行（黑名单迁往 `lyric_block`）
    pub async fn legacy_disabled() -> crate::error::Result<Vec<(i32, i32, String)>> {
        use sea_orm::{ColumnTrait, QueryFilter};
        Ok(Self::find()
            .filter(Column::Disable.eq(true))
            .all(database())
            .await?
            .into_iter()
            .map(|m| (m.bid, m.sid, m.title))
            .collect())
    }

    /// 迁移收尾：清掉 `disable` 标志；若该行也没有偏移则整行删除。
    pub async fn clear_legacy_disable(bid: i32) -> crate::error::Result<()> {
        use sea_orm::{ColumnTrait, QueryFilter};
        Self::update_many()
            .col_expr(Column::Disable, sea_orm::sea_query::Expr::value(false))
            .filter(Column::Bid.eq(bid))
            .exec(database())
            .await?;
        Ok(())
    }

    /// 删除所有既无偏移也无用的空行（迁移收尾用，幂等）
    pub async fn prune_empty() -> crate::error::Result<u64> {
        use sea_orm::{ColumnTrait, QueryFilter};
        let result = Self::delete_many()
            .filter(Column::Offset.eq(0))
            .filter(Column::Disable.eq(false))
            .exec(database())
            .await?;
        Ok(result.rows_affected)
    }
}
