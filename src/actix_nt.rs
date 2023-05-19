use std::todo;

use actix_web::{post, web::{Data, Path, Json}, Responder, HttpResponse};

use crate::{State, auth_nt::Identity, neo_nt::Database};

#[post("/messages/{them}")]
pub async fn send_msg(ctx: Data<State>, me: Identity, them: Path<String>, body: Json<String>) -> impl Responder {
    let q = Database::send_msg(&ctx.graph, me.user_id, them.to_string(), body.to_string()).await;
    match q {
        Ok(()) => HttpResponse::Ok().json("sent"),
        Err(e) => HttpResponse::NotAcceptable().json(e.to_string())
    }
}

#[post("/chats/{them}")]
pub async fn open_chat(ctx: Data<State>, me: Identity, them: Path<String>) -> impl Responder {
    let q = Database::open_chat(&ctx.graph, me.user_id, them.to_string()).await;
    match q {
        Ok(()) => HttpResponse::Ok().json("opened"),
        Err(e) => HttpResponse::NotAcceptable().json(e.to_string())
    }
}
