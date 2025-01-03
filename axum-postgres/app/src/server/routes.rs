use super::handlers::{
    protected::protected, todos_create::todos_create, todos_delete::todos_delete,
    todos_list::todos_list, todos_update::todos_update,
};
use crate::AppState;
use axum::{
    routing::{get, post},
    Router,
};
pub fn new_router(app_state: AppState) -> Router {
    let api_routes = Router::new()
        .route("/todos", get(todos_list).post(todos_create))
        .route("/todos/:id", post(todos_update).delete(todos_delete))
        .route("/protected", get(protected));

    Router::new()
        .route("/status", get(|| async { "OK" }))
        .nest("/api/v1", api_routes)
        .with_state(app_state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datasources::database::{Database, MockDatabase};
    use crate::test_utils::test_get;
    use axum::body::to_bytes;
    use axum::http::StatusCode;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_status_endpoint() {
        let app_state = AppState {
            db: Arc::new(Database::Mock(MockDatabase::new())),
            credentials: vec![("user".to_string(), "password".to_string())],
        };
        let app = new_router(app_state);

        let response = test_get(app, format!("/status")).await;
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(body, "OK");
    }
}
