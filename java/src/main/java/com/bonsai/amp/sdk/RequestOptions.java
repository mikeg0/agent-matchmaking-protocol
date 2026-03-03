package com.bonsai.amp.sdk;

import java.time.Duration;
import java.util.Set;

/** Per-request transport overrides for timeout/retries/idempotency. */
public record RequestOptions(
    Duration timeout,
    Integer maxRetries,
    Duration retryBackoff,
    Set<Integer> retryStatusCodes,
    String idempotencyKey) {
  public RequestOptions {
    retryStatusCodes = retryStatusCodes == null ? null : Set.copyOf(retryStatusCodes);
  }
}
