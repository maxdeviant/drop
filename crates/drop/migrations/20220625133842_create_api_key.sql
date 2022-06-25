create table api_key (
    id text primary key not null,
    created_at text not null,
    updated_at text not null,
    user_id text references user (id),
    value text not null,
    expired_at text
);
