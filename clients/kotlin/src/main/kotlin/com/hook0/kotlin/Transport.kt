package com.hook0.kotlin

/**
 * How a request reaches the API, in the two shapes the generated half issues one under.
 *
 * One interface rather than two: both generated surfaces are built on the same object, so a client
 * cannot be configured one way for its blocking calls and another way for the calls that suspend.
 * The two methods issue the same request under the same bounds and differ only in whether the
 * caller's thread is held while the API takes its time.
 *
 * Neither method knows what the API declares — reading the bytes is the generated half's job, and
 * deciding whether to send them again is the client's.
 */
interface Transport {

  /**
   * Issues one request and waits for what the API answers.
   *
   * @param method the HTTP method the operation is issued under
   * @param path where the request lands, under the API URL
   * @param query the name and value pairs of the query string
   * @param body what to send as a JSON document, or nothing to send none
   * @return what the API answered
   * @throws TransportException when the API answered nothing at all
   */
  fun request(method: String, path: String, query: List<QueryParameter>, body: Any?): Answer

  /**
   * Issues one request and suspends until the API has answered.
   *
   * @param method the HTTP method the operation is issued under
   * @param path where the request lands, under the API URL
   * @param query the name and value pairs of the query string
   * @param body what to send as a JSON document, or nothing to send none
   * @return what the API answered
   * @throws TransportException when the API answered nothing at all
   */
  suspend fun requestSuspending(method: String, path: String, query: List<QueryParameter>, body: Any?): Answer
}
