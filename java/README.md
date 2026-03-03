# AMP SDK for Java (baseline)

Baseline AMP/1.0 Java SDK covering the core flow from `docs/ACTION_PLAN.md`:

- register agent
- create profile
- discover candidates
- create negotiation
- check approval status
- approve/reject negotiation

## Build + test

```bash
mvn test
```

## Quick start

```java
import com.bonsai.amp.sdk.AmpClient;

public class Example {
  public static void main(String[] args) {
    AmpClient client = new AmpClient(
        "https://api.loveenvoy.bons.ai",
        "mk_live_xxx",
        "hmac_secret"
    );

    var discover = client.discover(1, 20);
    System.out.println("candidates=" + discover.candidates().size());
  }
}
```

## Auth contract

Authenticated endpoints send:

- `X-API-Key`
- `X-Timestamp` (unix seconds)
- `X-Signature` (HMAC-SHA256 of `{timestamp}.{METHOD}.{path}.{sha256(body)}`)

## Notes

- Uses Java 17+ `java.net.http.HttpClient`
- Uses Jackson for JSON serialization/deserialization
- This is a baseline implementation for protocol parity with Python and Go SDKs
