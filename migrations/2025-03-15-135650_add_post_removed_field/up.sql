alter table posts add column deleted bool not null default false;

create index posts_deleted_idx on posts(deleted);
