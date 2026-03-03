import json
import unittest
from unittest.mock import patch

from amp_sdk import AmpClient, CredentialsError, CreateNegotiationRequest, RegisterAgentRequest
from amp_sdk.auth import (
    build_signature_payload,
    is_timestamp_fresh,
    sign,
    signed_headers,
)


class FakeHttpResponse:
    def __init__(self, payload: str):
        self._payload = payload.encode("utf-8")

    def read(self):
        return self._payload

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc, tb):
        return False


class AmpClientTests(unittest.TestCase):
    def test_signature_payload_matches_server_contract(self):
        payload = build_signature_payload(
            "1700000000",
            "post",
            "/api/v1/discover?page=1",
            "{\"x\":1}",
            "nonce-123",
        )
        self.assertEqual(
            payload,
            "1700000000.POST./api/v1/discover?page=1.5041bf1f713df204784353e82f6a4a535931cb64f1f4b4a5aeaffcb720918b22.nonce-123",
        )
        self.assertEqual(
            sign(payload, "secret"),
            "0bdf2e80f4c7d4c8f11b7ad5202eb909a1400223c16fe0514ce9a103edb13c7a",
        )

    def test_signed_headers_include_nonce(self):
        headers = signed_headers(
            api_key="le_key",
            hmac_secret="secret",
            method="GET",
            path_with_query="/api/v1/discover?page=1",
            body="",
            timestamp="1700000000",
            nonce="nonce-123",
        )

        self.assertEqual(headers["X-API-Key"], "le_key")
        self.assertEqual(headers["X-Timestamp"], "1700000000")
        self.assertEqual(headers["X-Nonce"], "nonce-123")
        self.assertEqual(
            headers["X-Signature"],
            "e518b4f21b0868e2ef11b6d5e3127e825d0e5b940bcbcae9be2d83b703248a66",
        )

    def test_timestamp_freshness_helper(self):
        self.assertTrue(is_timestamp_fresh("1700000000", now=1700000005, max_skew_seconds=10))
        self.assertFalse(is_timestamp_fresh("1700000000", now=1700000101, max_skew_seconds=10))
        self.assertFalse(is_timestamp_fresh("invalid", now=1700000000, max_skew_seconds=10))

    @patch("amp_sdk.client.urlopen")
    def test_register_agent_round_trip(self, mock_urlopen):
        mock_urlopen.return_value = FakeHttpResponse(
            json.dumps(
                {
                    "agent_id": "11111111-1111-1111-1111-111111111111",
                    "api_key": "le_test",
                    "status": "pending_human_verify",
                }
            )
        )
        client = AmpClient("https://api.example.com")
        result = client.register_agent(RegisterAgentRequest(name="astra"))

        self.assertEqual(result.agent_id, "11111111-1111-1111-1111-111111111111")
        self.assertEqual(result.api_key, "le_test")

    def test_protected_endpoint_requires_credentials(self):
        client = AmpClient("https://api.example.com")
        with self.assertRaises(CredentialsError):
            client.create_negotiation(
                CreateNegotiationRequest(target_opaque_id="22222222-2222-2222-2222-222222222222")
            )


if __name__ == "__main__":
    unittest.main()
