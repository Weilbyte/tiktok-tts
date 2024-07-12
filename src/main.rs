mod db;
mod http;
mod structs;
mod tiktok;

use std::{env, str::FromStr, sync::Arc};

use tokio::time;
use tracing::{debug, info, Level};
use tracing_subscriber::FmtSubscriber;

use crate::db::{setup_database, TikTokTTSDatabaseFunctions};
use crate::structs::{Limiter, SessionIdCache};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing::subscriber::set_global_default(
        FmtSubscriber::builder()
            .with_max_level(
                Level::from_str(env::var("RUST_LOG").unwrap_or_default().as_mut_str())
                    .unwrap_or(Level::INFO),
            )
            .finish(),
    )
    .unwrap();

    let database = Arc::new(setup_database().await);
    let session_id_cache = Arc::new(SessionIdCache::new());
    let limiter = Arc::new(Limiter::new(
        std::env::var("REQUEST_LIMIT_BYTES")
            .map(|s| s.parse::<u64>().unwrap_or_else(|_| 4000))
            .unwrap_or(4000),
    ));

    let database_clone = database.clone();
    let session_id_cache_clone = session_id_cache.clone();
    let limiter_clone = limiter.clone();
    tokio::spawn(async move {
        info!("Refresher spawned");
        loop {
            let session_ids_current = database_clone.get_session_ids().await;
            debug!("Fetched {} session IDs!", &session_ids_current.len());
            session_id_cache_clone
                .set_session_ids(session_ids_current)
                .await;

            limiter_clone.tick().await;
            tokio::time::sleep(time::Duration::from_secs(60)).await;
        }
    });

    http::serve(database, session_id_cache, limiter).await;
}
