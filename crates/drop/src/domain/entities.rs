use chrono::{DateTime, TimeZone, Utc};
use entity_id::EntityId;
use ulid::Ulid;

/// The ID of a drop.
#[derive(EntityId, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[entity_id(prefix = "drop")]
pub struct DropId(Ulid);

impl DropId {
    pub fn created_at(&self) -> DateTime<Utc> {
        Utc.timestamp(self.0.datetime().unix_timestamp(), 0)
    }
}

/// The ID of a user.
#[derive(EntityId, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[entity_id(prefix = "user")]
pub struct UserId(Ulid);

#[derive(Debug)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub full_name: Option<String>,
}

/// The ID of an API key.
#[derive(EntityId, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[entity_id(prefix = "key")]
pub struct ApiKeyId(Ulid);
