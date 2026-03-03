package com.bonsai.amp.sdk;

import java.nio.charset.StandardCharsets;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.HexFormat;
import javax.crypto.Mac;
import javax.crypto.spec.SecretKeySpec;

/** HMAC signing helpers for AMP requests. */
public final class AuthSigner {
  private AuthSigner() {}

  /**
   * Builds the canonical payload used by Love Envoy HMAC verification.
   *
   * <p>Payload format: "{timestamp}.{METHOD}.{path}.{sha256(body)}".
   */
  public static String buildSignaturePayload(
      String timestamp, String method, String pathWithQuery, String body) {
    return "%s.%s.%s.%s"
        .formatted(timestamp, method.toUpperCase(), pathWithQuery, sha256Hex(body == null ? "" : body));
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

  private static String sha256Hex(String value) {
    try {
      MessageDigest digest = MessageDigest.getInstance("SHA-256");
      return HexFormat.of().formatHex(digest.digest(value.getBytes(StandardCharsets.UTF_8)));
    } catch (NoSuchAlgorithmException e) {
      throw new IllegalStateException("SHA-256 unavailable", e);
    }
  }
}
