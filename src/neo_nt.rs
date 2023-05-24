use core::panic;
use std::{assert_eq, dbg, error::Error, println, sync::Arc, todo};

use neo4rs::{Graph, Path, Query};

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
    // // type Result<T> = Result<self::models::T, neo4rs::Error>;
    // pub async fn open_chat(
    //     db: &Arc<Graph>,
    //     me: String, // comes from Identity.user_id *only*
    //     them: String,
    // ) -> Result<(), neo4rs::Error> {
    //     db.run(
    //         Query::new(
    //             "match (me:Account {id:$i}) match (them: Account {username:$u}) create (me) -[:Member]-> (c:Chat {members:[$i, them.id]}) <-[:Member]- (them)".to_string()
    //         )
    //         .param("i", me)
    //         .param("u", them)
    //     )
    //     .await
    // }

    // pub async fn send_msg(
    //     db: &Arc<Graph>,
    //     me: String, // comes from Identity.user_id *only*
    //     them: String,
    //     body: String,
    // ) -> Result<(), neo4rs::Error> {
    //     db.run(
    //         Query::new(
    //             "match (from: Account {id:$f}) -[:Member]-> (c: Chat) <-[:Member]- (to: Account {username:$t}) create (c) -[:Data]-> (:Message {from:$f, to:$t, body:$b, time:$t})".to_string()
    //         )
    //         .param("f", me)
    //         .param("t", them)
    //         .param("b", body)
    //         .param("ti", chrono::offset::Utc::now().to_string())
    //     )
    //     .await
    // }

    /// Creates a new service.
    /// Called only by Admin. Redundant arg performs dev-time check.
    pub async fn new_service(
        db: &Arc<Graph>,
        me: Identity,
        topic: self::models::Topic,
    ) -> Result<self::models::Service, neo4rs::Error> {
        if me.auth != Authority::Admin {
            return Err(neo4rs::Error::AuthenticationError(
                "Remove this".to_string(),
            ));
        }
        let q = db.execute(
            Query::new(
                "create x = (s: Service {id: $sid}) -[: Serves {type: 'eh'}]-> (t: Topic {id: $tid}) return x"
                    .to_string(),
            )
            .param("sid", uuid::Uuid::new_v4().to_string())
            .param("tid", topic.id),
        )
        .await;

        match q {
            Ok(mut row) => {
                // let r = row.next().await.unwrap();
                let r = row.next().await?.unwrap();
                let x = r.get::<Path>("x").unwrap();
                Ok(self::models::Service {
                    id: x.nodes()[0].get("id").unwrap(),
                    typ: (&*x.rels()[0].get::<String>("type").unwrap()).into(),
                    topic: models::Topic {
                        id: x.nodes()[1].get("id").unwrap(),
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
        me: Identity,
        topic: self::models::Topic,
    ) -> Result<models::Subscription, neo4rs::Error> {
        if me.auth != Authority::Subscriber {
            return Err(neo4rs::Error::AuthenticationError(
                "Remove later.".to_string(),
            ));
        }
        let me = me.user_id;
        let q = db.execute(
            Query::new(
                "match (me: Account {id: $me}) match (this: Topic {id: $this}) create x = (me) -[out: follows {subs_id: $id, from: $from}]-> (this) return x"
                    .to_string()
            )
            .param("me", me.clone())
            .param("this", topic.id.clone())
            .param("id", uuid::Uuid::new_v4().to_string())
            .param("from", chrono::offset::Utc::now().to_string())
        )
        .await;
        match q {
            Ok(mut rs) => {
                let row = rs.next().await?.unwrap();
                let out = row.get::<Path>("out").unwrap();
                Ok(models::Subscription {
                    id: out.rels()[0].get("id").unwrap(),
                    topic,
                    user: me,
                })
            }
            Err(_) => Err(neo4rs::Error::ConversionError),
        }
        // todo!()
    }
}
