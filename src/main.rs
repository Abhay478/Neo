use actix_nt::*;
use actix_web::middleware::{Compress, Logger};
use actix_web::web::Data;
use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use async_graphql::Schema;
use auth_nt::handlers::*;
use graphql_nt::{Mutation, Query, Subscription};
use neo4rs::*;
use neo_nt::handlers::Database;
use std::sync::Arc;
use std::{env, io};

mod actix_nt;
mod auth_nt;
mod graphql_nt;
mod neo_nt;

async fn connect() -> Arc<Graph> {
    env::set_var("RUST_LOG", "debug");
    dotenvy::dotenv().ok();
    env_logger::init();
    // let uri = "127.0.0.1:7687";
    let uri = env::var("DATABASE_URI").unwrap();
    // let unm = "neo4j";
    let unm = env::var("USERNAME").unwrap();
    // let pswd = "neo4jabhay";
    let pswd = env::var("PASSWORD").unwrap();

    Arc::new(Graph::new(&uri, &unm, &pswd).await.unwrap())
}

#[get("/")]
async fn api() -> impl Responder {
    HttpResponse::Ok().body("1.0")
}

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expires_in: String,
    pub jwt_maxage: i32,
}

impl Config {
    pub fn init() -> Config {
        let database_url = std::env::var("DATABASE_URI").expect("DATABASE_URI must be set");
        let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let jwt_expires_in = std::env::var("JWT_EXPIRED_IN").expect("JWT_EXPIRED_IN must be set");
        let jwt_maxage = std::env::var("JWT_MAXAGE").expect("JWT_MAXAGE must be set");
        Config {
            database_url,
            jwt_secret,
            jwt_expires_in,
            jwt_maxage: jwt_maxage.parse::<i32>().unwrap(),
        }
    }
}

#[derive(Clone)]
pub struct State {
    pub graph: Arc<Graph>,
    pub env: Config,
}

impl State {
    async fn init() -> Self {
        let graph = connect().await;
        Database::constraints(&graph).await;

        Self {
            graph,
            env: Config::init(),
        }
    }
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    let state = State::init().await;
    let schema = Schema::build(Query, Mutation, Subscription)
        .data(state.clone())
        .finish();
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(state.clone()))
            .app_data(Data::new(schema.clone()))
            .wrap(Logger::default())
            .wrap(Compress::default())
            .service(api)
            .service(register)
            .service(login)
            .service(logout)
            .service(graphql)
    })
    .bind("localhost:8080")?
    .run()
    .await
}
