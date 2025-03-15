use chrono::NaiveDateTime;
use diesel::backend::Backend;
use diesel::deserialize::FromSql;
use diesel::prelude::*;
use diesel::serialize::{IsNull, Output, ToSql};
use diesel::sql_types::Text;
use diesel::sqlite::Sqlite;
use diesel::{AsExpression, FromSqlRow, deserialize, serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Default, Clone, Copy, FromSqlRow, AsExpression, Hash, Eq, PartialEq)]
#[diesel(sql_type = Text)]
pub struct UUID(pub uuid::Uuid);

impl<B: Backend> FromSql<Text, B> for UUID
where
    String: FromSql<Text, B>,
{
    fn from_sql(bytes: <B as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        let value = String::from_sql(bytes)?;
        uuid::Uuid::from_str(&value.as_str())
            .map(UUID)
            .map_err(|e| e.into())
    }
}

impl ToSql<Text, Sqlite> for UUID
where
    String: ToSql<Text, Sqlite>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        out.set_value(self.0.as_hyphenated().to_string());
        Ok(IsNull::No)
    }
}

impl From<UUID> for Uuid {
    fn from(value: UUID) -> Self {
        value.0
    }
}

impl From<Uuid> for UUID {
    fn from(value: Uuid) -> Self {
        Self { 0: value }
    }
}

#[derive(Debug, FromSqlRow, AsExpression, PartialEq, Clone)]
#[diesel(sql_type = Text)]
pub enum MediaType {
    Photo,
    Video,
    Animation,
}

impl<B: Backend> FromSql<Text, B> for MediaType
where
    String: FromSql<Text, B>,
{
    fn from_sql(bytes: <B as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        let value = String::from_sql(bytes)?;
        match value.as_str() {
            "photo" => Ok(MediaType::Photo),
            "video" => Ok(MediaType::Video),
            "anim" => Ok(MediaType::Animation),
            _ => Err("invalid MediaType variant".into()),
        }
    }
}

impl ToSql<Text, Sqlite> for MediaType
where
    String: ToSql<Text, Sqlite>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        out.set_value(match self {
            MediaType::Photo => "photo",
            MediaType::Video => "video",
            MediaType::Animation => "anim",
        });
        Ok(IsNull::No)
    }
}

impl Display for MediaType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            MediaType::Photo => "photo",
            MediaType::Video => "video",
            MediaType::Animation => "anim",
        })
    }
}

#[derive(Queryable, Selectable, Insertable, Clone)]
#[diesel(table_name = crate::database::schema::posts)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Post {
    #[diesel(serialize_as = UUID, deserialize_as = UUID)]
    pub id: Uuid,
    pub media_type: MediaType,
    pub file_id: String,
    pub is_sent: bool,
    pub created_datetime: NaiveDateTime,
    pub sent_datetime: Option<NaiveDateTime>,
    pub image_hash: Option<String>,
}

impl Post {
    pub fn is_photo(&self) -> bool {
        self.media_type == MediaType::Photo
    }
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::database::schema::upload_tasks)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct UploadTask {
    #[diesel(serialize_as = UUID, deserialize_as = UUID)]
    pub id: Uuid,
    pub media_type: MediaType,
    pub data: Vec<u8>,
    pub is_processed: bool,
    pub created_datetime: NaiveDateTime,
    pub processed_datetime: Option<NaiveDateTime>,
    pub image_hash: Option<String>,
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::database::schema::post_message_ids)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct PostMessageId {
    pub chat_id: i64,
    pub message_id: i32,
    #[diesel(serialize_as = UUID, deserialize_as = UUID)]
    pub post_id: Uuid,
}
