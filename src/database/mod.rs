mod migrations;
mod models;
mod schema;

use sea_query::{Alias, Expr, Query, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use tokio::sync::Notify;
use tokio_rusqlite::Connection;

use crate::database::migrations::get_migrations;
use crate::database::schema::{
    Post as PostSchema, PostMessageIds as PostMessageIdsSchema, UploadTask as UploadTaskSchema,
};
pub use models::{MediaType, Post, PostMessageId, UploadTask};
use uuid::Uuid;

#[derive(Debug)]
pub struct Database {
    pub conn: Connection,
    pub upload_task_added: Notify,
}

impl Database {
    pub async fn open(path: &String) -> anyhow::Result<Self> {
        Database::migrate(path)?;

        let conn = Connection::open(path).await?;
        Ok(Database {
            conn,
            upload_task_added: Notify::new(),
        })
    }

    pub fn migrate(path: &String) -> anyhow::Result<(), rusqlite_migration::Error> {
        use rusqlite::Connection;
        use rusqlite_migration::{M, Migrations};
        let mut db = Connection::open(path)?;
        let migrations_str = get_migrations();
        let migrations = migrations_str.iter().map(|e| M::up(e.as_str())).collect();
        Migrations::new(migrations).to_latest(&mut db)?;
        Ok(())
    }

    pub async fn create_post(&self, post: Post) -> anyhow::Result<()> {
        self.conn
            .call(move |conn| {
                let transaction = conn.transaction()?;

                let id = post.id.unwrap_or_else(|| Uuid::now_v7());

                let (sql, values) = Query::insert()
                    .into_table(PostSchema::Table)
                    .columns([
                        PostSchema::Id,
                        PostSchema::MediaType,
                        PostSchema::FileId,
                        PostSchema::Sent,
                        PostSchema::AddedDatetime,
                        PostSchema::ImageHash,
                    ])
                    .values_panic([
                        id.into(),
                        post.media_type.into(),
                        post.file_id.into(),
                        post.sent.into(),
                        post.added_datetime.into(),
                        post.image_hash.into(),
                    ])
                    .build_rusqlite(SqliteQueryBuilder);
                transaction.execute(sql.as_str(), &*values.as_params())?;

                let mut statement = Query::insert();
                let mut query_builder =
                    statement.into_table(PostMessageIdsSchema::Table).columns([
                        PostMessageIdsSchema::ChatId,
                        PostMessageIdsSchema::MessageId,
                        PostMessageIdsSchema::PostId,
                    ]);

                for message_id in post.message_ids {
                    query_builder = query_builder.values_panic([
                        message_id.chat_id.into(),
                        message_id.message_id.into(),
                        id.into(),
                    ])
                }

                let (sql, values) = query_builder.build_rusqlite(SqliteQueryBuilder);
                transaction.execute(sql.as_str(), &*values.as_params())?;

                transaction.commit()?;

                Ok(())
            })
            .await?;
        Ok(())
    }

    pub async fn add_message_id_for_post(
        &self,
        post_id: Uuid,
        message_id: PostMessageId,
    ) -> anyhow::Result<()> {
        self.conn
            .call(move |conn| {
                let (sql, values) = Query::insert()
                    .into_table(PostMessageIdsSchema::Table)
                    .columns([
                        PostMessageIdsSchema::ChatId,
                        PostMessageIdsSchema::MessageId,
                        PostMessageIdsSchema::PostId,
                    ])
                    .values_panic([
                        message_id.chat_id.into(),
                        message_id.message_id.into(),
                        post_id.into(),
                    ])
                    .build_rusqlite(SqliteQueryBuilder);

                conn.execute(sql.as_str(), &*values.as_params())?;
                Ok(())
            })
            .await?;
        Ok(())
    }

    pub async fn get_post_by_hash(&self, hash: String) -> anyhow::Result<Option<Post>> {
        Ok(self
            .conn
            .call(move |conn| {
                let (sql, values) = Query::select()
                    .columns([
                        PostSchema::Id,
                        PostSchema::MediaType,
                        PostSchema::FileId,
                        PostSchema::Sent,
                        PostSchema::AddedDatetime,
                        PostSchema::ImageHash,
                    ])
                    .expr_as(
                        Expr::col((PostMessageIdsSchema::Table, PostMessageIdsSchema::ChatId)),
                        Alias::new("chat_id"),
                    )
                    .column((PostMessageIdsSchema::Table, PostMessageIdsSchema::ChatId))
                    .expr_as(
                        Expr::col((PostMessageIdsSchema::Table, PostMessageIdsSchema::MessageId)),
                        Alias::new("message_id"),
                    )
                    .from(PostSchema::Table)
                    .left_join(
                        PostMessageIdsSchema::Table,
                        Expr::col(PostMessageIdsSchema::PostId).eq(Expr::col(PostSchema::Id)),
                    )
                    .cond_where(Expr::col(PostSchema::ImageHash).eq(hash))
                    .build_rusqlite(SqliteQueryBuilder);
                let mut stmt = conn.prepare(sql.as_str())?;
                let mut rows = stmt.query(&*values.as_params())?;
                Ok(match rows.next()? {
                    Some(row) => {
                        let mut post = Post::from(row);
                        post.message_ids.push(PostMessageId::from(row));
                        while let Some(row) = rows.next().unwrap() {
                            post.message_ids.push(PostMessageId::from(row));
                        }
                        Some(post)
                    }
                    None => None,
                })
            })
            .await?)
    }

    pub async fn post_with_hash_exists(&self, hash: String) -> anyhow::Result<bool> {
        Ok(self
            .conn
            .call(move |conn| {
                let (sql, values) = Query::select()
                    .expr(Expr::exists(
                        Query::select()
                            .expr(Expr::value(1))
                            .from(PostSchema::Table)
                            .cond_where(Expr::col(PostSchema::ImageHash).eq(hash))
                            .limit(1)
                            .take(),
                    ))
                    .build_rusqlite(SqliteQueryBuilder);
                Ok(conn.query_row(sql.as_str(), &*values.as_params(), |row| row.get(0))?)
            })
            .await?)
    }

    pub async fn mark_sent_post(&self, post: Post) -> anyhow::Result<()> {
        self.conn
            .call(move |conn| {
                let (sql, values) = Query::update()
                    .table(PostSchema::Table)
                    .values([(PostSchema::Sent, true.into())])
                    .cond_where(Expr::col(PostSchema::Id).eq(post.id.unwrap()))
                    .build_rusqlite(SqliteQueryBuilder);

                conn.execute(sql.as_str(), &*values.as_params())?;
                Ok(())
            })
            .await?;
        Ok(())
    }

    pub async fn fetch_unsent_post(&self) -> anyhow::Result<Option<Post>> {
        Ok(self
            .conn
            .call(move |conn| {
                let (sql, values) = Query::select()
                    .columns([
                        PostSchema::Id,
                        PostSchema::MediaType,
                        PostSchema::FileId,
                        PostSchema::Sent,
                        PostSchema::AddedDatetime,
                        PostSchema::ImageHash,
                    ])
                    .from(PostSchema::Table)
                    .cond_where(Expr::col(PostSchema::Sent).eq(false))
                    .build_rusqlite(SqliteQueryBuilder);
                let mut stmt = conn.prepare(sql.as_str())?;
                let mut rows = stmt.query(&*values.as_params())?;
                let post = rows.next()?.map(|row| Post::from(row));

                Ok(post)
            })
            .await?)
    }

    pub async fn fetch_unsent_post_with_media_type(
        &self,
        media_type: MediaType,
    ) -> anyhow::Result<Option<Post>> {
        Ok(self
            .conn
            .call(move |conn| {
                let (sql, values) = Query::select()
                    .columns([
                        PostSchema::Id,
                        PostSchema::MediaType,
                        PostSchema::FileId,
                        PostSchema::Sent,
                        PostSchema::AddedDatetime,
                        PostSchema::ImageHash,
                    ])
                    .from(PostSchema::Table)
                    .cond_where(Expr::col(PostSchema::Sent).eq(false))
                    .and_where(Expr::col(PostSchema::MediaType).eq(media_type))
                    .build_rusqlite(SqliteQueryBuilder);
                let mut stmt = conn.prepare(sql.as_str())?;
                let mut rows = stmt.query(&*values.as_params())?;
                let post = rows.next()?.map(|row| Post::from(row));

                Ok(post)
            })
            .await?)
    }

    pub async fn create_upload_task(&self, upload_task: UploadTask) -> anyhow::Result<()> {
        self.conn
            .call(move |conn| {
                let (sql, values) = Query::insert()
                    .into_table(UploadTaskSchema::Table)
                    .columns([
                        UploadTaskSchema::Id,
                        UploadTaskSchema::MediaType,
                        UploadTaskSchema::Data,
                        UploadTaskSchema::Processed,
                        UploadTaskSchema::ImageHash,
                    ])
                    .values_panic([
                        Uuid::now_v7().into(),
                        upload_task.media_type.into(),
                        upload_task.data.into(),
                        upload_task.processed.into(),
                        upload_task.image_hash.into(),
                    ])
                    .build_rusqlite(SqliteQueryBuilder);

                conn.execute(sql.as_str(), &*values.as_params())?;
                Ok(())
            })
            .await?;
        self.upload_task_added.notify_one();
        Ok(())
    }

    pub async fn mark_complete_upload_task(&self, upload_task: &UploadTask) -> anyhow::Result<()> {
        let id = upload_task.id.clone().unwrap();
        self.conn
            .call(move |conn| {
                let (sql, values) = Query::update()
                    .table(UploadTaskSchema::Table)
                    .values([
                        (UploadTaskSchema::Processed, true.into()),
                        (UploadTaskSchema::Data, "".into()),
                    ])
                    .cond_where(Expr::col(UploadTaskSchema::Id).eq(id))
                    .build_rusqlite(SqliteQueryBuilder);

                conn.execute(sql.as_str(), &*values.as_params())?;
                Ok(())
            })
            .await?;
        Ok(())
    }

    pub async fn fetch_unprocessed_upload_task(&self) -> anyhow::Result<Option<UploadTask>> {
        Ok(self
            .conn
            .call(move |conn| {
                let (sql, values) = Query::select()
                    .columns([
                        UploadTaskSchema::Id,
                        UploadTaskSchema::MediaType,
                        UploadTaskSchema::Data,
                        UploadTaskSchema::Processed,
                        UploadTaskSchema::ImageHash,
                    ])
                    .from(UploadTaskSchema::Table)
                    .cond_where(Expr::col(UploadTaskSchema::Processed).eq(false))
                    .build_rusqlite(SqliteQueryBuilder);
                let mut stmt = conn.prepare(sql.as_str())?;
                let mut rows = stmt.query(&*values.as_params())?;
                let upload_task = rows.next()?.map(|row| UploadTask::from(row));

                Ok(upload_task)
            })
            .await?)
    }
}
