mod handlers;
mod middleware;
mod prometheus;
mod templates;

use axum::extract::DefaultBodyLimit;
use axum::{
    routing::{get, post},
    Extension, Router,
};
use std::sync::Arc;

use sqlx::{Pool, Postgres};
use tower_http::compression::CompressionLayer;

use crate::http::handlers::{handle_api_generate, handle_api_status, handle_index, handle_static};
use crate::http::prometheus::setup_handle;
use crate::structs::{Limiter, SessionIdCache};
use tracing::{debug, error, info, warn};

pub async fn serve(
    database: Arc<Pool<Postgres>>,
    session_id_cache: Arc<SessionIdCache>,
    limiter: Arc<Limiter>,
) {
    let prometheus_handle = setup_handle();

    if std::env::var("ENABLE_METRICS").is_ok() {
        debug!("Spawning metrics thread");

        tokio::spawn(async move {
            let listener = match tokio::net::TcpListener::bind("0.0.0.0:3001").await {
                Ok(v) => v,
                Err(e) => {
                    warn!("Error starting metrics server: {}", e);
                    return;
                }
            };

            info!(
                "Metrics server listening on {}",
                listener.local_addr().unwrap()
            );

            axum::serve(
                listener,
                Router::new().route(
                    "/metrics",
                    get(|| async move { prometheus_handle.render() }),
                ),
            )
            .await
            .unwrap()
        });
    }

    let listener = match tokio::net::TcpListener::bind("0.0.0.0:3000").await {
        Ok(v) => v,
        Err(e) => {
            error!("Error starting server: {}", e);
            std::process::exit(1);
        }
    };

    info!(
        "Application server listening on {}",
        listener.local_addr().unwrap()
    );

    let mut app = Router::new()
        .route("/api/status", get(handle_api_status))
        .route(
            "/api/generate",
            post(handle_api_generate)
                .layer(DefaultBodyLimit::max(
                    std::env::var("MAXIMUM_BODY_LIMIT")
                        .map(|s| s.parse::<usize>().unwrap_or_else(|_| 5000))
                        .unwrap_or(5000),
                ))
                .layer(axum::middleware::from_fn(move |req, next| {
                    middleware::limiter(req, next, limiter.clone())
                })),
        );

    if std::env::var("ENABLE_FRONTEND").is_ok() {
        app = app
            .route("/", get(handle_index))
            .route("/static/:filepath", get(handle_static))
    }

    app = app
        .layer(Extension(database))
        .layer(Extension(session_id_cache))
        .layer(axum::middleware::from_fn(middleware::track_metrics))
        .layer(CompressionLayer::new().deflate(true).gzip(true));

    axum::serve(listener, app).await.unwrap();
}
