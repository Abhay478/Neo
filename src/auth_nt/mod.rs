use crate::neo_nt::handlers::Database;
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
use jsonwebtoken::{decode, DecodingKey, Validation};
use jsonwebtoken::{encode, EncodingKey, Header};
use neo4rs::{Graph, Node, Path, Query};
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use std::future::{ready, Ready};
use std::sync::Arc;

pub mod handlers;
pub mod models;
