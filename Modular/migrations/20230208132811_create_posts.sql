-- Add migration script here

CREATE TABLE IF NOT EXISTS posts (
    uuid UUID primary key,
    user_uuid UUID NOT NULL,
    post_type integer not null default 0,
    content varchar not null unique,
    created_at timestamptz not null default current_timestamp,
    foreign key (user_uuid) references "users" (uuid)
);