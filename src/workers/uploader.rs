use crate::config::Config;
use crate::database::{Database, MediaType, UploadTask};
use std::sync::Arc;
use teloxide::RequestError;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, InputFile};
use uuid::Uuid;

pub async fn run_uploader(bot: Bot, db: Arc<Database>, cfg: Config) -> anyhow::Result<()> {
    while let Ok(Some(upload_task)) = db.fetch_unprocessed_upload_task().await {
        log::info!("Uploading queued photo...");
        upload(
            bot.clone(),
            cfg.upload_chat_id.unwrap(),
            db.clone(),
            upload_task,
        )
        .await;
    }

    loop {
        db.upload_task_added.notified().await;
        while let Ok(Some(upload_task)) = db.fetch_unprocessed_upload_task().await {
            log::info!("Uploading photo...");
            upload(
                bot.clone(),
                cfg.upload_chat_id.unwrap(),
                db.clone(),
                upload_task,
            )
            .await;
        }
    }
}

async fn upload(bot: Bot, chat_id: i64, db: Arc<Database>, upload_task: UploadTask) {
    match upload_task.media_type {
        MediaType::Photo => {
            let post_id = Uuid::now_v7();

            let keyboard = InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
                "Delete",
                format!("del {}", post_id),
            )]]);

            let msg = match bot
                .send_photo(
                    ChatId(chat_id),
                    InputFile::memory(upload_task.data.as_slice().to_owned()),
                )
                .reply_markup(keyboard)
                .await
            {
                Ok(v) => v,
                Err(RequestError::RetryAfter(sec)) => {
                    log::warn!("Rate limit: {} sec", &sec);
                    tokio::time::sleep(sec.duration()).await;
                    return;
                }
                Err(e) => {
                    log::error!("Upload error: {e:?}");
                    return;
                }
            };

            let file_meta = &msg.photo().unwrap().last().unwrap().file;

            match db
                .post_with_hash_exists(upload_task.image_hash.clone().unwrap())
                .await
            {
                Ok(true) => {
                    match db.mark_complete_upload_task(upload_task.id).await {
                        Ok(_) => {}
                        Err(e) => log::error!("Database error: {e:?}"),
                    }
                    return;
                }
                Ok(false) => {}
                Err(e) => {
                    log::error!("Checking hash collision error: {e:?}");
                    return;
                }
            }

            match db
                .create_post(
                    Some(post_id),
                    MediaType::Photo,
                    file_meta.id.clone(),
                    Some(upload_task.image_hash.clone().unwrap()),
                    msg.chat.id.0,
                    msg.id.0,
                ).await
            {
                Ok(_) => {}
                Err(e) => {
                    log::error!("Database error: {e:?}");
                    return;
                }
            }

            match db.mark_complete_upload_task(upload_task.id).await {
                Ok(_) => log::info!("Uploaded a photo"),
                Err(e) => log::error!("Database error: {e:?}"),
            }
        }
        // MediaType::Animation => {
        //
        // },
        _ => {
            log::warn!("Unsupported media type: {}", upload_task.media_type);
            match db.mark_complete_upload_task(upload_task.id).await {
                Ok(_) => {}
                Err(e) => log::error!("Database error: {e:?}"),
            }
        }
    }
}
