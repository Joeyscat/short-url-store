use std::io::Result;

use crate::*;
use axum::{http::StatusCode, response::IntoResponse};
use base62num::encode;
use redis::Commands;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateLink {
    url: String,
}

#[derive(Serialize)]
struct Link {
    code: String,
    url: String,
}

#[derive(Serialize)]
struct CreateLinkResult {
    code: usize,
    msg: String,
    data: Option<Link>,
}

#[derive(Serialize)]
struct GetLinkResult {
    code: usize,
    msg: String,
    data: Option<Link>,
}

pub struct LinkHandler {}

impl LinkHandler {
    pub async fn create(extract::Json(payload): extract::Json<CreateLink>) -> impl IntoResponse {
        let code = generate_code().await.expect("generate_code error");

        let mut conn = REDIS.get_connection().expect("get_connection error");
        conn.set::<String, String, String>(code.clone(), payload.url.clone())
            .expect("set code:url error");

        let link = Link {
            code,
            url: payload.url,
        };

        (
            StatusCode::CREATED,
            response::Json(CreateLinkResult {
                code: 0,
                msg: "OK".to_string(),
                data: Option::Some(link),
            }),
        )
    }

    pub async fn get(extract::Path(code): extract::Path<String>) -> impl IntoResponse {
        let mut conn = REDIS.get_connection().expect("get_connection error");

        match conn.get(code.clone()) {
            Ok(v) => {
                let link = Link { code, url: v };
                (
                    StatusCode::OK,
                    response::Json(CreateLinkResult {
                        code: 0,
                        msg: "OK".to_string(),
                        data: Option::Some(link),
                    }),
                )
            }
            _ => (
                StatusCode::OK,
                response::Json(CreateLinkResult {
                    code: 500,
                    msg: "code not found".to_string(),
                    data: Option::None,
                }),
            ),
        }
    }

    pub async fn delete(extract::Path(_code): extract::Path<String>) -> impl IntoResponse {
        StatusCode::OK
    }
}

async fn generate_code() -> Result<String> {
    let mut conn = REDIS.get_connection().expect("get_connection error");
    let v = conn
        .incr::<String, u64, u64>("next.url.id".to_string(), 1)
        .expect("get next.url.id error");

    Ok(encode(v as usize))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::routing::BoxRoute;
    use http::StatusCode;
    use serde_json::json;
    use tower::ServiceExt; // for `app.oneshot()`
    use tower_http::trace::TraceLayer;

    #[allow(dead_code)]
    fn app() -> BoxRoute<Body> {
        route("/", get(|| async { "Hello, World!" }))
            .route(
                "/link",
                post(LinkHandler::create).delete(LinkHandler::delete),
            )
            .route("/link/:code", get(LinkHandler::get))
            .layer(TraceLayer::new_for_http())
            .boxed()
    }

    #[tokio::test]
    async fn link_get() {
        let app = app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/link/B")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let expected_body = serde_json::to_vec(&json!({
            "code": 0,
            "msg": "OK",
            "data": {
                "code": "B",
                "url": "https://github.com/tokio-rs/axum/blob/main/examples/todos/src/main.rs"
            }
        }))
        .unwrap();

        assert_eq!(&body[..], expected_body);
    }

    #[tokio::test]
    async fn link_create() {
        let app = app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/link")
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .body(Body::from(
                        serde_json::to_string(&json!({"url":"jojo"})).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let expected_body = serde_json::to_vec(&json!({
            "code": 0,
            "msg": "OK",
            "data": {
                "code": "W",
                "url": "https://github.com/tokio-rs/axum/blob/main/examples/todos/src/main.rs"
            }
        }))
        .unwrap();

        assert_eq!(&body[..], expected_body);
    }
}
