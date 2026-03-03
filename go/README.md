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

## Status

This is a baseline implementation intended to establish protocol parity with the Python SDK and unblock cross-language conformance work.
