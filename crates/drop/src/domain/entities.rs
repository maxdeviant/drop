use std::fmt::Display;

use chrono::{DateTime, TimeZone, Utc};
use ulid::Ulid;

fn unprefix_id(value: &str) -> &str {
    value.split('_').last().to_owned().unwrap_or(value)
}

macro_rules! entity_id {
    ($entity_id:ty, $prefix:expr) => {
        impl $entity_id {
            pub fn new() -> Self {
                Self(Ulid::new())
            }

            pub fn unprefixed(&self) -> String {
                self.0.to_string().to_lowercase()
            }

            pub fn created_at(&self) -> DateTime<Utc> {
                Utc.timestamp(self.0.datetime().unix_timestamp(), 0)
            }
        }

        impl Display for $entity_id {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}_{}", $prefix, self.0.to_string().to_lowercase())
            }
        }

        impl TryFrom<String> for $entity_id {
            type Error = ulid::DecodeError;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                let value = unprefix_id(&value);

                Ok(Self(Ulid::from_string(value)?))
            }
        }

        impl TryFrom<&str> for $entity_id {
            type Error = ulid::DecodeError;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                let value = unprefix_id(value);

                Ok(Self(Ulid::from_string(value)?))
            }
        }
    };
}

/// The ID of a drop.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DropId(Ulid);

entity_id!(DropId, "drop");

/// The ID of a user.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UserId(Ulid);

entity_id!(UserId, "user");

#[derive(Debug)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub full_name: Option<String>,
}

/// The ID of an API key.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ApiKeyId(Ulid);

entity_id!(ApiKeyId, "key");
