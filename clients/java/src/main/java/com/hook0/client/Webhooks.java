package com.hook0.client;

import java.time.Duration;
import java.time.Instant;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;

/**
 * Verifying that a webhook came from Hook0, and that nothing in it changed on the way.
 *
 * <p>The clock window is bilateral. A moment too far in the future is refused exactly like one too far in the past, so
 * the window a given delivery is accepted in stays the width the caller asked for, whichever way a clock drifted. A
 * window that only looked backwards would be one a sender widens by dating its own delivery in the future.
 */
public final class Webhooks {

  private Webhooks() {}

  /**
   * Verifies a webhook against the current moment.
   *
   * @param signature the value of the {@code X-Hook0-Signature} header
   * @param payload the raw body of the webhook request
   * @param headers the headers of the webhook request, in the order they arrived
   * @param subscriptionSecret the signing secret of the subscription it was delivered for
   * @param tolerance how far, in either direction, the moment the signature names may sit from now. Five minutes is a
   *     reasonable trade-off between tolerating clock drift and bounding how long a captured delivery can be replayed
   * @throws ClientException for every reason a webhook may be refused
   */
  public static void verify(
      String signature,
      String payload,
      List<Map.Entry<String, String>> headers,
      String subscriptionSecret,
      Duration tolerance) {
    verifyAt(signature, payload, headers, subscriptionSecret, tolerance, Instant.now());
  }

  /**
   * Verifies a webhook whose headers arrived under names a map already keeps apart.
   *
   * @param signature the value of the {@code X-Hook0-Signature} header
   * @param payload the raw body of the webhook request
   * @param headers the headers of the webhook request
   * @param subscriptionSecret the signing secret of the subscription it was delivered for
   * @param tolerance how far, in either direction, the moment the signature names may sit from now
   * @throws ClientException for every reason a webhook may be refused
   */
  public static void verify(
      String signature,
      String payload,
      Map<String, String> headers,
      String subscriptionSecret,
      Duration tolerance) {
    verifyAt(signature, payload, pairs(headers), subscriptionSecret, tolerance, Instant.now());
  }

  /**
   * Verifies a webhook against a moment the caller names.
   *
   * @param signature the value of the {@code X-Hook0-Signature} header
   * @param payload the raw body of the webhook request
   * @param headers the headers of the webhook request, in the order they arrived
   * @param subscriptionSecret the signing secret of the subscription it was delivered for
   * @param tolerance how far, in either direction, the moment the signature names may sit from {@code now}
   * @param now what to hold the signature's moment against
   * @throws ClientException for every reason a webhook may be refused
   */
  public static void verifyAt(
      String signature,
      String payload,
      List<Map.Entry<String, String>> headers,
      String subscriptionSecret,
      Duration tolerance,
      Instant now) {
    Signature parsed = Signature.parse(signature);

    Map<String, String> delivered = deliveredHeaders(headers);
    List<String> coveredValues = new ArrayList<>(parsed.coveredHeaders().size());
    for (String name : parsed.coveredHeaders()) {
      String value = delivered.get(name);
      if (value == null) {
        throw ClientException.refusedDelivery(
            "the `" + name + "` header the signature covers was not delivered");
      }
      coveredValues.add(value);
    }

    if (!parsed.matches(payload == null ? "" : payload, coveredValues, subscriptionSecret)) {
      throw ClientException.refusedDelivery("the signature does not match what the subscription secret produces");
    }

    Duration drift = Duration.between(Instant.ofEpochSecond(parsed.timestamp()), now);
    if (drift.abs().compareTo(tolerance) > 0) {
      throw ClientException.refusedDelivery(
          "the signature was made "
              + drift.toSeconds()
              + " seconds from now, outside the "
              + tolerance.toSeconds()
              + " accepted");
    }
  }

  /**
   * The headers of the request, under the names a signature refers to them by.
   *
   * <p>A later value wins over an earlier one under the same name, which is what a map built by the caller would have
   * done.
   */
  private static Map<String, String> deliveredHeaders(List<Map.Entry<String, String>> headers) {
    Map<String, String> delivered = new LinkedHashMap<>();
    if (headers == null) {
      return delivered;
    }
    for (Map.Entry<String, String> header : headers) {
      if (header == null || header.getKey() == null || header.getValue() == null) {
        throw ClientException.refusedDelivery("a header carries no name or no value");
      }
      delivered.put(header.getKey().toLowerCase(Locale.ROOT), header.getValue());
    }
    return delivered;
  }

  private static List<Map.Entry<String, String>> pairs(Map<String, String> headers) {
    List<Map.Entry<String, String>> written = new ArrayList<>();
    if (headers == null) {
      return written;
    }
    for (Map.Entry<String, String> header : headers.entrySet()) {
      written.add(Map.entry(header.getKey(), header.getValue()));
    }
    return written;
  }
}
