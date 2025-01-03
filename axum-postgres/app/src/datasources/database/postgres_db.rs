use super::{
    models::{DbNewTodo, DbTodo, DbUpdateTodo},
    DatabaseError,
};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::time::Duration;
use uuid::Uuid;

pub struct PostgresDB {
    pool: Pool<Postgres>,
}

impl PostgresDB {
    pub async fn new(connection_url: String, max_connections: u32) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .acquire_timeout(Duration::from_secs(3))
            .connect(&connection_url)
            .await?;
        Ok(PostgresDB { pool })
    }

    pub async fn get_values(&self) -> Result<Vec<DbTodo>, DatabaseError> {
        let rows = sqlx::query_as::<_, DbTodo>("SELECT id, text, completed FROM todos")
            .fetch_all(&self.pool)
            .await?;
        Ok(rows)
    }

    pub async fn insert(&self, todo: DbNewTodo) -> Result<DbTodo, DatabaseError> {
        let row = sqlx::query_as::<_, DbTodo>(
            "INSERT INTO todos (id, text, completed) VALUES ($1, $2, $3) RETURNING id, text, completed",
        )
        .bind(uuid::Uuid::new_v4())
        .bind(todo.text)
        .bind(false) // default completed to false
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn remove(&self, id: Uuid) -> Result<(), DatabaseError> {
        let result = sqlx::query("DELETE FROM todos WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(DatabaseError::NotFound { id });
        }
        Ok(())
    }

    pub async fn update(&self, id: Uuid, todo: DbUpdateTodo) -> Result<DbTodo, DatabaseError> {
        let row = sqlx::query_as::<_, DbTodo>(
            "UPDATE todos SET text = COALESCE($1, text), completed = COALESCE($2, completed) WHERE id = $3 RETURNING id, text, completed"
        )
        .bind(todo.text)
        .bind(todo.completed)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => DatabaseError::NotFound { id },
            e => DatabaseError::Internal(e.to_string()),
        })?;
        Ok(row)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::migrate::Migrator;
    use testcontainers_modules::{
        postgres,
        testcontainers::{runners::AsyncRunner, ContainerAsync},
    };

    const MIGRATOR: Migrator = sqlx::migrate!("./migrations");

    async fn setup() -> (ContainerAsync<postgres::Postgres>, PostgresDB) {
        let postgres_node = postgres::Postgres::default().start().await.unwrap();

        let connection_url = &format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            postgres_node.get_host_port_ipv4(5432).await.unwrap()
        );
        let db = PostgresDB::new(connection_url.to_string(), 1)
            .await
            .unwrap();
        MIGRATOR.run(&db.pool).await.unwrap();

        (postgres_node, db)
    }

    async fn shutdown(node: ContainerAsync<postgres::Postgres>) {
        node.rm().await.unwrap();
    }

    #[tokio::test]
    async fn test_insert() {
        let (postgres_node, db) = setup().await;

        let new_todo = DbNewTodo {
            text: "Test todo".to_string(),
        };
        let inserted_todo = db.insert(new_todo).await.unwrap();
        assert_eq!(inserted_todo.text, "Test todo");
        assert_eq!(inserted_todo.completed, false);

        shutdown(postgres_node).await;
    }

    #[tokio::test]
    async fn test_get_values_empty() {
        let (postgres_node, db) = setup().await;

        let todos = db.get_values().await.unwrap();
        assert_eq!(todos.len(), 0);

        shutdown(postgres_node).await;
    }

    #[tokio::test]
    async fn test_get_values() {
        let (postgres_node, db) = setup().await;

        let new_todo = DbNewTodo {
            text: "Test todo".to_string(),
        };
        db.insert(new_todo).await.unwrap();

        let todos = db.get_values().await.unwrap();
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].text, "Test todo");
        assert_eq!(todos[0].completed, false);

        shutdown(postgres_node).await;
    }

    #[tokio::test]
    async fn test_update() {
        let (postgres_node, db) = setup().await;

        let new_todo = DbNewTodo {
            text: "Test todo".to_string(),
        };
        let inserted_todo = db.insert(new_todo).await.unwrap();

        let update_todo = DbUpdateTodo {
            text: Some("Updated todo".to_string()),
            completed: Some(true),
        };
        let updated_todo = db.update(inserted_todo.id, update_todo).await.unwrap();
        assert_eq!(updated_todo.text, "Updated todo");
        assert_eq!(updated_todo.completed, true);

        shutdown(postgres_node).await;
    }

    #[tokio::test]
    async fn test_update_not_found() {
        let (postgres_node, db) = setup().await;

        let not_found_id = Uuid::new_v4();

        let update_todo = DbUpdateTodo {
            text: Some("Updated todo".to_string()),
            completed: Some(true),
        };
        let result = db.update(not_found_id, update_todo).await;
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            DatabaseError::NotFound { id: _not_found_id }
        ));

        shutdown(postgres_node).await;
    }

    #[tokio::test]
    async fn test_remove() {
        let (postgres_node, db) = setup().await;

        let new_todo = DbNewTodo {
            text: "Test todo".to_string(),
        };
        let inserted_todo = db.insert(new_todo).await.unwrap();

        db.remove(inserted_todo.id).await.unwrap();
        let todos = db.get_values().await.unwrap();
        assert_eq!(todos.len(), 0);

        shutdown(postgres_node).await;
    }

    #[tokio::test]
    async fn test_remove_not_found() {
        let (postgres_node, db) = setup().await;

        let not_found_id = Uuid::new_v4();
        let result = db.remove(not_found_id).await;
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            DatabaseError::NotFound { id: _not_found_id }
        ));

        shutdown(postgres_node).await;
    }
}
