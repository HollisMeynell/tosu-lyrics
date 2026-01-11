use crate::database::database;
use crate::database::entity::DB_ERROR_MESSAGE;
use sea_orm::entity::prelude::*;
use sea_orm::sea_query::OnConflict;
use sea_orm::{ColumnTrait, QueryFilter};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "lyric_config")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub bid: i32,
    #[sea_orm(indexed)]
    pub sid: i32,
    #[sea_orm(indexed)]
    pub title: String,
    pub disable: bool,
    pub offset: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Entity {
    pub async fn get_by_bid(bid: i32) -> crate::error::Result<Option<(bool, i32)>> {
        let db = database();
        // 优先根据 bid 查询
        if let Some(model) = Self::find_by_id(bid).one(db).await? {
            Ok(Some((model.disable, model.offset)))
        } else {
            Ok(None)
        }
    }
    async fn find_first(bid: i32, sid: i32, title: &str) -> crate::error::Result<Option<Model>> {
        let db = database();
        // 优先根据 bid 查询
        if let Some(model) = Self::find_by_id(bid).one(db).await? {
            return Ok(Some(model));
        }

        // bid 未找到, 再找 title
        if let Some(model) = Self::find()
            .filter(Column::Title.eq(title))
            .one(db)
            .await?
        {
            return Ok(Some(model));
        }

        // 都未找到，根据 sid 查询第一个
        if let Some(model) = Self::find()
            .filter(Column::Sid.eq(sid))
            .one(db)
            .await?
        {
            return Ok(Some(model));
        }
        Ok(None)
    }
    pub async fn find_setting(bid: i32, sid: i32, title: &str) -> crate::error::Result<(bool, i32)> {
        if let Some(m) = Entity::find_first(bid, sid, title).await? {
            Ok((m.disable, m.offset))
        } else {
            // 都没找到，返回默认值 0
            Ok((false, 0))
        }
    }

    pub async fn save_setting(bid: i32, sid: i32, title: &str, block: bool, offset: i32) -> crate::error::Result<()> {
        let model = ActiveModel::from(Model {
            bid,
            sid,
            title: title.to_string(),
            disable: block,
            offset,
        });

        let mut on_conflict = OnConflict::column(Column::Bid);
        on_conflict
            .update_column(Column::Sid)
            .update_column(Column::Title)
            .update_column(Column::Disable)
            .update_column(Column::Offset);

        Self::insert(model)
            .on_conflict(on_conflict)
            .exec(database())
            .await?;
        Ok(())
    }

    pub async fn delete_by_bid(bid: i32) -> crate::error::Result<()> {
        Self::delete_by_id(bid)
            .exec(database())
            .await?;
        Ok(())
    }

    pub async fn get_all_disable() -> crate::error::Result<Vec<(i32, i32, String)>> {
        Ok(Self::find()
            .filter(Column::Disable.eq(true))
            .all(database())
            .await?
            .into_iter()
            .map(|m| (m.bid, m.sid, m.title))
            .collect())
    }
}
