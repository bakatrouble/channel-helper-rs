create table posts (
    id uuid_text not null primary key,
    media_type media_type_text not null,
    file_id text not null,
    is_sent boolean not null default false,
    created_datetime timestamp not null default current_timestamp,
    sent_datetime timestamp null,
    image_hash text null
);

create index posts_media_type_idx on posts(media_type);
create index posts_is_sent_idx on posts(is_sent);
create index posts_image_hash_idx on posts(image_hash);

create table upload_tasks (
    id uuid_text not null primary key,
    media_type media_type_text not null,
    data blob not null,
    is_processed boolean not null default false,
    created_datetime timestamp not null default current_timestamp,
    processed_datetime timestamp null,
    image_hash text null
);

create index upload_tasks_processed_idx on upload_tasks(is_processed);

create table post_message_ids (
    chat_id bigint not null,
    message_id integer not null,
    post_id uuid_text not null,
    foreign key (post_id) references posts(id)
);

create unique index post_message_ids_idx on post_message_ids(chat_id, message_id);
