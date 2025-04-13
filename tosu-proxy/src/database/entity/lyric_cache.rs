use crate::database::database;
use sea_orm::ActiveValue;
use sea_orm::entity::prelude::*;
use sea_orm::sea_query::OnConflict;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "lyric_cache")]
pub struct Model {
    pub sid: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub bid: i32,
    #[sea_orm(column_type = "Blob", nullable)]
    pub cache: Option<Vec<u8>>,
    pub title: String,
    /// ms
    pub audio_length: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Entity {
    pub async fn find_by_sid(sid: i32) -> Option<Model> {
        Self::find()
            .filter(Column::Sid.eq(sid))
            .one(database())
            .await
            .expect("无法查询数据库")
    }

    pub async fn find_by_bid(bid: i32) -> Option<Model> {
        Self::find()
            .filter(Column::Bid.eq(bid))
            .one(database())
            .await
            .expect("无法查询数据库")
    }

    pub async fn find_by_title_like(title: &str) -> Vec<Model> {
        Self::find()
            .filter(Column::Title.contains(title))
            .all(database())
            .await
            .expect("无法查询数据库")
    }

    pub async fn save(
        sid: i32,
        bid: i32,
        cache: Option<Vec<u8>>,
        title: String,
        audio_length: i32,
    ) -> crate::error::Result<()> {
        let model = ActiveModel {
            sid: ActiveValue::Set(sid),
            bid: ActiveValue::Set(bid),
            cache: ActiveValue::Set(cache),
            title: ActiveValue::Set(title),
            audio_length: ActiveValue::Set(audio_length),
        };

        // 如果存在相同的 bid，则更新记录
        let mut on_conflict = OnConflict::column(Column::Bid);
        on_conflict
            .update_column(Column::Sid)
            .update_column(Column::Cache)
            .update_column(Column::Title)
            .update_column(Column::AudioLength);

        Self::insert(model)
            .on_conflict(on_conflict)
            .exec(database())
            .await?;
        Ok(())
    }
}
