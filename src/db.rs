use std::env;
use sqlx::{Pool, Postgres};
use tracing::{error, info};

pub async fn setup_database() -> Pool<Postgres> {
    let db_url = match env::var("DATABASE_URL") {
        Ok(url) => {
            url
        },
        Err(_) => {
            error!("FATAL: DATABASE_URL environment variable is missing");
            std::process::exit(1);
        }
    };

    return match sqlx::postgres::PgPoolOptions::new()
        .connect(&db_url)
        .await {
        Ok(v) => {
            info!("Connected to database; running migrations");

            sqlx::migrate!(".//migrations")
                .run(&v)
                .await
                .map_err(|e| {
                    error!("FATAL: Error running database migrations: {}", e);
                    std::process::exit(1);
                })
                .ok();

            v
        },
        Err(e) => {
            error!("Could not connect to database: {}", e);
            std::process::exit(1);
        }
    };
}

pub trait TikTokTTSDatabaseFunctions {
    async fn get_session_ids(&self) -> Vec<String>;
    async fn remove_session_id(&self, id: &str);
}

impl TikTokTTSDatabaseFunctions for Pool<Postgres> {
    async fn get_session_ids(&self) -> Vec<String> {
        let result = sqlx::query!(
            // language=PostgreSQL
            r#"SELECT session_id FROM sessionids WHERE is_active = true;"#
        ).fetch_all(self).await.expect("Could not fetch session IDs from database");

        let mut session_ids: Vec<String> = Vec::new();
        for record in result {
            session_ids.push(record.session_id);
        }

        session_ids
    }

    async fn remove_session_id(&self, id: &str) {
        sqlx::query!(
            // language=PostgreSQL
            r#"UPDATE sessionids SET is_active = false WHERE session_id = $1;"#,
            id
        ).execute(self).await.expect("Could not remove session ID from database");
    }
}