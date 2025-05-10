use super::database;
use sea_orm::ActiveValue;
use sea_orm::entity::prelude::*;
use sea_orm::sea_query::OnConflict;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "config")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub key: String,
    pub setting: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Entity {
    pub async fn get_config(key:&str) -> Option<String> {
        let query = Self::find_by_id(key);
        let result = query
            .one(database())
            .await
            .expect("can not read database")?;
        Some(result.setting)
    }

    pub async fn save_config(key:String, setting: String) {
        let model = ActiveModel {
            key: ActiveValue::set(key),
            setting: ActiveValue::set(setting),
        };
        let mut on_conflict = OnConflict::column(Column::Key);
        on_conflict.update_column(Column::Setting);
        Self::insert(model)
            .on_conflict(on_conflict)
            .exec(database())
            .await
            .expect("can not write database");
    }
}
