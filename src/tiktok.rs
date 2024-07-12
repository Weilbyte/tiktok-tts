use std::sync::OnceLock;
use reqwest::Client;
use tracing::{error, warn};
use urlencoding::encode;
use base64_light::base64_decode;
use crate::structs::{DeviceIdentity,TikTokResponseError, TikTokResponseRoot};

const TIKTOK_ENDPOINT : &str = "https://api16-normal-v6.tiktokv.com";

static REQWEST_CLIENT : OnceLock<Client> = OnceLock::new();

fn get_random_device_identity() -> DeviceIdentity {
    const MODELS: &[&str] = &[
        "SM-G988N"
    ];

    const OS_VERSIONS: &[&str] = &[
        "Android 7.1.2"
    ];

    const LANGUAGES: &[&str] = &[
        "es_ES"
    ];

    DeviceIdentity {
        model: MODELS[rand::random::<usize>() % MODELS.len()],
        os_version: OS_VERSIONS[rand::random::<usize>() % OS_VERSIONS.len()],
        language: LANGUAGES[rand::random::<usize>() % LANGUAGES.len()],
    }
}

pub async fn generate(text: &str, voice: &str, session_id: &str) -> Result<Vec<u8>, TikTokResponseError> {
    let client = REQWEST_CLIENT.get_or_init(|| {
        Client::builder().build().expect("Could not build reqwest client")
    });

    let identity = get_random_device_identity();

    let request = client.post(format!(
            "{TIKTOK_ENDPOINT}/media/api/text/speech/invoke/?text_speaker={}&req_text={}&aid=1233",
        encode(voice),
        encode(text)
        ))
        .header("User-Agent", format!(
            "com.zhiliaoapp.musically/2022600030 (Linux; U; Android {}; {}; {}; tt-ok/3.12.13.1)", identity.os_version, identity.language, identity.model
        )).header("Cookie", format!("sessionid={session_id}")).send().await;

    match request {
        Ok(resp) => {
            return match resp.json::<TikTokResponseRoot>().await {
                Ok(root) => {
                    let label = [
                        ("status_code", root.status_code.to_string()),
                        ("status_msg", root.status_msg.clone())
                    ];
                    metrics::counter!("tts_generation_status_code_responses", &label).increment(1);

                    match root.status_code {
                        0 => {
                            Ok(base64_decode(&root.data.unwrap().v_str))
                        },
                        1 => {
                            // Match "Text-to-speech isn’t supported for this language"
                            if root.status_msg.contains("this language") {
                                Err(TikTokResponseError::UnsupportedLanguage)
                            } else {
                                Err(TikTokResponseError::BanError)
                            }
                        },
                        2 => {
                            // Should not occur; indicates chunking error!
                            Err(TikTokResponseError::TextTooLongError).into()
                        },
                        3 => {
                            Err(TikTokResponseError::CouldntLoadSpeechError)
                        },
                        4 => {
                            Err(TikTokResponseError::UnavailableVoiceError)
                        }

                        _ => {
                            warn!("Unknown TikTok API response code: {}, {}", root.status_code, root.status_msg);
                            Err(TikTokResponseError::UnknownError(root.status_msg))
                        }
                    }
                },
                Err(e) => {
                    error!("Error parsing TikTok API request body: {e}");
                    Err(TikTokResponseError::UnknownError("Error parsing API request body".to_string()))
                }
            }
        },
        Err(error) => {
            error!("TikTok API request failed: {error}");
            Err(TikTokResponseError::UnknownError("Error sending API request".to_string()))
        }
    }
}