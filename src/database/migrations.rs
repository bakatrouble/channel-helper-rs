use crate::database::schema::{Post, PostMessageIds, State, UploadTask};
use sea_query::{ColumnDef, ForeignKey, Index, SqliteQueryBuilder, Table};

pub fn get_migrations() -> Vec<String> {
    vec![
        // create Post table and indices
        vec![
            Table::create()
                .table(Post::Table)
                .col(ColumnDef::new(Post::Id).uuid().not_null().primary_key())
                .col(ColumnDef::new(Post::MediaType).text().not_null())
                .col(ColumnDef::new(Post::FileId).text().not_null())
                .col(ColumnDef::new(Post::Sent).boolean().not_null())
                .col(ColumnDef::new(Post::AddedDatetime).date_time().not_null())
                .col(ColumnDef::new(Post::ImageHash).text().null())
                .build(SqliteQueryBuilder),
            Index::create()
                .name("post_media_type_idx")
                .table(Post::Table)
                .col(Post::MediaType)
                .build(SqliteQueryBuilder),
            Index::create()
                .name("post_sent_idx")
                .table(Post::Table)
                .col(Post::Sent)
                .build(SqliteQueryBuilder),
            Index::create()
                .name("post_image_hash_idx")
                .table(Post::Table)
                .col(Post::ImageHash)
                .build(SqliteQueryBuilder),
        ]
        .join("; "),
        // create UploadTask table and indices
        vec![
            Table::create()
                .table(UploadTask::Table)
                .col(
                    ColumnDef::new(UploadTask::Id)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(ColumnDef::new(UploadTask::MediaType).text().not_null())
                .col(ColumnDef::new(UploadTask::Data).blob().not_null())
                .col(ColumnDef::new(UploadTask::Processed).boolean().not_null())
                .col(ColumnDef::new(UploadTask::ImageHash).text().null())
                .build(SqliteQueryBuilder),
            Index::create()
                .name("upload_task_processed_idx")
                .table(UploadTask::Table)
                .col(UploadTask::Processed)
                .build(SqliteQueryBuilder),
        ]
        .join("; "),
        // create PostMessageIds table and indices
        vec![
            Table::create()
                .table(PostMessageIds::Table)
                .col(
                    ColumnDef::new(PostMessageIds::ChatId)
                        .big_integer()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(PostMessageIds::MessageId)
                        .integer()
                        .not_null(),
                )
                .col(ColumnDef::new(PostMessageIds::PostId).uuid().not_null())
                .foreign_key(
                    ForeignKey::create()
                        .from(PostMessageIds::Table, PostMessageIds::PostId)
                        .to(Post::Table, Post::Id),
                )
                .build(SqliteQueryBuilder),
            Index::create()
                .unique()
                .name("post_message_ids_idx")
                .table(PostMessageIds::Table)
                .col(PostMessageIds::ChatId)
                .col(PostMessageIds::MessageId)
                .build(SqliteQueryBuilder),
        ]
        .join("; "),
        // create State table
        vec![
            Table::create()
                .table(State::Table)
                .col(ColumnDef::new(State::LastSent).date_time().not_null())
                .build(SqliteQueryBuilder),
        ]
        .join("; "),
    ]
}
