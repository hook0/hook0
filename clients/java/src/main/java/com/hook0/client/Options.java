package com.hook0.client;

import java.time.Duration;

/**
 * Every bound a client applies to one send.
 *
 * <p>The first three bound what this client does; the last four bound what the other end may cost it. A server that is
 * broken or hostile can otherwise stream a body, a head or a single header of any length into a caller's memory, and a
 * client with no ceiling has no answer to that.
 *
 * <p>The four are not interchangeable. {@code maxHeadBytes} is the one that bounds what a head can cost, because it
 * bounds the total; a line count and a size per line multiply, and 64 lines of 64 KiB is 4 MiB of head admitted by
 * both. Those two earn their place by refusing sooner — on the first line past the count, or the first line too long —
 * rather than after the whole head has been read.
 *
 * @param retryPolicy how the attempts of one send are spaced out
 * @param requestTimeout how long one attempt is given
 * @param maxPayloadBytes the largest payload sent, refused before a socket is opened
 * @param maxResponseBytes the largest answer read off a socket
 * @param maxHeadBytes the largest head of an answer, every line of it taken together
 * @param maxResponseHeaders how many header lines an answer may carry
 * @param maxHeaderBytes the longest one header line may be
 */
public record Options(
    RetryPolicy retryPolicy,
    Duration requestTimeout,
    int maxPayloadBytes,
    long maxResponseBytes,
    int maxHeadBytes,
    int maxResponseHeaders,
    int maxHeaderBytes) {

  /**
   * Largest event payload the client agrees to send, in bytes.
   *
   * <p>Hook0's API refuses request bodies above 2 MiB, so a payload above 1 MiB is already at risk of being refused
   * once the JSON envelope around it — metadata, labels, identifiers — is counted. The client rules such an event out
   * rather than spending a round trip, and every retry after it, on a request that cannot be accepted.
   */
  public static final int DEFAULT_MAX_PAYLOAD_BYTES = 1024 * 1024;

  /** Largest response body read off a socket, in bytes. */
  public static final long DEFAULT_MAX_RESPONSE_BYTES = 8L * 1024 * 1024;

  /**
   * Largest head of an answer read off a socket, every line of it taken together, in bytes.
   *
   * <p>The number is the ceiling of the strictest runtime any Hook0 SDK runs on rather than a round figure, so that
   * every target can apply the same one in library code instead of inheriting whatever its runtime happens to do. This
   * one bounds what a head can cost; the two below it refuse sooner.
   */
  public static final int DEFAULT_MAX_HEAD_BYTES = 16 * 1024;

  /** How many header lines an answer may carry before it is refused. */
  public static final int DEFAULT_MAX_RESPONSE_HEADERS = 64;

  /** Longest one header line may be, name and value together, in bytes. */
  public static final int DEFAULT_MAX_HEADER_BYTES = 64 * 1024;

  /**
   * Longest one attempt at reaching the API is given before it is abandoned.
   *
   * <p>Ten seconds is far above what ingesting an event takes when the API is healthy, and short enough that a stuck
   * connection does not hold a caller for a noticeable time.
   */
  public static final Duration DEFAULT_REQUEST_TIMEOUT = Duration.ofSeconds(10);

  /**
   * The bounds a client applies when the caller names none.
   *
   * @return the defaults, which are the numbers the shared conformance corpus writes down
   */
  public static Options defaults() {
    return new Options(
        RetryPolicy.defaults(),
        DEFAULT_REQUEST_TIMEOUT,
        DEFAULT_MAX_PAYLOAD_BYTES,
        DEFAULT_MAX_RESPONSE_BYTES,
        DEFAULT_MAX_HEAD_BYTES,
        DEFAULT_MAX_RESPONSE_HEADERS,
        DEFAULT_MAX_HEADER_BYTES);
  }

  /**
   * The same bounds, under another schedule.
   *
   * @param chosen the schedule to apply
   * @return the bounds with that schedule
   */
  public Options withRetryPolicy(RetryPolicy chosen) {
    return new Options(
        chosen, requestTimeout, maxPayloadBytes, maxResponseBytes, maxHeadBytes, maxResponseHeaders, maxHeaderBytes);
  }

  /**
   * The same bounds, giving one attempt that long.
   *
   * @param chosen how long one attempt is given
   * @return the bounds with that timeout
   */
  public Options withRequestTimeout(Duration chosen) {
    return new Options(
        retryPolicy, chosen, maxPayloadBytes, maxResponseBytes, maxHeadBytes, maxResponseHeaders, maxHeaderBytes);
  }

  /**
   * The same bounds, sending at most that many bytes of payload.
   *
   * @param chosen the largest payload sent
   * @return the bounds with that ceiling
   */
  public Options withMaxPayloadBytes(int chosen) {
    return new Options(
        retryPolicy, requestTimeout, chosen, maxResponseBytes, maxHeadBytes, maxResponseHeaders, maxHeaderBytes);
  }

  /**
   * The same bounds, reading at most that many bytes of answer.
   *
   * @param chosen the largest answer read
   * @return the bounds with that ceiling
   */
  public Options withMaxResponseBytes(long chosen) {
    return new Options(
        retryPolicy, requestTimeout, maxPayloadBytes, chosen, maxHeadBytes, maxResponseHeaders, maxHeaderBytes);
  }

  /**
   * The same bounds, reading at most that large a head.
   *
   * @param chosen the largest head read, every line of it taken together
   * @return the bounds with that ceiling
   */
  public Options withMaxHeadBytes(int chosen) {
    return new Options(
        retryPolicy, requestTimeout, maxPayloadBytes, maxResponseBytes, chosen, maxResponseHeaders, maxHeaderBytes);
  }

  /**
   * The same bounds, reading at most that many header lines.
   *
   * @param chosen how many header lines an answer may carry
   * @return the bounds with that ceiling
   */
  public Options withMaxResponseHeaders(int chosen) {
    return new Options(
        retryPolicy, requestTimeout, maxPayloadBytes, maxResponseBytes, maxHeadBytes, chosen, maxHeaderBytes);
  }

  /**
   * The same bounds, reading at most that long a header line.
   *
   * @param chosen the longest one header line may be
   * @return the bounds with that ceiling
   */
  public Options withMaxHeaderBytes(int chosen) {
    return new Options(
        retryPolicy, requestTimeout, maxPayloadBytes, maxResponseBytes, maxHeadBytes, maxResponseHeaders, chosen);
  }
}
