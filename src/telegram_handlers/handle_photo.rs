use crate::database::{Database, MediaType};
use crate::utils::{download_file, image_hash};
use std::error::Error;
use std::sync::Arc;
use teloxide::types::{InputFile, ReactionType, ReplyParameters};
use teloxide::{Bot, prelude::*};

pub async fn handle_photo(
    bot: Bot,
    message: Message,
    db: Arc<Database>,
) -> Result<(), Box<dyn Error + Sync + Send>> {
    let reply_parameters = ReplyParameters::new(message.id);

    let file_meta = &message.photo().unwrap().last().unwrap().file;

    let file = match bot.get_file(&file_meta.id).await {
        Ok(file) => file,
        Err(e) => {
            log::error!("Error fetching file: {e:?}");
            return Err(e.into());
        }
    };

    let download_resp = match download_file(&file, bot.token()).await {
        Ok(resp) => resp,
        Err(e) => {
            log::error!("Error downloading photo: {e:?}");
            return Err(e.into());
        }
    };

    let hash = image_hash(download_resp.bytes().await.unwrap().as_ref())?;

    if !message.caption().unwrap_or("").contains("force") {
        match db.get_post_by_hash(hash.clone()).await {
            Ok(Some(post)) => {
                log::warn!("Hash {hash} already exists");

                match db
                    .add_message_id_for_post(post.id, message.chat.id.0, message.id.0)
                    .await
                {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("Error saving post message id: {e:?}");
                    }
                }

                match bot
                    .send_photo(message.chat.id, InputFile::file_id(post.file_id.clone()))
                    .caption(format!(
                        "Duplicate from {}",
                        post.created_datetime.and_utc().to_rfc3339()
                    ))
                    .reply_parameters(reply_parameters)
                    .await
                {
                    Ok(msg) => {
                        log::info!("Sent duplicate notification");

                        match db
                            .add_message_id_for_post(post.id, msg.chat.id.0, msg.id.0)
                            .await
                        {
                            Ok(_) => {}
                            Err(e) => {
                                log::error!("Error saving post message id: {e:?}");
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Error sending duplicate notification: {e:?}");
                        return Err(e.into());
                    }
                }

                return Ok(());
            }
            Ok(None) => {}
            Err(e) => {
                log::error!("Error checking hash presence: {e:?}");
                return Err(e.into());
            }
        }
    }

    let create_post_future = db.create_post(
        None,
        MediaType::Photo,
        file_meta.id.clone(),
        Some(hash.clone()),
        message.chat.id.0,
        message.id.0,
    );
    match create_post_future.await {
        Ok(_) => {
            log::info!("Post saved");

            bot.set_message_reaction(message.chat.id, message.id)
                .reaction(vec![ReactionType::Emoji {
                    emoji: "👍".to_string(),
                }])
                .await?;
        }
        Err(e) => {
            log::error!("Error saving post: {e:?}");
        }
    };

    Ok(())
}
