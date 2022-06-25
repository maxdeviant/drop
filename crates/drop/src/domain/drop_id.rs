use std::fmt::Display;

use chrono::{DateTime, TimeZone, Utc};
use ulid::Ulid;

/// The ID of a drop.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DropId(Ulid);

impl DropId {
    /// Returns a drop [`DropId`].
    pub fn new() -> Self {
        Self(Ulid::new())
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        Utc.timestamp(self.0.datetime().unix_timestamp(), 0)
    }
}

impl Display for DropId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_string().to_lowercase())
    }
}

impl TryFrom<String> for DropId {
    type Error = ulid::DecodeError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Self(Ulid::from_string(&value)?))
    }
}

impl TryFrom<&str> for DropId {
    type Error = ulid::DecodeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Self(Ulid::from_string(value)?))
    }
}
