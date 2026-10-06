use crate::database::database;
use sea_orm::entity::prelude::*;
use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveValue, ColumnTrait, QueryFilter};

type Res<T> = crate::error::Result<T>;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "lyric_binding")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub sid: i32,
    pub source_type: String,
    pub source_key: String,
    #[sea_orm(default_value = "")]
    pub title: String,
    #[sea_orm(default_value = "")]
    pub artist: String,
    /// 绑定时的谱面 id，仅用于排查，不参与匹配
    #[sea_orm(default_value = 0)]
    pub bid: i32,
    #[sea_orm(default_value = 0)]
    pub created_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Entity {
    pub async fn get(sid: i32) -> Res<Option<Model>> {
        Ok(Self::find_by_id(sid).one(database()).await?)
    }

    /// 幂等写入：同一 sid 重复绑定只覆盖为最新选择
    pub async fn upsert(
        sid: i32,
        source_type: &str,
        source_key: &str,
        title: &str,
        artist: &str,
        bid: i32,
    ) -> Res<Model> {
        let now = sea_orm::sqlx::types::chrono::Utc::now().timestamp_millis();
        let model = ActiveModel {
            sid: ActiveValue::Set(sid),
            source_type: ActiveValue::Set(source_type.to_string()),
            source_key: ActiveValue::Set(source_key.to_string()),
            title: ActiveValue::Set(title.to_string()),
            artist: ActiveValue::Set(artist.to_string()),
            bid: ActiveValue::Set(bid),
            created_at: ActiveValue::Set(now),
        };

        let mut on_conflict = OnConflict::column(Column::Sid);
        on_conflict
            .update_column(Column::SourceType)
            .update_column(Column::SourceKey)
            .update_column(Column::Title)
            .update_column(Column::Artist)
            .update_column(Column::Bid)
            .update_column(Column::CreatedAt);

        Self::insert(model)
            .on_conflict(on_conflict)
            .exec(database())
            .await?;

        Self::find_by_id(sid)
            .one(database())
            .await?
            .ok_or(crate::error::Error::Impossible)
    }

    /// 幂等删除，返回是否真的删掉了一行
    pub async fn remove(sid: i32) -> Res<bool> {
        let result = <Self as EntityTrait>::delete_by_id(sid)
            .exec(database())
            .await?;
        Ok(result.rows_affected > 0)
    }

    pub async fn count() -> Res<u64> {
        Ok(Self::find().count(database()).await?)
    }

    pub async fn list_all() -> Res<Vec<Model>> {
        Ok(Self::find()
            .filter(Column::Sid.gt(0))
            .all(database())
            .await?)
    }
}
