use crate::server::domain::todos::Todo;
use crate::server::domain::todos::TodosResponse;
use crate::server::errors::AppError;
use crate::SharedState;
use axum::{extract::State, Json};

pub async fn todos_list(State(state): State<SharedState>) -> Result<Json<TodosResponse>, AppError> {
    let db_todos = state.db.get_values().await?;
    let todos: Vec<Todo> = db_todos.into_iter().map(|db_todo| db_todo.into()).collect();
    Ok(Json(TodosResponse { todos }))
}

#[cfg(test)]
mod tests {
    use crate::datasources::database::models::DbTodo;
    use crate::datasources::database::MockDatabase;
    use crate::server::domain::todos::TodosResponse;
    use crate::server::handlers::todos_list::todos_list;
    use crate::test_utils::{init_router, read_response_body, test_get};
    use axum::http::StatusCode;
    use axum::routing::get;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_todos_list_empty() {
        let mut mock_db = MockDatabase::new();
        mock_db.expect_get_values().returning(|| Ok(vec![]));
        let app = init_router(mock_db, "/todos", get(todos_list)).await;

        let response = test_get(app, "/todos").await;
        assert_eq!(response.status(), StatusCode::OK);

        let response_body: TodosResponse = read_response_body(response).await;
        assert!(response_body.todos.is_empty());
    }

    #[tokio::test]
    async fn test_todos_list_results() {
        let id = Uuid::new_v4();

        let mut mock_db = MockDatabase::new();
        mock_db.expect_get_values().returning(move || {
            Ok(vec![DbTodo {
                id: id.clone(),
                text: "test".to_string(),
                completed: false,
            }])
        });
        let app = init_router(mock_db, "/todos", get(todos_list)).await;

        let response = test_get(app, "/todos").await;
        assert_eq!(response.status(), StatusCode::OK);

        let response_body: TodosResponse = read_response_body(response).await;
        assert_eq!(response_body.todos.len(), 1);
        let todo = &response_body.todos[0];
        assert_eq!(todo.id, id.to_string());
        assert_eq!(todo.text, "test");
        assert!(!todo.completed);
    }
}
