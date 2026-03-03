package com.bonsai.amp.sdk;

import com.bonsai.amp.sdk.model.ApprovalDecisionResponse;
import com.bonsai.amp.sdk.model.ApprovalStatus;
import com.bonsai.amp.sdk.model.ApproveNegotiationRequest;
import com.bonsai.amp.sdk.model.CreateNegotiationRequest;
import com.bonsai.amp.sdk.model.CreateProfileRequest;
import com.bonsai.amp.sdk.model.CreateProfileResponse;
import com.bonsai.amp.sdk.model.DiscoveryResponse;
import com.bonsai.amp.sdk.model.HealthResponse;
import com.bonsai.amp.sdk.model.Negotiation;
import com.bonsai.amp.sdk.model.RegisterAgentRequest;
import com.bonsai.amp.sdk.model.RegisterAgentResponse;
import com.bonsai.amp.sdk.model.RejectNegotiationRequest;
import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.databind.DeserializationFeature;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.datatype.jsr310.JavaTimeModule;
import java.io.IOException;
import java.net.URI;
import java.net.URLEncoder;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.charset.StandardCharsets;
import java.time.Duration;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Set;

/** Baseline synchronous HTTP client for AMP/1.0 servers. */
public final class AmpClient {
  private static final Duration DEFAULT_TIMEOUT = Duration.ofSeconds(30);
  private static final int DEFAULT_MAX_RETRIES = 2;
  private static final Duration DEFAULT_RETRY_BACKOFF = Duration.ofMillis(250);
  private static final Set<Integer> DEFAULT_RETRY_STATUS_CODES = Set.of(429, 500, 502, 503, 504);

  private final String baseUrl;
  private final HttpClient httpClient;
  private final ObjectMapper objectMapper;
  private final Duration timeout;
  private final int maxRetries;
  private final Duration retryBackoff;
  private final Set<Integer> retryStatusCodes;
  private String apiKey;
  private String hmacSecret;

  public AmpClient(String baseUrl) {
    this(baseUrl, null, null);
  }

  public AmpClient(String baseUrl, String apiKey, String hmacSecret) {
    this(
        baseUrl,
        apiKey,
        hmacSecret,
        DEFAULT_TIMEOUT,
        DEFAULT_MAX_RETRIES,
        DEFAULT_RETRY_BACKOFF,
        DEFAULT_RETRY_STATUS_CODES);
  }

  public AmpClient(
      String baseUrl,
      String apiKey,
      String hmacSecret,
      Duration timeout,
      int maxRetries,
      Duration retryBackoff,
      Set<Integer> retryStatusCodes) {
    this.baseUrl = trimTrailingSlash(baseUrl);
    this.apiKey = apiKey;
    this.hmacSecret = hmacSecret;
    this.timeout = timeout == null ? DEFAULT_TIMEOUT : timeout;
    this.maxRetries = Math.max(0, maxRetries);
    this.retryBackoff = retryBackoff == null ? DEFAULT_RETRY_BACKOFF : retryBackoff;
    this.retryStatusCodes =
        retryStatusCodes == null ? Set.copyOf(DEFAULT_RETRY_STATUS_CODES) : Set.copyOf(retryStatusCodes);
    this.httpClient = HttpClient.newBuilder().connectTimeout(this.timeout).build();
    this.objectMapper =
        new ObjectMapper()
            .registerModule(new JavaTimeModule())
            .configure(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES, false);
  }

  public void setCredentials(String apiKey, String hmacSecret) {
    this.apiKey = apiKey;
    this.hmacSecret = hmacSecret;
  }

  public HealthResponse health() {
    return health(null);
  }

  public HealthResponse health(RequestOptions requestOptions) {
    return request("GET", "/health", null, false, HealthResponse.class, requestOptions);
  }

  public RegisterAgentResponse registerAgent(RegisterAgentRequest request) {
    return registerAgent(request, null);
  }

  public RegisterAgentResponse registerAgent(RegisterAgentRequest request, RequestOptions requestOptions) {
    return request(
        "POST", "/api/v1/agents/register", request, false, RegisterAgentResponse.class, requestOptions);
  }

  public CreateProfileResponse createProfile(CreateProfileRequest request) {
    return createProfile(request, null);
  }

  public CreateProfileResponse createProfile(CreateProfileRequest request, RequestOptions requestOptions) {
    return request("POST", "/api/v1/profiles", request, true, CreateProfileResponse.class, requestOptions);
  }

  public DiscoveryResponse discover(Integer page, Integer limit) {
    return discover(page, limit, null);
  }

  public DiscoveryResponse discover(Integer page, Integer limit, RequestOptions requestOptions) {
    String path = "/api/v1/discover";
    List<String> query = new ArrayList<>();
    if (page != null) {
      query.add("page=" + page);
    }
    if (limit != null) {
      query.add("limit=" + limit);
    }
    if (!query.isEmpty()) {
      path += "?" + String.join("&", query);
    }

    return request("GET", path, null, true, DiscoveryResponse.class, requestOptions);
  }

  public Negotiation createNegotiation(CreateNegotiationRequest request) {
    return createNegotiation(request, null);
  }

  public Negotiation createNegotiation(
      CreateNegotiationRequest request, RequestOptions requestOptions) {
    NegotiationEnvelope envelope =
        request("POST", "/api/v1/negotiations", request, true, NegotiationEnvelope.class, requestOptions);
    return envelope == null ? null : envelope.negotiation();
  }

  public ApprovalStatus approvalStatus(String negotiationId) {
    return approvalStatus(negotiationId, null);
  }

  public ApprovalStatus approvalStatus(String negotiationId, RequestOptions requestOptions) {
    return request(
        "GET",
        "/api/v1/approvals/" + urlEncode(negotiationId),
        null,
        true,
        ApprovalStatus.class,
        requestOptions);
  }

  public ApprovalDecisionResponse approveNegotiation(
      String negotiationId, ApproveNegotiationRequest request) {
    return approveNegotiation(negotiationId, request, null);
  }

  public ApprovalDecisionResponse approveNegotiation(
      String negotiationId, ApproveNegotiationRequest request, RequestOptions requestOptions) {
    return request(
        "POST",
        "/api/v1/approvals/" + urlEncode(negotiationId) + "/approve",
        request,
        true,
        ApprovalDecisionResponse.class,
        requestOptions);
  }

  public ApprovalDecisionResponse rejectNegotiation(
      String negotiationId, RejectNegotiationRequest request) {
    return rejectNegotiation(negotiationId, request, null);
  }

  public ApprovalDecisionResponse rejectNegotiation(
      String negotiationId, RejectNegotiationRequest request, RequestOptions requestOptions) {
    return request(
        "POST",
        "/api/v1/approvals/" + urlEncode(negotiationId) + "/reject",
        request,
        true,
        ApprovalDecisionResponse.class,
        requestOptions);
  }

  private <T> T request(
      String method,
      String pathWithQuery,
      Object payload,
      boolean authRequired,
      Class<T> responseType,
      RequestOptions requestOptions) {
    String canonicalPath = pathWithQuery.startsWith("/") ? pathWithQuery : "/" + pathWithQuery;

    String bodyText = "";
    if (payload != null) {
      try {
        bodyText = objectMapper.writeValueAsString(payload);
      } catch (IOException e) {
        throw new IllegalArgumentException("failed to serialize request payload", e);
      }
    }

    ResolvedRequestOptions options = resolveRequestOptions(requestOptions);

    for (int attempt = 0; ; attempt++) {
      HttpRequest.BodyPublisher bodyPublisher =
          payload == null
              ? HttpRequest.BodyPublishers.noBody()
              : HttpRequest.BodyPublishers.ofString(bodyText, StandardCharsets.UTF_8);

      HttpRequest.Builder builder =
          HttpRequest.newBuilder()
              .uri(URI.create(baseUrl + canonicalPath))
              .timeout(options.timeout())
              .header("Accept", "application/json")
              .method(method.toUpperCase(), bodyPublisher);

      if (payload != null) {
        builder.header("Content-Type", "application/json");
      }

      if (options.idempotencyKey() != null) {
        builder.header("Idempotency-Key", options.idempotencyKey());
      }

      if (authRequired) {
        if (isBlank(apiKey) || isBlank(hmacSecret)) {
          throw new MissingCredentialsException();
        }
        var authHeaders =
            AuthSigner.signedHeaders(apiKey, hmacSecret, method, canonicalPath, bodyText, null, null);
        authHeaders.forEach(builder::header);
      }

      HttpResponse<String> response;
      try {
        response =
            httpClient.send(
                builder.build(), HttpResponse.BodyHandlers.ofString(StandardCharsets.UTF_8));
      } catch (IOException e) {
        if (shouldRetryAttempt(attempt, options.maxRetries())) {
          waitBackoff(options.retryBackoff(), attempt);
          continue;
        }
        throw new NetworkException("network error: " + e.getMessage(), e);
      } catch (InterruptedException e) {
        Thread.currentThread().interrupt();
        throw new NetworkException("request interrupted", e);
      }

      String responseBody = response.body() == null ? "" : response.body().trim();

      if (response.statusCode() < 200 || response.statusCode() >= 300) {
        if (shouldRetryAttempt(attempt, options.maxRetries())
            && shouldRetryStatus(response.statusCode(), options.retryStatusCodes())) {
          waitBackoff(options.retryBackoff(), attempt);
          continue;
        }

        String message =
            responseBody.isEmpty() ? "request failed" : responseBody.lines().findFirst().orElse("request failed");
        throw new HttpStatusException(response.statusCode(), message, responseBody);
      }

      if (responseType == Void.class || responseBody.isEmpty()) {
        return null;
      }

      try {
        return objectMapper.readValue(responseBody, responseType);
      } catch (IOException e) {
        throw new IllegalStateException("failed to parse JSON response", e);
      }
    }
  }

  private ResolvedRequestOptions resolveRequestOptions(RequestOptions requestOptions) {
    Duration resolvedTimeout = timeout;
    int resolvedMaxRetries = maxRetries;
    Duration resolvedRetryBackoff = retryBackoff;
    Set<Integer> resolvedRetryStatusCodes = new HashSet<>(retryStatusCodes);
    String resolvedIdempotencyKey = null;

    if (requestOptions != null) {
      if (requestOptions.timeout() != null) {
        resolvedTimeout = requestOptions.timeout();
      }
      if (requestOptions.maxRetries() != null) {
        resolvedMaxRetries = requestOptions.maxRetries();
      }
      if (requestOptions.retryBackoff() != null) {
        resolvedRetryBackoff = requestOptions.retryBackoff();
      }
      if (requestOptions.retryStatusCodes() != null) {
        resolvedRetryStatusCodes = new HashSet<>(requestOptions.retryStatusCodes());
      }
      if (requestOptions.idempotencyKey() != null) {
        String trimmed = requestOptions.idempotencyKey().trim();
        resolvedIdempotencyKey = trimmed.isEmpty() ? null : trimmed;
      }
    }

    if (resolvedTimeout.isZero() || resolvedTimeout.isNegative()) {
      throw new IllegalArgumentException("timeout must be > 0");
    }
    if (resolvedMaxRetries < 0) {
      throw new IllegalArgumentException("maxRetries must be >= 0");
    }
    if (resolvedRetryBackoff.isNegative()) {
      throw new IllegalArgumentException("retryBackoff must be >= 0");
    }

    return new ResolvedRequestOptions(
        resolvedTimeout,
        resolvedMaxRetries,
        Set.copyOf(resolvedRetryStatusCodes),
        resolvedRetryBackoff,
        resolvedIdempotencyKey);
  }

  private static boolean shouldRetryAttempt(int attempt, int maxRetries) {
    return attempt < maxRetries;
  }

  private static boolean shouldRetryStatus(int statusCode, Set<Integer> retryStatusCodes) {
    return retryStatusCodes.contains(statusCode);
  }

  private static void waitBackoff(Duration retryBackoff, int attempt) {
    if (retryBackoff == null || retryBackoff.isZero() || retryBackoff.isNegative()) {
      return;
    }

    long factor = 1L << Math.min(attempt, 30);
    long delayMillis;
    try {
      delayMillis = Math.multiplyExact(retryBackoff.toMillis(), factor);
    } catch (ArithmeticException e) {
      delayMillis = Long.MAX_VALUE;
    }

    try {
      Thread.sleep(delayMillis);
    } catch (InterruptedException e) {
      Thread.currentThread().interrupt();
      throw new NetworkException("retry backoff interrupted", e);
    }
  }

  private static boolean isBlank(String value) {
    return value == null || value.isBlank();
  }

  private static String trimTrailingSlash(String value) {
    if (value == null || value.isBlank()) {
      throw new IllegalArgumentException("baseUrl is required");
    }
    if (value.endsWith("/")) {
      return value.substring(0, value.length() - 1);
    }
    return value;
  }

  private static String urlEncode(String value) {
    return URLEncoder.encode(value, StandardCharsets.UTF_8);
  }

  @JsonIgnoreProperties(ignoreUnknown = true)
  private record NegotiationEnvelope(@JsonProperty("negotiation") Negotiation negotiation) {}

  private record ResolvedRequestOptions(
      Duration timeout,
      int maxRetries,
      Set<Integer> retryStatusCodes,
      Duration retryBackoff,
      String idempotencyKey) {}
}
