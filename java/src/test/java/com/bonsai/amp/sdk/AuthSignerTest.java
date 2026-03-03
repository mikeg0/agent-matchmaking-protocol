package com.bonsai.amp.sdk;

import static org.junit.jupiter.api.Assertions.assertEquals;

import org.junit.jupiter.api.Test;

class AuthSignerTest {
  @Test
  void buildSignaturePayloadMatchesServerContract() {
    String payload =
        AuthSigner.buildSignaturePayload(
            "1700000000", "post", "/api/v1/discover?page=1", "{\"x\":1}");

    assertEquals(
        "1700000000.POST./api/v1/discover?page=1.5041bf1f713df204784353e82f6a4a535931cb64f1f4b4a5aeaffcb720918b22",
        payload);

    String signature = AuthSigner.sign(payload, "secret");
    assertEquals("91cabb616cba2f5c780d8d3f08569bfefd26380639d41ebc3a38a1745fec4016", signature);
  }
}
