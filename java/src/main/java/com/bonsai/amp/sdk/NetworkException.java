package com.bonsai.amp.sdk;

/** Raised when a request fails before a valid HTTP response is returned. */
public final class NetworkException extends RuntimeException {
  public NetworkException(String message, Throwable cause) {
    super(message, cause);
  }
}
