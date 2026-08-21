package com.hook0.kotlin

/**
 * An answer that is not what the API declares it answers.
 *
 * A member the document requires and the API did not send, a string where a number was declared, a
 * value outside a closed list: each of them stops the read here rather than travelling on as a
 * value nobody checked.
 */
class DecodeException : Hook0Exception {

  /**
   * Builds one out of what could not be read.
   *
   * @param detail what to say about the failure
   */
  constructor(detail: String) : super(detail)

  /**
   * Builds one out of what could not be read and what reported it.
   *
   * @param detail what to say about the failure
   * @param cause what this failure was raised out of
   */
  constructor(detail: String, cause: Throwable) : super(detail, cause)
}
