use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Config {
    pub bot_token: String,
    pub db_name: String,
    pub target_chat_id: i64,
    // pub admin_id: i64,
    pub allowed_sender_chats: Vec<i64>,
    pub interval: Duration,
    pub group_threshold: i64,
    pub with_api: bool,
    pub api_port: Option<u16>,
    pub upload_chat_id: Option<i64>,
}
