use sea_query::Iden;

#[derive(Iden)]
pub enum Post {
    Table,
    Id,
    MediaType,
    FileId,
    Sent,
    AddedDatetime,
    ImageHash,
}

#[derive(Iden)]
pub enum UploadTask {
    Table,
    Id,
    MediaType,
    Data,
    Processed,
    ImageHash,
}

#[derive(Iden)]
pub enum PostMessageIds {
    Table,
    ChatId,
    MessageId,
    PostId,
}

#[derive(Iden)]
pub enum State {
    Table,
    LastSent,
}
