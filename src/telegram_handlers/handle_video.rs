use crate::database::{Database, MediaType, Post, PostMessageId};
use chrono::Utc;
use std::error::Error;
use std::sync::Arc;
use teloxide::prelude::*;

pub async fn handle_video(
    message: Message,
    db: Arc<Database>,
) -> Result<(), Box<dyn Error + Sync + Send>> {
    let file_meta = &message.video().unwrap().file;

    let post = Post {
        id: None,
        media_type: MediaType::Video,
        file_id: file_meta.id.clone(),
        message_ids: vec![PostMessageId {
            chat_id: message.chat.id.0,
            message_id: message.id.0,
        }],
        sent: false,
        added_datetime: Utc::now().naive_utc(),
        image_hash: None,
    };
    match db.create_post(post).await {
        Ok(_) => {
            log::info!("Post saved");
        }
        Err(e) => {
            log::error!("Error saving post: {e:?}");
        }
    }

    Ok(())
}
