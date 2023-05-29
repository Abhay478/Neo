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
    auth_nt::models::Identity,
    graphql_nt::{Mutation, Query, Subscription},
    neo_nt::handlers::Database,
    State,
};

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
