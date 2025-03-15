use crate::config::Config;
use crate::database::Database;
use crate::telegram_handlers::{
    handle_animation, handle_del, handle_photo, handle_unknown, handle_video,
};
use std::sync::Arc;
use teloxide::dispatching::{Dispatcher, UpdateFilterExt};
use teloxide::dptree::case;
use teloxide::macros::BotCommands;
use teloxide::prelude::*;
use teloxide::types::ReplyParameters;
use teloxide::{Bot, dptree};

#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "lowercase")]
pub enum Commands {
    #[command(aliases = ["del", "delete", "rem", "remove"])]
    Delete,
}

pub async fn run_bot(bot: Bot, db: Arc<Database>, cfg: Config) {
    Dispatcher::builder(
        bot,
        Update::filter_message()
            .branch(
                Update::filter_message()
                    .filter(|msg: Message, cfg: Config| {
                        !cfg.allowed_sender_chats.contains(&msg.chat.id.0)
                    })
                    .endpoint(|msg: Message, bot: Bot| async move {
                        bot.send_message(msg.chat.id, "gtfo")
                            .reply_parameters(ReplyParameters::new(msg.id))
                            .await?;
                        Ok(())
                    }),
            )
            .branch(
                Update::filter_message()
                    .filter_command::<Commands>()
                    .branch(case![Commands::Delete].endpoint(handle_del)),
            )
            .branch(Message::filter_photo().endpoint(handle_photo))
            .branch(Message::filter_video().endpoint(handle_video))
            .branch(Message::filter_animation().endpoint(handle_animation))
            .branch(dptree::endpoint(handle_unknown)),
    )
    .dependencies(dptree::deps![db.clone(), cfg.clone()])
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;
}
