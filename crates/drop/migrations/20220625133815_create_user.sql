create table user (
    id text primary key not null,
    created_at text not null,
    updated_at text not null,
    username text not null unique,
    full_name text
);
