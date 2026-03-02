package ampsdk

import (
	"crypto/hmac"
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"strings"
)

// BuildSignaturePayload builds the canonical payload used by Love Envoy HMAC verification.
//
// Payload format: "{timestamp}.{METHOD}.{path}.{sha256(body)}".
func BuildSignaturePayload(timestamp, method, pathWithQuery, body string) string {
	bodyHash := sha256.Sum256([]byte(body))
	return fmt.Sprintf("%s.%s.%s.%x", timestamp, strings.ToUpper(method), pathWithQuery, bodyHash)
}

// Sign returns a hex-encoded HMAC-SHA256 digest.
func Sign(payload, secret string) string {
	mac := hmac.New(sha256.New, []byte(secret))
	_, _ = mac.Write([]byte(payload))
	return hex.EncodeToString(mac.Sum(nil))
}
