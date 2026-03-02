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

// Client is a synchronous HTTP client for AMP/1.0 servers.
type Client struct {
	baseURL    string
	apiKey     string
	hmacSecret string
	httpClient *http.Client
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

// NewClient creates a new AMP client.
func NewClient(baseURL string, opts ...Option) *Client {
	c := &Client{
		baseURL: strings.TrimRight(baseURL, "/"),
		httpClient: &http.Client{
			Timeout: 30 * time.Second,
		},
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

func (c *Client) Health(ctx context.Context) (HealthResponse, error) {
	var out HealthResponse
	err := c.doJSON(ctx, http.MethodGet, "/health", nil, false, &out)
	return out, err
}

func (c *Client) RegisterAgent(ctx context.Context, request RegisterAgentRequest) (RegisterAgentResponse, error) {
	var out RegisterAgentResponse
	err := c.doJSON(ctx, http.MethodPost, "/api/v1/agents/register", request, false, &out)
	return out, err
}

func (c *Client) CreateProfile(ctx context.Context, request CreateProfileRequest) (CreateProfileResponse, error) {
	var out CreateProfileResponse
	err := c.doJSON(ctx, http.MethodPost, "/api/v1/profiles", request, true, &out)
	return out, err
}

type DiscoverParams struct {
	Page     *int
	Limit    *int
	MinScore *float64
}

func (c *Client) Discover(ctx context.Context, params DiscoverParams) (DiscoveryResponse, error) {
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
	err := c.doJSON(ctx, http.MethodGet, path, nil, true, &out)
	return out, err
}

func (c *Client) CreateNegotiation(ctx context.Context, request CreateNegotiationRequest) (Negotiation, error) {
	var envelope struct {
		Negotiation Negotiation `json:"negotiation"`
	}
	err := c.doJSON(ctx, http.MethodPost, "/api/v1/negotiations", request, true, &envelope)
	return envelope.Negotiation, err
}

func (c *Client) ApprovalStatus(ctx context.Context, negotiationID string) (ApprovalStatus, error) {
	var out ApprovalStatus
	err := c.doJSON(ctx, http.MethodGet, "/api/v1/approvals/"+negotiationID, nil, true, &out)
	return out, err
}

func (c *Client) ApproveNegotiation(ctx context.Context, negotiationID string, request ApproveNegotiationRequest) (ApprovalDecisionResponse, error) {
	var out ApprovalDecisionResponse
	err := c.doJSON(ctx, http.MethodPost, "/api/v1/approvals/"+negotiationID+"/approve", request, true, &out)
	return out, err
}

func (c *Client) RejectNegotiation(ctx context.Context, negotiationID string, request RejectNegotiationRequest) (ApprovalDecisionResponse, error) {
	var out ApprovalDecisionResponse
	err := c.doJSON(ctx, http.MethodPost, "/api/v1/approvals/"+negotiationID+"/reject", request, true, &out)
	return out, err
}

func (c *Client) doJSON(
	ctx context.Context,
	method string,
	pathWithQuery string,
	payload any,
	authRequired bool,
	out any,
) error {
	canonicalPath := pathWithQuery
	if !strings.HasPrefix(canonicalPath, "/") {
		canonicalPath = "/" + canonicalPath
	}

	body := ""
	var bodyReader io.Reader
	if payload != nil {
		encoded, err := json.Marshal(payload)
		if err != nil {
			return err
		}
		body = string(encoded)
		bodyReader = bytes.NewReader(encoded)
	}

	req, err := http.NewRequestWithContext(ctx, method, c.baseURL+canonicalPath, bodyReader)
	if err != nil {
		return err
	}
	req.Header.Set("Accept", "application/json")
	if payload != nil {
		req.Header.Set("Content-Type", "application/json")
	}

	if authRequired {
		if c.apiKey == "" || c.hmacSecret == "" {
			return ErrMissingCredentials
		}
		timestamp := strconv.FormatInt(time.Now().Unix(), 10)
		signaturePayload := BuildSignaturePayload(timestamp, method, canonicalPath, body)
		signature := Sign(signaturePayload, c.hmacSecret)

		req.Header.Set("X-API-Key", c.apiKey)
		req.Header.Set("X-Timestamp", timestamp)
		req.Header.Set("X-Signature", signature)
	}

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	respBody, err := io.ReadAll(resp.Body)
	if err != nil {
		return err
	}
	respBodyText := strings.TrimSpace(string(respBody))

	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
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
