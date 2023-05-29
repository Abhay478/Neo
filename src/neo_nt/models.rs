use super::*;
use std::fmt::Display;

use serde_derive::{Deserialize, Serialize};

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct Message {
//     from: String,
//     to: String,
//     body: String,
//     time: String, // chrono::DateTime<Utc> does not derive Serde. Sigh.
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub id: String,
    pub body: Frame,
    pub time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Add more fields?
pub struct Frame {
    pub title: String,
    pub body: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
/// For now, only so many fields. May add more.
pub struct Topic {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Obviously have to add more types.
pub enum ServiceType {
    Unknown,
    Eh,
}

impl From<&str> for ServiceType {
    fn from(value: &str) -> Self {
        match value {
            "eh" => Self::Eh,
            _ => Self::Unknown,
        }
    }
}

impl Display for ServiceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Eh => write!(f, "Eh"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub id: String, // uuid
    pub typ: ServiceType,
    pub topic: Topic, // debating whether or not to remove this, so the service doesn't really know it's topic. We aren't using this field anywhere, ig.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: String,   // uuid
    pub user: String, // different uuid.
    pub topic: Topic,
}
