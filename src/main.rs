mod cli;
mod config;
mod database;
mod telegram_handlers;
mod utils;
mod workers;

use crate::{
    database::Database,
    workers::{run_bot, run_sender, run_server, run_uploader},
};
use dotenvy::dotenv;
use std::sync::Arc;
use teloxide::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let cfg = cli::parse_args();

    pretty_env_logger::init_timed();
    log::info!("Initializing...");

    let db_path = format!("dbs/{}.sqlite3", &cfg.db_name);
    let db = Arc::new(Database::open(&db_path)?);

    let bot = Bot::new(&cfg.bot_token);

    if cfg.with_api {
        tokio::spawn(run_server(cfg.clone(), db.clone()));
        tokio::spawn(run_uploader(bot.clone(), db.clone(), cfg.clone()));
    }

    tokio::spawn(run_sender(bot.clone(), db.clone(), cfg.clone()));

    run_bot(bot.clone(), db.clone(), cfg.clone()).await;

    Ok(())
}
