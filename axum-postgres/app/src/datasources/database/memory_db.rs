use super::{DatabaseError, DbNewTodo, DbTodo, DbUpdateTodo};
use std::{collections::HashMap, sync::RwLock};
use uuid::Uuid;

pub struct MemoryDB {
    todo_map: RwLock<HashMap<Uuid, DbTodo>>,
}

impl MemoryDB {
    pub fn new() -> Self {
        MemoryDB {
            todo_map: RwLock::new(HashMap::new()),
        }
    }

    pub async fn get_values(&self) -> Result<Vec<DbTodo>, DatabaseError> {
        let rows = self.todo_map
            .read()
            .map(|map| map.values().cloned().collect())?;
        Ok(rows)
    }

    pub async fn insert(&self, todo: DbNewTodo) -> Result<DbTodo, DatabaseError> {
        let todo = DbTodo {
            id: uuid::Uuid::new_v4(),
            text: todo.text,
            completed: false,
        };
        self.todo_map
            .write()
            .map(|mut map| map.insert(todo.id, todo.clone()))?;
        Ok(todo)
    }

    pub async fn remove(&self, id: Uuid) -> Result<(), DatabaseError> {
        self.todo_map
            .write()
            .map(|mut map| map.remove(&id))?
            .ok_or(DatabaseError::NotFound { id })?;
        Ok(())
    }

    pub async fn update(&self, id: Uuid, todo: DbUpdateTodo) -> Result<DbTodo, DatabaseError> {
        let mut map = self.todo_map.write()?;
        if let Some(existing_todo) = map.get_mut(&id) {
            if let Some(text) = todo.text {
                existing_todo.text = text;
            }
            if let Some(completed) = todo.completed {
                existing_todo.completed = completed;
            }
            Ok(existing_todo.clone())
        } else {
            Err(DatabaseError::NotFound { id })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::datasources::database::{
        memory_db::MemoryDB, DatabaseError, DbNewTodo, DbUpdateTodo,
    };
    use uuid::Uuid;

    #[tokio::test]
    async fn test_get_values() {
        let db = MemoryDB::new();
        let new_todo = DbNewTodo {
            text: String::from("Test todo"),
        };
        db.insert(new_todo).await.unwrap();

        let todos = db.get_values().await.unwrap();
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].text, "Test todo");
    }

    #[tokio::test]
    async fn test_insert() {
        let db = MemoryDB::new();
        let new_todo = DbNewTodo {
            text: String::from("Test todo"),
        };
        let inserted_todo = db.insert(new_todo).await.unwrap();

        assert_eq!(inserted_todo.text, "Test todo");
        assert!(!inserted_todo.completed);
    }

    #[tokio::test]
    async fn test_remove() {
        let db = MemoryDB::new();
        let new_todo = DbNewTodo {
            text: String::from("Test todo"),
        };
        let inserted_todo = db.insert(new_todo).await.unwrap();

        db.remove(inserted_todo.id).await.unwrap();
        let todos = db.get_values().await.unwrap();
        assert!(todos.is_empty());
    }

    #[tokio::test]
    async fn test_remove_not_found() {
        let db = MemoryDB::new();
        let result = db.remove(Uuid::new_v4()).await;
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            DatabaseError::NotFound { id: _ }
        ));
    }

    #[tokio::test]
    async fn test_update() {
        let db = MemoryDB::new();
        let new_todo = DbNewTodo {
            text: String::from("Test todo"),
        };
        let inserted_todo = db.insert(new_todo).await.unwrap();

        let update_todo = DbUpdateTodo {
            text: Some(String::from("Updated todo")),
            completed: Some(true),
        };
        let updated_todo = db.update(inserted_todo.id, update_todo).await.unwrap();

        assert_eq!(updated_todo.text, "Updated todo");
        assert!(updated_todo.completed);
    }

    #[tokio::test]
    async fn test_update_not_found() {
        let db = MemoryDB::new();
        let update_todo = DbUpdateTodo {
            text: Some(String::from("Updated todo")),
            completed: Some(true),
        };
        let result = db.update(Uuid::new_v4(), update_todo).await;
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            DatabaseError::NotFound { id: _ }
        ));
    }
}
