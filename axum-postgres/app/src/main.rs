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

#[derive(Clone)]
struct AppState {
    db: Arc<Database>,
    credentials: Vec<(String, String)>,
}

#[tokio::main]
async fn main() {
    let config = Config::new();

    logger::init(config.log_level);

    let db = new_database(config.database_url, config.database_max_connections)
        .await
        .expect("database initialization failed");
    let app_state = AppState {
        db: Arc::new(db),
        credentials: config.credentials,
    };

    let app = new_router(app_state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.port))
        .await
        .expect("failed to bind to port");
    tracing::info!(
        "listening on {}",
        listener.local_addr().expect("could not get local address")
    );
    axum::serve(listener, app).await.expect("server failed");
}
