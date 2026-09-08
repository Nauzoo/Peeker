use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260908_020252_directories"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Directory::Table)
                    .if_not_exists()
                    .col(pk_auto(Directory::Id))
                    .col(string(Directory::Name))
                    .col(string(Directory::Path))
                    .col(string(Directory::Creator))
                    .foreign_key(
                        ForeignKey::create()
                            .name("creator_id")
                            .from(Directory::Table, Directory::Creator)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Directory::Table).to_owned())
            .await
    }
}
#[derive(DeriveIden)]
enum Directory {
    Table,
    Id,
    Name,
    Path,
    Creator,
}
#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
