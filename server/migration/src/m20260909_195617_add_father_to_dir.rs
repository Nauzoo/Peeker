use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260909_000001_add_father_to_directory"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Directory::Table)
                    .add_column(
                        // Nullable because root directories won't have a parent/father
                        ColumnDef::new(Directory::Father)
                            .big_integer() // Use .big_integer() to match Directory::Id (i64), or .string() if storing paths/names
                            .null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Directory::Table)
                    .drop_column(Directory::Father)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Directory {
    Table,
    Father,
}
