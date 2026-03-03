use std::{collections::HashSet, sync::Arc, time::Duration};

use reqwest::{
    header::{AUTHORIZATION, CONTENT_TYPE},
    Method, Url,
};
use serde::{de::DeserializeOwned, Serialize};
use tokio::time::sleep;
use uuid::Uuid;

use crate::{
    auth::{HmacSigner, OAuthTokenManager, TokenProvider},
    error::{AmpError, Result},
    models::*,
};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_MAX_RETRIES: u32 = 2;
const DEFAULT_RETRY_BACKOFF: Duration = Duration::from_millis(250);
const DEFAULT_RETRY_STATUS_CODES: [u16; 5] = [429, 500, 502, 503, 504];

#[derive(Debug, Clone, Default)]
pub struct RequestOptions {
    pub timeout: Option<Duration>,
    pub max_retries: Option<u32>,
    pub retry_backoff: Option<Duration>,
    pub retry_status_codes: Option<Vec<u16>>,
    pub idempotency_key: Option<String>,
}

#[derive(Clone)]
pub struct AmpClient {
    http: reqwest::Client,
    base_url: Url,
    api_key: Option<String>,
    signer: Option<HmacSigner>,
    token_manager: Option<OAuthTokenManager>,
    timeout: Duration,
    max_retries: u32,
    retry_backoff: Duration,
    retry_status_codes: HashSet<u16>,
}

pub struct AmpClientBuilder {
    base_url: String,
    api_key: Option<String>,
    hmac_secret: Option<String>,
    timeout: Duration,
    max_retries: u32,
    retry_backoff: Duration,
    retry_status_codes: HashSet<u16>,
    token_provider: Option<Arc<dyn TokenProvider>>,
}

#[derive(Clone)]
struct ResolvedRequestOptions {
    timeout: Duration,
    max_retries: u32,
    retry_backoff: Duration,
    retry_status_codes: HashSet<u16>,
    idempotency_key: Option<String>,
}

impl AmpClientBuilder {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            api_key: None,
            hmac_secret: None,
            timeout: DEFAULT_TIMEOUT,
            max_retries: DEFAULT_MAX_RETRIES,
            retry_backoff: DEFAULT_RETRY_BACKOFF,
            retry_status_codes: DEFAULT_RETRY_STATUS_CODES.into_iter().collect(),
            token_provider: None,
        }
    }

    pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    pub fn hmac_secret(mut self, hmac_secret: impl Into<String>) -> Self {
        self.hmac_secret = Some(hmac_secret.into());
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn retry_policy(
        mut self,
        max_retries: u32,
        retry_backoff: Duration,
        retry_status_codes: Vec<u16>,
    ) -> Self {
        self.max_retries = max_retries;
        self.retry_backoff = retry_backoff;
        self.retry_status_codes = retry_status_codes.into_iter().collect();
        self
    }

    pub fn token_provider(mut self, provider: Arc<dyn TokenProvider>) -> Self {
        self.token_provider = Some(provider);
        self
    }

    pub fn build(self) -> Result<AmpClient> {
        let base_url =
            Url::parse(&self.base_url).map_err(|_| AmpError::InvalidUrl(self.base_url.clone()))?;
        let http = reqwest::Client::builder().build()?;

        Ok(AmpClient {
            http,
            base_url,
            api_key: self.api_key,
            signer: self.hmac_secret.map(HmacSigner::new),
            token_manager: self.token_provider.map(OAuthTokenManager::new),
            timeout: self.timeout,
            max_retries: self.max_retries,
            retry_backoff: self.retry_backoff,
            retry_status_codes: self.retry_status_codes,
        })
    }
}

impl AmpClient {
    pub fn builder(base_url: impl Into<String>) -> AmpClientBuilder {
        AmpClientBuilder::new(base_url)
    }

    pub async fn health(&self) -> Result<HealthResponse> {
        self.health_with_options(None).await
    }

    pub async fn health_with_options(
        &self,
        request_options: Option<&RequestOptions>,
    ) -> Result<HealthResponse> {
        self.send_no_body(Method::GET, "/health", false, request_options)
            .await
    }

    pub async fn register_agent(&self, request: &RegisterAgentRequest) -> Result<RegisterAgentResponse> {
        self.register_agent_with_options(request, None).await
    }

    pub async fn register_agent_with_options(
        &self,
        request: &RegisterAgentRequest,
        request_options: Option<&RequestOptions>,
    ) -> Result<RegisterAgentResponse> {
        self.send(
            Method::POST,
            "/api/v1/agents/register",
            Some(request),
            false,
            request_options,
        )
        .await
    }

    pub async fn link_human(&self, request: &LinkHumanRequest) -> Result<GenericResponse> {
        self.link_human_with_options(request, None).await
    }

    pub async fn link_human_with_options(
        &self,
        request: &LinkHumanRequest,
        request_options: Option<&RequestOptions>,
    ) -> Result<GenericResponse> {
        self.send(
            Method::POST,
            "/api/v1/agents/link-human",
            Some(request),
            true,
            request_options,
        )
        .await
    }

    pub async fn verify_human(&self, request: &VerifyHumanRequest) -> Result<GenericResponse> {
        self.verify_human_with_options(request, None).await
    }

    pub async fn verify_human_with_options(
        &self,
        request: &VerifyHumanRequest,
        request_options: Option<&RequestOptions>,
    ) -> Result<GenericResponse> {
        self.send(
            Method::POST,
            "/api/v1/agents/verify",
            Some(request),
            true,
            request_options,
        )
        .await
    }

    pub async fn rotate_key(&self, request: &RotateKeyRequest) -> Result<RotateKeyResponse> {
        self.rotate_key_with_options(request, None).await
    }

    pub async fn rotate_key_with_options(
        &self,
        request: &RotateKeyRequest,
        request_options: Option<&RequestOptions>,
    ) -> Result<RotateKeyResponse> {
        self.send(
            Method::POST,
            "/api/v1/agents/rotate-key",
            Some(request),
            true,
            request_options,
        )
        .await
    }

    pub async fn agent_status(&self) -> Result<AgentStatus> {
        self.agent_status_with_options(None).await
    }

    pub async fn agent_status_with_options(
        &self,
        request_options: Option<&RequestOptions>,
    ) -> Result<AgentStatus> {
        self.send_no_body(Method::GET, "/api/v1/agents/status", true, request_options)
            .await
    }

    pub async fn trust_score(&self, agent_id: Uuid) -> Result<TrustScoreEnvelope> {
        self.trust_score_with_options(agent_id, None).await
    }

    pub async fn trust_score_with_options(
        &self,
        agent_id: Uuid,
        request_options: Option<&RequestOptions>,
    ) -> Result<TrustScoreEnvelope> {
        self.send_no_body(
            Method::GET,
            &format!("/api/v1/agents/{agent_id}/trust"),
            true,
            request_options,
        )
        .await
    }

    pub async fn create_profile(&self, request: &CreateProfileRequest) -> Result<CreateProfileResponse> {
        self.create_profile_with_options(request, None).await
    }

    pub async fn create_profile_with_options(
        &self,
        request: &CreateProfileRequest,
        request_options: Option<&RequestOptions>,
    ) -> Result<CreateProfileResponse> {
        self.send(
            Method::POST,
            "/api/v1/profiles",
            Some(request),
            true,
            request_options,
        )
        .await
    }

    pub async fn get_profile(&self, profile_id: Uuid) -> Result<ProfileResponse> {
        self.get_profile_with_options(profile_id, None).await
    }

    pub async fn get_profile_with_options(
        &self,
        profile_id: Uuid,
        request_options: Option<&RequestOptions>,
    ) -> Result<ProfileResponse> {
        self.send_no_body(
            Method::GET,
            &format!("/api/v1/profiles/{profile_id}"),
            true,
            request_options,
        )
        .await
    }

    pub async fn update_profile(
        &self,
        profile_id: Uuid,
        request: &CreateProfileRequest,
    ) -> Result<GenericResponse> {
        self.update_profile_with_options(profile_id, request, None).await
    }

    pub async fn update_profile_with_options(
        &self,
        profile_id: Uuid,
        request: &CreateProfileRequest,
        request_options: Option<&RequestOptions>,
    ) -> Result<GenericResponse> {
        self.send(
            Method::PATCH,
            &format!("/api/v1/profiles/{profile_id}"),
            Some(request),
            true,
            request_options,
        )
        .await
    }

    pub async fn discover(&self, page: Option<u32>, limit: Option<u32>) -> Result<DiscoveryResponse> {
        self.discover_with_options(page, limit, None).await
    }

    pub async fn discover_with_options(
        &self,
        page: Option<u32>,
        limit: Option<u32>,
        request_options: Option<&RequestOptions>,
    ) -> Result<DiscoveryResponse> {
        let mut path = String::from("/api/v1/discover");
        let mut first = true;
        if let Some(p) = page {
            path.push(if first { '?' } else { '&' });
            path.push_str(&format!("page={p}"));
            first = false;
        }
        if let Some(l) = limit {
            path.push(if first { '?' } else { '&' });
            path.push_str(&format!("limit={l}"));
        }

        self.send_no_body(Method::GET, &path, true, request_options)
            .await
    }

    pub async fn signal_candidate(
        &self,
        profile_id: Uuid,
        request: &SignalRequest,
    ) -> Result<SignalResponse> {
        self.signal_candidate_with_options(profile_id, request, None)
            .await
    }

    pub async fn signal_candidate_with_options(
        &self,
        profile_id: Uuid,
        request: &SignalRequest,
        request_options: Option<&RequestOptions>,
    ) -> Result<SignalResponse> {
        self.send(
            Method::POST,
            &format!("/api/v1/discover/signal/{profile_id}"),
            Some(request),
            true,
            request_options,
        )
        .await
    }

    pub async fn create_negotiation(
        &self,
        request: &CreateNegotiationRequest,
    ) -> Result<NegotiationEnvelope> {
        self.create_negotiation_with_options(request, None).await
    }

    pub async fn create_negotiation_with_options(
        &self,
        request: &CreateNegotiationRequest,
        request_options: Option<&RequestOptions>,
    ) -> Result<NegotiationEnvelope> {
        self.send(
            Method::POST,
            "/api/v1/negotiations",
            Some(request),
            true,
            request_options,
        )
        .await
    }

    pub async fn list_negotiations_for_agent(&self, agent_id: Uuid) -> Result<NegotiationsEnvelope> {
        self.list_negotiations_for_agent_with_options(agent_id, None)
            .await
    }

    pub async fn list_negotiations_for_agent_with_options(
        &self,
        agent_id: Uuid,
        request_options: Option<&RequestOptions>,
    ) -> Result<NegotiationsEnvelope> {
        self.send_no_body(
            Method::GET,
            &format!("/api/v1/negotiations/agent/{agent_id}"),
            true,
            request_options,
        )
        .await
    }

    pub async fn get_negotiation(&self, negotiation_id: Uuid) -> Result<NegotiationDetailEnvelope> {
        self.get_negotiation_with_options(negotiation_id, None).await
    }

    pub async fn get_negotiation_with_options(
        &self,
        negotiation_id: Uuid,
        request_options: Option<&RequestOptions>,
    ) -> Result<NegotiationDetailEnvelope> {
        self.send_no_body(
            Method::GET,
            &format!("/api/v1/negotiations/{negotiation_id}"),
            true,
            request_options,
        )
        .await
    }

    pub async fn respond_negotiation(
        &self,
        negotiation_id: Uuid,
        request: &RespondNegotiationRequest,
    ) -> Result<GenericResponse> {
        self.respond_negotiation_with_options(negotiation_id, request, None)
            .await
    }

    pub async fn respond_negotiation_with_options(
        &self,
        negotiation_id: Uuid,
        request: &RespondNegotiationRequest,
        request_options: Option<&RequestOptions>,
    ) -> Result<GenericResponse> {
        self.send(
            Method::POST,
            &format!("/api/v1/negotiations/{negotiation_id}/respond"),
            Some(request),
            true,
            request_options,
        )
        .await
    }

    pub async fn disclose_negotiation(
        &self,
        negotiation_id: Uuid,
        request: &NotesRequest,
    ) -> Result<DisclosureResponse> {
        self.disclose_negotiation_with_options(negotiation_id, request, None)
            .await
    }

    pub async fn disclose_negotiation_with_options(
        &self,
        negotiation_id: Uuid,
        request: &NotesRequest,
        request_options: Option<&RequestOptions>,
    ) -> Result<DisclosureResponse> {
        self.send(
            Method::POST,
            &format!("/api/v1/negotiations/{negotiation_id}/disclose"),
            Some(request),
            true,
            request_options,
        )
        .await
    }

    pub async fn propose_meeting(
        &self,
        negotiation_id: Uuid,
        request: &NotesRequest,
    ) -> Result<GenericResponse> {
        self.propose_meeting_with_options(negotiation_id, request, None)
            .await
    }

    pub async fn propose_meeting_with_options(
        &self,
        negotiation_id: Uuid,
        request: &NotesRequest,
        request_options: Option<&RequestOptions>,
    ) -> Result<GenericResponse> {
        self.send(
            Method::POST,
            &format!("/api/v1/negotiations/{negotiation_id}/propose-meeting"),
            Some(request),
            true,
            request_options,
        )
        .await
    }

    pub async fn accept_meeting(&self, negotiation_id: Uuid) -> Result<GenericResponse> {
        self.accept_meeting_with_options(negotiation_id, None).await
    }

    pub async fn accept_meeting_with_options(
        &self,
        negotiation_id: Uuid,
        request_options: Option<&RequestOptions>,
    ) -> Result<GenericResponse> {
        self.send_no_body(
            Method::POST,
            &format!("/api/v1/negotiations/{negotiation_id}/accept-meeting"),
            true,
            request_options,
        )
        .await
    }

    pub async fn activate_negotiation(
        &self,
        negotiation_id: Uuid,
        request: &NotesRequest,
    ) -> Result<GenericResponse> {
        self.activate_negotiation_with_options(negotiation_id, request, None)
            .await
    }

    pub async fn activate_negotiation_with_options(
        &self,
        negotiation_id: Uuid,
        request: &NotesRequest,
        request_options: Option<&RequestOptions>,
    ) -> Result<GenericResponse> {
        self.send(
            Method::POST,
            &format!("/api/v1/negotiations/{negotiation_id}/activate"),
            Some(request),
            true,
            request_options,
        )
        .await
    }

    pub async fn withdraw_negotiation(
        &self,
        negotiation_id: Uuid,
        request: &WithdrawRequest,
    ) -> Result<GenericResponse> {
        self.withdraw_negotiation_with_options(negotiation_id, request, None)
            .await
    }

    pub async fn withdraw_negotiation_with_options(
        &self,
        negotiation_id: Uuid,
        request: &WithdrawRequest,
        request_options: Option<&RequestOptions>,
    ) -> Result<GenericResponse> {
        self.send(
            Method::POST,
            &format!("/api/v1/negotiations/{negotiation_id}/withdraw"),
            Some(request),
            true,
            request_options,
        )
        .await
    }

    pub async fn human_approve_negotiation(
        &self,
        negotiation_id: Uuid,
        request: &HumanApproveRequest,
    ) -> Result<GenericResponse> {
        self.human_approve_negotiation_with_options(negotiation_id, request, None)
            .await
    }

    pub async fn human_approve_negotiation_with_options(
        &self,
        negotiation_id: Uuid,
        request: &HumanApproveRequest,
        request_options: Option<&RequestOptions>,
    ) -> Result<GenericResponse> {
        self.send(
            Method::POST,
            &format!("/api/v1/negotiations/{negotiation_id}/human-approve"),
            Some(request),
            true,
            request_options,
        )
        .await
    }

    pub async fn send_message(
        &self,
        negotiation_id: Uuid,
        request: &SendMessageRequest,
    ) -> Result<GenericResponse> {
        self.send_message_with_options(negotiation_id, request, None).await
    }

    pub async fn send_message_with_options(
        &self,
        negotiation_id: Uuid,
        request: &SendMessageRequest,
        request_options: Option<&RequestOptions>,
    ) -> Result<GenericResponse> {
        self.send(
            Method::POST,
            &format!("/api/v1/negotiations/{negotiation_id}/messages"),
            Some(request),
            true,
            request_options,
        )
        .await
    }

    pub async fn list_messages(&self, negotiation_id: Uuid) -> Result<MessagesEnvelope> {
        self.list_messages_with_options(negotiation_id, None).await
    }

    pub async fn list_messages_with_options(
        &self,
        negotiation_id: Uuid,
        request_options: Option<&RequestOptions>,
    ) -> Result<MessagesEnvelope> {
        self.send_no_body(
            Method::GET,
            &format!("/api/v1/negotiations/{negotiation_id}/messages"),
            true,
            request_options,
        )
        .await
    }

    pub async fn approval_status(&self, negotiation_id: Uuid) -> Result<ApprovalStatus> {
        self.approval_status_with_options(negotiation_id, None).await
    }

    pub async fn approval_status_with_options(
        &self,
        negotiation_id: Uuid,
        request_options: Option<&RequestOptions>,
    ) -> Result<ApprovalStatus> {
        self.send_no_body(
            Method::GET,
            &format!("/api/v1/approvals/{negotiation_id}"),
            true,
            request_options,
        )
        .await
    }

    pub async fn approve_negotiation(
        &self,
        negotiation_id: Uuid,
        request: &HumanApproveRequest,
    ) -> Result<ApprovalDecisionResponse> {
        self.approve_negotiation_with_options(negotiation_id, request, None)
            .await
    }

    pub async fn approve_negotiation_with_options(
        &self,
        negotiation_id: Uuid,
        request: &HumanApproveRequest,
        request_options: Option<&RequestOptions>,
    ) -> Result<ApprovalDecisionResponse> {
        self.send(
            Method::POST,
            &format!("/api/v1/approvals/{negotiation_id}/approve"),
            Some(request),
            true,
            request_options,
        )
        .await
    }

    pub async fn reject_negotiation(
        &self,
        negotiation_id: Uuid,
        request: &NotesRequest,
    ) -> Result<ApprovalDecisionResponse> {
        self.reject_negotiation_with_options(negotiation_id, request, None)
            .await
    }

    pub async fn reject_negotiation_with_options(
        &self,
        negotiation_id: Uuid,
        request: &NotesRequest,
        request_options: Option<&RequestOptions>,
    ) -> Result<ApprovalDecisionResponse> {
        self.send(
            Method::POST,
            &format!("/api/v1/approvals/{negotiation_id}/reject"),
            Some(request),
            true,
            request_options,
        )
        .await
    }

    fn resolve_request_options(&self, request_options: Option<&RequestOptions>) -> ResolvedRequestOptions {
        let mut resolved = ResolvedRequestOptions {
            timeout: self.timeout,
            max_retries: self.max_retries,
            retry_backoff: self.retry_backoff,
            retry_status_codes: self.retry_status_codes.clone(),
            idempotency_key: None,
        };

        if let Some(options) = request_options {
            if let Some(timeout) = options.timeout {
                resolved.timeout = timeout;
            }
            if let Some(max_retries) = options.max_retries {
                resolved.max_retries = max_retries;
            }
            if let Some(retry_backoff) = options.retry_backoff {
                resolved.retry_backoff = retry_backoff;
            }
            if let Some(retry_status_codes) = &options.retry_status_codes {
                resolved.retry_status_codes = retry_status_codes.iter().copied().collect();
            }
            resolved.idempotency_key = options
                .idempotency_key
                .as_ref()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty());
        }

        resolved
    }

    async fn send_no_body<T>(
        &self,
        method: Method,
        path: &str,
        authenticated: bool,
        request_options: Option<&RequestOptions>,
    ) -> Result<T>
    where
        T: DeserializeOwned,
    {
        self.send::<T, serde_json::Value>(method, path, None, authenticated, request_options)
            .await
    }

    async fn send<T, B>(
        &self,
        method: Method,
        path: &str,
        body: Option<&B>,
        authenticated: bool,
        request_options: Option<&RequestOptions>,
    ) -> Result<T>
    where
        T: DeserializeOwned,
        B: Serialize + ?Sized,
    {
        let canonical_path = if path.starts_with('/') {
            path.to_string()
        } else {
            format!("/{path}")
        };

        let url = self
            .base_url
            .join(canonical_path.trim_start_matches('/'))
            .map_err(|_| AmpError::InvalidUrl(canonical_path.clone()))?;

        let body_json = match body {
            Some(payload) => Some(serde_json::to_string(payload)?),
            None => None,
        };

        let resolved_options = self.resolve_request_options(request_options);

        let mut attempt: u32 = 0;
        loop {
            let mut req = self
                .http
                .request(method.clone(), url.clone())
                .header("Accept", "application/json")
                .timeout(resolved_options.timeout);

            if let Some(json) = body_json.as_deref() {
                req = req
                    .header(CONTENT_TYPE, "application/json")
                    .body(json.to_string());
            }

            if let Some(idempotency_key) = resolved_options.idempotency_key.as_deref() {
                req = req.header("Idempotency-Key", idempotency_key);
            }

            if authenticated {
                let (api_key, signer) = match (&self.api_key, &self.signer) {
                    (Some(k), Some(s)) => (k, s),
                    _ => return Err(AmpError::MissingCredentials),
                };

                let timestamp = HmacSigner::unix_timestamp_now();
                let nonce = HmacSigner::generate_nonce();
                let signature = signer.sign_with_nonce(
                    &timestamp,
                    method.as_str(),
                    &canonical_path,
                    body_json.as_deref().unwrap_or(""),
                    &nonce,
                );

                req = req
                    .header("X-API-Key", api_key.as_str())
                    .header("X-Timestamp", timestamp)
                    .header("X-Nonce", nonce)
                    .header("X-Signature", signature);

                if let Some(tokens) = &self.token_manager {
                    let token = tokens.get_token().await?;
                    req = req.header(AUTHORIZATION, token.bearer_value());
                }
            }

            let response = match req.send().await {
                Ok(response) => response,
                Err(error) => {
                    if should_retry_attempt(attempt, resolved_options.max_retries) {
                        wait_backoff(resolved_options.retry_backoff, attempt).await;
                        attempt += 1;
                        continue;
                    }
                    return Err(AmpError::Request(error));
                }
            };

            let status = response.status();
            let body = response.text().await?;

            if !status.is_success() {
                if should_retry_attempt(attempt, resolved_options.max_retries)
                    && should_retry_status(status.as_u16(), &resolved_options.retry_status_codes)
                {
                    wait_backoff(resolved_options.retry_backoff, attempt).await;
                    attempt += 1;
                    continue;
                }

                let message = body.lines().next().unwrap_or("request failed").to_string();
                return Err(AmpError::HttpStatus {
                    status: status.as_u16(),
                    message,
                    body: if body.is_empty() { None } else { Some(body) },
                });
            }

            let normalized = if body.trim().is_empty() {
                "{}".to_string()
            } else {
                body
            };

            return Ok(serde_json::from_str(&normalized)?);
        }
    }
}

fn should_retry_attempt(attempt: u32, max_retries: u32) -> bool {
    attempt < max_retries
}

fn should_retry_status(status: u16, retry_status_codes: &HashSet<u16>) -> bool {
    retry_status_codes.contains(&status)
}

async fn wait_backoff(retry_backoff: Duration, attempt: u32) {
    if retry_backoff.is_zero() {
        return;
    }

    let factor = 2_u32.saturating_pow(attempt);
    let delay = retry_backoff * factor;
    sleep(delay).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_retry_status_codes_include_503() {
        let client = AmpClient::builder("https://example.com").build().unwrap();
        assert!(client.retry_status_codes.contains(&503));
    }

    #[test]
    fn request_options_override_defaults() {
        let client = AmpClient::builder("https://example.com")
            .timeout(Duration::from_secs(30))
            .retry_policy(2, Duration::from_millis(250), vec![503])
            .build()
            .unwrap();

        let options = RequestOptions {
            timeout: Some(Duration::from_secs(5)),
            max_retries: Some(4),
            retry_backoff: Some(Duration::from_millis(10)),
            retry_status_codes: Some(vec![429, 500]),
            idempotency_key: Some(" idem-123 ".to_string()),
        };

        let resolved = client.resolve_request_options(Some(&options));

        assert_eq!(resolved.timeout, Duration::from_secs(5));
        assert_eq!(resolved.max_retries, 4);
        assert_eq!(resolved.retry_backoff, Duration::from_millis(10));
        assert!(resolved.retry_status_codes.contains(&429));
        assert!(!resolved.retry_status_codes.contains(&503));
        assert_eq!(resolved.idempotency_key.as_deref(), Some("idem-123"));
    }

    #[test]
    fn should_retry_attempt_stops_after_max_retries() {
        assert!(should_retry_attempt(0, 2));
        assert!(should_retry_attempt(1, 2));
        assert!(!should_retry_attempt(2, 2));
    }
}
