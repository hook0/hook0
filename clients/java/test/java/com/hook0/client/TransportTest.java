package com.hook0.client;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.time.Duration;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.Timeout;

/**
 * What a server on the other end is not allowed to cost, measured against a real one.
 *
 * <p>Each bound is provoked over a loopback socket rather than reported: the answer is written by a server this suite
 * controls, and what is asserted is the cause this client read it as, since that is what decides whether the request is
 * issued again.
 */
@Timeout(120)
final class TransportTest {

  private static final String INGESTED_ID = "01961234-5678-7abc-8def-0123456789ac";

  private static Options once() {
    return Options.defaults().withRetryPolicy(RetryPolicy.disabled()).withRequestTimeout(Duration.ofSeconds(5));
  }

  private static Answer issued(FakeApi api, Options options) {
    try (HttpTransport transport = new HttpTransport(api.baseUrl(), "token-xyz", options)) {
      return transport.request("GET", "somewhere", List.of(), null);
    }
  }

  private static TransportException refused(FakeApi api, Options options) {
    return assertThrows(TransportException.class, () -> issued(api, options));
  }

  /** A head of roughly that many bytes, spread across as many lines as it takes. */
  private static Map<String, String> headOf(int bytes, int perLine) {
    Map<String, String> headers = new LinkedHashMap<>();
    int written = 0;
    for (int index = 0; written < bytes; index++) {
      String name = "x-pad-" + index;
      String value = "y".repeat(perLine);
      headers.put(name, value);
      written += name.length() + value.length() + 4;
    }
    return headers;
  }

  @Test
  void aPolicyHoldingNumbersItsHeaderCannotStateIsStillStatedOnTheWire() {
    // Three numbers a caller should not write and can: more attempts than the policy will ever make,
    // a delay too large for the milliseconds it is stated in, and a negative one. What reaches the
    // socket is the schedule the policy would actually apply, rather than an exception raised while
    // a header was being composed.
    RetryPolicy absurd =
        new RetryPolicy(
            1000, Duration.ofSeconds(Long.MAX_VALUE), Duration.ofMillis(-5), Duration.ofSeconds(Long.MAX_VALUE));

    try (FakeApi api = new FakeApi()) {
      api.willAnswer(FakeApi.Scripted.of(200, Map.of()));
      issued(api, once().withRetryPolicy(absurd));

      assertEquals(
          "attempts=" + RetryPolicy.MAX_ATTEMPTS_CAP + ",backoff=" + Long.MAX_VALUE + ",ceiling=0,budget="
              + Long.MAX_VALUE,
          api.received().get(0).headers().get("hook0-client-options"));
    }
  }

  @Test
  void anAnswerAboveTheBodyBoundIsRefusedAsOneThatWouldDrawTheSameAnswerAgain() {
    try (FakeApi api = new FakeApi()) {
      api.willAnswer(FakeApi.Scripted.of(200, Map.of("padding", "x".repeat(4096))));

      TransportException refused = refused(api, once().withMaxResponseBytes(256));

      assertEquals(TransportException.ANSWER_ABOVE_A_BOUND, refused.causeName());
      assertEquals(false, refused.retryable());
      assertTrue(refused.getMessage().contains("256"), refused.getMessage());
    }
  }

  @Test
  void anAnswerCarryingMoreHeaderLinesThanTheBoundIsRefused() {
    try (FakeApi api = new FakeApi()) {
      api.willAnswer(FakeApi.Scripted.of(200, Map.of(), headOf(600, 10)));

      TransportException refused = refused(api, once().withMaxResponseHeaders(4));

      assertEquals(TransportException.ANSWER_ABOVE_A_BOUND, refused.causeName());
      assertTrue(refused.getMessage().contains("header lines"), refused.getMessage());
    }
  }

  @Test
  void anAnswerCarryingAHeaderLineAboveTheBoundIsRefused() {
    try (FakeApi api = new FakeApi()) {
      api.willAnswer(FakeApi.Scripted.of(200, Map.of(), Map.of("x-long", "y".repeat(600))));

      TransportException refused = refused(api, once().withMaxHeaderBytes(128));

      assertEquals(TransportException.ANSWER_ABOVE_A_BOUND, refused.causeName());
      assertTrue(refused.getMessage().contains("x-long"), refused.getMessage());
    }
  }

  @Test
  void anAnswerWhoseHeadIsAboveTheBoundIsRefusedEvenThoughNoLineIs() {
    // What the total bound is for: sixty-four lines of sixty-four kilobytes sits inside both of the
    // component bounds and is four megabytes of head.
    try (FakeApi api = new FakeApi()) {
      api.willAnswer(FakeApi.Scripted.of(200, Map.of(), headOf(4096, 100)));

      TransportException refused =
          refused(api, once().withMaxHeadBytes(1024).withMaxResponseHeaders(1024).withMaxHeaderBytes(65536));

      assertEquals(TransportException.ANSWER_ABOVE_A_BOUND, refused.causeName());
      assertTrue(refused.getMessage().contains("head above"), refused.getMessage());
    }
  }

  @Test
  void theBoundThisClientAppliesToAHeadIsTheOneThatDecides() {
    // The runtime has a ceiling of its own — 393216 bytes, which this suite measures below — and the
    // one this client applies sits far under it, so what refuses a large head is this client rather
    // than whichever runtime it happens to be running on. That is the whole reason the bound is
    // applied in library code instead of inherited.
    assertTrue(
        Options.DEFAULT_MAX_HEAD_BYTES < measuredRuntimeHeadBound(),
        "the head bound this client applies sits above the one its runtime applies, so the runtime "
            + "would refuse a head before this client had a say in it");

    try (FakeApi api = new FakeApi()) {
      api.willAnswer(FakeApi.Scripted.of(200, Map.of(), headOf(300 * 1024, 200)));

      TransportException refused = refused(api, once());

      assertEquals(
          TransportException.ANSWER_ABOVE_A_BOUND,
          refused.causeName(),
          "a head of 300 KiB was read as something other than an answer above a bound, which means "
              + "the runtime refused it before this client did");
    }
  }

  /**
   * What the runtime bounds a head at, measured rather than looked up.
   *
   * <p>{@code java.net.http.HttpClient} bounds the whole head of an answer and nothing else: not the number of header
   * lines, not the length of one of them, and not the body at all. The ceiling is settled by
   * {@code jdk.http.maxHeaderSize}, which is unset by default and falls back to the number this measures — so an
   * application can move it, which is the second reason this client does not lean on it.
   */
  private static int measuredRuntimeHeadBound() {
    // Bisected between a head the runtime is known to read and one it is known to refuse, so that a
    // release changing the number changes what this suite asserts rather than failing on it.
    int accepted = 8 * 1024;
    int refused = 4 * 1024 * 1024;
    while (refused - accepted > 64 * 1024) {
      int tried = accepted + (refused - accepted) / 2;
      if (runtimeReads(tried)) {
        accepted = tried;
      } else {
        refused = tried;
      }
    }
    return accepted;
  }

  private static boolean runtimeReads(int headBytes) {
    try (FakeApi api = new FakeApi()) {
      api.willAnswer(FakeApi.Scripted.of(200, Map.of("event_id", INGESTED_ID), headOf(headBytes, 1024)));
      // Every bound of this client is opened up so that whatever refuses the head is the runtime.
      Options unbounded =
          once()
              .withMaxHeadBytes(Integer.MAX_VALUE)
              .withMaxResponseHeaders(Integer.MAX_VALUE)
              .withMaxHeaderBytes(Integer.MAX_VALUE);
      try {
        return issued(api, unbounded).status() == 200;
      } catch (TransportException refused) {
        return false;
      }
    }
  }
}
