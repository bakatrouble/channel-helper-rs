use std::error::Error;
use teloxide::Bot;
use teloxide::prelude::*;
use teloxide::types::ReplyParameters;

pub async fn handle_unknown(bot: Bot, message: Message) -> Result<(), Box<dyn Error + Sync + Send>> {
    let reply_parameters = ReplyParameters::new(message.id);

    bot.send_message(message.chat.id, "Unknown message type")
        .reply_parameters(reply_parameters)
        .await?;

    Ok(())
}
