use tracing_subscriber::{prelude::*, Layer};

pub fn init(log_level: String, use_json_format: bool) {
    let mut layers = Vec::new();
    if use_json_format {
        let layer = tracing_subscriber::fmt::layer()
            .json()
            .with_current_span(false)
            .with_target(false)
            .with_ansi(false)
            .without_time()
            .boxed();
        layers.push(layer);
    } else {
        let layer = tracing_subscriber::fmt::layer().boxed();
        layers.push(layer);
    }

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!(
                    "{}={log_level},tower_http={log_level},axum::rejection=trace",
                    env!("CARGO_CRATE_NAME")
                )
                .into()
            }),
        )
        .with(layers)
        .init();
}
