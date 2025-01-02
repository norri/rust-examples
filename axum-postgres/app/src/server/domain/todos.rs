use crate::datasources::database::models::DbTodo;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Deserialize, Serialize)]
pub struct TodosResponse {
    pub todos: Vec<Todo>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Todo {
    pub id: String,
    pub text: String,
    pub completed: bool,
}

impl From<DbTodo> for Todo {
    fn from(db_todo: DbTodo) -> Self {
        Todo {
            id: db_todo.id.to_string(),
            text: db_todo.text,
            completed: db_todo.completed,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct NewTodo {
    #[validate(length(min = 1, max = 200, message = "length must be between 1 and 200"))]
    pub text: String,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct UpdateTodo {
    #[validate(length(min = 1, max = 200, message = "length must be between 1 and 200"))]
    pub text: Option<String>,
    pub completed: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn test_new_todo_validation() {
        let valid_todo = NewTodo {
            text: "Valid todo".to_string(),
        };
        assert!(valid_todo.validate().is_ok());

        let empty_todo = NewTodo {
            text: "".to_string(),
        };
        assert!(empty_todo.validate().is_err());
        assert_validation_error_message(empty_todo, "length must be between 1 and 200");

        let long_todo = NewTodo {
            text: "a".repeat(201),
        };
        assert!(long_todo.validate().is_err());
        assert_validation_error_message(long_todo, "length must be between 1 and 200");
    }

    #[test]
    fn test_update_todo_validation() {
        let valid_todo = UpdateTodo {
            text: Some("Valid todo".to_string()),
            completed: Some(true),
        };
        assert!(valid_todo.validate().is_ok());

        let empty_todo = UpdateTodo {
            text: Some("".to_string()),
            completed: Some(false),
        };
        assert!(empty_todo.validate().is_err());
        assert_validation_error_message(empty_todo, "length must be between 1 and 200");

        let long_todo = UpdateTodo {
            text: Some("a".repeat(201)),
            completed: Some(true),
        };
        assert!(long_todo.validate().is_err());
        assert_validation_error_message(long_todo, "length must be between 1 and 200");

        let no_text_todo = UpdateTodo {
            text: None,
            completed: Some(true),
        };
        assert!(no_text_todo.validate().is_ok());

        let nothing_todo = UpdateTodo {
            text: None,
            completed: None,
        };
        assert!(nothing_todo.validate().is_ok()); // this needs to be validate separately
    }

    fn assert_validation_error_message<T: Validate>(item: T, expected_message: &str) {
        let error = item.validate().err().unwrap().to_string();
        assert!(error.contains(expected_message));
    }
}
