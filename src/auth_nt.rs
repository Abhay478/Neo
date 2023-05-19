use core::fmt;
use std::future::{ready, Ready};

// use crate::diesel_ch::{dupe_acc, get_creds, makeme, Creds};
use crate::neo_nt::{dupe_acc, get_account, makeme};
use crate::State;
use actix_web::error::ErrorUnauthorized;
use actix_web::{dev::Payload, Error as ActixWebError};
use actix_web::{http, web, FromRequest, HttpMessage, HttpRequest};

use jsonwebtoken::{decode, DecodingKey, Validation};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
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

pub struct JwtMiddleware {
    pub user_id: String,
}

impl FromRequest for JwtMiddleware {
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
        req.extensions_mut().insert::<String>(user_id.clone());

        ready(Ok(JwtMiddleware { user_id }))
    }
}

use actix_web::{
    cookie::{time::Duration as AWD, Cookie},
    get, post, HttpResponse, Responder,
};
use argon2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Argon2,
};
use chrono::{prelude::*, Duration};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Creds {
    pub username: String,
    pub password: String,
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
            let token = get_token(&user.obj.to_string(), data);

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

fn get_token(id: &str, data: web::Data<State>) -> String {
    let now = Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + Duration::minutes(60)).timestamp() as usize;
    let claims = TokenClaims {
        sub: id.to_string(),
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

    let token = get_token(&user.obj, data);

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
pub async fn logout(_: JwtMiddleware) -> impl Responder {
    let cookie = Cookie::build("token", "")
        .path("/")
        .max_age(AWD::new(-1, 0))
        .http_only(true)
        .finish();

    HttpResponse::Ok()
        .cookie(cookie)
        .json(json!({"status": "success"}))
}
