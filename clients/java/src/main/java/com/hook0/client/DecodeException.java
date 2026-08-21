package com.hook0.client;

/**
 * What the API answered is not what it declares it answers.
 *
 * <p>Reading is deliberately strict. A member the document declares as a string and the API answered as a number stops
 * the read with the name of that member, rather than yielding a value whose documentation lies about what it holds.
 * Every failure of that kind is one of these, so a caller has one thing to catch whatever the shape of the answer was.
 */
public class DecodeException extends Hook0Exception {

  private static final long serialVersionUID = 1L;

  /**
   * Builds one out of what could not be read.
   *
   * @param detail what to say about the failure
   */
  public DecodeException(String detail) {
    super(detail);
  }

  /**
   * Builds one out of what could not be read and what stopped the read.
   *
   * @param detail what to say about the failure
   * @param cause what this failure was raised out of
   */
  public DecodeException(String detail, Throwable cause) {
    super(detail, cause);
  }
}
