package ampsdk

import (
	"crypto/hmac"
	"crypto/rand"
	"crypto/sha256"
	"encoding/base64"
	"encoding/hex"
	"fmt"
	"strconv"
	"strings"
	"time"
)

const defaultClockSkew = 5 * time.Minute

// BuildSignaturePayload builds the canonical payload used by Love Envoy HMAC verification.
//
// Payload format: "{timestamp}.{METHOD}.{path}.{sha256(body)}.{nonce}".
func BuildSignaturePayload(timestamp, method, pathWithQuery, body string, nonce ...string) string {
	requestNonce := ""
	if len(nonce) > 0 {
		requestNonce = nonce[0]
	}

	bodyHash := sha256.Sum256([]byte(body))
	return fmt.Sprintf("%s.%s.%s.%x.%s", timestamp, strings.ToUpper(method), pathWithQuery, bodyHash, requestNonce)
}

// Sign returns a hex-encoded HMAC-SHA256 digest.
func Sign(payload, secret string) string {
	mac := hmac.New(sha256.New, []byte(secret))
	_, _ = mac.Write([]byte(payload))
	return hex.EncodeToString(mac.Sum(nil))
}

// UnixTimestampNow returns the current unix timestamp (seconds) as a string.
func UnixTimestampNow() string {
	return strconv.FormatInt(time.Now().Unix(), 10)
}

// IsTimestampFresh validates that timestamp is within maxSkew of now.
func IsTimestampFresh(timestamp string, now time.Time, maxSkew time.Duration) bool {
	ts, err := strconv.ParseInt(timestamp, 10, 64)
	if err != nil {
		return false
	}
	if maxSkew <= 0 {
		maxSkew = defaultClockSkew
	}

	deltaSeconds := now.Unix() - ts
	if deltaSeconds < 0 {
		deltaSeconds = -deltaSeconds
	}

	return time.Duration(deltaSeconds)*time.Second <= maxSkew
}

// GenerateNonce creates a URL-safe nonce suitable for X-Nonce/Idempotency-Key.
func GenerateNonce() (string, error) {
	raw := make([]byte, 24)
	if _, err := rand.Read(raw); err != nil {
		return "", err
	}
	return base64.RawURLEncoding.EncodeToString(raw), nil
}

// SignedHeaders returns HMAC auth headers for a request.
func SignedHeaders(apiKey, hmacSecret, method, pathWithQuery, body, timestamp, nonce string) (map[string]string, error) {
	ts := timestamp
	if ts == "" {
		ts = UnixTimestampNow()
	}

	requestNonce := nonce
	if requestNonce == "" {
		var err error
		requestNonce, err = GenerateNonce()
		if err != nil {
			return nil, err
		}
	}

	payload := BuildSignaturePayload(ts, method, pathWithQuery, body, requestNonce)
	signature := Sign(payload, hmacSecret)

	headers := map[string]string{
		"X-API-Key":    apiKey,
		"X-Timestamp":  ts,
		"X-Signature":  signature,
		"X-Nonce":      requestNonce,
	}

	return headers, nil
}
