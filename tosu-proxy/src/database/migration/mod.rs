use sea_orm_migration::MigratorTrait;

mod m20240101_000001_create_tables;
mod m20240101_000002_add_updated_at_to_cache;
mod m20240101_000003_add_reason_to_block;
mod m20240101_000004_create_lyric_block_unique_index;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
        vec![
            Box::new(m20240101_000001_create_tables::Migration),
            Box::new(m20240101_000002_add_updated_at_to_cache::Migration),
            Box::new(m20240101_000003_add_reason_to_block::Migration),
            Box::new(m20240101_000004_create_lyric_block_unique_index::Migration),
        ]
    }
}
