package ampsdk

import (
	"errors"
	"fmt"
)

var (
	// ErrMissingCredentials is returned when an authenticated endpoint is called
	// without both API key and HMAC secret configured.
	ErrMissingCredentials = errors.New("authenticated endpoint requested without api_key and hmac_secret")
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
