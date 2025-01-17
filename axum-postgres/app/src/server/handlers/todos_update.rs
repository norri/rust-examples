use crate::server::domain::todos::Todo;
use crate::server::domain::todos::UpdateTodo;
use crate::server::errors::AppError;
use crate::server::extractors::request_json::ValidatedJson;
use crate::SharedState;
use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

pub async fn todos_update(
    Path(id): Path<String>,
    State(state): State<SharedState>,
    ValidatedJson(input): ValidatedJson<UpdateTodo>,
) -> Result<Json<Todo>, AppError> {
    let todo_id =
        Uuid::parse_str(&id) // validate id is UUID
            .map_err(|_| AppError::BadRequest(format!("id is not valid uuid: {}", id)))?;
    if input.text.is_none() && input.completed.is_none() {
        return Err(AppError::BadRequest(
            "either text or completed must be present".to_string(),
        ));
    }

    let updated_todo = state.db.update(todo_id, input.into()).await?;
    let todo: Todo = updated_todo.into();
    Ok(Json(todo))
}

#[cfg(test)]
mod tests {
    use crate::datasources::database::models::DbTodo;
    use crate::datasources::database::{DatabaseError, MockDatabase};
    use crate::server::domain::errors::ErrorResponse;
    use crate::server::domain::todos::{Todo, UpdateTodo};
    use crate::server::handlers::todos_update::todos_update;
    use crate::test_utils::{init_router, read_response_body, test_post};
    use axum::http::StatusCode;
    use axum::routing::post;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_todos_update() {
        let mut mock_db = MockDatabase::new();
        mock_db.expect_update().returning(|_, update_todo| {
            Ok(DbTodo {
                id: Uuid::new_v4(),
                text: update_todo.text.unwrap(),
                completed: update_todo.completed.unwrap(),
            })
        });
        let app = init_router(mock_db, "/todos/{id}", post(todos_update)).await;

        let update_todo = UpdateTodo {
            text: Some("updated".to_string()),
            completed: Some(true),
        };
        let id = Uuid::new_v4().to_string();
        let response = test_post(app, &format!("/todos/{}", id), update_todo).await;
        assert_eq!(response.status(), StatusCode::OK);

        let todo: Todo = read_response_body(response).await;
        assert_eq!(todo.text, "updated");
        assert!(todo.completed);
    }

    #[tokio::test]
    async fn test_todos_update_not_found() {
        let id = Uuid::new_v4();

        let mut mock_db = MockDatabase::new();
        mock_db
            .expect_update()
            .returning(move |_, _| Err(DatabaseError::NotFound { id }));
        let app = init_router(mock_db, "/todos/{id}", post(todos_update)).await;

        let update_todo = UpdateTodo {
            text: Some("updated".to_string()),
            completed: Some(true),
        };
        let response = test_post(app, &format!("/todos/{}", id), update_todo).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_todos_update_invalid_id() {
        let mock_db = MockDatabase::new();
        let app = init_router(mock_db, "/todos/{id}", post(todos_update)).await;

        let update_todo = UpdateTodo {
            text: Some("updated".to_string()),
            completed: Some(true),
        };
        let response = test_post(app, &format!("/todos/{}", "invalid"), update_todo).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let response_body: ErrorResponse = read_response_body(response).await;
        assert_eq!(response_body.error, "id is not valid uuid: invalid");
    }

    #[tokio::test]
    async fn test_todos_update_invalid_request() {
        let id = Uuid::new_v4();

        let mock_db = MockDatabase::new();
        let app = init_router(mock_db, "/todos/{id}", post(todos_update)).await;

        #[derive(serde::Serialize)]
        struct InvalidRequest {
            text: i32,
        }
        let update_todo = InvalidRequest { text: 100 };
        let response = test_post(app, &format!("/todos/{}", id), update_todo).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let response_body: ErrorResponse = read_response_body(response).await;
        assert_eq!(response_body.error, "failed to read json");
    }

    #[tokio::test]
    async fn test_todos_update_empty_changes() {
        let id = Uuid::new_v4();

        let mock_db = MockDatabase::new();
        let app = init_router(mock_db, "/todos/{id}", post(todos_update)).await;

        let update_todo = UpdateTodo {
            text: None,
            completed: None,
        };
        let response = test_post(app, &format!("/todos/{}", id), update_todo).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let response_body: ErrorResponse = read_response_body(response).await;
        assert_eq!(
            response_body.error,
            "either text or completed must be present"
        );
    }
}
