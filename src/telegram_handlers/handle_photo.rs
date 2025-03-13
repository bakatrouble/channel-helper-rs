use std::error::Error;
use std::sync::Arc;
use chrono::Utc;
use teloxide::{
    prelude::*,
    Bot
};
use teloxide::types::{InputFile, ReplyParameters};
use crate::database::{Database, MediaType, Post, PostMessageId};
use crate::utils::{download_file, image_hash};

pub async fn handle_photo(bot: Bot, message: Message, db: Arc<Database>) -> Result<(), Box<dyn Error + Sync + Send>> {
    let reply_parameters = ReplyParameters::new(message.id);

    let file_meta = &message.photo().unwrap().last().unwrap().file;

    let file = match bot.get_file(&file_meta.id).await {
        Ok(file) => file,
        Err(e) => {
            log::error!("Error fetching file: {e:?}");
            return Err(e.into());
        },
    };

    let download_resp = match download_file(&file, bot.token()).await {
        Ok(resp) => resp,
        Err(e) => {
            log::error!("Error downloading photo: {e:?}");
            return Err(e.into());
        },
    };

    let hash = image_hash(download_resp.bytes().await.unwrap().as_ref())?;

    match db.get_post_by_hash(hash.clone()).await {
        Ok(Some(post)) => {
            log::warn!("Hash {hash} already exists");

            match db.add_message_id_for_post(post.id.unwrap(), PostMessageId::from(&message)).await {
                Ok(_) => {},
                Err(e) => {
                    log::error!("Error saving post message id: {e:?}");
                }
            }

            match bot
                .send_photo(message.chat.id, InputFile::file_id(post.file_id.clone()))
                .caption(format!("Duplicate from {}", post.added_datetime))
                .reply_parameters(reply_parameters)
                .await {

                Ok(msg) => {
                    log::info!("Sent duplicate notification");

                    match db.add_message_id_for_post(post.id.unwrap(), PostMessageId::from(&msg)).await {
                        Ok(_) => {},
                        Err(e) => {
                            log::error!("Error saving post message id: {e:?}");
                        }
                    }
                },
                Err(e) => {
                    log::error!("Error sending duplicate notification: {e:?}");
                    return Err(e.into());
                },
            }

            return Ok(());
        },
        Ok(None) => {},
        Err(e) => {
            log::error!("Error checking hash presence: {e:?}");
            return Err(e.into());
        }
    }

    let post = Post {
        id: None,
        media_type: MediaType::Photo,
        file_id: file.id.clone(),
        message_ids: vec![
            PostMessageId {
                chat_id: message.chat.id.0,
                message_id: message.id.0,
            },
        ],
        sent: false,
        added_datetime: Utc::now().naive_utc(),
        image_hash: Some(hash),
    };
    match db.create_post(post).await {
        Ok(_) => {
            log::info!("Post saved");
        },
        Err(e) => {
            log::error!("Error saving post: {e:?}");
        }
    }

    Ok(())
}