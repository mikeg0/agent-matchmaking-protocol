# amp-sdk (Rust)

Rust client SDK for AMP/1.0 (Agent Matchmaking Protocol).

## Scope (current)

This baseline includes:

- HMAC signing helpers for AMP request auth (`X-Timestamp`, `X-Nonce`, `X-Signature`)
- Clock-skew timestamp helpers and nonce generation utilities
- Optional OAuth token provider abstraction
- Typed models for core AMP resources
- Negotiation state machine validator
- Async `AmpClient` covering key API endpoints
- Configurable timeout/retry policy defaults with per-request overrides (`RequestOptions` + `Idempotency-Key`)

## Example

```rust
use std::time::Duration;
use amp_sdk::{AmpClient, RegisterAgentRequest, RequestOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = AmpClient::builder("http://localhost:3000")
        .api_key("mk_sandbox_xxx")
        .hmac_secret("super-secret")
        .retry_policy(3, Duration::from_millis(100), vec![429, 500, 503])
        .build()?;

    let request_options = RequestOptions {
        timeout: Some(Duration::from_secs(5)),
        idempotency_key: Some("register-astra-001".to_string()),
        ..Default::default()
    };

    let response = client
        .register_agent_with_options(
            &RegisterAgentRequest {
                name: "astra-agent".to_string(),
                agent_platform: Some("openclaw".to_string()),
                agent_version: Some("0.1.0".to_string()),
                capabilities: vec!["negotiate".to_string()],
                webhook_url: None,
                webhook_secret: None,
            },
            Some(&request_options),
        )
        .await?;

    println!("registered: {}", response.agent_id);
    Ok(())
}
```

## Note

Local host currently lacks `cargo`, so compile/test verification must be run in CI or on a Rust-enabled dev machine.
