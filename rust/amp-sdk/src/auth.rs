use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::error::Result;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone)]
pub struct HmacSigner {
    secret: Vec<u8>,
}

impl HmacSigner {
    pub fn new(secret: impl AsRef<[u8]>) -> Self {
        Self {
            secret: secret.as_ref().to_vec(),
        }
    }

    pub fn canonical_message(timestamp: &str, method: &str, path: &str, body: &str) -> String {
        Self::canonical_payload(timestamp, method, path, body, "")
    }

    pub fn canonical_payload(
        timestamp: &str,
        method: &str,
        path: &str,
        body: &str,
        nonce: &str,
    ) -> String {
        let body_hash = Sha256::digest(body.as_bytes());
        format!(
            "{}.{}.{}.{:x}.{}",
            timestamp,
            method.to_ascii_uppercase(),
            path,
            body_hash,
            nonce
        )
    }

    pub fn sign(&self, timestamp: &str, method: &str, path: &str, body: &str) -> String {
        self.sign_with_nonce(timestamp, method, path, body, "")
    }

    pub fn sign_with_nonce(
        &self,
        timestamp: &str,
        method: &str,
        path: &str,
        body: &str,
        nonce: &str,
    ) -> String {
        let message = Self::canonical_payload(timestamp, method, path, body, nonce);
        let mut mac = HmacSha256::new_from_slice(&self.secret)
            .expect("HMAC accepts secrets of any size for SHA256");
        mac.update(message.as_bytes());
        let bytes = mac.finalize().into_bytes();
        hex::encode(bytes)
    }

    pub fn verify(
        &self,
        timestamp: &str,
        method: &str,
        path: &str,
        body: &str,
        expected_hex: &str,
    ) -> bool {
        self.verify_with_nonce(timestamp, method, path, body, expected_hex, "")
    }

    pub fn verify_with_nonce(
        &self,
        timestamp: &str,
        method: &str,
        path: &str,
        body: &str,
        expected_hex: &str,
        nonce: &str,
    ) -> bool {
        let sig = self.sign_with_nonce(timestamp, method, path, body, nonce);
        sig.eq_ignore_ascii_case(expected_hex)
    }

    pub fn unix_timestamp_now() -> String {
        Utc::now().timestamp().to_string()
    }

    pub fn is_timestamp_fresh(timestamp: &str, now: DateTime<Utc>, max_skew_seconds: i64) -> bool {
        let parsed = match timestamp.parse::<i64>() {
            Ok(value) => value,
            Err(_) => return false,
        };

        let skew = if max_skew_seconds <= 0 {
            300
        } else {
            max_skew_seconds
        };

        (now.timestamp() - parsed).abs() <= skew
    }

    pub fn generate_nonce() -> String {
        Uuid::new_v4().simple().to_string()
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OAuthToken {
    pub access_token: String,
    #[serde(default)]
    pub token_type: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub expires_at: Option<DateTime<Utc>>,
}

impl OAuthToken {
    pub fn bearer_value(&self) -> String {
        match self.token_type.as_deref() {
            Some(kind) if !kind.trim().is_empty() => {
                format!("{} {}", kind.trim(), self.access_token)
            }
            _ => format!("Bearer {}", self.access_token),
        }
    }

    pub fn is_expired(&self, skew_seconds: i64) -> bool {
        match self.expires_at {
            Some(exp) => Utc::now() + chrono::Duration::seconds(skew_seconds) >= exp,
            None => false,
        }
    }
}

#[async_trait]
pub trait TokenProvider: Send + Sync {
    async fn token(&self) -> Result<OAuthToken>;
}

#[derive(Clone)]
pub struct OAuthTokenManager {
    provider: Arc<dyn TokenProvider>,
    cache: Arc<Mutex<Option<OAuthToken>>>,
    refresh_skew_seconds: i64,
}

impl OAuthTokenManager {
    pub fn new(provider: Arc<dyn TokenProvider>) -> Self {
        Self {
            provider,
            cache: Arc::new(Mutex::new(None)),
            refresh_skew_seconds: 30,
        }
    }

    pub fn with_refresh_skew_seconds(mut self, seconds: i64) -> Self {
        self.refresh_skew_seconds = seconds.max(0);
        self
    }

    pub async fn get_token(&self) -> Result<OAuthToken> {
        {
            let current = self.cache.lock().await;
            if let Some(token) = current.as_ref() {
                if !token.is_expired(self.refresh_skew_seconds) {
                    return Ok(token.clone());
                }
            }
        }

        let fresh = self.provider.token().await?;
        let mut current = self.cache.lock().await;
        *current = Some(fresh.clone());
        Ok(fresh)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::HmacSigner;

    #[test]
    fn signs_canonical_payload_with_nonce() {
        let signer = HmacSigner::new("secret");
        let payload = HmacSigner::canonical_payload(
            "1700000000",
            "post",
            "/api/v1/profiles",
            "{\"x\":1}",
            "nonce-123",
        );

        assert_eq!(
            payload,
            "1700000000.POST./api/v1/profiles.5041bf1f713df204784353e82f6a4a535931cb64f1f4b4a5aeaffcb720918b22.nonce-123"
        );

        let sig = signer.sign_with_nonce(
            "1700000000",
            "post",
            "/api/v1/profiles",
            "{\"x\":1}",
            "nonce-123",
        );

        assert_eq!(
            sig,
            "64f0f00c4f4dca1c28d74281479a433d3060c2959a3614430e78bbf10d3a53df"
        );

        assert!(signer.verify_with_nonce(
            "1700000000",
            "POST",
            "/api/v1/profiles",
            "{\"x\":1}",
            &sig,
            "nonce-123"
        ));
    }

    #[test]
    fn timestamp_freshness_helper_respects_clock_skew() {
        let now = Utc.timestamp_opt(1_700_000_010, 0).unwrap();
        assert!(HmacSigner::is_timestamp_fresh("1700000005", now, 10));
        assert!(!HmacSigner::is_timestamp_fresh("1700000000", now, 5));
        assert!(!HmacSigner::is_timestamp_fresh("invalid", now, 10));
    }
}
