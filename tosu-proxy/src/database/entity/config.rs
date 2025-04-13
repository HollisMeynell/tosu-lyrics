use super::database;
use sea_orm::ActiveValue;
use sea_orm::entity::prelude::*;
use sea_orm::sea_query::OnConflict;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "config")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub config: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Entity {
    pub async fn get_config() -> Option<String> {
        let query = Self::find_by_id(0);
        let result = query
            .one(database())
            .await
            .expect("can not read database")?;
        Some(result.config)
    }

    pub async fn save_config<T: ToString>(config: T) {
        let model = ActiveModel {
            id: ActiveValue::set(0),
            config: ActiveValue::set(config.to_string()),
        };
        let mut on_conflict = OnConflict::column(Column::Id);
        on_conflict.update_column(Column::Config);
        Self::insert(model)
            .on_conflict(on_conflict)
            .exec(database())
            .await
            .expect("can not write database");
    }
}
