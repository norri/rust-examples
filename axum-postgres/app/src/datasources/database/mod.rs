use memory_db::MemoryDB;
use mockall::automock;
use models::{DbNewTodo, DbTodo, DbUpdateTodo};
use postgres_db::PostgresDB;
use std::sync::PoisonError;
use uuid::Uuid;

mod memory_db;
pub mod models;
mod postgres_db;

#[derive(thiserror::Error, Debug)]
pub enum DatabaseError {
    #[error("database item not found with id: {id}")]
    NotFound { id: Uuid },
    #[error("database query failed: {0}")]
    Internal(String),
}

impl<T> From<PoisonError<T>> for DatabaseError {
    fn from(error: PoisonError<T>) -> Self {
        DatabaseError::Internal(error.to_string())
    }
}

impl From<sqlx::Error> for DatabaseError {
    fn from(error: sqlx::Error) -> Self {
        DatabaseError::Internal(error.to_string())
    }
}

pub enum Database {
    Postgres(PostgresDB),
    Memory(MemoryDB),
    #[cfg(test)]
    Mock(MockDatabase),
}

#[automock]
impl Database {
    pub async fn get_values(&self) -> Result<Vec<DbTodo>, DatabaseError> {
        match self {
            Database::Postgres(pg) => pg.get_values().await,
            Database::Memory(memdb) => memdb.get_values().await,
            #[cfg(test)]
            Database::Mock(mock) => mock.get_values().await,
        }
    }

    pub async fn insert(&self, todo: DbNewTodo) -> Result<DbTodo, DatabaseError> {
        match self {
            Database::Postgres(pg) => pg.insert(todo).await,
            Database::Memory(memdb) => memdb.insert(todo).await,
            #[cfg(test)]
            Database::Mock(mock) => mock.insert(todo).await,
        }
    }

    pub async fn remove(&self, id: Uuid) -> Result<(), DatabaseError> {
        match self {
            Database::Postgres(pg) => pg.remove(id).await,
            Database::Memory(memdb) => memdb.remove(id).await,
            #[cfg(test)]
            Database::Mock(mock) => mock.remove(id).await,
        }
    }

    pub async fn update(&self, id: Uuid, todo: DbUpdateTodo) -> Result<DbTodo, DatabaseError> {
        match self {
            Database::Postgres(pg) => pg.update(id, todo).await,
            Database::Memory(memdb) => memdb.update(id, todo).await,
            #[cfg(test)]
            Database::Mock(mock) => mock.update(id, todo).await,
        }
    }
}

pub async fn new_database(
    database_url: Option<String>,
    max_connections: u32,
) -> Result<Database, String> {
    match database_url {
        Some(url) => {
            if url.starts_with("postgres://") {
                tracing::info!("Using Postgres database with url: {}", url,);
                match PostgresDB::new(url, max_connections).await {
                    Ok(db) => Ok(Database::Postgres(db)),
                    Err(e) => Err(format!("Failed to connect to Postgres database: {}", e)),
                }
            } else {
                Err("Unsupported database URL".to_string())
            }
        }
        None => {
            tracing::info!("Using in-memory database");
            Ok(Database::Memory(MemoryDB::new()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::Error as SqlxError;
    use testcontainers_modules::{postgres, testcontainers::runners::AsyncRunner};

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
    fn test_database_error_sqlx_query_failed() {
        let sqlx_error = SqlxError::RowNotFound;
        let error = DatabaseError::Internal(sqlx_error.to_string());
        assert!(format!("{}", error).contains("database query failed: no rows returned"));
    }

    #[test]
    fn test_database_error_poison_error() {
        let poison_error = PoisonError::new("Poisoned");
        let error = DatabaseError::Internal(poison_error.to_string());
        assert!(format!("{}", error).contains("database query failed: poisoned lock"));
    }
}
