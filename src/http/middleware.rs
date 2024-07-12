use crate::structs::Limiter;
use axum::body::HttpBody;
use axum::http::{header, StatusCode};
use axum::{
    extract::{MatchedPath, Request},
    middleware::Next,
    response::IntoResponse,
};
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::time::Instant;

pub async fn track_metrics(req: Request, next: Next) -> impl IntoResponse {
    let start = Instant::now();
    let path = if let Some(matched_path) = req.extensions().get::<MatchedPath>() {
        matched_path.as_str().to_owned()
    } else {
        req.uri().path().to_owned()
    };
    let method = req.method().clone();

    let response = next.run(req).await;

    let latency = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    let labels = [
        ("method", method.to_string()),
        ("path", path),
        ("status", status),
    ];

    metrics::counter!("http_requests_total", &labels).increment(1);
    metrics::histogram!("http_requests_duration_seconds", &labels).record(latency);

    response
}

pub async fn limiter(req: Request, next: Next, limiter: Arc<Limiter>) -> impl IntoResponse {
    static HEADER_ENV_VAR: Lazy<Option<String>> =
        Lazy::new(|| std::env::var("REQUEST_LIMIT_HEADER").ok());

    if let Some(header) = &*HEADER_ENV_VAR {
        let header_clone = req.headers().clone();
        let src = match header_clone.get(header) {
            Some(value) => value,
            None => {
                return (StatusCode::BAD_REQUEST, "Missing proxy header").into_response();
            }
        };

        if limiter.is_over_limit(&src.to_str().unwrap()).await {
            let retry_after = limiter.get_next_window().await + rand::random::<u64>() % 10;
            return (
                StatusCode::TOO_MANY_REQUESTS,
                [(header::RETRY_AFTER, retry_after.to_string())],
            )
                .into_response();
        } else {
            let size = req.body().size_hint().lower().clone();
            let response = next.run(req).await;
            limiter.count(src.to_str().unwrap(), &size).await;
            return response.into_response();
        }
    } else {
        return next.run(req).await;
    }
}
