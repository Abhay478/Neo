use super::*;
use std::fmt::Display;

use async_graphql::{Enum, InputObject, SimpleObject};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct Frame {
    pub id: String,
    pub body: Page,
    pub time: String,
    pub by: String, // the service that wrote it.
}

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject, InputObject)]
#[graphql(input_name = "InputPage")]
/// Add more fields?
pub struct Page {
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
/// For now, only so many fields. May add more.
pub struct Topic {
    pub id: String,
    pub info: TopicInfo,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
/// This struct contains any miscellaneous information, like #subs, #pages, maybe a short description, etc.
/// ~TODO: The cypher to actually update this.
pub struct TopicInfo {
    pub pages: i64,
    pub subs: i64,
    pub time: String,
    pub desc: String,
}

#[derive(Debug, Clone, Serialize, Copy, PartialEq, Eq, Deserialize, Enum)]
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

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject, InputObject)]
#[graphql(input_name = "InputService")]
pub struct Service {
    pub id: String, // uuid
    pub typ: ServiceType,
    pub topic: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct FollowRequest {
    pub id: String,   // uuid
    pub user: String, // different uuid.
    pub topic: String,
}

#[derive(SimpleObject, Clone)]
pub struct TopicList {
    pub fd: Vec<Topic>,
}

#[derive(SimpleObject, Clone)]
pub struct Book {
    pub fd: Vec<Page>,
}

#[derive(SimpleObject, Clone)]
pub struct Album {
    pub fd: Vec<Frame>,
}
