use config::Config;
use metrics::new_metrics_router;
use routes::new_router;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod metrics;
mod routes;

#[tokio::main]
async fn main() {
    let config = Config::new();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let (_main_server, _metrics_server) = tokio::join!(
        start_main_server(config.port),
        start_metrics_server(config.metrics_port)
    );
}

async fn start_main_server(port: String) {
    let app = new_router();

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .expect("Failed to bind TCP listener");
    tracing::debug!(
        "listening on {}",
        listener.local_addr().expect("Failed to get local address")
    );
    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}

async fn start_metrics_server(port: String) {
    let app = new_metrics_router();

    // NOTE: expose metrics endpoint on a different port
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .expect("Failed to bind metrics TCP listener");
    tracing::debug!(
        "listening metrics on {}",
        listener
            .local_addr()
            .expect("Failed to get metrics local address")
    );
    axum::serve(listener, app)
        .await
        .expect("Failed to start metrics server");
}
