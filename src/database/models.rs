use rusqlite::Row;
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ValueRef};
use std::fmt::{Display, Formatter};
use teloxide::types::Message;
use uuid::Uuid;

#[derive(Debug, Copy, Clone)]
pub enum MediaType {
    Photo,
    Video,
    Animation,
}

impl Into<sea_query::Value> for MediaType {
    fn into(self) -> sea_query::Value {
        sea_query::Value::String(match self {
            MediaType::Photo => Some(Box::new("photo".to_string())),
            MediaType::Video => Some(Box::new("video".to_string())),
            MediaType::Animation => Some(Box::new("anim".to_string())),
        })
    }
}

impl FromSql for MediaType {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value.as_str()? {
            "photo" => FromSqlResult::Ok(Self::Photo),
            "video" => FromSqlResult::Ok(Self::Video),
            "anim" => FromSqlResult::Ok(Self::Video),
            _ => FromSqlResult::Err(FromSqlError::InvalidType),
        }
    }
}

impl Display for MediaType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaType::Photo => f.write_str("MediaType[Photo]"),
            MediaType::Video => f.write_str("MediaType[Video]"),
            MediaType::Animation => f.write_str("MediaType[Animation]"),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PostMessageId {
    pub chat_id: i64,
    pub message_id: i32,
}

impl From<&Row<'_>> for PostMessageId {
    fn from(row: &Row<'_>) -> Self {
        Self {
            chat_id: row.get_unwrap("chat_id"),
            message_id: row.get_unwrap("message_id"),
        }
    }
}

impl From<&Message> for PostMessageId {
    fn from(value: &Message) -> Self {
        Self {
            chat_id: value.chat.id.0,
            message_id: value.id.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Post {
    pub id: Option<Uuid>,
    pub media_type: MediaType,
    pub file_id: String,
    pub message_ids: Vec<PostMessageId>,
    pub sent: bool,
    pub added_datetime: chrono::NaiveDateTime,
    pub image_hash: Option<String>,
}

impl From<&Row<'_>> for Post {
    fn from(row: &Row<'_>) -> Self {
        Self {
            id: row.get_unwrap("id"),
            media_type: row.get_unwrap("media_type"),
            file_id: row.get_unwrap("file_id"),
            message_ids: vec![],
            sent: row.get_unwrap("sent"),
            added_datetime: row.get_unwrap("added_datetime"),
            image_hash: row.get_unwrap("image_hash"),
        }
    }
}

#[derive(Debug)]
pub struct UploadTask {
    pub id: Option<Uuid>,
    pub media_type: MediaType,
    pub data: Vec<u8>,
    pub processed: bool,
    pub image_hash: Option<String>,
}

impl From<&Row<'_>> for UploadTask {
    fn from(row: &Row<'_>) -> Self {
        Self {
            id: row.get_unwrap("id"),
            media_type: row.get_unwrap("media_type"),
            data: row.get_unwrap("data"),
            processed: row.get_unwrap("processed"),
            image_hash: row.get_unwrap("image_hash"),
        }
    }
}
