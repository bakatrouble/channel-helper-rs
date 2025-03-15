use std::sync::{Arc};
use chrono::Utc;
use diesel::dsl::{exists};
use diesel::prelude::*;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use tokio::sync::{Notify, Mutex};
use uuid::Uuid;

mod schema;
mod models;

pub use models::{MediaType, Post, UploadTask, PostMessageId};
use crate::database::models::UUID;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
define_sql_function!(fn random() -> Text);

pub struct Database {
    conn: Arc<Mutex<SqliteConnection>>,
    pub upload_task_added: Notify,
}

impl Database {
    pub fn open(db_name: &str) -> anyhow::Result<Self> {
        let mut conn = SqliteConnection::establish(db_name)?;
        conn.run_pending_migrations(MIGRATIONS).expect("Unable to apply migrations");

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            upload_task_added: Notify::new(),
        })
    }

    pub async fn create_post(&self, id: Option<Uuid>, media_type: MediaType, file_id: String, image_hash: Option<String>, chat_id: i64, message_id: i32) -> anyhow::Result<Post> {
        use crate::database::schema::{posts, post_message_ids};

        let post_id = id.unwrap_or(Uuid::now_v7());

        let new_post = Post {
            id: post_id,
            media_type,
            file_id,
            is_sent: false,
            created_datetime: Utc::now().naive_utc(),
            sent_datetime: None,
            image_hash,
        };

        let new_message_id = PostMessageId {
            chat_id,
            message_id,
            post_id,
        };

        self.conn.lock().await.transaction(|conn| {
            let post = diesel::insert_into(posts::table)
                .values(new_post)
                .returning(Post::as_returning())
                .get_result(conn)
                .expect("error saving new post");

            diesel::insert_into(post_message_ids::table)
                .values(new_message_id)
                .execute(conn)
                .expect("error saving message id");

            Ok(post)
        })
    }

    pub async fn get_post_by_hash(&self, hash: String) -> anyhow::Result<Option<Post>> {
        use crate::database::schema::posts::dsl::{posts, image_hash};

        self.conn.lock().await.transaction(|conn| {
            Ok(
                posts
                    .filter(image_hash.eq(hash))
                    .limit(1)
                    .select(Post::as_select())
                    .load(conn)
                    .expect("error fetching post")
                    .pop()
            )
        })
    }

    pub async fn add_message_id_for_post(&self, post_id: Uuid, chat_id: i64, message_id: i32) -> anyhow::Result<()> {
        use crate::database::schema::{post_message_ids};

        let new_post_message_id = PostMessageId {
            chat_id,
            message_id,
            post_id,
        };

        self.conn.lock().await.transaction(|conn| {
            diesel::insert_into(post_message_ids::table)
                .values(new_post_message_id)
                .execute(conn)
                .expect("error saving message id");

            Ok(())
        })
    }

    pub async fn post_with_hash_exists(&self, hash: String) -> anyhow::Result<bool> {
        use crate::database::schema::posts::dsl::{posts, image_hash};

        self.conn.lock().await.transaction(|conn| {
            Ok(
                diesel::select(exists(posts.filter(image_hash.eq(hash))))
                    .get_result::<bool>(conn)
                    .expect("error checking hash existence")
            )
        })
    }

    pub async fn create_upload_task(&self, media_type: MediaType, data: Vec<u8>, image_hash: Option<String>) -> anyhow::Result<UploadTask> {
        use crate::database::schema::{upload_tasks};

        let new_post = UploadTask {
            id: Uuid::now_v7(),
            media_type,
            data,
            is_processed: false,
            created_datetime: Utc::now().naive_utc(),
            processed_datetime: None,
            image_hash,
        };

        self.conn.lock().await.transaction(|conn| {
            Ok(
                diesel::insert_into(upload_tasks::table)
                    .values(new_post)
                    .returning(UploadTask::as_returning())
                    .get_result(conn)
                    .expect("error saving new upload task")
            )
        })
    }

    pub async fn fetch_unprocessed_upload_task(&self) -> anyhow::Result<Option<UploadTask>> {
        use crate::database::schema::upload_tasks::dsl::{upload_tasks, is_processed};

        self.conn.lock().await.transaction(|conn| {
            Ok(
                upload_tasks
                    .filter(is_processed.eq(false))
                    .limit(1)
                    .select(UploadTask::as_select())
                    .load(conn)
                    .expect("error fetching upload tasks")
                    .pop()
            )
        })
    }

    pub async fn mark_complete_upload_task(&self, id: Uuid) -> anyhow::Result<()> {
        use crate::database::schema::upload_tasks::dsl::{upload_tasks, is_processed, processed_datetime};

        self.conn.lock().await.transaction(|conn| {
            diesel::update(upload_tasks.find(UUID::from(id)))
                .set((
                     is_processed.eq(true),
                     processed_datetime.eq(Utc::now().naive_utc())
                ))
                .execute(conn)
                .expect("error marking upload task as complete");
            Ok(())
        })
    }

    pub async fn unsent_posts_count(&self) -> anyhow::Result<i64> {
        use crate::database::schema::posts::dsl::{posts, is_sent};

        self.conn.lock().await.transaction(|conn| {
            Ok(
                posts
                    .filter(is_sent.eq(false))
                    .count()
                    .get_result(conn)
                    .expect("error getting unsent posts count")
            )
        })
    }

    pub async fn fetch_unsent_post(&self) -> anyhow::Result<Option<Post>> {
        use crate::database::schema::posts::dsl::{posts, is_sent};

        self.conn.lock().await.transaction(|conn| {
            Ok(
                posts
                    .filter(is_sent.eq(false))
                    .limit(1)
                    .order_by(random())
                    .select(Post::as_select())
                    .load(conn)
                    .expect("error fetching unsent post")
                    .pop()
            )
        })
    }

    pub async fn fetch_ten_unsent_photo_posts(&self) -> anyhow::Result<Vec<Post>> {
        use crate::database::schema::posts::dsl::{posts, is_sent, media_type};

        self.conn.lock().await.transaction(|conn| {
            Ok(
                posts
                    .filter(
                        is_sent.eq(false)
                            .and(media_type.eq(MediaType::Photo))
                    )
                    .limit(10)
                    .order_by(random())
                    .select(Post::as_select())
                    .load(conn)
                    .expect("error fetching unsent photo posts")
            )
        })
    }

    pub async fn mark_sent_posts<T>(&self, ids: T) -> anyhow::Result<()>
    where
        T: IntoIterator<Item = Uuid>
    {
        use crate::database::schema::posts::dsl::{posts, id, is_sent, sent_datetime};

        self.conn.lock().await.transaction(|conn| {
            diesel::update(posts.filter(id.eq_any(ids.into_iter().map(|v| UUID::from(v)))))
                .set((
                    is_sent.eq(true),
                    sent_datetime.eq(Utc::now().naive_utc())
                ))
                .execute(conn)
                .expect("error marking post as sent");
            Ok(())
        })
    }
}
