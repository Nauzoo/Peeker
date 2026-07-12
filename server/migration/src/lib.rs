pub use sea_orm_migration::prelude::*;

mod m20220101_000001_users;
mod m20260712_061208_files;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_users::Migration),
            Box::new(m20260712_061208_files::Migration),
        ]
    }
}
