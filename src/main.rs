use actix_web::middleware::Logger;
use actix_web::web::Data;
use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use auth_nt::*;
use neo4rs::*;
use std::sync::Arc;
use std::{env, io};

mod auth_nt;
mod neo_nt;
mod actix_nt;

async fn connect() -> Arc<Graph> {
    env::set_var("RUST_LOG", "debug");
    dotenvy::dotenv().ok();
    env_logger::init();
    let uri = "127.0.0.1:7687";
    let unm = "neo4j";
    let pswd = "neo4jabhay";
    Arc::new(
        Graph::new(
            &uri,
            &unm,
            &pswd,
        )
        .await
        .unwrap(),
    )
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
    graph: Arc<Graph>,
    env: Config,
}

impl State {
    async fn init() -> Self {
        let graph = connect().await;
        graph.run(Query::new("create(q: Test {num: 2})".to_string())).await.unwrap();
        Self {
            graph,
            env: Config::init(),
        }
    }
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    let state = State::init().await;
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(state.clone()))
            .wrap(Logger::default())
            .service(api)
            .service(register)
            .service(login)
            .service(logout)
    })
    .bind("localhost:8080")?
    .run()
    .await
    // Ok(())
}
