use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. Criamos a tabela de Tags primeiro
        manager
            .create_table(
                Table::create()
                    .table(Tags::Table)
                    .if_not_exists()
                    .col(pk_auto(Tags::Id))
                    // O nome da tag deve ser único (ex: "férias", "documentos")
                    .col(string(Tags::Name).unique_key())
                    .to_owned(),
            )
            .await?;

        // 2. Criamos a tabela de Junção (FileTags)
        manager
            .create_table(
                Table::create()
                    .table(FileTags::Table)
                    .if_not_exists()
                    .col(integer(FileTags::FileId))
                    .col(integer(FileTags::TagId))
                    // Chave Primária Composta: A união do FileId com TagId é única
                    .primary_key(
                        Index::create()
                            .name("file_tags")
                            .col(FileTags::FileId)
                            .col(FileTags::TagId),
                    )
                    // Chave Estrangeira apontando para Arquivos
                    .foreign_key(
                        ForeignKey::create()
                            .name("file_tags-file_id")
                            .from(FileTags::Table, FileTags::FileId)
                            .to(Files::Table, Files::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    // Chave Estrangeira apontando para Tags
                    .foreign_key(
                        ForeignKey::create()
                            .name("file_tags-tag_id")
                            .from(FileTags::Table, FileTags::TagId)
                            .to(Tags::Table, Tags::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Na hora de desfazer, apagamos na ordem inversa
        manager
            .drop_table(Table::drop().table(FileTags::Table).to_owned())
            .await?;
            
        manager
            .drop_table(Table::drop().table(Tags::Table).to_owned())
            .await
    }
}

// Enums Identificadores
#[derive(DeriveIden)]
enum Tags {
    Table,
    Id,
    Name,
}

#[derive(DeriveIden)]
enum FileTags {
    Table,
    FileId,
    TagId,
}

// Precisamos referenciar a tabela de Arquivos que criamos na migration passada
#[derive(DeriveIden)]
enum Files {
    Table,
    Id,
}