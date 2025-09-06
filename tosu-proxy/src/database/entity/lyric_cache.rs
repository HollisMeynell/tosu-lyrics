use crate::database::database;
use crate::lyric::Lyric;
use sea_orm::ActiveValue;
use sea_orm::entity::prelude::*;
use sea_orm::sea_query::OnConflict;
use crate::database::entity::DB_ERROR_MESSAGE;

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
            .expect(DB_ERROR_MESSAGE)
    }

    pub async fn find_by_bid(bid: i32) -> Option<Model> {
        Self::find()
            .filter(Column::Bid.eq(bid))
            .one(database())
            .await
            .expect(DB_ERROR_MESSAGE)
    }

    pub async fn find_by_title_like(title: &str) -> Vec<Model> {
        Self::find()
            .filter(Column::Title.contains(title))
            .all(database())
            .await
            .expect(DB_ERROR_MESSAGE)
    }

    pub async fn all_count() -> u64 {
        Self::find()
            .count(database()).await
            .expect(DB_ERROR_MESSAGE)
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
            .update_column(Column::AudioLength);

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
