use std::sync::Arc;
use teloxide::dispatching::{Dispatcher, UpdateFilterExt};
use teloxide::{dptree, Bot};
use teloxide::prelude::*;
use teloxide::types::ReplyParameters;
use crate::config::Config;
use crate::database::Database;
use crate::telegram_handlers::{handle_animation, handle_photo, handle_unknown, handle_video};

pub async fn run_bot(bot: Bot, db: Arc<Database>, cfg: Config) {
    Dispatcher::builder(
        bot,
        Update::filter_message()
            .branch(
                dptree::filter(|msg: Message, cfg: Config| {
                    !cfg.allowed_sender_chats.contains(&msg.chat.id.0)
                }).chain(dptree::endpoint(|msg: Message, bot: Bot| async move {
                    bot.send_message(msg.chat.id, "gtfo")
                        .reply_parameters(ReplyParameters::new(msg.id))
                        .await?;
                    Ok(())
                }))
            )
            .branch(Message::filter_photo().endpoint(handle_photo))
            .branch(Message::filter_video().endpoint(handle_video))
            .branch(Message::filter_animation().endpoint(handle_animation))
            .branch(dptree::endpoint(handle_unknown))
    )
        .dependencies(dptree::deps![db.clone(), cfg.clone()])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
