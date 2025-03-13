use std::time::Duration;
use clap::{arg, value_parser, ArgAction, Command};
use crate::config::Config;

pub fn parse_args() -> Config {
    let matches = Command::new("channel-helper-rs")
        .arg(
            arg!(-t --token <BOT_TOKEN>)
                .id("bot_token")
                .env("BOT_TOKEN")
                .value_parser(value_parser!(String))
                .required(true)
        )
        .arg(
            arg!(-d --db <DB_NAME>)
                .id("db_name")
                .env("DB_NAME")
                .value_parser(value_parser!(String))
                .required(true)
        )
        .arg(
            arg!(-a --admin <ADMIN_ID>)
                .id("admin_id")
                .env("ADMIN_ID")
                .value_parser(value_parser!(i64))
                .required(true)
        )
        .arg(
            arg!(-s --senders <ALLOWED_SENDERS>)
                .id("allowed_senders")
                .env("ALLOWED_SENDERS")
                .value_delimiter(',')
                .required(true)
        )
        .arg(
            arg!(-i --interval)
                .id("interval")
                .env("INTERVAL")
                .value_parser(humantime::parse_duration)
                .required(true)
        )
        .arg(
            arg!(-w --api)
                .id("with_api")
                .env("WITH_API")
                .action(ArgAction::SetTrue)
                .requires("api_port")
                .requires("upload_chat_id")
        )
        .arg(
            arg!(-p --port <API_PORT>)
                .id("api_port")
                .env("API_PORT")
                .value_parser(value_parser!(u16))
                .required(false)
        )
        .arg(
            arg!(-u --"upload-chat" <UPLOAD_CHAT_ID>)
                .id("upload_chat_id")
                .env("UPLOAD_CHAT_ID")
                .value_parser(value_parser!(i64))
                .required(false)
        )
        .get_matches();
    let bot_token = matches.get_one::<String>("bot_token").unwrap();
    let db_name = matches.get_one::<String>("db_name").unwrap();
    let admin_id = matches.get_one::<i64>("admin_id").unwrap();
    let allowed_sender_chats: Vec<i64> = matches.get_many::<String>("allowed_senders")
        .unwrap()
        .map(|v| v.parse().unwrap())
        .collect();
    let interval = matches.get_one::<Duration>("interval").unwrap();
    let with_api = matches.get_one::<bool>("with_api").unwrap();
    let api_port = matches.get_one::<u16>("api_port");
    let upload_chat_id = matches.get_one::<i64>("upload_chat_id");
    Config {
        bot_token: bot_token.clone(),
        db_name: db_name.clone(),
        admin_id: admin_id.clone(),
        allowed_sender_chats,
        interval: interval.clone(),
        with_api: with_api.clone(),
        api_port: api_port.map(|v| *v).clone(),
        upload_chat_id: upload_chat_id.map(|v| *v).clone(),
    }
}