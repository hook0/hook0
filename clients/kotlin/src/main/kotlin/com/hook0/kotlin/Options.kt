package com.hook0.kotlin

import java.time.Duration

/**
 * Every bound a client applies to one send.
 *
 * The first three bound what this client does; the last four bound what the other end may cost it. A
 * server that is broken or hostile can otherwise stream a body, a head or a single header of any
 * length into a caller's memory, and a client with no ceiling has no answer to that.
 *
 * The four are not interchangeable. [maxHeadBytes] is the one that bounds what a head can cost,
 * because it bounds the total; a line count and a size per line multiply, and 64 lines of 64 KiB is
 * 4 MiB of head admitted by both. Those two earn their place by refusing sooner — on the first line
 * past the count, or the first line too long — rather than after the whole head has been read.
 *
 * A caller lowers any of them with `copy`, which is what a data class already gives it: there is no
 * hand-written setter here for the same reason there is no hand-written equality.
 *
 * @property retryPolicy how the attempts of one send are spaced out
 * @property requestTimeout how long one attempt is given
 * @property maxPayloadBytes the largest payload sent, refused before a socket is opened
 * @property maxResponseBytes the largest answer read off a socket
 * @property maxHeadBytes the largest head of an answer, every line of it taken together
 * @property maxResponseHeaders how many header lines an answer may carry
 * @property maxHeaderBytes the longest one header line may be
 */
data class Options(
  val retryPolicy: RetryPolicy = RetryPolicy.defaults(),
  val requestTimeout: Duration = DEFAULT_REQUEST_TIMEOUT,
  val maxPayloadBytes: Int = DEFAULT_MAX_PAYLOAD_BYTES,
  val maxResponseBytes: Long = DEFAULT_MAX_RESPONSE_BYTES,
  val maxHeadBytes: Int = DEFAULT_MAX_HEAD_BYTES,
  val maxResponseHeaders: Int = DEFAULT_MAX_RESPONSE_HEADERS,
  val maxHeaderBytes: Int = DEFAULT_MAX_HEADER_BYTES
) {

  companion object {
    /**
     * Largest event payload the client agrees to send, in bytes.
     *
     * Hook0's API refuses request bodies above 2 MiB, so a payload above 1 MiB is already at risk of
     * being refused once the JSON envelope around it — metadata, labels, identifiers — is counted.
     * The client rules such an event out rather than spending a round trip, and every retry after
     * it, on a request that cannot be accepted.
     */
    const val DEFAULT_MAX_PAYLOAD_BYTES = 1024 * 1024

    /** Largest response body read off a socket, in bytes. */
    const val DEFAULT_MAX_RESPONSE_BYTES = 8L * 1024 * 1024

    /**
     * Largest head of an answer read off a socket, every line of it taken together, in bytes.
     *
     * The number is the ceiling of the strictest runtime any Hook0 SDK runs on rather than a round
     * figure, so that every target can apply the same one in library code instead of inheriting
     * whatever its runtime happens to do. This one bounds what a head can cost; the two below it
     * refuse sooner.
     */
    const val DEFAULT_MAX_HEAD_BYTES = 16 * 1024

    /** How many header lines an answer may carry before it is refused. */
    const val DEFAULT_MAX_RESPONSE_HEADERS = 64

    /** Longest one header line may be, name and value together, in bytes. */
    const val DEFAULT_MAX_HEADER_BYTES = 64 * 1024

    /**
     * Longest one attempt at reaching the API is given before it is abandoned.
     *
     * Ten seconds is far above what ingesting an event takes when the API is healthy, and short
     * enough that a stuck connection does not hold a caller for a noticeable time.
     */
    val DEFAULT_REQUEST_TIMEOUT: Duration = Duration.ofSeconds(10)

    /**
     * The bounds a client applies when the caller names none.
     *
     * @return the defaults, which are the numbers the shared conformance corpus writes down
     */
    fun defaults(): Options = Options()
  }
}
