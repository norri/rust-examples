use axum::{routing::get, Router};
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};
use metrics_process::Collector;
use std::future::ready;

pub fn new_metrics_router() -> Router {
    let recorder_handle = setup_metrics_recorder();

    let collector = Collector::default();
    // Call `describe()` method to register help string.
    collector.describe();

    Router::new().route(
        "/metrics",
        get(move || {
            collector.collect();
            ready(recorder_handle.render())
        }),
    )
}

fn setup_metrics_recorder() -> PrometheusHandle {
    const EXPONENTIAL_SECONDS: &[f64] = &[
        0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
    ];

    PrometheusBuilder::new()
        .set_buckets_for_metric(
            Matcher::Full("http_requests_duration_seconds".to_string()),
            EXPONENTIAL_SECONDS,
        )
        .expect("failed to set bucket")
        .install_recorder()
        .expect("failed to install recorder")
}
