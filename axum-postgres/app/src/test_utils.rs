use crate::{datasources::database::MockDatabase, AppState};
use axum::{body::{to_bytes, Body}, extract::Request, response::Response, routing::MethodRouter, Router};
use std::sync::Arc;
use tower::ServiceExt;

pub async fn init_router(
    mock_db: MockDatabase,
    uri: String,
    router: MethodRouter<Arc<AppState>>,
) -> Router {
    let app_state = Arc::new(AppState {
        db: Box::new(mock_db),
    });
    Router::new()
        .route(uri.as_str(), router)
        .with_state(app_state)
}

pub async fn test_get(app: Router, uri: String) -> Response<Body> {
    app.oneshot(
        Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
            .unwrap(),
    )
    .await
    .unwrap()
}

pub async fn test_post<T>(app: Router, uri: String, body: T) -> Response<Body>
where
    T: serde::Serialize,
{
    app.oneshot(
        Request::builder()
            .method("POST")
            .uri(uri)
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap(),
    )
    .await
    .unwrap()
}

pub async fn test_delete(app: Router, uri: String) -> Response<Body> {
    app.oneshot(
        Request::builder()
            .method("DELETE")
            .uri(uri)
            .body(Body::empty())
            .unwrap(),
    )
    .await
    .unwrap()
}

pub async fn read_response_body<T>(response: axum::response::Response) -> T
where
    T: serde::de::DeserializeOwned,
{
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}