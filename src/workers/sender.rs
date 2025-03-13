use crate::config::Config;
use crate::database::Database;
use std::ops::Add;
use std::sync::Arc;
use teloxide::Bot;
use tokio::time::Instant;

pub async fn run_sender(bot: Bot, db: Arc<Database>, cfg: Config) {
    loop {
        log::info!("sender iteration");
        tokio::time::sleep_until(Instant::now().add(cfg.interval)).await;
    }
}
