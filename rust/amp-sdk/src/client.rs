use std::{sync::Arc, time::Duration};

use chrono::Utc;
use reqwest::{
    header::{AUTHORIZATION, CONTENT_TYPE},
    Method, Url,
};
use serde::{de::DeserializeOwned, Serialize};
use uuid::Uuid;

use crate::{
    auth::{HmacSigner, OAuthTokenManager, TokenProvider},
    error::{AmpError, Result},
    models::*,
};

#[derive(Clone)]
pub struct AmpClient {
    http: reqwest::Client,
    base_url: Url,
    api_key: Option<String>,
    signer: Option<HmacSigner>,
    token_manager: Option<OAuthTokenManager>,
}

pub struct AmpClientBuilder {
    base_url: String,
    api_key: Option<String>,
    hmac_secret: Option<String>,
    timeout: Duration,
    token_provider: Option<Arc<dyn TokenProvider>>,
}

impl AmpClientBuilder {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            api_key: None,
            hmac_secret: None,
            timeout: Duration::from_secs(30),
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

    pub fn token_provider(mut self, provider: Arc<dyn TokenProvider>) -> Self {
        self.token_provider = Some(provider);
        self
    }

    pub fn build(self) -> Result<AmpClient> {
        let base_url = Url::parse(&self.base_url)
            .map_err(|_| AmpError::InvalidUrl(self.base_url.clone()))?;
        let http = reqwest::Client::builder().timeout(self.timeout).build()?;

        Ok(AmpClient {
            http,
            base_url,
            api_key: self.api_key,
            signer: self.hmac_secret.map(HmacSigner::new),
            token_manager: self.token_provider.map(OAuthTokenManager::new),
        })
    }
}

impl AmpClient {
    pub fn builder(base_url: impl Into<String>) -> AmpClientBuilder {
        AmpClientBuilder::new(base_url)
    }

    pub async fn health(&self) -> Result<HealthResponse> {
        self.send_no_body(Method::GET, "/health", false).await
    }

    pub async fn register_agent(
        &self,
        request: &RegisterAgentRequest,
    ) -> Result<RegisterAgentResponse> {
        self.send(Method::POST, "/api/v1/agents/register", Some(request), false)
            .await
    }

    pub async fn link_human(&self, request: &LinkHumanRequest) -> Result<GenericResponse> {
        self.send(Method::POST, "/api/v1/agents/link-human", Some(request), true)
            .await
    }

    pub async fn verify_human(&self, request: &VerifyHumanRequest) -> Result<GenericResponse> {
        self.send(Method::POST, "/api/v1/agents/verify", Some(request), true)
            .await
    }

    pub async fn rotate_key(&self, request: &RotateKeyRequest) -> Result<RotateKeyResponse> {
        self.send(Method::POST, "/api/v1/agents/rotate-key", Some(request), true)
            .await
    }

    pub async fn agent_status(&self) -> Result<AgentStatus> {
        self.send_no_body(Method::GET, "/api/v1/agents/status", true)
            .await
    }

    pub async fn trust_score(&self, agent_id: Uuid) -> Result<TrustScoreEnvelope> {
        self.send_no_body(
            Method::GET,
            &format!("/api/v1/agents/{agent_id}/trust"),
            true,
        )
        .await
    }

    pub async fn create_profile(
        &self,
        request: &CreateProfileRequest,
    ) -> Result<CreateProfileResponse> {
        self.send(Method::POST, "/api/v1/profiles", Some(request), true)
            .await
    }

    pub async fn get_profile(&self, profile_id: Uuid) -> Result<ProfileResponse> {
        self.send_no_body(Method::GET, &format!("/api/v1/profiles/{profile_id}"), true)
            .await
    }

    pub async fn update_profile(
        &self,
        profile_id: Uuid,
        request: &CreateProfileRequest,
    ) -> Result<GenericResponse> {
        self.send(
            Method::PATCH,
            &format!("/api/v1/profiles/{profile_id}"),
            Some(request),
            true,
        )
        .await
    }

    pub async fn discover(&self, page: Option<u32>, limit: Option<u32>) -> Result<DiscoveryResponse> {
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

        self.send_no_body(Method::GET, &path, true).await
    }

    pub async fn signal_candidate(
        &self,
        profile_id: Uuid,
        request: &SignalRequest,
    ) -> Result<SignalResponse> {
        self.send(
            Method::POST,
            &format!("/api/v1/discover/signal/{profile_id}"),
            Some(request),
            true,
        )
        .await
    }

    pub async fn create_negotiation(
        &self,
        request: &CreateNegotiationRequest,
    ) -> Result<NegotiationEnvelope> {
        self.send(Method::POST, "/api/v1/negotiations", Some(request), true)
            .await
    }

    pub async fn list_negotiations_for_agent(
        &self,
        agent_id: Uuid,
    ) -> Result<NegotiationsEnvelope> {
        self.send_no_body(
            Method::GET,
            &format!("/api/v1/negotiations/agent/{agent_id}"),
            true,
        )
        .await
    }

    pub async fn get_negotiation(&self, negotiation_id: Uuid) -> Result<NegotiationDetailEnvelope> {
        self.send_no_body(
            Method::GET,
            &format!("/api/v1/negotiations/{negotiation_id}"),
            true,
        )
        .await
    }

    pub async fn respond_negotiation(
        &self,
        negotiation_id: Uuid,
        request: &RespondNegotiationRequest,
    ) -> Result<GenericResponse> {
        self.send(
            Method::POST,
            &format!("/api/v1/negotiations/{negotiation_id}/respond"),
            Some(request),
            true,
        )
        .await
    }

    pub async fn disclose_negotiation(
        &self,
        negotiation_id: Uuid,
        request: &NotesRequest,
    ) -> Result<DisclosureResponse> {
        self.send(
            Method::POST,
            &format!("/api/v1/negotiations/{negotiation_id}/disclose"),
            Some(request),
            true,
        )
        .await
    }

    pub async fn propose_meeting(
        &self,
        negotiation_id: Uuid,
        request: &NotesRequest,
    ) -> Result<GenericResponse> {
        self.send(
            Method::POST,
            &format!("/api/v1/negotiations/{negotiation_id}/propose-meeting"),
            Some(request),
            true,
        )
        .await
    }

    pub async fn accept_meeting(&self, negotiation_id: Uuid) -> Result<GenericResponse> {
        self.send_no_body(
            Method::POST,
            &format!("/api/v1/negotiations/{negotiation_id}/accept-meeting"),
            true,
        )
        .await
    }

    pub async fn activate_negotiation(
        &self,
        negotiation_id: Uuid,
        request: &NotesRequest,
    ) -> Result<GenericResponse> {
        self.send(
            Method::POST,
            &format!("/api/v1/negotiations/{negotiation_id}/activate"),
            Some(request),
            true,
        )
        .await
    }

    pub async fn withdraw_negotiation(
        &self,
        negotiation_id: Uuid,
        request: &WithdrawRequest,
    ) -> Result<GenericResponse> {
        self.send(
            Method::POST,
            &format!("/api/v1/negotiations/{negotiation_id}/withdraw"),
            Some(request),
            true,
        )
        .await
    }

    pub async fn human_approve_negotiation(
        &self,
        negotiation_id: Uuid,
        request: &HumanApproveRequest,
    ) -> Result<GenericResponse> {
        self.send(
            Method::POST,
            &format!("/api/v1/negotiations/{negotiation_id}/human-approve"),
            Some(request),
            true,
        )
        .await
    }

    pub async fn send_message(
        &self,
        negotiation_id: Uuid,
        request: &SendMessageRequest,
    ) -> Result<GenericResponse> {
        self.send(
            Method::POST,
            &format!("/api/v1/negotiations/{negotiation_id}/messages"),
            Some(request),
            true,
        )
        .await
    }

    pub async fn list_messages(&self, negotiation_id: Uuid) -> Result<MessagesEnvelope> {
        self.send_no_body(
            Method::GET,
            &format!("/api/v1/negotiations/{negotiation_id}/messages"),
            true,
        )
        .await
    }

    pub async fn approval_status(&self, negotiation_id: Uuid) -> Result<ApprovalStatus> {
        self.send_no_body(
            Method::GET,
            &format!("/api/v1/approvals/{negotiation_id}"),
            true,
        )
        .await
    }

    pub async fn approve_negotiation(
        &self,
        negotiation_id: Uuid,
        request: &HumanApproveRequest,
    ) -> Result<ApprovalDecisionResponse> {
        self.send(
            Method::POST,
            &format!("/api/v1/approvals/{negotiation_id}/approve"),
            Some(request),
            true,
        )
        .await
    }

    pub async fn reject_negotiation(
        &self,
        negotiation_id: Uuid,
        request: &NotesRequest,
    ) -> Result<ApprovalDecisionResponse> {
        self.send(
            Method::POST,
            &format!("/api/v1/approvals/{negotiation_id}/reject"),
            Some(request),
            true,
        )
        .await
    }

    async fn send_no_body<T>(&self, method: Method, path: &str, authenticated: bool) -> Result<T>
    where
        T: DeserializeOwned,
    {
        self.send::<T, serde_json::Value>(method, path, None, authenticated)
            .await
    }

    async fn send<T, B>(
        &self,
        method: Method,
        path: &str,
        body: Option<&B>,
        authenticated: bool,
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

        let mut req = self.http.request(method.clone(), url);

        if let Some(json) = body_json.as_deref() {
            req = req
                .header(CONTENT_TYPE, "application/json")
                .body(json.to_string());
        }

        if authenticated {
            let (api_key, signer) = match (&self.api_key, &self.signer) {
                (Some(k), Some(s)) => (k, s),
                _ => return Err(AmpError::MissingCredentials),
            };

            let timestamp = Utc::now().timestamp().to_string();
            let signature = signer.sign(
                &timestamp,
                method.as_str(),
                &canonical_path,
                body_json.as_deref().unwrap_or(""),
            );

            req = req
                 .header("X-API-Key", api_key.as_str())
                .header("X-Timestamp", timestamp)
                .header("X-Signature", signature);

            if let Some(tokens) = &self.token_manager {
                let token = tokens.get_token().await?;
                req = req.header(AUTHORIZATION, token.bearer_value());
            }
        }

        let response = req.send().await?;
        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            let message = body
                .lines()
                .next()
                .unwrap_or("request failed")
                .to_string();
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

        Ok(serde_json::from_str(&normalized)?)
    }
}
