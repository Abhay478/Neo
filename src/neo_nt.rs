#![allow(unused_imports)]
#![allow(dead_code)]
use core::panic;
use std::{assert_eq, dbg, error::Error, println, sync::Arc, todo};

use neo4rs::{Graph, Path, Query, RowStream, Node};

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
type NeoResult<T> = Result<T, neo4rs::Error>;
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
    ) -> NeoResult<self::models::Service> {
        // --
        if me.auth != Authority::Admin {
            return Err(neo4rs::Error::AuthenticationError(
                "Remove this".to_string(),
            ));
        }
        // --
        let mut rs = db
            .execute(
                Query::new(Self::read_query("new_service"))
                    .param("sid", uuid::Uuid::new_v4().to_string())
                    .param("tname", topic.clone()),
            )
            .await?;

        let r = rs.next().await?.unwrap();
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

    /// Create a new subscription to the given topic
    pub async fn subscribe_to(
        db: &Arc<Graph>,
        me: Identity, // change to String.
        topic: String,
    ) -> NeoResult<models::Subscription> {
        // --
        if me.auth != Authority::Subscriber {
            return Err(neo4rs::Error::AuthenticationError(
                "Remove later.".to_string(),
            ));
        }
        let me = me.user_id;
        // --
        let mut rs = db
            .execute(
                Query::new(Self::read_query("subscribe_to"))
                    .param("me", me.clone())
                    .param("this", topic.clone())
                    .param("id", uuid::Uuid::new_v4().to_string())
                    .param("from", chrono::offset::Utc::now().to_string()),
            )
            .await?;
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

    /// Also admin-only.
    pub async fn new_topic(
        db: &Arc<Graph>,
        me: Identity,
        topic: String,
    ) -> NeoResult<self::models::Topic> {
        // --
        if me.auth != Authority::Admin {
            return Err(neo4rs::Error::AuthenticationError(
                "Remove this".to_string(),
            ));
        }
        // --
        let mut rs = db
            .execute(
                Query::new(Self::read_query("new_topic"))
                    .param("id", uuid::Uuid::new_v4().to_string())
                    .param("name", topic),
            )
            .await?;

        let row = rs.next().await?.unwrap();
        let x = row.get::<Path>("t").unwrap();
        Ok(models::Topic {
            id: x.nodes()[0].get("id").unwrap(),
            name: x.nodes()[0].get("name").unwrap(),
        })
    }

    /// Remove the entire function - this is just in case dev wants to run an arbitrary query.
    pub async fn naked_query(db: &Arc<Graph>, me: Identity, query: String) -> NeoResult<RowStream> {
        if me.auth != Authority::Admin {
            return Err(neo4rs::Error::AuthenticationError(
                "Remove this".to_string(),
            ));
        }
        db.execute(Query::new(query)).await
    }

    pub async fn unsubscribe(db: &Arc<Graph>, me: String, topic: String) -> NeoResult<()> {
        db.run(
            Query::new(Self::read_query("unsubscribe"))
                .param("me", me)
                .param("tname", topic),
        )
        .await?;
        Ok(())
    }
    /// Admin-only, again.
    pub async fn kill_service(db: &Arc<Graph>, id: String) -> NeoResult<()> {
        db.run(Query::new(Self::read_query("kill_service")).param("id", id))
            .await?;
        Ok(())
    }

    /// Admin-only, again.
    pub async fn retire_topic(db: &Arc<Graph>, topic: String) -> NeoResult<()> {
        db.run(Query::new(Self::read_query("retire_topic")).param("id", topic))
            .await?;
        Ok(())
    }

    pub async fn get_topics(db: &Arc<Graph>, me: String) -> NeoResult<Vec<models::Topic>> {
        let mut rs = db
            .execute(Query::new(Self::read_query("get_topics")).param("me", me))
            .await?;

        // Wanted to do this part with maps and iterators, but apparrntly RowStream does not implement Iterator or IntoIterator.
        let mut out = vec![];
        while let Ok(Some(row)) = rs.next().await {
            let node = row.get::<Node>("t").unwrap();
            out.push(models::Topic {
                id: node.get("id").unwrap(),
                name: node.get("name").unwrap()
            });
        }

        Ok(out)
        // todo!()
    }
}
