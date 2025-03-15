use teloxide::{Bot, prelude::*, types::ReplyParameters};

pub async fn handle_unknown(bot: Bot, message: Message) -> anyhow::Result<()> {
    let reply_parameters = ReplyParameters::new(message.id);

    bot.send_message(message.chat.id, "Unknown message type")
        .reply_parameters(reply_parameters)
        .await?;

    Ok(())
}
