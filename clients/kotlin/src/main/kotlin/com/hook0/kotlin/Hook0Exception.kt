package com.hook0.kotlin

/**
 * What every failure this SDK reports is a kind of.
 *
 * Unchecked, which in Kotlin is the only kind there is. Half of this client suspends, and a failure
 * raised out of a suspending call has to reach the caller as the same thing a blocking call raises:
 * one hierarchy is what keeps the two surfaces from reporting the same failure in two ways.
 *
 * Four kinds sit under it. [ClientException] is what this client refuses to do; [TransportException]
 * is a request that never produced an answer to read; [DecodeException] is an answer that is not
 * what the API declares it answers; and the generated problem hierarchy is what the API itself
 * reported.
 */
open class Hook0Exception : RuntimeException {

  /**
   * Builds one out of what went wrong.
   *
   * @param detail what to say about the failure
   */
  constructor(detail: String) : super(detail)

  /**
   * Builds one out of what went wrong and what caused it.
   *
   * @param detail what to say about the failure
   * @param cause what this failure was raised out of
   */
  constructor(detail: String, cause: Throwable) : super(detail, cause)
}
