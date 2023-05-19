use core::panic;
use std::{error::Error, sync::Arc, todo, println, dbg};

use neo4rs::{Graph, Query, Path};
use serde_derive::{Deserialize, Serialize};

use crate::auth_nt::Creds;

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    pub obj: String,
    pub creds: Creds,
}

pub async fn dupe_acc(db: &Arc<Graph>, uu: &str) -> bool {
    let c = db
        .execute(
            Query::new(
                "match (acc:Account {username:$unm}) return count(acc) as count".to_string(),
            )
            .param("unm", uu),
        )
        .await;
    match c {
        Ok(mut rs) => {let row = rs.next().await.unwrap(); dbg!(&row); row.unwrap().get::<i64>("count").unwrap() != 0},
        Err(e) => {println!("{}", e.to_string()); panic!("")},
    }
}

pub fn hash(s: &str) -> String {
    use argon2::{
        password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
        Argon2,
    };
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(s.as_bytes(), &salt)
        .expect("Error while hashing password")
        .to_string()
}

pub async fn makeme(db: &Arc<Graph>, new: Creds) -> Result<Account, neo4rs::Error> {
    let mut c = db
        .execute(
            Query::new(
                "create x = (:Account {id:$obj, username:$unm, password:$pswd}) return x"
                    .to_string(),
            )
            .param("obj", uuid::Uuid::new_v4().to_string())
            .param("unm", new.username)
            .param("pswd", hash(&*new.password)),
        )
        .await?;

    let rs = c.next().await?;
    dbg!(&rs);
    match rs {
        Some(cr) => {
            let x = &cr.get::<Path>("x").unwrap().nodes()[0];
            
            Ok(Account {
            obj: x.get("id").unwrap(),
            creds: Creds {
                username: x.get("username").unwrap(),
                password: x.get("password").unwrap(),
            },
        })},
        None => Err(neo4rs::Error::UnexpectedMessage("Ayo wut.".to_string())),
    }
}

pub async fn get_account(db: &Arc<Graph>, username: &str) -> Result<Account, neo4rs::Error> {
    let mut c = db
        .execute(
            Query::new(
                "match (a:Account {username:$unm}) return a.id as obj, a.username as username, a.password as password".to_string(),
            )
            .param("unm", username),
        )
        .await?;

    let rs = c.next().await?;
    match rs {
        Some(row) => Ok(Account {
            obj: row.get("obj").unwrap(),
            creds: Creds {
                username: row.get("username").unwrap(),
                password: row.get("password").unwrap(),
            },
        }),
        None => Err(neo4rs::Error::AuthenticationError("..".to_string())),
    }
    // todo!()
}

/// Application specific
pub struct Database;

pub mod models {
    use chrono::Utc;

    pub struct Message{
        from: String,
        to: String,
        body: String,
        time: chrono::DateTime<Utc>
    }
}
/// To write a new application, just comment this impl out.
/// So this one is a social networking app (?)
impl Database {

    pub async fn send_msg()
}
