package com.hook0.client;

/**
 * The one failure {@link Json} reports, whichever way it was reading or writing.
 *
 * <p>It covers a document that is not JSON, one that crosses a ceiling {@link Json} sets for itself, and a value this
 * client was asked to write that JSON has no way to carry. Nothing else ever leaves that reader: a caller that catches
 * this has caught every way a document can fail to be one.
 */
public final class JsonException extends Hook0Exception {

  private static final long serialVersionUID = 1L;

  /**
   * Builds one out of what could not be read or written.
   *
   * @param detail what to say about the failure
   */
  public JsonException(String detail) {
    super(detail);
  }
}
