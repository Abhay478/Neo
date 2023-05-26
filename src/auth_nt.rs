use crate::neo_nt::Database;
use crate::State;
use actix_web::error::ErrorUnauthorized;
use actix_web::{
    cookie::{time::Duration as AWD, Cookie},
    get, post, HttpResponse, Responder,
};
use actix_web::{dev::Payload, Error as ActixWebError};
use actix_web::{http, web, FromRequest, HttpMessage, HttpRequest};
use argon2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Argon2,
};
use chrono::{prelude::*, Duration};
use core::fmt;
use jsonwebtoken::{decode, DecodingKey, Validation};
use jsonwebtoken::{encode, EncodingKey, Header};
use neo4rs::{Graph, Node, Path, Query};
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use std::future::{ready, Ready};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    pub obj: String,
    pub creds: Creds,
}

/// Maybe more fields, like #topics subscribed to, active_since?
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub username: String,
    pub disp_name: String,
}

pub async fn dupe_acc(db: &Arc<Graph>, uu: &str) -> bool {
    let c = db
        .execute(Query::new(Database::read_query("dupe_acc")).param("unm", uu))
        .await;
    match c {
        Ok(mut rs) => {
            let row = rs.next().await.unwrap();
            dbg!(&row);
            row.unwrap().get::<i64>("count").unwrap() != 0
        }
        Err(e) => {
            println!("{}", e.to_string());
            panic!("")
        }
    }
}

pub fn hash(s: &str) -> String {
    use argon2::password_hash::{rand_core::OsRng, PasswordHasher, SaltString};
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(s.as_bytes(), &salt)
        .expect("Error while hashing password")
        .to_string()
}

pub async fn makeme(db: &Arc<Graph>, new: Creds) -> Result<Account, neo4rs::Error> {
    let mut c = db
        .execute(
            Query::new(Database::read_query("makeme"))
                .param("obj", uuid::Uuid::new_v4().to_string())
                .param("unm", new.username)
                .param("pswd", hash(&*new.password))
                .param("dnm", new.disp_name),
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
                    disp_name: x.get("disp_name").unwrap(),
                    auth: (&*x.get::<String>("auth").unwrap()).into(),
                },
            })
        }
        None => Err(neo4rs::Error::UnexpectedMessage("Ayo wut.".to_string())),
    }
}

pub async fn get_account(db: &Arc<Graph>, username: &str) -> Result<Account, neo4rs::Error> {
    let mut c = db
        .execute(Query::new(Database::read_query("get_account")).param("unm", username))
        .await?;

    let rs = c.next().await?;
    match rs {
        Some(row) => {
            let x = row.get::<Node>("a").unwrap();
            Ok(Account {
                obj: x.get("obj").unwrap(),
                creds: Creds {
                    username: x.get("username").unwrap(),
                    password: x.get("password").unwrap(),
                    disp_name: x.get("disp_name").unwrap(),
                    auth: (&*x.get::<String>("auth").unwrap()).into(),
                },
            })
        }
        None => Err(neo4rs::Error::AuthenticationError("..".to_string())),
    }
    // todo!()
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
/// Will have to add intermediate authorities, obviously.
pub enum Authority {
    Unknown,
    Subscriber,
    Admin,
}

impl From<&str> for Authority {
    // type Err = neo4rs::Error;
    fn from(s: &str) -> Self {
        match s {
            "Subscriber" => Self::Subscriber,
            "Admin" => Self::Admin,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub auth: Authority,
    pub iat: usize,
    pub exp: usize,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    status: String,
    message: String,
}

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
    }
}

pub struct Identity {
    pub user_id: String,
    pub auth: Authority,
}

impl FromRequest for Identity {
    type Error = ActixWebError;
    type Future = Ready<Result<Self, Self::Error>>;
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let data = req.app_data::<web::Data<State>>().unwrap();

        let token = req
            .cookie("token")
            .map(|c| c.value().to_string())
            .or_else(|| {
                req.headers()
                    .get(http::header::AUTHORIZATION)
                    .map(|h| h.to_str().unwrap().split_at(7).1.to_string())
            });

        if token.is_none() {
            let json_error = ErrorResponse {
                status: "fail".to_string(),
                message: "Roses are red,\n violets are blue,\n please enter your credentials,\n we'd love to have you.".to_string(),
            };
            return ready(Err(ErrorUnauthorized(json_error)));
        }

        let claims = match decode::<TokenClaims>(
            &token.unwrap(),
            &DecodingKey::from_secret(data.env.jwt_secret.as_ref()),
            &Validation::default(),
        ) {
            Ok(c) => c.claims,
            Err(_) => {
                let json_error = ErrorResponse {
                    status: "fail".to_string(),
                    message: "I find your lack of faith...disturbing".to_string(),
                };
                return ready(Err(ErrorUnauthorized(json_error)));
            }
        };

        let user_id = claims.sub;
        let auth = claims.auth;
        req.extensions_mut().insert::<String>(user_id.clone());

        ready(Ok(Identity { user_id, auth }))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Creds {
    pub username: String,
    pub password: String,
    pub disp_name: String,
    pub auth: Authority,
}

#[post("/auth/register")]
pub async fn register(mut body: web::Json<Creds>, data: web::Data<State>) -> impl Responder {
    let db = &data.graph;
    let exists = dupe_acc(db, &body.username).await;
    if exists {
        return HttpResponse::Conflict()
            .json(serde_json::json!({"status": "fail","message": "Doppleganger alert."}));
    }

    // Empty password means set to username.
    if body.password == "" {
        body.password = body.username.clone();
    }

    let res = makeme(db, body.0).await;

    match res {
        Ok(user) => {
            let token = get_token(&user.obj.to_string(), data, user.creds.auth);

            let cookie = Cookie::build("token", token.to_owned())
                .path("/")
                .max_age(AWD::new(60 * 60, 0))
                .http_only(true)
                .finish();
            HttpResponse::Ok().cookie(cookie).json(user)
        }
        Err(e) => HttpResponse::InternalServerError()
            .json(serde_json::json!({"status": "error","message": format!("{:?}", e)})),
    }
}

fn get_token(id: &str, data: web::Data<State>, auth: Authority) -> String {
    let now = Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + Duration::minutes(60)).timestamp() as usize;
    let claims = TokenClaims {
        sub: id.to_string(),
        auth,
        exp,
        iat,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(data.env.jwt_secret.as_ref()),
    )
    .unwrap()
}

#[post("/auth/login")]
pub async fn login(body: web::Json<Creds>, data: web::Data<State>) -> impl Responder {
    let db = &data.graph;

    let query_result = get_account(db, &*body.username).await;

    match &query_result {
        Ok(user) => {
            let othertemp = body.clone().password;
            let temp = &user.clone().creds.password;
            let hash = PasswordHash::new(&temp).unwrap();
            let is_valid = Argon2::default()
                .verify_password(othertemp.as_bytes(), &hash)
                .is_ok();

            if !is_valid {
                // wrong password
                return HttpResponse::BadRequest()
                    .json(json!({"status": "fail", "message": "These are not the droids we are looking for."}));
            }
        }
        Err(_e) => {
            // user not found
            return HttpResponse::NotFound()
                .json(json!({"status": "fail", "message": "No record."}));
        }
    }

    let user = query_result.unwrap();

    let token = get_token(&user.obj, data, user.creds.auth);

    let cookie = Cookie::build("token", token.to_owned())
        .path("/")
        .max_age(AWD::new(60 * 60, 0))
        .http_only(true)
        .finish();

    HttpResponse::Ok()
        .cookie(cookie)
        .json(json!({"status": "success", "token": token}))
}

#[get("/auth/logout")]
pub async fn logout(_: Identity) -> impl Responder {
    let cookie = Cookie::build("token", "")
        .path("/")
        .max_age(AWD::new(-1, 0))
        .http_only(true)
        .finish();

    HttpResponse::Ok()
        .cookie(cookie)
        .json(json!({"status": "success"}))
}
