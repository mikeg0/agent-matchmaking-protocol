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
import java.time.Instant;
import java.util.ArrayList;
import java.util.List;

/** Baseline synchronous HTTP client for AMP/1.0 servers. */
public final class AmpClient {
  private static final Duration DEFAULT_TIMEOUT = Duration.ofSeconds(30);

  private final String baseUrl;
  private final HttpClient httpClient;
  private final ObjectMapper objectMapper;
  private String apiKey;
  private String hmacSecret;

  public AmpClient(String baseUrl) {
    this(baseUrl, null, null);
  }

  public AmpClient(String baseUrl, String apiKey, String hmacSecret) {
    this.baseUrl = trimTrailingSlash(baseUrl);
    this.apiKey = apiKey;
    this.hmacSecret = hmacSecret;
    this.httpClient = HttpClient.newBuilder().connectTimeout(DEFAULT_TIMEOUT).build();
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
    return request("GET", "/health", null, false, HealthResponse.class);
  }

  public RegisterAgentResponse registerAgent(RegisterAgentRequest request) {
    return request("POST", "/api/v1/agents/register", request, false, RegisterAgentResponse.class);
  }

  public CreateProfileResponse createProfile(CreateProfileRequest request) {
    return request("POST", "/api/v1/profiles", request, true, CreateProfileResponse.class);
  }

  public DiscoveryResponse discover(Integer page, Integer limit) {
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

    return request("GET", path, null, true, DiscoveryResponse.class);
  }

  public Negotiation createNegotiation(CreateNegotiationRequest request) {
    NegotiationEnvelope envelope =
        request("POST", "/api/v1/negotiations", request, true, NegotiationEnvelope.class);
    return envelope == null ? null : envelope.negotiation();
  }

  public ApprovalStatus approvalStatus(String negotiationId) {
    return request(
        "GET",
        "/api/v1/approvals/" + urlEncode(negotiationId),
        null,
        true,
        ApprovalStatus.class);
  }

  public ApprovalDecisionResponse approveNegotiation(
      String negotiationId, ApproveNegotiationRequest request) {
    return request(
        "POST",
        "/api/v1/approvals/" + urlEncode(negotiationId) + "/approve",
        request,
        true,
        ApprovalDecisionResponse.class);
  }

  public ApprovalDecisionResponse rejectNegotiation(
      String negotiationId, RejectNegotiationRequest request) {
    return request(
        "POST",
        "/api/v1/approvals/" + urlEncode(negotiationId) + "/reject",
        request,
        true,
        ApprovalDecisionResponse.class);
  }

  private <T> T request(
      String method, String pathWithQuery, Object payload, boolean authRequired, Class<T> responseType) {
    String canonicalPath = pathWithQuery.startsWith("/") ? pathWithQuery : "/" + pathWithQuery;

    String bodyText = "";
    HttpRequest.BodyPublisher bodyPublisher = HttpRequest.BodyPublishers.noBody();
    if (payload != null) {
      try {
        bodyText = objectMapper.writeValueAsString(payload);
      } catch (IOException e) {
        throw new IllegalArgumentException("failed to serialize request payload", e);
      }
      bodyPublisher = HttpRequest.BodyPublishers.ofString(bodyText, StandardCharsets.UTF_8);
    }

    HttpRequest.Builder builder =
        HttpRequest.newBuilder()
            .uri(URI.create(baseUrl + canonicalPath))
            .timeout(DEFAULT_TIMEOUT)
            .header("Accept", "application/json")
            .method(method.toUpperCase(), bodyPublisher);

    if (payload != null) {
      builder.header("Content-Type", "application/json");
    }

    if (authRequired) {
      if (isBlank(apiKey) || isBlank(hmacSecret)) {
        throw new MissingCredentialsException();
      }
      String timestamp = Long.toString(Instant.now().getEpochSecond());
      String signaturePayload =
          AuthSigner.buildSignaturePayload(timestamp, method, canonicalPath, bodyText);
      builder
          .header("X-API-Key", apiKey)
          .header("X-Timestamp", timestamp)
          .header("X-Signature", AuthSigner.sign(signaturePayload, hmacSecret));
    }

    HttpResponse<String> response;
    try {
      response = httpClient.send(builder.build(), HttpResponse.BodyHandlers.ofString(StandardCharsets.UTF_8));
    } catch (IOException e) {
      throw new RuntimeException("network error: " + e.getMessage(), e);
    } catch (InterruptedException e) {
      Thread.currentThread().interrupt();
      throw new RuntimeException("request interrupted", e);
    }

    String responseBody = response.body() == null ? "" : response.body().trim();

    if (response.statusCode() < 200 || response.statusCode() >= 300) {
      String message = responseBody.isEmpty() ? "request failed" : responseBody.lines().findFirst().orElse("request failed");
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
}
