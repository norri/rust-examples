use axum::async_trait;
use memory_db::MemoryDB;
use mockall::automock;
use models::{DbNewTodo, DbTodo, DbUpdateTodo};
use postgres_db::PostgresDB;
use thiserror::Error;
use uuid::Uuid;

mod memory_db;
pub mod models;
mod postgres_db;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("database item not found with id: {id}")]
    NotFound { id: Uuid },
    #[error("database query failed: {0}")]
    SqlxError(#[from] sqlx::Error),
}

#[automock]
#[async_trait]
pub trait Database: Send + Sync {
    async fn get_values(&self) -> Result<Vec<DbTodo>, DatabaseError>;
    async fn insert(&self, todo: DbNewTodo) -> Result<DbTodo, DatabaseError>;
    async fn remove(&self, id: Uuid) -> Result<(), DatabaseError>;
    async fn update(&self, id: Uuid, todo: DbUpdateTodo) -> Result<DbTodo, DatabaseError>;
}

pub async fn new_database(
    database_url: Option<String>,
    max_connections: u32,
) -> Result<Box<dyn Database>, String> {
    match database_url {
        Some(url) => {
            if url.starts_with("postgres://") {
                tracing::info!("Using Postgres database with url: {}", url,);
                match PostgresDB::new(url, max_connections).await {
                    Ok(db) => Ok(Box::new(db) as Box<dyn Database>),
                    Err(e) => Err(format!("Failed to connect to Postgres database: {}", e)),
                }
            } else {
                Err("Unsupported database URL".to_string())
            }
        }
        None => {
            tracing::info!("Using in-memory database");
            Ok(Box::new(MemoryDB::new()) as Box<dyn Database>)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::Error as SqlxError;
    use testcontainers_modules::{
        postgres,
        testcontainers::runners::AsyncRunner,
    };

    #[tokio::test]
    async fn test_new_database_postgres() {
        let postgres_node = postgres::Postgres::default().start().await.unwrap();
        let connection_url = &format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            postgres_node.get_host_port_ipv4(5432).await.unwrap()
        );

        let db_result = new_database(Some(connection_url.to_string()), 1).await;
        assert!(db_result.is_ok());
        // TODO test that the database is a PostgresDB

        postgres_node.rm().await.unwrap();
    }

    #[tokio::test]
    async fn test_new_database_unknown() {
        let db_result = new_database(Some("invalid://localhost".to_string()), 1).await;
        assert!(db_result.is_err());
        assert_eq!(
            db_result.err().unwrap(),
            "Unsupported database URL".to_string()
        );
    }

    #[tokio::test]
    async fn test_new_database_without_url() {
        let db_result = new_database(None, 1).await;
        assert!(db_result.is_ok());
        // TODO test that the database is a MemoryDB
    }

    #[test]
    fn test_database_error_not_found() {
        let id = uuid::Uuid::new_v4();
        let error = DatabaseError::NotFound { id };
        assert_eq!(
            format!("{}", error),
            format!("database item not found with id: {}", id)
        );
    }

    #[test]
    fn test_database_error_query_failed() {
        let sqlx_error = SqlxError::RowNotFound;
        let error = DatabaseError::SqlxError(sqlx_error);
        assert!(format!("{}", error).contains("database query failed: no rows returned"));
    }
}
