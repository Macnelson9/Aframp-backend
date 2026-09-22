pub mod mock;
pub mod termii;

use async_trait::async_trait;

#[async_trait]
pub trait OtpProvider: Send + Sync {
    async fn send_sms(&self, phone: &str, message: &str) -> Result<(), String>;
}
