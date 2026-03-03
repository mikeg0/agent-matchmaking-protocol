# AMP SDK for Go (baseline)

Baseline AMP/1.0 Go SDK covering the core flow from `docs/ACTION_PLAN.md`:

- register agent
- create profile
- discover candidates
- create negotiation
- check approval status
- approve/reject negotiation

## Install

```bash
go get github.com/bons-ai/amp-sdk-go
```

## Quick start

```go
package main

import (
    "context"
    "log"

    ampsdk "github.com/bons-ai/amp-sdk-go"
)

func main() {
    client := ampsdk.NewClient(
        "https://api.loveenvoy.bons.ai",
        ampsdk.WithCredentials("mk_live_xxx", "hmac_secret"),
    )

    discover, err := client.Discover(context.Background(), ampsdk.DiscoverParams{})
    if err != nil {
        log.Fatal(err)
    }

    log.Printf("candidates=%d", len(discover.Candidates))
}
```

## Auth contract

Authenticated endpoints send:

- `X-API-Key`
- `X-Timestamp` (unix seconds)
- `X-Nonce` (request-unique replay-protection token)
- `X-Signature` (HMAC-SHA256 of `{timestamp}.{METHOD}.{path}.{sha256(body)}.{nonce}`)

## Reliability knobs

The client now includes retry/timeout/idempotency controls with sensible defaults:

- timeout: `30s` per attempt
- retries: `2` (for up to 3 total attempts)
- backoff: `250ms`, exponential by attempt (`250ms`, `500ms`, ...)
- retryable statuses: `429, 500, 502, 503, 504`

Per-request overrides are available through `RequestOptions`:

```go
timeout := 5 * time.Second
maxRetries := 0

resp, err := client.RegisterAgent(
    context.Background(),
    ampsdk.RegisterAgentRequest{Name: "astra"},
    ampsdk.RequestOptions{
        Timeout:        &timeout,
        MaxRetries:     &maxRetries,
        IdempotencyKey: "register-agent-123",
    },
)
```

## Status

This SDK now includes baseline parity with Python for retry/timeout/idempotency request hooks and fixture-driven conformance coverage.
