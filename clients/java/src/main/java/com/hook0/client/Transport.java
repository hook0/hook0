package com.hook0.client;

import java.util.List;
import java.util.concurrent.CompletableFuture;

/**
 * How a request reaches the API, in the two shapes the generated half issues one under.
 *
 * <p>One interface rather than two: both generated surfaces are built on the same object, so a client cannot be
 * configured one way for its blocking calls and another way for the calls that answer a future. The two methods issue
 * the same request under the same bounds and differ only in when the caller hears about it.
 *
 * <p>Neither method knows what the API declares — reading the bytes is the generated half's job, and deciding whether
 * to send them again is the client's.
 */
public interface Transport {

  /**
   * Issues one request and waits for what the API answers.
   *
   * @param method the HTTP method the operation is issued under
   * @param path where the request lands, under the API URL
   * @param query the name and value pairs of the query string
   * @param body what to send as a JSON document, or {@code null} to send none
   * @return what the API answered
   * @throws TransportException when the API answered nothing at all
   */
  Answer request(String method, String path, List<QueryParameter> query, Object body);

  /**
   * Issues one request and hands back what the API will answer.
   *
   * <p>A request that cannot be built at all completes the future exceptionally rather than throwing: a caller holding
   * a future should never have to catch as well as compose.
   *
   * @param method the HTTP method the operation is issued under
   * @param path where the request lands, under the API URL
   * @param query the name and value pairs of the query string
   * @param body what to send as a JSON document, or {@code null} to send none
   * @return what the API will answer
   */
  CompletableFuture<Answer> requestAsync(String method, String path, List<QueryParameter> query, Object body);
}
