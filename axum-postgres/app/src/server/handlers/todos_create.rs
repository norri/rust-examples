use crate::server::errors::AppError;
use crate::server::extractors::request_json::ValidatedJson;
use crate::{
    datasources::database::models::DbNewTodo,
    server::domain::todos::{NewTodo, Todo},
    AppState,
};
use axum::{extract::State, http::StatusCode, Json};
use std::sync::Arc;

pub async fn todos_create(
    State(state): State<Arc<AppState>>,
    ValidatedJson(input): ValidatedJson<NewTodo>,
) -> Result<(StatusCode, Json<Todo>), AppError> {
    let new_todo: DbNewTodo = input.into();
    let db_todo = state.db.insert(new_todo).await?;

    let todo: Todo = db_todo.into();
    Ok((StatusCode::CREATED, Json(todo)))
}

#[cfg(test)]
mod tests {
    use crate::datasources::database::models::DbTodo;
    use crate::datasources::database::MockDatabase;
    use crate::server::domain::errors::ErrorResponse;
    use crate::server::domain::todos::{NewTodo, Todo};
    use crate::server::handlers::todos_create::todos_create;
    use crate::test_utils::{init_router, read_response_body, test_post};
    use axum::http::StatusCode;
    use axum::routing::post;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_todos_create() {
        let mut mock_db = MockDatabase::new();
        mock_db.expect_insert().returning(|new_todo| {
            Ok(DbTodo {
                id: Uuid::new_v4(),
                text: new_todo.text,
                completed: false,
            })
        });
        let app = init_router(mock_db, format!("/todos"), post(todos_create)).await;

        let new_todo = NewTodo {
            text: "test".to_string(),
        };
        let response = test_post(app, format!("/todos"), new_todo).await;
        assert_eq!(response.status(), StatusCode::CREATED);

        let todo: Todo = read_response_body(response).await;
        assert_eq!(todo.text, "test");
        assert!(!todo.completed);
    }

    #[tokio::test]
    async fn test_todos_create_invalid_text_too_short() {
        let mock_db = MockDatabase::new();
        let app = init_router(mock_db, format!("/todos"), post(todos_create)).await;

        let invalid_todo = NewTodo {
            text: "".to_string(),
        };
        let response = test_post(app, format!("/todos"), invalid_todo).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let response_body: ErrorResponse = read_response_body(response).await;
        assert_eq!(
            response_body.error,
            "text: length must be between 1 and 200"
        );
    }

    #[tokio::test]
    async fn test_todos_create_invalid_text_too_long() {
        let mock_db = MockDatabase::new();
        let app = init_router(mock_db, format!("/todos"), post(todos_create)).await;

        let invalid_todo = NewTodo {
            text: "a".repeat(201),
        };
        let response = test_post(app, format!("/todos"), invalid_todo).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let response_body: ErrorResponse = read_response_body(response).await;
        assert_eq!(
            response_body.error,
            "text: length must be between 1 and 200"
        );
    }
}
