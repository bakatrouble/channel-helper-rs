use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Config {
    pub bot_token: String,
    pub db_name: String,
    pub admin_id: i64,
    pub allowed_sender_chats: Vec<i64>,
    pub interval: Duration,
    pub group_threshold: Option<u32>,
    pub with_api: bool,
    pub api_port: Option<u16>,
    pub upload_chat_id: Option<i64>,
}
