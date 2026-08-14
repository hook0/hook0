package com.hook0.kotlin

import java.time.Duration
import java.time.Instant
import java.util.Locale

/**
 * Verifying that a webhook came from Hook0, and that nothing in it changed on the way.
 *
 * The clock window is bilateral. A moment too far in the future is refused exactly like one too far
 * in the past, so the window a given delivery is accepted in stays the width the caller asked for,
 * whichever way a clock drifted. A window that only looked backwards would be one a sender widens by
 * dating its own delivery in the future.
 */
object Webhooks {

  /**
   * Verifies a webhook against the current moment.
   *
   * @param signature the value of the `X-Hook0-Signature` header
   * @param payload the raw body of the webhook request
   * @param headers the headers of the webhook request, in the order they arrived
   * @param subscriptionSecret the signing secret of the subscription it was delivered for
   * @param tolerance how far, in either direction, the moment the signature names may sit from now.
   *     Five minutes is a reasonable trade-off between tolerating clock drift and bounding how long
   *     a captured delivery can be replayed
   * @throws ClientException for every reason a webhook may be refused
   */
  fun verify(
    signature: String,
    payload: String,
    headers: List<Pair<String, String>>,
    subscriptionSecret: String,
    tolerance: Duration
  ) {
    verifyAt(signature, payload, headers, subscriptionSecret, tolerance, Instant.now())
  }

  /**
   * Verifies a webhook whose headers arrived under names a map already keeps apart.
   *
   * @param signature the value of the `X-Hook0-Signature` header
   * @param payload the raw body of the webhook request
   * @param headers the headers of the webhook request
   * @param subscriptionSecret the signing secret of the subscription it was delivered for
   * @param tolerance how far, in either direction, the moment the signature names may sit from now
   * @throws ClientException for every reason a webhook may be refused
   */
  fun verify(
    signature: String,
    payload: String,
    headers: Map<String, String>,
    subscriptionSecret: String,
    tolerance: Duration
  ) {
    verifyAt(
      signature,
      payload,
      headers.map { (name, value) -> name to value },
      subscriptionSecret,
      tolerance,
      Instant.now()
    )
  }

  /**
   * Verifies a webhook against a moment the caller names.
   *
   * @param signature the value of the `X-Hook0-Signature` header
   * @param payload the raw body of the webhook request
   * @param headers the headers of the webhook request, in the order they arrived
   * @param subscriptionSecret the signing secret of the subscription it was delivered for
   * @param tolerance how far, in either direction, the moment the signature names may sit from [now]
   * @param now what to hold the signature's moment against
   * @throws ClientException for every reason a webhook may be refused
   */
  fun verifyAt(
    signature: String,
    payload: String,
    headers: List<Pair<String, String>>,
    subscriptionSecret: String,
    tolerance: Duration,
    now: Instant
  ) {
    val parsed = Signature.parse(signature)

    val delivered = deliveredHeaders(headers)
    val coveredValues =
      parsed.coveredHeaders.map { name ->
        delivered[name]
          ?: throw ClientException.refusedDelivery(
            "the `$name` header the signature covers was not delivered"
          )
      }

    if (!parsed.matches(payload, coveredValues, subscriptionSecret)) {
      throw ClientException.refusedDelivery(
        "the signature does not match what the subscription secret produces"
      )
    }

    val drift = Duration.between(Instant.ofEpochSecond(parsed.timestamp), now)
    if (drift.abs() > tolerance) {
      throw ClientException.refusedDelivery(
        "the signature was made ${drift.toSeconds()} seconds from now, outside the " +
          "${tolerance.toSeconds()} accepted"
      )
    }
  }

  /**
   * The headers of the request, under the names a signature refers to them by.
   *
   * A later value wins over an earlier one under the same name, which is what a map built by the
   * caller would have done.
   */
  private fun deliveredHeaders(headers: List<Pair<String, String>>): Map<String, String> {
    val delivered = LinkedHashMap<String, String>()
    for ((name, value) in headers) {
      delivered[name.lowercase(Locale.ROOT)] = value
    }
    return delivered
  }
}
