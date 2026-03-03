package com.bonsai.amp.sdk;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.bonsai.amp.sdk.model.CreateNegotiationRequest;
import com.bonsai.amp.sdk.model.RegisterAgentRequest;
import com.bonsai.amp.sdk.model.RegisterAgentResponse;
import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.sun.net.httpserver.HttpExchange;
import com.sun.net.httpserver.HttpServer;
import java.io.IOException;
import java.net.InetSocketAddress;
import java.nio.charset.StandardCharsets;
import java.time.Duration;
import java.util.Map;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;

class AmpClientTest {
  private static final ObjectMapper OBJECT_MAPPER = new ObjectMapper();

  @Test
  void registerAgentRoundTrip() throws Exception {
    HttpServer server = HttpServer.create(new InetSocketAddress(0), 0);
    server.createContext(
        "/api/v1/agents/register",
        exchange -> {
          assertEquals("POST", exchange.getRequestMethod());

          String body = new String(exchange.getRequestBody().readAllBytes(), StandardCharsets.UTF_8);
          Map<String, Object> request =
              OBJECT_MAPPER.readValue(body, new TypeReference<Map<String, Object>>() {});
          assertEquals("astra", request.get("name"));

          writeJson(
              exchange,
              201,
              """
              {"agent_id":"11111111-1111-1111-1111-111111111111","api_key":"le_test","status":"pending_human_verify"}
              """);
        });
    server.start();

    try {
      String baseUrl = "http://127.0.0.1:" + server.getAddress().getPort();
      AmpClient client = new AmpClient(baseUrl);

      RegisterAgentResponse response =
          client.registerAgent(new RegisterAgentRequest("astra", null, null, null, null, null));

      assertEquals("11111111-1111-1111-1111-111111111111", response.agentId());
      assertEquals("le_test", response.apiKey());
    } finally {
      server.stop(0);
    }
  }

  @Test
  void protectedEndpointRequiresCredentials() {
    AmpClient client = new AmpClient("https://api.example.com");

    assertThrows(
        MissingCredentialsException.class,
        () ->
            client.createNegotiation(
                new CreateNegotiationRequest("22222222-2222-2222-2222-222222222222", null)));
  }

  @Test
  void discoverIncludesQueryAndAuthHeaders() throws Exception {
    HttpServer server = HttpServer.create(new InetSocketAddress(0), 0);
    server.createContext(
        "/api/v1/discover",
        exchange -> {
          assertEquals("GET", exchange.getRequestMethod());
          assertTrue(exchange.getRequestURI().getRawQuery().contains("page=1"));
          assertTrue(exchange.getRequestURI().getRawQuery().contains("limit=20"));
          assertEquals("le_key", exchange.getRequestHeaders().getFirst("X-API-Key"));
          assertNotNull(exchange.getRequestHeaders().getFirst("X-Timestamp"));
          assertNotNull(exchange.getRequestHeaders().getFirst("X-Signature"));
          assertNotNull(exchange.getRequestHeaders().getFirst("X-Nonce"));

          writeJson(exchange, 200, "{\"candidates\":[],\"page\":1,\"total_estimate\":0,\"source\":\"live\"}");
        });
    server.start();

    try {
      String baseUrl = "http://127.0.0.1:" + server.getAddress().getPort();
      AmpClient client = new AmpClient(baseUrl, "le_key", "secret");

      var response = client.discover(1, 20);
      assertEquals(1, response.page());
      assertEquals("live", response.source());
    } finally {
      server.stop(0);
    }
  }

  @Test
  void registerAgentRetriesAndSendsIdempotencyKey() throws Exception {
    AtomicInteger attempts = new AtomicInteger();
    HttpServer server = HttpServer.create(new InetSocketAddress(0), 0);
    server.createContext(
        "/api/v1/agents/register",
        exchange -> {
          assertEquals("idem-123", exchange.getRequestHeaders().getFirst("Idempotency-Key"));

          int attempt = attempts.incrementAndGet();
          if (attempt == 1) {
            writeText(exchange, 503, "temporary failure");
            return;
          }

          writeJson(
              exchange,
              201,
              """
              {"agent_id":"11111111-1111-1111-1111-111111111111","api_key":"le_test","status":"pending_human_verify"}
              """);
        });
    server.start();

    try {
      String baseUrl = "http://127.0.0.1:" + server.getAddress().getPort();
      AmpClient client = new AmpClient(baseUrl);

      RequestOptions options = new RequestOptions(null, null, Duration.ZERO, null, "idem-123");
      RegisterAgentResponse response =
          client.registerAgent(new RegisterAgentRequest("astra", null, null, null, null, null), options);

      assertEquals("le_test", response.apiKey());
      assertEquals(2, attempts.get());
    } finally {
      server.stop(0);
    }
  }

  @Test
  void requestOptionsTimeoutOverride() throws Exception {
    HttpServer server = HttpServer.create(new InetSocketAddress(0), 0);
    server.createContext(
        "/health",
        exchange -> {
          try {
            Thread.sleep(40);
          } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
          }
          writeJson(exchange, 200, "{\"ok\":true}");
        });
    server.start();

    try {
      String baseUrl = "http://127.0.0.1:" + server.getAddress().getPort();
      AmpClient client = new AmpClient(baseUrl);

      RequestOptions options =
          new RequestOptions(Duration.ofMillis(5), 0, Duration.ZERO, null, null);

      assertThrows(NetworkException.class, () -> client.health(options));
    } finally {
      server.stop(0);
    }
  }

  private static void writeJson(HttpExchange exchange, int status, String json) throws IOException {
    byte[] payload = json.strip().getBytes(StandardCharsets.UTF_8);
    exchange.getResponseHeaders().set("Content-Type", "application/json");
    exchange.sendResponseHeaders(status, payload.length);
    try (var output = exchange.getResponseBody()) {
      output.write(payload);
    }
  }

  private static void writeText(HttpExchange exchange, int status, String body) throws IOException {
    byte[] payload = body.getBytes(StandardCharsets.UTF_8);
    exchange.sendResponseHeaders(status, payload.length);
    try (var output = exchange.getResponseBody()) {
      output.write(payload);
    }
  }
}
