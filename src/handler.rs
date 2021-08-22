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

pub struct LinkHandler {}

impl LinkHandler {
    pub async fn create(extract::Json(payload): extract::Json<CreateLink>) -> impl IntoResponse {
        let code = generate_code().await.expect("generate_code error");

        save_code(code.clone(), payload.url.clone())
            .await
            .expect("save_code error");

        let link = Link {
            code,
            url: payload.url,
        };

        (StatusCode::CREATED, response::Json(link))
    }

    pub async fn get(extract::Path(code): extract::Path<String>) -> impl IntoResponse {
        let link = Link {
            code,
            url: "jojo".to_string(),
        };
        (StatusCode::OK, response::Json(link))
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

async fn save_code(code: String, url: String) -> Result<()> {
    let mut conn = REDIS.get_connection().expect("get_connection error");
    conn.set::<String, String, String>(code, url)
        .expect("set code:url error");
    Ok(())
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
                    .uri("/link/xx")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();

        assert_eq!(&body[..], b"{\"code\":\"xx\",\"url\":\"jojo\"}");
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
        let expected_body = serde_json::to_vec(&json!({"code":"xx","url":"jojo"})).unwrap();

        assert_eq!(&body[..], expected_body);
    }
}
