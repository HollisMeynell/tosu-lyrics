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
    // 兼容旧库；黑名单迁移后不再参与匹配。
    pub disable: bool,
    pub offset: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Entity {
    pub async fn get_offset(bid: i32) -> crate::error::Result<i32> {
        Ok(Self::find_by_id(bid)
            .one(database())
            .await?
            .map(|m| m.offset)
            .unwrap_or(0))
    }

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

    pub async fn get_by_bid(bid: i32) -> crate::error::Result<Option<i32>> {
        Ok(Self::find_by_id(bid)
            .one(database())
            .await?
            .map(|m| m.offset))
    }

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

    pub async fn clear_legacy_disable(bid: i32) -> crate::error::Result<()> {
        use sea_orm::{ColumnTrait, QueryFilter};
        Self::update_many()
            .col_expr(Column::Disable, sea_orm::sea_query::Expr::value(false))
            .filter(Column::Bid.eq(bid))
            .exec(database())
            .await?;
        Ok(())
    }

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
