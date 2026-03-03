package com.bonsai.amp.sdk;

import java.nio.charset.StandardCharsets;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.security.SecureRandom;
import java.time.Instant;
import java.util.Base64;
import java.util.HashMap;
import java.util.HexFormat;
import java.util.Map;
import javax.crypto.Mac;
import javax.crypto.spec.SecretKeySpec;

/** HMAC signing helpers for AMP requests. */
public final class AuthSigner {
  private static final int DEFAULT_CLOCK_SKEW_SECONDS = 300;
  private static final SecureRandom RANDOM = new SecureRandom();

  private AuthSigner() {}

  /**
   * Builds the canonical payload used by Love Envoy HMAC verification.
   *
   * <p>Payload format: "{timestamp}.{METHOD}.{path}.{sha256(body)}.{nonce}".
   */
  public static String buildSignaturePayload(
      String timestamp, String method, String pathWithQuery, String body) {
    return buildSignaturePayload(timestamp, method, pathWithQuery, body, "");
  }

  /**
   * Builds the canonical payload used by Love Envoy HMAC verification.
   *
   * <p>Payload format: "{timestamp}.{METHOD}.{path}.{sha256(body)}.{nonce}".
   */
  public static String buildSignaturePayload(
      String timestamp, String method, String pathWithQuery, String body, String nonce) {
    return "%s.%s.%s.%s.%s"
        .formatted(
            timestamp,
            method.toUpperCase(),
            pathWithQuery,
            sha256Hex(body == null ? "" : body),
            nonce == null ? "" : nonce);
  }

  /** Returns a hex-encoded HMAC-SHA256 digest. */
  public static String sign(String payload, String secret) {
    try {
      Mac mac = Mac.getInstance("HmacSHA256");
      mac.init(new SecretKeySpec(secret.getBytes(StandardCharsets.UTF_8), "HmacSHA256"));
      return HexFormat.of().formatHex(mac.doFinal(payload.getBytes(StandardCharsets.UTF_8)));
    } catch (Exception e) {
      throw new IllegalStateException("failed to sign payload", e);
    }
  }

  /** Returns current unix timestamp (seconds) as a string. */
  public static String currentUnixTimestamp() {
    return Long.toString(Instant.now().getEpochSecond());
  }

  /** Returns true when timestamp is within acceptable clock skew. */
  public static boolean isTimestampFresh(String timestamp, long nowEpochSeconds, int maxSkewSeconds) {
    try {
      long ts = Long.parseLong(timestamp);
      int skew = Math.max(0, maxSkewSeconds <= 0 ? DEFAULT_CLOCK_SKEW_SECONDS : maxSkewSeconds);
      return Math.abs(nowEpochSeconds - ts) <= skew;
    } catch (NumberFormatException e) {
      return false;
    }
  }

  /** Generates a URL-safe nonce for X-Nonce / Idempotency-Key. */
  public static String generateNonce() {
    byte[] random = new byte[24];
    RANDOM.nextBytes(random);
    return Base64.getUrlEncoder().withoutPadding().encodeToString(random);
  }

  /** Build auth headers for a request, auto-filling timestamp + nonce if omitted. */
  public static Map<String, String> signedHeaders(
      String apiKey,
      String secret,
      String method,
      String pathWithQuery,
      String body,
      String timestamp,
      String nonce) {
    String ts = (timestamp == null || timestamp.isBlank()) ? currentUnixTimestamp() : timestamp;
    String requestNonce = (nonce == null || nonce.isBlank()) ? generateNonce() : nonce;

    String payload = buildSignaturePayload(ts, method, pathWithQuery, body, requestNonce);
    String signature = sign(payload, secret);

    Map<String, String> headers = new HashMap<>();
    headers.put("X-API-Key", apiKey);
    headers.put("X-Timestamp", ts);
    headers.put("X-Signature", signature);
    headers.put("X-Nonce", requestNonce);
    return headers;
  }

  private static String sha256Hex(String value) {
    try {
      MessageDigest digest = MessageDigest.getInstance("SHA-256");
      return HexFormat.of().formatHex(digest.digest(value.getBytes(StandardCharsets.UTF_8)));
    } catch (NoSuchAlgorithmException e) {
      throw new IllegalStateException("SHA-256 unavailable", e);
    }
  }
}
