use super::handlers::{
    protected::protected, todos_create::todos_create, todos_delete::todos_delete,
    todos_list::todos_list, todos_update::todos_update,
};
use crate::SharedState;
use axum::{
    http::{HeaderName, Request},
    routing::{get, post},
    Router,
};
use tower::ServiceBuilder;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::TraceLayer;
use tracing::{debug_span, error};

const REQUEST_ID_HEADER: &str = "x-request-id";

pub fn new_router(app_state: SharedState) -> Router {
    let x_request_id = HeaderName::from_static(REQUEST_ID_HEADER);

    let middleware = ServiceBuilder::new()
        .layer(SetRequestIdLayer::new(
            x_request_id.clone(),
            MakeRequestUuid,
        ))
        .layer(
            TraceLayer::new_for_http().make_span_with(|req: &Request<_>| {
                let request_id = req.headers().get(REQUEST_ID_HEADER);
                let method = req.method();
                let uri = req.uri();
                // user is added if the request is authenticated
                let user = tracing::field::Empty;

                if let Some(request_id) = request_id {
                    debug_span!("http_request", request_id = ?request_id, %method, %uri, user)
                } else {
                    error!("could not extract request_id");
                    debug_span!("http_request", %method, %uri, user)
                }
            }),
        )
        .layer(PropagateRequestIdLayer::new(x_request_id));

    let api_routes = Router::new()
        .route("/todos", get(todos_list).post(todos_create))
        .route("/todos/:id", post(todos_update).delete(todos_delete))
        .route("/protected", get(protected));

    Router::new()
        .route("/status", get(|| async { "OK" }))
        .nest("/api/v1", api_routes)
        .layer(middleware)
        .with_state(app_state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datasources::database::{Database, MockDatabase};
    use crate::test_utils::test_get;
    use crate::AppState;
    use axum::body::to_bytes;
    use axum::http::StatusCode;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_status_endpoint() {
        let app_state = Arc::new(AppState {
            db: Database::Mock(MockDatabase::new()),
            credentials: vec![("user".to_string(), "password".to_string())],
        });
        let app = new_router(app_state);

        let response = test_get(app, format!("/status")).await;
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(body, "OK");
    }
}
