// @generated automatically by Diesel CLI.

diesel::table! {
    post_message_ids (rowid) {
        rowid -> Integer,
        chat_id -> BigInt,
        message_id -> Integer,
        post_id -> Text,
    }
}

diesel::table! {
    posts (id) {
        id -> Text,
        media_type -> Text,
        file_id -> Text,
        is_sent -> Bool,
        created_datetime -> Timestamp,
        sent_datetime -> Nullable<Timestamp>,
        image_hash -> Nullable<Text>,
        deleted -> Bool,
    }
}

diesel::table! {
    upload_tasks (id) {
        id -> Text,
        media_type -> Text,
        data -> Binary,
        is_processed -> Bool,
        created_datetime -> Timestamp,
        processed_datetime -> Nullable<Timestamp>,
        image_hash -> Nullable<Text>,
    }
}

diesel::joinable!(post_message_ids -> posts (post_id));

diesel::allow_tables_to_appear_in_same_query!(post_message_ids, posts, upload_tasks,);
