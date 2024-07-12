use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};
use tracing::error;

pub fn setup_handle() -> PrometheusHandle {
    const EXPONENTIAL_SECONDS: &[f64] = &[
        0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
    ];

    PrometheusBuilder::new()
        .set_buckets_for_metric(
            Matcher::Full("http_requests_duration_seconds".to_string()),
            EXPONENTIAL_SECONDS,
        )
        .unwrap()
        .install_recorder()
        .map_err(|e| {
            error!("FATAL: Unable to build Prometheus recorder: {}", e);
            std::process::exit(1);
        })
        .unwrap()
}