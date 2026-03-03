package com.bonsai.amp.sdk;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.Test;

class AuthSignerTest {
  @Test
  void buildSignaturePayloadMatchesServerContract() {
    String payload =
        AuthSigner.buildSignaturePayload(
            "1700000000", "post", "/api/v1/discover?page=1", "{\"x\":1}", "nonce-123");

    assertEquals(
        "1700000000.POST./api/v1/discover?page=1.5041bf1f713df204784353e82f6a4a535931cb64f1f4b4a5aeaffcb720918b22.nonce-123",
        payload);

    String signature = AuthSigner.sign(payload, "secret");
    assertEquals("0bdf2e80f4c7d4c8f11b7ad5202eb909a1400223c16fe0514ce9a103edb13c7a", signature);
  }

  @Test
  void timestampFreshnessHelperRespectsClockSkew() {
    assertTrue(AuthSigner.isTimestampFresh("1700000000", 1700000008L, 10));
    assertFalse(AuthSigner.isTimestampFresh("1700000000", 1700000012L, 10));
    assertFalse(AuthSigner.isTimestampFresh("invalid", 1700000012L, 10));
  }
}
