#![allow(unused_imports, dead_code)]
use std::{time::Duration, todo};

use actix_web::{
    post, route,
    web::{self, Data, Json, Path},
    HttpRequest, HttpResponse, Responder,
};

use async_graphql::{http::GraphiQLSource, Schema};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
// use juniper_graphql_ws::ConnectionConfig;

use crate::{
    auth_nt::Identity,
    graphql_nt::{Mutation, Query, Subscription},
    neo_nt::Database,
    State,
};

// #[post("/messages/{them}")]
// pub async fn send_msg(ctx: Data<State>, me: Identity, them: Path<String>, body: Json<String>) -> impl Responder {
//     let q = Database::send_msg(&ctx.graph, me.user_id, them.to_string(), body.to_string()).await;
//     match q {
//         Ok(()) => HttpResponse::Ok().json("sent"),
//         Err(e) => HttpResponse::NotAcceptable().json(e.to_string())
//     }
// }

// #[post("/chats/{them}")]
// pub async fn open_chat(ctx: Data<State>, me: Identity, them: Path<String>) -> impl Responder {
//     let q = Database::open_chat(&ctx.graph, me.user_id, them.to_string()).await;
//     match q {
//         Ok(()) => HttpResponse::Ok().json("opened"),
//         Err(e) => HttpResponse::NotAcceptable().json(e.to_string())
//     }
// }
// type Schema = RootNode<'static, Query, Mutation, Subscription>;

type Sch = Schema<Query, Mutation, Subscription>;

#[route("/pg", method = "GET", method = "POST")]
async fn playground(_: Identity) -> impl Responder {
    HttpResponse::Ok()
        .content_type("html; charset=utf-8")
        .body(GraphiQLSource::build().endpoint("/").finish())
}

#[post("/gql")]
async fn graphql(req: GraphQLRequest, schema: web::Data<Sch>, me: Identity) -> impl Responder {
    HttpResponse::Ok().json(schema.execute(req.into_inner().data(me)).await)
}

async fn index_ws(
    schema: web::Data<Sch>,
    req: HttpRequest,
    payload: web::Payload,
    _me: Identity,
) -> impl Responder {
    GraphQLSubscription::new(Schema::clone(&*schema)).start(&req, payload)
}
