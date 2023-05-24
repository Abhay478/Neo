#![allow(unused_imports)]
#![allow(dead_code)]
use core::panic;
use std::{assert_eq, dbg, error::Error, println, sync::Arc, todo};

use neo4rs::{Graph, Path, Query, RowStream};

use crate::auth_nt::{Authority, Creds, Identity};

pub mod models {
    use serde_derive::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Message {
        from: String,
        to: String,
        body: String,
        time: String, // chrono::DateTime<Utc> does not derive Serde. Sigh.
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

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Service {
        pub id: String, // uuid
        pub typ: ServiceType,
        pub topic: Topic,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Subscription {
        pub id: String,   // uuid
        pub user: String, // different uuid.
        pub topic: Topic,
    }
}

/// Application specific
pub struct Database;

/// To write a new application, just comment this impl out.
impl Database {
    /// I do NOT want to write cypher queries in strings. 
    /// They're all in .cypher files, with proper formatting and syntax highlighting.
    pub fn read_query(name: &str) -> String {
        std::fs::read_to_string(format!("src/cypher/{name}.cypher")).unwrap()
    }

    pub async fn constraints(db: &Arc<Graph>) {
        db.run(Query::new(Self::read_query("simple_graph_for_subs")))
            .await
            .unwrap();

        db.run(Query::new(Self::read_query("unique_topics")))
            .await
            .unwrap();

        db.run(Query::new(Self::read_query("simple_graph_for_services")))
            .await
            .unwrap();
    }

    /// Creates a new service.
    /// Called only by Admin. Redundant arg performs dev-time check.
    pub async fn new_service(
        db: &Arc<Graph>,
        me: Identity,
        // topic name
        topic: String,
    ) -> Result<self::models::Service, neo4rs::Error> {
        // --
        if me.auth != Authority::Admin {
            return Err(neo4rs::Error::AuthenticationError(
                "Remove this".to_string(),
            ));
        }
        // --
        let q = db
            .execute(
                Query::new(Self::read_query("new_service"))
                    .param("sid", uuid::Uuid::new_v4().to_string())
                    .param("tname", topic.clone()),
            )
            .await;

        match q {
            Ok(mut row) => {
                let r = row.next().await?.unwrap();
                let x = r.get::<Path>("x").unwrap();
                Ok(self::models::Service {
                    id: x.nodes()[0].get("id").unwrap(),
                    typ: (&*x.rels()[0].get::<String>("type").unwrap()).into(),
                    topic: models::Topic {
                        id: x.nodes()[1].get("id").unwrap(),
                        name: x.nodes()[1].get("name").unwrap(),
                    },
                })
            }
            Err(_) => Err(neo4rs::Error::ConversionError),
        }
        // todo!()
    }

    /// Create a new subscription to the given topic
    pub async fn subscribe_to(
        db: &Arc<Graph>,
        me: Identity, // change to String.
        topic: String,
    ) -> Result<models::Subscription, neo4rs::Error> {
        // --
        if me.auth != Authority::Subscriber {
            return Err(neo4rs::Error::AuthenticationError(
                "Remove later.".to_string(),
            ));
        }
        let me = me.user_id;
        // --
        let q = db
            .execute(
                Query::new(Self::read_query("subscribe_to"))
                    .param("me", me.clone())
                    .param("this", topic.clone())
                    .param("id", uuid::Uuid::new_v4().to_string())
                    .param("from", chrono::offset::Utc::now().to_string()),
            )
            .await;
        match q {
            Ok(mut rs) => {
                let row = rs.next().await?.unwrap();
                let out = row.get::<Path>("out").unwrap();
                Ok(models::Subscription {
                    id: out.rels()[0].get("id").unwrap(),
                    topic: models::Topic {
                        id: out.nodes()[1].get("id").unwrap(),
                        name: out.nodes()[1].get("name").unwrap(),
                    },
                    user: me,
                })
            }
            Err(_) => Err(neo4rs::Error::ConversionError),
        }
        // todo!()
    }

    pub async fn new_topic(
        db: &Arc<Graph>,
        me: Identity,
        topic: String,
    ) -> Result<self::models::Topic, neo4rs::Error> {
        // --
        if me.auth != Authority::Admin {
            return Err(neo4rs::Error::AuthenticationError(
                "Remove this".to_string(),
            ));
        }
        // --
        let q = db.execute(
            Query::new(
                Self::read_query("new_topic")
            )
            .param("id", uuid::Uuid::new_v4().to_string())
            .param("name", topic)
        )
        .await;

        match q {
            Ok(mut rs) => {
                let row = rs.next().await?.unwrap();
                let x = row.get::<Path>("t").unwrap();
                Ok(models::Topic {
                    id: x.nodes()[0].get("id").unwrap(),
                    name: x.nodes()[0].get("name").unwrap(),
                })
            },
            Err(_) => Err(neo4rs::Error::ConversionError)
        }
        // todo!()
    }

    /// Remove the entire function - this is just in case dev wants to run an arbitrary query.
    pub async fn naked_query(db: &Arc<Graph>, me: Identity, query: String) -> Result<RowStream, neo4rs::Error> {
        if me.auth != Authority::Admin {
            return Err(neo4rs::Error::AuthenticationError(
                "Remove this".to_string(),
            ));
        }
        db.execute(Query::new(query)).await
    }
}
