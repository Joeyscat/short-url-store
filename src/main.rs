use axum::prelude::*;
use redis::Client;
use std::net::SocketAddr;

#[macro_use]
extern crate lazy_static;

use crate::handler::LinkHandler;

mod handler;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = route("/", get(|| async { "short-link-service v0.0.1" }))
        .route(
            "/link",
            post(LinkHandler::create).delete(LinkHandler::delete),
        )
        .route("/link/:code", get(LinkHandler::get));

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

pub struct AppConf {
    pub redis_uri: String,
}

impl AppConf {
    fn new(_config_path: &str) -> Self {
        Self {
            redis_uri: "".to_string(),
        }
    }
}

lazy_static! {
    //全局配置文件
    pub static ref GLOBAL_CONF: AppConf = AppConf::new("app.toml");
    // 全局redis client
    pub static ref REDIS: Client = {
        let redis_client = redis::Client::open(GLOBAL_CONF.redis_uri.as_str()).unwrap();
        redis_client
    };
}