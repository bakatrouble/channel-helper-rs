use crate::config::Config;
use crate::database::{Database, MediaType, Post};
use std::ops::Add;
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::{InputFile, InputMedia, InputMediaPhoto};
use tokio::time::Instant;

pub async fn run_sender(bot: Bot, db: Arc<Database>, cfg: Config) {
    loop {
        log::info!("sender iteration");

        let unsent_posts_count = db.unsent_posts_count().await.unwrap();

        match db.fetch_unsent_post().await {
            Ok(Some(post)) => {
                if post.is_photo() && cfg.group_threshold > 0 && unsent_posts_count > cfg.group_threshold {
                    match db.fetch_ten_unsent_photo_posts().await {
                        Ok(posts) => {
                            match send_group_photo_post(posts.clone(), bot.clone(), cfg.clone()).await {
                                Ok(_) => {
                                    match db.mark_sent_posts(posts.iter().map(|p| p.id)).await {
                                        Ok(_) => log::info!("Marked as sent"),
                                        Err(e) => log::error!("Unable to mark posts as sent: {e:?}"),
                                    }
                                }
                                Err(e) => log::error!("Error sending multiple posts: {e:?}")
                            }
                        }
                        Err(e) => log::error!("Error fetching multiple posts: {e:?}"),
                    }
                } else {
                    match send_post(post.clone(), bot.clone(), cfg.clone()).await {
                        Ok(_) => {
                            match db.mark_sent_posts(vec![post.id]).await {
                                Ok(_) => log::info!("Marked as sent"),
                                Err(e) => log::error!("Unable to mark post as sent: {e:?}"),
                            }
                        }
                        Err(e) => log::error!("Error sending post: {e:?}")
                    }
                }
            }
            Ok(None) => log::info!("Nothing to send"),
            Err(e) => log::error!("Error fetching unsent posts: {e:?}")
        }

        tokio::time::sleep_until(Instant::now().add(cfg.interval)).await;
    }
}

async fn send_post(post: Post, bot: Bot, cfg: Config) -> anyhow::Result<()> {
    let recipient = ChatId(cfg.target_chat_id);
    let input_file = InputFile::file_id(post.file_id);

    match post.media_type {
        MediaType::Photo => {
            bot.send_photo(recipient, input_file).await?;
        },
        MediaType::Video => {
            bot.send_video(recipient, input_file).await?;
        },
        MediaType::Animation => {
            bot.send_animation(recipient, input_file).await?;
        },
    }
    Ok(())
}

async fn send_group_photo_post(posts: Vec<Post>, bot: Bot, cfg: Config) -> anyhow::Result<()> {
    if posts.len() == 0 {
        return Ok(());
    } else if posts.len() == 1 {
        return send_post(posts[0].clone(), bot, cfg).await;
    }

    let recipient = ChatId(cfg.target_chat_id);
    let group: Vec<InputMedia> = posts.iter()
        .map(|p| InputMedia::Photo(InputMediaPhoto::new(InputFile::file_id(&p.file_id))))
        .collect();

    bot.send_media_group(recipient, group).await?;
    Ok(())
}
