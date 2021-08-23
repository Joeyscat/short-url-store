use axum::prelude::*;
use redis::Client;
use std::{env, net::SocketAddr};

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

lazy_static! {
    pub static ref REDIS: Client = {
        let redis_client = redis::Client::open(
            env::var("REDIS_URI").expect("missing environment variable REDIS_URI"),
        )
        .expect("failed to connect to Redis");
        redis_client
    };
}
