mod lyric_cache;
mod lyric_config;
mod setting;

use crate::database::table_exists;
use crate::error::Result;
use sea_orm::{ConnectionTrait, EntityName, Schema};

use super::database;
pub use lyric_cache::Entity as LyricCacheEntity;
pub use lyric_config::Entity as LyricConfigEntity;
pub use setting::Entity as SettingEntity;

macro_rules! init_entity {
    ($($fn_name:ident($entity:ident),)*) => {
        pub(super)async fn init_all_table() -> Result<()> {
            $($fn_name().await?;)*
            Ok(())
        }
        $(
        async fn $fn_name() -> Result<()> {
            let default = $entity::default();
            let table_name = EntityName::table_name(&default);
            if !table_exists(table_name).await? {
                let db = super::database();
                let backend = db.get_database_backend();
                let schema = Schema::new(backend)
                    .create_table_from_entity($entity);
                db.execute(backend.build(&schema)).await?;
            }
            Ok(())
        }
        )*
    };
}
init_entity! {
    init_setting(SettingEntity),
    init_lyric_cache(LyricCacheEntity),
    init_lyric_config(LyricConfigEntity),
}
