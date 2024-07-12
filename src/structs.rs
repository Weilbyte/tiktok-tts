use std::{error, fmt};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::debug;

pub struct SessionIdCache {
    session_ids: RwLock<Vec<String>>,
}

impl SessionIdCache {
    pub fn new() -> Self {
        Self {
            session_ids: RwLock::new(Vec::new()),
        }
    }

    pub async fn set_session_ids(&self, session_ids_new: Vec<String>) {
        let mut session_ids = self.session_ids.write().await;
        *session_ids = session_ids_new;
    }

    pub async fn get_random_session_id(&self) -> Option<String> {
        let session_ids = self.session_ids.read().await;
        if session_ids.is_empty() {
            None
        } else {
            let index = rand::random::<usize>() % session_ids.len();
            Some(session_ids[index].clone())
        }
    }

    pub async fn remove_session_id(&self, session_id: &str) {
        let mut session_ids = self.session_ids.write().await;
        debug!("Removing from cache: {}", &session_id);
        session_ids.retain(|id| id != session_id);
    }

    pub async fn is_empty(&self) -> bool {
        return self.session_ids.read().await.is_empty()
    }
}


pub struct LimiterUsageRecord {
    pub source: String,
    pub amount: u64
}

pub struct Limiter {
    next_window: RwLock<u64>,
    records: RwLock<Vec<LimiterUsageRecord>>,
    bandwidth_limit: u64,
}

impl Limiter {
    pub fn new(limit_bytes: u64) -> Self {
        Self {
            next_window: RwLock::new(0),
            records: RwLock::new(Vec::new()),
            bandwidth_limit: limit_bytes
        }
    }

    pub async fn tick(&self) {
        let mut next_window = self.next_window.write().await;
        *next_window = Self::current_time() + 60;

        let mut records = self.records.write().await;
        *records = Vec::new();
    }

    pub async fn count(&self, source: &str, amount: &u64) {
        let mut records = self.records.write().await;

        for record in records.iter_mut() {
            if record.source == source {
                record.amount = record.amount + amount;
                return;
            }
        }

        records.push(LimiterUsageRecord {
            source: source.to_string(),
            amount: amount.clone()
        });
    }

    pub async fn is_over_limit(&self, source: &str) -> bool {
        let mut records = self.records.read().await;

        for record in records.iter() {
            if record.source == source {
                return record.amount > self.bandwidth_limit
            }
        }

        false
    }

    pub async fn get_next_window(&self) -> u64 {
        let mut next_window = self.next_window.read().await;
        next_window.clone()
    }

    fn current_time() -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
    }
}

#[derive(Deserialize)]
pub struct GenerationRequest {
    pub text: String,
    pub voice: String,
    pub base64: Option<bool>
}

pub struct DeviceIdentity {
    pub model: &'static str,
    pub os_version: &'static str,
    pub language: &'static str,
}

#[derive(Serialize, Deserialize)]
pub struct TikTokResponseRoot {
    pub data: Option<TikTokResponseData>,
    pub message: String,
    pub status_code: i8,
    pub status_msg: String,
}

#[derive(Serialize, Deserialize)]
pub struct TikTokResponseData {
    pub v_str: String,
}

#[derive(Debug)]
pub enum TikTokResponseError {
    UnavailableVoiceError,
    BanError,
    CouldntLoadSpeechError,
    TextTooLongError,
    UnsupportedLanguage,
    UnknownError(String), // For other API related errors with a message
}

impl fmt::Display for TikTokResponseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TikTokResponseError::UnavailableVoiceError => write!(f, "This voice is unavailable now"),
            TikTokResponseError::BanError => write!(f, "TikTok authentication issue. Try again."),
            TikTokResponseError::CouldntLoadSpeechError => write!(f, "Couldn't load speech. Try again."),
            TikTokResponseError::TextTooLongError => write!(f, "Text too long to create speech."),
            TikTokResponseError::UnsupportedLanguage => write!(f, "Text-to-speech isn’t supported for this language"),
            TikTokResponseError::UnknownError(msg) => write!(f, "Unknown error: {}", msg),
        }
    }
}

impl error::Error for TikTokResponseError {}
