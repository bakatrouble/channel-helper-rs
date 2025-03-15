use crate::database::{Database, MediaType};
use std::error::Error;
use std::sync::Arc;
use teloxide::prelude::*;

pub async fn handle_video(
    message: Message,
    db: Arc<Database>,
) -> Result<(), Box<dyn Error + Sync + Send>> {
    let file_meta = &message.video().unwrap().file;

    let create_post_future = db.create_post(
        None,
        MediaType::Video,
        file_meta.id.clone(),
        None,
        message.chat.id.0,
        message.id.0,
    );
    match create_post_future.await {
        Ok(_) => {
            log::info!("Post saved");
        }
        Err(e) => {
            log::error!("Error saving post: {e:?}");
        }
    }

    Ok(())
}
