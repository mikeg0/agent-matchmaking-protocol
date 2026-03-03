package com.bonsai.amp.sdk;

/** Raised when the server returns a non-2xx HTTP status. */
public final class HttpStatusException extends RuntimeException {
  private final int statusCode;
  private final String body;

  public HttpStatusException(int statusCode, String message, String body) {
    super("HTTP " + statusCode + ": " + (message == null || message.isBlank() ? "request failed" : message));
    this.statusCode = statusCode;
    this.body = body;
  }

  public int getStatusCode() {
    return statusCode;
  }

  public String getBody() {
    return body;
  }
}
