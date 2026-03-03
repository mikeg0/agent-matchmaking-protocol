package com.bonsai.amp.sdk;

/** Raised when authenticated operations are attempted without API key and HMAC secret. */
public final class MissingCredentialsException extends RuntimeException {
  public MissingCredentialsException() {
    super("authenticated endpoint requested without api_key and hmac_secret");
  }
}
