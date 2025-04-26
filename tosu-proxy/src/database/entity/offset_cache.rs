use crate::database::database;
use sea_orm::entity::prelude::*;
use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveValue, ColumnTrait, Condition, QueryFilter, QuerySelect};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "offset_cache")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub bid: i32,
    #[sea_orm(indexed)]
    pub sid: i32,
    pub offset: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Entity {
    pub async fn find_offset(bid: i32, sid: i32) -> i32 {
        let db = database();
        // 优先根据 bid 查询
        if let Some(model) = Self::find_by_id(bid)
            .one(db)
            .await
            .expect("查询 offset_cache (bid) 失败")
        {
            return model.offset;
        }

        // bid 未找到，根据 sid 查询第一个
        if let Some(model) = Self::find()
            .filter(Column::Sid.eq(sid))
            .one(db)
            .await
            .expect("查询 offset_cache (sid) 失败")
        {
            return model.offset;
        }

        // 都没找到，返回默认值 0
        0
    }

    pub async fn save_offset(bid: i32, sid: i32, offset: i32) {
        let model = ActiveModel {
            bid: ActiveValue::Set(bid),
            sid: ActiveValue::Set(sid),
            offset: ActiveValue::Set(offset),
        };

        let mut on_conflict = OnConflict::column(Column::Bid);
        on_conflict
            .update_column(Column::Sid)
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