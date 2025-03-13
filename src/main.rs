mod telegram_handlers;
mod database;
mod utils;
mod cli;
mod config;
mod workers;

use std::sync::Arc;
use dotenvy::dotenv;
use teloxide::prelude::*;
use crate::database::Database;
use crate::workers::{run_bot, run_sender, run_server, run_uploader};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let cfg = cli::parse_args();

    pretty_env_logger::init_timed();
    log::info!("Initializing...");

    let db_path = format!("dbs/{}.sqlite3", &cfg.db_name);
    let db = Arc::new(Database::open(&db_path).await?);

    let bot = Bot::new(&cfg.bot_token);

    if cfg.with_api {
        tokio::spawn(run_server(cfg.clone(), db.clone()));
        tokio::spawn(run_uploader(bot.clone(), db.clone(), cfg.clone()));
    }

    tokio::spawn(run_sender(bot.clone(), db.clone(), cfg.clone()));

    run_bot(bot.clone(), db.clone(), cfg.clone()).await;

    Ok(())
}
