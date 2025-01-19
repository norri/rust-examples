use crate::{
    server::{domain::errors::ErrorResponse, errors::AppError, openapi::TODO_TAG},
    SharedState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
};
use uuid::Uuid;

/// Delete Todo item by id
///
/// Delete Todo item from in-memory storage by id. Returns either 200 success of 404 with TodoError if Todo is not found.
#[utoipa::path(
    delete,
    path = "/{id}",
    tag = TODO_TAG,
    responses(
        (status = 200, description = "Todo deleted successfully"),
        (status = 404, description = "Todo not found"),
        (status = 500, description = "Internal error", body = ErrorResponse)
    ),
    params(
        ("id" = String, Path, description = "Todo id")
    )
)]
pub async fn todos_delete(
    Path(id): Path<String>,
    State(state): State<SharedState>,
) -> Result<StatusCode, AppError> {
    let todo_id =
        Uuid::parse_str(&id) // validate id is UUID
            .map_err(|_| AppError::BadRequest(format!("id is not valid uuid: {}", id)))?;

    state.db.remove(todo_id).await?;
    Ok(StatusCode::OK)
}

#[cfg(test)]
mod tests {
    use crate::datasources::database::{DatabaseError, MockDatabase};
    use crate::server::domain::errors::ErrorResponse;
    use crate::server::handlers::todos_delete::todos_delete;
    use crate::test_utils::{init_router, read_response_body, test_delete};
    use axum::http::StatusCode;
    use axum::routing::delete;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_todos_delete() {
        let mut mock_db = MockDatabase::new();
        mock_db.expect_remove().returning(|_| Ok(()));
        let app = init_router(mock_db, "/todos/{id}", delete(todos_delete)).await;

        let id = Uuid::new_v4().to_string();
        let response = test_delete(app, &format!("/todos/{}", id)).await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_todos_delete_not_found() {
        let mut mock_db = MockDatabase::new();
        mock_db
            .expect_remove()
            .returning(|_| Err(DatabaseError::NotFound { id: Uuid::new_v4() }));
        let app = init_router(mock_db, "/todos/{id}", delete(todos_delete)).await;

        let id = Uuid::new_v4().to_string();
        let response = test_delete(app, &format!("/todos/{}", id)).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_todos_delete_invalid_id() {
        let mock_db = MockDatabase::new();
        let app = init_router(mock_db, "/todos/{id}", delete(todos_delete)).await;

        let response = test_delete(app, &format!("/todos/{}", "invalid")).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let response_body: ErrorResponse = read_response_body(response).await;
        assert_eq!(response_body.error, "id is not valid uuid: invalid");
    }
}
