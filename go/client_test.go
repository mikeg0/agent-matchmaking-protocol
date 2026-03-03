package ampsdk

import (
	"context"
	"encoding/json"
	"errors"
	"net/http"
	"net/http/httptest"
	"strings"
	"sync/atomic"
	"testing"
	"time"
)

func TestBuildSignaturePayloadMatchesServerContract(t *testing.T) {
	payload := BuildSignaturePayload("1700000000", "post", "/api/v1/discover?page=1", "{\"x\":1}", "nonce-123")
	if payload != "1700000000.POST./api/v1/discover?page=1.5041bf1f713df204784353e82f6a4a535931cb64f1f4b4a5aeaffcb720918b22.nonce-123" {
		t.Fatalf("unexpected payload: %s", payload)
	}

	sig := Sign(payload, "secret")
	if sig != "0bdf2e80f4c7d4c8f11b7ad5202eb909a1400223c16fe0514ce9a103edb13c7a" {
		t.Fatalf("unexpected signature: %s", sig)
	}
}

func TestRegisterAgentRoundTrip(t *testing.T) {
	t.Parallel()

	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			t.Fatalf("unexpected method: %s", r.Method)
		}
		if r.URL.Path != "/api/v1/agents/register" {
			t.Fatalf("unexpected path: %s", r.URL.Path)
		}

		var req RegisterAgentRequest
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			t.Fatalf("decode request: %v", err)
		}
		if req.Name != "astra" {
			t.Fatalf("unexpected name: %s", req.Name)
		}

		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusCreated)
		_ = json.NewEncoder(w).Encode(RegisterAgentResponse{
			AgentID: "11111111-1111-1111-1111-111111111111",
			APIKey:  "le_test",
			Status:  "pending_human_verify",
		})
	}))
	defer srv.Close()

	client := NewClient(srv.URL)
	resp, err := client.RegisterAgent(context.Background(), RegisterAgentRequest{Name: "astra"})
	if err != nil {
		t.Fatalf("register agent: %v", err)
	}
	if resp.AgentID != "11111111-1111-1111-1111-111111111111" {
		t.Fatalf("unexpected agent id: %s", resp.AgentID)
	}
	if resp.APIKey != "le_test" {
		t.Fatalf("unexpected api key: %s", resp.APIKey)
	}
}

func TestProtectedEndpointRequiresCredentials(t *testing.T) {
	t.Parallel()

	client := NewClient("https://api.example.com")
	_, err := client.CreateNegotiation(context.Background(), CreateNegotiationRequest{
		TargetOpaqueID: "22222222-2222-2222-2222-222222222222",
	})
	if !errors.Is(err, ErrMissingCredentials) {
		t.Fatalf("expected ErrMissingCredentials, got: %v", err)
	}
}

func TestIsTimestampFresh(t *testing.T) {
	now := time.Unix(1700000010, 0)
	if !IsTimestampFresh("1700000005", now, 10*time.Second) {
		t.Fatal("expected timestamp to be fresh")
	}
	if IsTimestampFresh("1700000000", now, 5*time.Second) {
		t.Fatal("expected timestamp to be stale")
	}
	if IsTimestampFresh("invalid", now, 5*time.Second) {
		t.Fatal("expected invalid timestamp to be rejected")
	}
}

func TestDiscoverIncludesQueryAndAuthHeaders(t *testing.T) {
	t.Parallel()

	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodGet {
			t.Fatalf("unexpected method: %s", r.Method)
		}
		if r.URL.Path != "/api/v1/discover" {
			t.Fatalf("unexpected path: %s", r.URL.Path)
		}
		if !strings.Contains(r.URL.RawQuery, "page=1") || !strings.Contains(r.URL.RawQuery, "limit=20") {
			t.Fatalf("unexpected query: %s", r.URL.RawQuery)
		}
		if got := r.Header.Get("X-API-Key"); got != "le_key" {
			t.Fatalf("missing api key header, got %q", got)
		}
		if r.Header.Get("X-Timestamp") == "" {
			t.Fatal("missing X-Timestamp")
		}
		if r.Header.Get("X-Signature") == "" {
			t.Fatal("missing X-Signature")
		}
		if r.Header.Get("X-Nonce") == "" {
			t.Fatal("missing X-Nonce")
		}

		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write([]byte(`{"candidates":[],"page":1,"total_estimate":0,"source":"live"}`))
	}))
	defer srv.Close()

	page := 1
	limit := 20
	client := NewClient(srv.URL, WithCredentials("le_key", "secret"))
	resp, err := client.Discover(context.Background(), DiscoverParams{Page: &page, Limit: &limit})
	if err != nil {
		t.Fatalf("discover: %v", err)
	}
	if resp.Page != 1 {
		t.Fatalf("unexpected page: %d", resp.Page)
	}
}

func TestRegisterAgentRetriesRetryableStatus(t *testing.T) {
	t.Parallel()

	var attempts int32
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		current := atomic.AddInt32(&attempts, 1)
		if current <= 2 {
			w.WriteHeader(http.StatusServiceUnavailable)
			_, _ = w.Write([]byte("temporarily unavailable"))
			return
		}
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusCreated)
		_ = json.NewEncoder(w).Encode(RegisterAgentResponse{AgentID: "ok", APIKey: "k", Status: "pending_human_verify"})
	}))
	defer srv.Close()

	client := NewClient(srv.URL, WithRetryPolicy(2, 0, []int{http.StatusServiceUnavailable}))
	_, err := client.RegisterAgent(context.Background(), RegisterAgentRequest{Name: "astra"})
	if err != nil {
		t.Fatalf("register agent: %v", err)
	}
	if got := atomic.LoadInt32(&attempts); got != 3 {
		t.Fatalf("expected 3 attempts, got %d", got)
	}
}

func TestRegisterAgentRequestOptionsCanDisableRetries(t *testing.T) {
	t.Parallel()

	var attempts int32
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		atomic.AddInt32(&attempts, 1)
		w.WriteHeader(http.StatusServiceUnavailable)
		_, _ = w.Write([]byte("temporarily unavailable"))
	}))
	defer srv.Close()

	zero := 0
	client := NewClient(srv.URL, WithRetryPolicy(2, 0, []int{http.StatusServiceUnavailable}))
	_, err := client.RegisterAgent(
		context.Background(),
		RegisterAgentRequest{Name: "astra"},
		RequestOptions{MaxRetries: &zero},
	)
	if err == nil {
		t.Fatal("expected error")
	}
	if got := atomic.LoadInt32(&attempts); got != 1 {
		t.Fatalf("expected 1 attempt, got %d", got)
	}
}

func TestRegisterAgentRequestOptionsIdempotencyKey(t *testing.T) {
	t.Parallel()

	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if got := r.Header.Get("Idempotency-Key"); got != "idem-123" {
			t.Fatalf("expected idempotency key header, got %q", got)
		}
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusCreated)
		_ = json.NewEncoder(w).Encode(RegisterAgentResponse{AgentID: "ok", APIKey: "k", Status: "pending_human_verify"})
	}))
	defer srv.Close()

	client := NewClient(srv.URL)
	_, err := client.RegisterAgent(
		context.Background(),
		RegisterAgentRequest{Name: "astra"},
		RequestOptions{IdempotencyKey: "idem-123"},
	)
	if err != nil {
		t.Fatalf("register agent: %v", err)
	}
}

func TestHealthRequestOptionsTimeout(t *testing.T) {
	t.Parallel()

	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		time.Sleep(40 * time.Millisecond)
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write([]byte(`{"ok":true}`))
	}))
	defer srv.Close()

	timeout := 5 * time.Millisecond
	zero := 0
	client := NewClient(srv.URL)
	_, err := client.Health(context.Background(), RequestOptions{Timeout: &timeout, MaxRetries: &zero})
	if err == nil {
		t.Fatal("expected timeout error")
	}
	if !errors.Is(err, context.DeadlineExceeded) {
		t.Fatalf("expected deadline exceeded, got: %v", err)
	}
}

func TestRequestOptionsValidation(t *testing.T) {
	t.Parallel()

	invalidRetries := -1
	client := NewClient("https://api.example.com")
	_, err := client.Health(context.Background(), RequestOptions{MaxRetries: &invalidRetries})
	if !errors.Is(err, ErrInvalidRequestOptions) {
		t.Fatalf("expected ErrInvalidRequestOptions, got: %v", err)
	}
}
