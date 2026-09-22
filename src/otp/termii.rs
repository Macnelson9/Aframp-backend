use async_trait::async_trait;
use serde::Deserialize;

use super::OtpProvider;

const BASE_URL: &str = "https://api.ng.termii.com/api";

pub struct TermiiProvider {
    api_key: String,
    sender_id: String,
    http: reqwest::Client,
}

impl TermiiProvider {
    pub fn new(api_key: String, sender_id: String) -> Self {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(20))
            .build()
            .expect("failed to build Termii HTTP client");
        Self { api_key, sender_id, http }
    }
}

#[derive(Debug, Deserialize)]
struct SendResponse {
    code: Option<String>,
    message: Option<String>,
}

#[async_trait]
impl OtpProvider for TermiiProvider {
    async fn send_sms(&self, phone: &str, message: &str) -> Result<(), String> {
        let response = self
            .http
            .post(format!("{BASE_URL}/sms/send"))
            .json(&serde_json::json!({
                "api_key": self.api_key,
                "to": phone,
                "from": self.sender_id,
                "sms": message,
                "type": "plain",
                // The generic route is promotional-only and won't deliver to
                // numbers on Do-Not-Disturb; OTP/transactional traffic must
                // go over the DND route.
                "channel": "dnd",
            }))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let status = response.status();
        let raw = response.text().await.map_err(|e| e.to_string())?;
        let body: SendResponse = serde_json::from_str(&raw).map_err(|e| {
            format!("Termii returned an unexpected response (HTTP {status}): {e} — body: {raw}")
        })?;

        if !status.is_success() || body.code.as_deref() != Some("ok") {
            let message = body.message.unwrap_or_else(|| raw.clone());
            return Err(format!("Termii error (HTTP {status}): {message}"));
        }
        Ok(())
    }
}
