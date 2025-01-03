use config::Config;
use datasources::database::{new_database, Database};
use server::routes::new_router;
use std::sync::Arc;

mod config;
mod datasources;
mod logger;
mod server;

#[cfg(test)]
mod test_utils;

struct AppState {
    db: Database,
}

#[tokio::main]
async fn main() {
    let config = Config::new();

    logger::init(config.log_level);

    let db = new_database(config.database_url, config.database_max_connections)
        .await
        .unwrap();
    let app_state = Arc::new(AppState { db });

    let app = new_router(app_state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.port))
        .await
        .unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
