use std::sync::Arc;
use axum::{
    http::{header, StatusCode},
    response::{Html, IntoResponse},
    extract::Path,
    Extension,
    extract
};
use base64_light::base64_encode_bytes;

use sailfish::TemplateOnce;
use futures::future::join_all;
use sqlx::{Pool, Postgres};
use tracing::{debug, info, warn};
use crate::db::TikTokTTSDatabaseFunctions;
use crate::http::templates;
use crate::structs::{GenerationRequest, SessionIdCache, TikTokResponseError};
use crate::tiktok::generate;

pub async fn handle_index() -> impl IntoResponse {
    Html(
        templates::IndexPage {}.render_once().unwrap()
    )
}

pub async fn handle_static(Path(filepath): Path<String>) -> impl IntoResponse {
    return match templates::StaticAsset::get(filepath.as_str()) {
        Some(v) => {
            (
                StatusCode::OK,
                [
                    (header::CONTENT_TYPE, mime_guess::from_path(filepath).first_or_text_plain().to_string()),
                    (header::CACHE_CONTROL, "public, max-age=43200;".to_string()) // Static assets cached for 12h
                ],
                v.data
            ).into_response()
        },
        None => {
            (StatusCode::NOT_FOUND).into_response()
        }
    }
}

pub async fn handle_api_status(Extension(session_id_cache): Extension<Arc<SessionIdCache>>) -> impl IntoResponse {
    if session_id_cache.is_empty().await {
        (StatusCode::SERVICE_UNAVAILABLE, "Temporarily unavailable due to TikTok authentication issue.").into_response()
    } else {
        (StatusCode::OK).into_response()
    }
}

pub async fn handle_api_generate(Extension(database): Extension<Arc<Pool<Postgres>>>, Extension(session_id_cache): Extension<Arc<SessionIdCache>>, extract::Json(payload): extract::Json<GenerationRequest>) -> impl IntoResponse {
    if session_id_cache.is_empty().await {
        return (StatusCode::SERVICE_UNAVAILABLE, "Temporarily unavailable due to TikTok authentication issue.").into_response()
    }

    let session_id = &session_id_cache.get_random_session_id().await.expect("No session IDs available for request.");

    let mut chunks= Vec::new();

    let mut selection = String::new();
    for character in payload.text.chars() {
        let character_length = character.len_utf8();
        if selection.len() + character_length > 300 {
            chunks.push(selection.clone());
            selection.clear();
        }

        selection.push(character)
    }

    if !selection.is_empty() {
        chunks.push(selection.clone());
        selection.clear();
    }

    let mut generated_all: Vec<u8> = Vec::new();

    let mut chunk_futures = Vec::new();
    for chunk in &chunks {
        debug!("Chunk {}: {chunk}", chunk.len());
        chunk_futures.push(generate(&chunk, &payload.voice, &session_id));
    }

    let chunk_futures_results = join_all(chunk_futures).await;
    for result in chunk_futures_results {
        match result {
            Ok(mut generation) => generated_all.append(&mut generation),
            Err(e) => {
                warn!("Generation failed: {e}");

                let status = match e {
                    TikTokResponseError::TextTooLongError |
                    TikTokResponseError::UnavailableVoiceError |
                    TikTokResponseError::CouldntLoadSpeechError |
                    TikTokResponseError::UnsupportedLanguage => StatusCode::BAD_REQUEST,
                    _ => StatusCode::INTERNAL_SERVER_ERROR
                };

                if let TikTokResponseError::BanError = e {
                    info!("Removing session ID {} due to detected ban.", &session_id);
                    session_id_cache.remove_session_id(&session_id).await;
                    database.remove_session_id(session_id).await;
                }

                return (status, format!("Could not generate: {e}")).into_response();
            }
        }
    }

    let byte_count = generated_all.len() as u64;

    metrics::counter!("tts_generation_bandwidth_estimate_bytes").increment(
        byte_count +
            (byte_count + (byte_count * 33 / 100)) + // TikTok API response base64 overhead
            1700 // Average non-audio data
    );

    if payload.base64.is_some_and(|x| x == true) {
        let encoded = base64_encode_bytes(&generated_all.as_slice());

        metrics::counter!("tts_generation_bandwidth_estimate_bytes").increment(
            encoded.len() as u64 - byte_count
        );
        metrics::counter!("tts_generation_base64_generations").increment(1);

        return (StatusCode::OK, encoded).into_response()
    } else {
        return (StatusCode::OK, generated_all).into_response()
    }
}