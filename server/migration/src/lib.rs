pub use sea_orm_migration::prelude::*;

mod m20220101_000001_users;
mod m20260712_061208_files;
mod m20260712_184826_tags;
mod m20260908_020252_directories;
mod m20260909_195617_add_father_to_dir;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_users::Migration),
            Box::new(m20260712_061208_files::Migration),
            Box::new(m20260712_184826_tags::Migration),
            Box::new(m20260908_020252_directories::Migration),
            Box::new(m20260909_195617_add_father_to_dir::Migration),
        ]
    }
}
