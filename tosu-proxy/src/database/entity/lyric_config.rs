use crate::database::database;
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
    async fn find_first(bid: i32, sid: i32, title: &str) -> Option<Model> {
        let db = database();
        // 优先根据 bid 查询
        if let Some(model) = Self::find_by_id(bid)
            .one(db)
            .await
            .expect("查询 offset_cache (bid) 失败")
        {
            return Some(model);
        }

        // bid 未找到, 再找 title
        if let Some(model) = Self::find()
            .filter(Column::Title.eq(title))
            .one(db)
            .await
            .expect("查询 offset_cache (title) 失败")
        {
            return Some(model);
        }

        // 都未找到，根据 sid 查询第一个
        if let Some(model) = Self::find()
            .filter(Column::Sid.eq(sid))
            .one(db)
            .await
            .expect("查询 offset_cache (sid) 失败")
        {
            return Some(model);
        }
        None
    }
    pub async fn find_setting(bid: i32, sid: i32, title: &str) -> (bool, i32) {
        if let Some(m) = Entity::find_first(bid, sid, title).await {
            (m.disable, m.offset)
        } else {
            // 都没找到，返回默认值 0
            (false, 0)
        }
    }

    pub async fn save_setting(data: Model) {
        let model = ActiveModel::from(data);

        let mut on_conflict = OnConflict::column(Column::Bid);
        on_conflict
            .update_column(Column::Sid)
            .update_column(Column::Title)
            .update_column(Column::Disable)
            .update_column(Column::Offset);

        Self::insert(model)
            .on_conflict(on_conflict)
            .exec(database())
            .await
            .expect("保存 offset_cache 失败");
    }

    pub async fn delete_by_bid(bid: i32) {
        Self::delete_by_id(bid)
            .exec(database())
            .await
            .expect("删除 offset_cache 失败");
    }
}
