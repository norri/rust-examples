use crate::server::domain::todos::{NewTodo, UpdateTodo};
use serde::Serialize;

use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow, Serialize, Clone)]
pub struct DbTodo {
    pub id: Uuid,
    pub text: String,
    pub completed: bool,
}

pub struct DbNewTodo {
    pub text: String,
}

impl From<NewTodo> for DbNewTodo {
    fn from(new_todo: NewTodo) -> Self {
        DbNewTodo {
            text: new_todo.text,
        }
    }
}

pub struct DbUpdateTodo {
    pub text: Option<String>,
    pub completed: Option<bool>,
}

impl From<UpdateTodo> for DbUpdateTodo {
    fn from(update_todo: UpdateTodo) -> Self {
        DbUpdateTodo {
            text: update_todo.text,
            completed: update_todo.completed,
        }
    }
}
