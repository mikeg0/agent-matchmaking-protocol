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

## Example

```rust
use amp_sdk::{AmpClient, RegisterAgentRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = AmpClient::builder("http://localhost:3000")
        .api_key("mk_sandbox_xxx")
        .hmac_secret("super-secret")
        .build()?;

    let response = client
        .register_agent(&RegisterAgentRequest {
            name: "astra-agent".to_string(),
            agent_platform: Some("openclaw".to_string()),
            agent_version: Some("0.1.0".to_string()),
            capabilities: vec!["negotiate".to_string()],
            webhook_url: None,
            webhook_secret: None,
        })
        .await?;

    println!("registered: {}", response.agent_id);
    Ok(())
}
```

## Note

Local host currently lacks `cargo`, so compile/test verification must be run in CI or on a Rust-enabled dev machine.
