use config::Config;
use datasources::database::{new_database, Database};
use server::routes::new_router;
use std::sync::Arc;
use tokio::signal;

mod config;
mod datasources;
mod logger;
mod server;

#[cfg(test)]
mod test_utils;

struct AppState {
    db: Database,
    credentials: Vec<(String, String)>,
}
type SharedState = Arc<AppState>;

#[tokio::main]
async fn main() {
    let config = Config::new();

    logger::init(config.log_level, !config.environment.is_local());

    let db = new_database(config.database_url, config.database_max_connections)
        .await
        .expect("database initialization failed");
    let app_state = Arc::new(AppState {
        db,
        credentials: config.credentials,
    });

    let router = new_router(app_state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.port))
        .await
        .expect("failed to bind to port");
    tracing::info!(
        "listening on {}",
        listener.local_addr().expect("could not get local address")
    );
    axum::serve(listener, router.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("server failed");
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
