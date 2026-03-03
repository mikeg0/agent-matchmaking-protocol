package ampsdk

import (
	"bytes"
	"context"
	"encoding/json"
	"io"
	"net/http"
	"net/url"
	"strconv"
	"strings"
	"time"
)

var defaultRetryStatusCodes = []int{
	http.StatusTooManyRequests,
	http.StatusInternalServerError,
	http.StatusBadGateway,
	http.StatusServiceUnavailable,
	http.StatusGatewayTimeout,
}

// RequestOptions configures per-request transport overrides.
//
// Any nil field falls back to the client's default transport settings.
// RetryStatusCodes overrides the retryable status set when non-nil.
// Set RetryStatusCodes to an empty slice to disable status-based retries.
type RequestOptions struct {
	Timeout          *time.Duration
	MaxRetries       *int
	RetryBackoff     *time.Duration
	RetryStatusCodes []int
	IdempotencyKey   string
}

// Client is a synchronous HTTP client for AMP/1.0 servers.
type Client struct {
	baseURL          string
	apiKey           string
	hmacSecret       string
	httpClient       *http.Client
	timeout          time.Duration
	maxRetries       int
	retryBackoff     time.Duration
	retryStatusCodes map[int]struct{}
}

// Option configures a Client.
type Option func(*Client)

// WithCredentials sets default API key + HMAC secret used for authenticated requests.
func WithCredentials(apiKey, hmacSecret string) Option {
	return func(c *Client) {
		c.apiKey = apiKey
		c.hmacSecret = hmacSecret
	}
}

// WithHTTPClient overrides the underlying http.Client.
func WithHTTPClient(httpClient *http.Client) Option {
	return func(c *Client) {
		if httpClient != nil {
			c.httpClient = httpClient
		}
	}
}

// WithTimeout sets the default per-attempt request timeout.
func WithTimeout(timeout time.Duration) Option {
	return func(c *Client) {
		if timeout >= 0 {
			c.timeout = timeout
		}
	}
}

// WithRetryPolicy sets default retry behavior.
//
// retryStatusCodes == nil keeps the existing status set. Passing an empty slice
// disables status-code retries.
func WithRetryPolicy(maxRetries int, retryBackoff time.Duration, retryStatusCodes []int) Option {
	return func(c *Client) {
		if maxRetries >= 0 {
			c.maxRetries = maxRetries
		}
		if retryBackoff >= 0 {
			c.retryBackoff = retryBackoff
		}
		if retryStatusCodes != nil {
			c.retryStatusCodes = statusCodeSet(retryStatusCodes)
		}
	}
}

// NewClient creates a new AMP client.
func NewClient(baseURL string, opts ...Option) *Client {
	c := &Client{
		baseURL:      strings.TrimRight(baseURL, "/"),
		httpClient:   &http.Client{},
		timeout:      30 * time.Second,
		maxRetries:   2,
		retryBackoff: 250 * time.Millisecond,
		retryStatusCodes: statusCodeSet(defaultRetryStatusCodes),
	}
	for _, opt := range opts {
		opt(c)
	}
	return c
}

// SetCredentials updates credentials after construction.
func (c *Client) SetCredentials(apiKey, hmacSecret string) {
	c.apiKey = apiKey
	c.hmacSecret = hmacSecret
}

func (c *Client) Health(ctx context.Context, requestOptions ...RequestOptions) (HealthResponse, error) {
	var out HealthResponse
	err := c.doJSON(ctx, http.MethodGet, "/health", nil, false, &out, firstRequestOptions(requestOptions))
	return out, err
}

func (c *Client) RegisterAgent(ctx context.Context, request RegisterAgentRequest, requestOptions ...RequestOptions) (RegisterAgentResponse, error) {
	var out RegisterAgentResponse
	err := c.doJSON(ctx, http.MethodPost, "/api/v1/agents/register", request, false, &out, firstRequestOptions(requestOptions))
	return out, err
}

func (c *Client) CreateProfile(ctx context.Context, request CreateProfileRequest, requestOptions ...RequestOptions) (CreateProfileResponse, error) {
	var out CreateProfileResponse
	err := c.doJSON(ctx, http.MethodPost, "/api/v1/profiles", request, true, &out, firstRequestOptions(requestOptions))
	return out, err
}

type DiscoverParams struct {
	Page     *int
	Limit    *int
	MinScore *float64
}

func (c *Client) Discover(ctx context.Context, params DiscoverParams, requestOptions ...RequestOptions) (DiscoveryResponse, error) {
	path := "/api/v1/discover"
	values := url.Values{}
	if params.Page != nil {
		values.Set("page", strconv.Itoa(*params.Page))
	}
	if params.Limit != nil {
		values.Set("limit", strconv.Itoa(*params.Limit))
	}
	if params.MinScore != nil {
		values.Set("min_score", strconv.FormatFloat(*params.MinScore, 'f', -1, 64))
	}
	if encoded := values.Encode(); encoded != "" {
		path += "?" + encoded
	}

	var out DiscoveryResponse
	err := c.doJSON(ctx, http.MethodGet, path, nil, true, &out, firstRequestOptions(requestOptions))
	return out, err
}

func (c *Client) CreateNegotiation(ctx context.Context, request CreateNegotiationRequest, requestOptions ...RequestOptions) (Negotiation, error) {
	var envelope struct {
		Negotiation Negotiation `json:"negotiation"`
	}
	err := c.doJSON(ctx, http.MethodPost, "/api/v1/negotiations", request, true, &envelope, firstRequestOptions(requestOptions))
	return envelope.Negotiation, err
}

func (c *Client) ApprovalStatus(ctx context.Context, negotiationID string, requestOptions ...RequestOptions) (ApprovalStatus, error) {
	var out ApprovalStatus
	err := c.doJSON(ctx, http.MethodGet, "/api/v1/approvals/"+negotiationID, nil, true, &out, firstRequestOptions(requestOptions))
	return out, err
}

func (c *Client) ApproveNegotiation(ctx context.Context, negotiationID string, request ApproveNegotiationRequest, requestOptions ...RequestOptions) (ApprovalDecisionResponse, error) {
	var out ApprovalDecisionResponse
	err := c.doJSON(ctx, http.MethodPost, "/api/v1/approvals/"+negotiationID+"/approve", request, true, &out, firstRequestOptions(requestOptions))
	return out, err
}

func (c *Client) RejectNegotiation(ctx context.Context, negotiationID string, request RejectNegotiationRequest, requestOptions ...RequestOptions) (ApprovalDecisionResponse, error) {
	var out ApprovalDecisionResponse
	err := c.doJSON(ctx, http.MethodPost, "/api/v1/approvals/"+negotiationID+"/reject", request, true, &out, firstRequestOptions(requestOptions))
	return out, err
}

func firstRequestOptions(options []RequestOptions) *RequestOptions {
	if len(options) == 0 {
		return nil
	}
	return &options[0]
}

type resolvedRequestOptions struct {
	timeout          time.Duration
	maxRetries       int
	retryBackoff     time.Duration
	retryStatusCodes map[int]struct{}
	idempotencyKey   string
}

func (c *Client) resolveRequestOptions(requestOptions *RequestOptions) (resolvedRequestOptions, error) {
	resolved := resolvedRequestOptions{
		timeout:          c.timeout,
		maxRetries:       c.maxRetries,
		retryBackoff:     c.retryBackoff,
		retryStatusCodes: cloneStatusCodeSet(c.retryStatusCodes),
	}

	if requestOptions == nil {
		return resolved, nil
	}

	if requestOptions.Timeout != nil {
		resolved.timeout = *requestOptions.Timeout
	}
	if requestOptions.MaxRetries != nil {
		resolved.maxRetries = *requestOptions.MaxRetries
	}
	if requestOptions.RetryBackoff != nil {
		resolved.retryBackoff = *requestOptions.RetryBackoff
	}
	if requestOptions.RetryStatusCodes != nil {
		resolved.retryStatusCodes = statusCodeSet(requestOptions.RetryStatusCodes)
	}
	resolved.idempotencyKey = strings.TrimSpace(requestOptions.IdempotencyKey)

	if resolved.maxRetries < 0 {
		return resolvedRequestOptions{}, ErrInvalidRequestOptions
	}
	if resolved.timeout < 0 {
		return resolvedRequestOptions{}, ErrInvalidRequestOptions
	}
	if resolved.retryBackoff < 0 {
		return resolvedRequestOptions{}, ErrInvalidRequestOptions
	}

	return resolved, nil
}

func (c *Client) doJSON(
	ctx context.Context,
	method string,
	pathWithQuery string,
	payload any,
	authRequired bool,
	out any,
	requestOptions *RequestOptions,
) error {
	canonicalPath := pathWithQuery
	if !strings.HasPrefix(canonicalPath, "/") {
		canonicalPath = "/" + canonicalPath
	}

	resolvedOptions, err := c.resolveRequestOptions(requestOptions)
	if err != nil {
		return err
	}

	body := ""
	var bodyBytes []byte
	if payload != nil {
		encoded, err := json.Marshal(payload)
		if err != nil {
			return err
		}
		body = string(encoded)
		bodyBytes = encoded
	}

	for attempt := 0; ; attempt++ {
		attemptCtx := ctx
		var cancel context.CancelFunc
		if resolvedOptions.timeout > 0 {
			attemptCtx, cancel = context.WithTimeout(ctx, resolvedOptions.timeout)
		}

		var bodyReader io.Reader
		if payload != nil {
			bodyReader = bytes.NewReader(bodyBytes)
		}

		req, err := http.NewRequestWithContext(attemptCtx, method, c.baseURL+canonicalPath, bodyReader)
		if err != nil {
			if cancel != nil {
				cancel()
			}
			return err
		}
		req.Header.Set("Accept", "application/json")
		if payload != nil {
			req.Header.Set("Content-Type", "application/json")
		}
		if resolvedOptions.idempotencyKey != "" {
			req.Header.Set("Idempotency-Key", resolvedOptions.idempotencyKey)
		}

		if authRequired {
			if c.apiKey == "" || c.hmacSecret == "" {
				if cancel != nil {
					cancel()
				}
				return ErrMissingCredentials
			}

			headers, err := SignedHeaders(c.apiKey, c.hmacSecret, method, canonicalPath, body, "", "")
			if err != nil {
				if cancel != nil {
					cancel()
				}
				return err
			}
			for key, value := range headers {
				req.Header.Set(key, value)
			}
		}

		resp, err := c.httpClient.Do(req)
		if err != nil {
			if cancel != nil {
				cancel()
			}
			if shouldRetryAttempt(attempt, resolvedOptions.maxRetries) {
				if err := waitBackoff(ctx, resolvedOptions.retryBackoff, attempt); err != nil {
					return err
				}
				continue
			}
			if ctxErr := ctx.Err(); ctxErr != nil {
				return ctxErr
			}
			return &NetworkError{Err: err}
		}

		respBody, readErr := io.ReadAll(resp.Body)
		closeErr := resp.Body.Close()
		if cancel != nil {
			cancel()
		}
		if readErr != nil {
			return readErr
		}
		if closeErr != nil {
			return closeErr
		}

		respBodyText := strings.TrimSpace(string(respBody))

		if resp.StatusCode < 200 || resp.StatusCode >= 300 {
			if shouldRetryAttempt(attempt, resolvedOptions.maxRetries) && shouldRetryStatus(resp.StatusCode, resolvedOptions.retryStatusCodes) {
				if err := waitBackoff(ctx, resolvedOptions.retryBackoff, attempt); err != nil {
					return err
				}
				continue
			}

			message := resp.Status
			if respBodyText != "" {
				message = strings.Split(respBodyText, "\n")[0]
			}
			return &StatusError{
				StatusCode: resp.StatusCode,
				Message:    message,
				Body:       respBodyText,
			}
		}

		if out == nil || respBodyText == "" {
			return nil
		}

		return json.Unmarshal(respBody, out)
	}
}

func shouldRetryAttempt(attempt, maxRetries int) bool {
	return attempt < maxRetries
}

func shouldRetryStatus(statusCode int, retryStatusCodes map[int]struct{}) bool {
	_, ok := retryStatusCodes[statusCode]
	return ok
}

func waitBackoff(ctx context.Context, backoff time.Duration, attempt int) error {
	if backoff <= 0 {
		return nil
	}
	delay := backoff * time.Duration(1<<attempt)
	timer := time.NewTimer(delay)
	defer timer.Stop()

	select {
	case <-ctx.Done():
		return ctx.Err()
	case <-timer.C:
		return nil
	}
}

func statusCodeSet(codes []int) map[int]struct{} {
	set := make(map[int]struct{}, len(codes))
	for _, code := range codes {
		set[code] = struct{}{}
	}
	return set
}

func cloneStatusCodeSet(source map[int]struct{}) map[int]struct{} {
	if source == nil {
		return map[int]struct{}{}
	}
	clone := make(map[int]struct{}, len(source))
	for key := range source {
		clone[key] = struct{}{}
	}
	return clone
}
