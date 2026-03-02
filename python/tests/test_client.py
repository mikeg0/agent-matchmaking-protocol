import json
import unittest
from unittest.mock import patch

from amp_sdk import AmpClient, CredentialsError, CreateNegotiationRequest, RegisterAgentRequest
from amp_sdk.auth import build_signature_payload, sign


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
        payload = build_signature_payload("1700000000", "post", "/api/v1/discover?page=1", "{\"x\":1}")
        self.assertEqual(
            payload,
            "1700000000.POST./api/v1/discover?page=1.5041bf1f713df204784353e82f6a4a535931cb64f1f4b4a5aeaffcb720918b22",
        )
        self.assertEqual(
            sign(payload, "secret"),
            "91cabb616cba2f5c780d8d3f08569bfefd26380639d41ebc3a38a1745fec4016",
        )

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
