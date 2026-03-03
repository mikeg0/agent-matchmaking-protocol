package ampsdk

import (
	"errors"
	"fmt"
)

var (
	// ErrMissingCredentials is returned when an authenticated endpoint is called
	// without both API key and HMAC secret configured.
	ErrMissingCredentials = errors.New("authenticated endpoint requested without api_key and hmac_secret")

	// ErrInvalidRequestOptions is returned when request-level transport options
	// are invalid (for example, negative retries/timeouts/backoff).
	ErrInvalidRequestOptions = errors.New("invalid request options")
)

// StatusError is returned for non-2xx HTTP responses.
type StatusError struct {
	StatusCode int
	Message    string
	Body       string
}

func (e *StatusError) Error() string {
	if e == nil {
		return "http status error"
	}
	if e.Message == "" {
		return fmt.Sprintf("http status %d", e.StatusCode)
	}
	return fmt.Sprintf("http status %d: %s", e.StatusCode, e.Message)
}

// NetworkError is returned when a request fails before an HTTP response is received.
type NetworkError struct {
	Err error
}

func (e *NetworkError) Error() string {
	if e == nil || e.Err == nil {
		return "network error"
	}
	return fmt.Sprintf("network error: %v", e.Err)
}

func (e *NetworkError) Unwrap() error {
	if e == nil {
		return nil
	}
	return e.Err
}
