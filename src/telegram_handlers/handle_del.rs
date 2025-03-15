use crate::database::Database;
use std::sync::Arc;
use teloxide::{prelude::*, types::ReplyParameters};

pub async fn handle_del(bot: Bot, message: Message, db: Arc<Database>) -> anyhow::Result<()> {
    let Some(reply_message) = message.reply_to_message() else {
        let reply_parameters = ReplyParameters::new(message.id);
        bot.send_message(message.chat.id, "Reply required")
            .reply_parameters(reply_parameters)
            .await?;
        return Ok(());
    };

    let reply_parameters = ReplyParameters::new(reply_message.id);

    match db
        .fetch_post_by_message_id(reply_message.chat.id.0, reply_message.id.0)
        .await
    {
        Ok(Some(post)) => match db.delete_post(post.id).await {
            Ok(_) => {
                bot.send_message(message.chat.id, "Post deleted")
                    .reply_parameters(reply_parameters)
                    .await?;
            }
            Err(e) => log::error!("failed to delete post: {e:?}"),
        },
        Ok(None) => {
            bot.send_message(message.chat.id, "Post was not found (already deleted?)")
                .reply_parameters(reply_parameters)
                .await?;
        }
        Err(e) => log::error!("failed to fetch post: {e:?}"),
    }

    Ok(())
}
