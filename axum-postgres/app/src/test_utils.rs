use crate::{
    datasources::database::{Database, MockDatabase},
    AppState, SharedState,
};
use axum::{
    body::{to_bytes, Body},
    extract::Request,
    response::Response,
    routing::MethodRouter,
    Router,
};
use std::sync::Arc;
use tower::ServiceExt;

pub async fn init_router(
    mock_db: MockDatabase,
    uri: &str,
    router: MethodRouter<SharedState>,
) -> Router {
    let app_state = Arc::new(AppState {
        db: Database::Mock(mock_db),
        credentials: vec![("user".to_string(), "pass".to_string())],
    });
    Router::new().route(uri, router).with_state(app_state)
}

pub async fn test_get(app: Router, uri: &str) -> Response<Body> {
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

pub async fn test_post<T>(app: Router, uri: &str, body: T) -> Response<Body>
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

pub async fn test_delete(app: Router, uri: &str) -> Response<Body> {
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

pub async fn test_authenticated(
    app: Router,
    uri: &str,
    method: &str,
    auth_header: &str,
) -> Response<Body> {
    app.oneshot(
        Request::builder()
            .method(method)
            .uri(uri)
            .header("Authorization", auth_header)
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
