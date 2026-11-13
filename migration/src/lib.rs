pub use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        // Add new migrations here, for example:
        // vec![Box::new(m20260101_000001_create_table::Migration)]
        vec![]
    }
}
