package com.hook0.kotlin

import java.time.Duration
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertFalse
import org.junit.jupiter.api.Assertions.assertThrows
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.Timeout

/**
 * What a server on the other end is not allowed to cost, measured against a real one.
 *
 * Each bound is provoked over a loopback socket rather than reported: the answer is written by a
 * server this suite controls, and what is asserted is the cause this client read it as, since that
 * is what decides whether the request is issued again.
 */
@Timeout(180)
class TransportTest {

  @Test
  fun whatTheHeaderStatesIsWhatTheScheduleWaits() {
    // The header is worth nothing if it describes a schedule the client does not keep. Every delay
    // is drawn against the whole of its ceiling, which is what the drawn schedule looks like at its
    // longest, and the numbers the wire carries are held against it rather than against the policy
    // they were both read from.
    val policy = RetryPolicy(3, Duration.ofMillis(200), Duration.ofMillis(400), Duration.ofSeconds(5))

    FakeApi().use { api ->
      api.willAnswer(FakeApi.Scripted.of(200, emptyMap<String, Any?>()))
      issued(api, once().copy(retryPolicy = policy))

      val stated = api.received()[0].headers["hook0-client-options"]!!
        .split(",")
        .associate { part -> part.substringBefore('=') to part.substringAfter('=').toLong() }

      val longest = policy.delaysMillis(doubleArrayOf(1.0, 1.0))

      assertEquals(
        stated["backoff"],
        policy.backoffCeilingMillis(1),
        "the header states a first delay the schedule does not hold to"
      )
      for (waited in longest) {
        assertTrue(
          waited <= stated["ceiling"]!!,
          "the schedule waits ${waited}ms where the header states a ceiling of ${stated["ceiling"]}"
        )
      }
      assertTrue(
        longest.sum() <= stated["budget"]!!,
        "the schedule spends ${longest.sum()}ms where the header states a budget of ${stated["budget"]}"
      )
    }
  }

  @Test
  fun aPolicyHoldingNumbersItsHeaderCannotStateIsStillStatedOnTheWire() {
    // Three numbers a caller should not write and can: more attempts than the policy will ever make,
    // a delay too large for the milliseconds it is stated in, and a negative one. What reaches the
    // socket is the schedule the policy would actually apply, rather than an exception raised while
    // a header was being composed.
    val absurd = RetryPolicy(
      1000,
      Duration.ofSeconds(Long.MAX_VALUE),
      Duration.ofMillis(-5),
      Duration.ofSeconds(Long.MAX_VALUE)
    )

    // The schedule reads those same numbers, so it has to survive them too: a header stating a
    // budget the scheduler raises on rather than waits describes a send that dies. What it holds
    // to is what the header states — a ceiling of nothing, waited before each of its attempts.
    val waited = absurd.delaysMillis(doubleArrayOf(1.0, 1.0, 1.0))
    assertEquals(RetryPolicy.MAX_ATTEMPTS_CAP - 1, waited.size)
    assertEquals(0L, waited.sum())

    FakeApi().use { api ->
      api.willAnswer(FakeApi.Scripted.of(200, emptyMap<String, Any?>()))
      issued(api, once().copy(retryPolicy = absurd))

      assertEquals(
        "attempts=${RetryPolicy.MAX_ATTEMPTS_CAP},backoff=${Long.MAX_VALUE},ceiling=0," +
          "budget=${Long.MAX_VALUE}",
        api.received()[0].headers["hook0-client-options"]
      )
    }
  }

  @Test
  fun anAnswerAboveTheBodyBoundIsRefusedAsOneThatWouldDrawTheSameAnswerAgain() {
    FakeApi().use { api ->
      api.willAnswer(FakeApi.Scripted.of(200, mapOf("padding" to "x".repeat(4096))))

      val refused = refused(api, once().copy(maxResponseBytes = 256))

      assertEquals(TransportException.ANSWER_ABOVE_A_BOUND, refused.causeName)
      assertEquals(false, refused.retryable)
      assertTrue(refused.message?.contains("256") == true, refused.message)
    }
  }

  @Test
  fun anAnswerCarryingMoreHeaderLinesThanTheBoundIsRefused() {
    FakeApi().use { api ->
      api.willAnswer(FakeApi.Scripted.of(200, emptyMap<String, Any?>(), headOf(600, 10)))

      val refused = refused(api, once().copy(maxResponseHeaders = 4))

      assertEquals(TransportException.ANSWER_ABOVE_A_BOUND, refused.causeName)
      assertTrue(refused.message?.contains("header lines") == true, refused.message)
    }
  }

  @Test
  fun anAnswerCarryingAHeaderLineAboveTheBoundIsRefused() {
    FakeApi().use { api ->
      api.willAnswer(
        FakeApi.Scripted.of(200, emptyMap<String, Any?>(), mapOf("x-long" to "y".repeat(600)))
      )

      val refused = refused(api, once().copy(maxHeaderBytes = 128))

      assertEquals(TransportException.ANSWER_ABOVE_A_BOUND, refused.causeName)
      assertTrue(refused.message?.contains("x-long") == true, refused.message)
    }
  }

  @Test
  fun anAnswerWhoseHeadIsAboveTheBoundIsRefusedEvenThoughNoLineIs() {
    // What the total bound is for: sixty-four lines of sixty-four kilobytes sits inside both of the
    // component bounds and is four megabytes of head.
    FakeApi().use { api ->
      api.willAnswer(FakeApi.Scripted.of(200, emptyMap<String, Any?>(), headOf(4096, 100)))

      val refused =
        refused(
          api,
          once().copy(maxHeadBytes = 1024, maxResponseHeaders = 1024, maxHeaderBytes = 65536)
        )

      assertEquals(TransportException.ANSWER_ABOVE_A_BOUND, refused.causeName)
      assertTrue(refused.message?.contains("head above") == true, refused.message)
    }
  }

  @Test
  fun theBoundThisClientAppliesToAHeadIsTheOneThatDecides() {
    // The runtime has a ceiling of its own — which this suite measures below — and the one this
    // client applies sits far under it, so what refuses a large head is this client rather than
    // whichever runtime it happens to be running on. That is the whole reason the bound is applied
    // in library code instead of inherited.
    assertTrue(
      Options.DEFAULT_MAX_HEAD_BYTES < measuredRuntimeHeadBound(),
      "the head bound this client applies sits above the one its runtime applies, so the runtime " +
        "would refuse a head before this client had a say in it"
    )

    FakeApi().use { api ->
      api.willAnswer(FakeApi.Scripted.of(200, emptyMap<String, Any?>(), headOf(300 * 1024, 200)))

      val refused = refused(api, once())

      assertEquals(
        TransportException.ANSWER_ABOVE_A_BOUND,
        refused.causeName,
        "a head of 300 KiB was read as something other than an answer above a bound, which means " +
          "the runtime refused it before this client did"
      )
    }
  }

  @Test
  fun theSuspendingSurfaceIsHeldToTheSameBoundAsTheBlockingOne() {
    // The two surfaces share one transport and one set of ceilings, and this is what says so: the
    // same oversized answer, read the other way round, is refused as the same cause.
    FakeApi().use { api ->
      api.willAnswer(FakeApi.Scripted.of(200, mapOf("padding" to "x".repeat(4096))))

      val refused =
        assertThrows(TransportException::class.java) {
          HttpTransport(api.baseUrl(), "token-xyz", once().copy(maxResponseBytes = 256)).use { transport ->
            Surface.awaiting { transport.requestSuspending("GET", "somewhere", emptyList(), null) }
          }
        }

      assertEquals(TransportException.ANSWER_ABOVE_A_BOUND, refused.causeName)
    }
  }

  @Test
  fun anApiUrlThatIsNotSomewhereARequestCanGoIsRefusedBeforeASocketIsOpened() {
    // Each of the ways a base URL can fail to be one this transport can reach: not a URL at all, a
    // scheme nothing is sent over, and no host to send it to. All are refused when the request is
    // built rather than when a connection is attempted, so a caller hears why.
    for (written in listOf("not a url at all", "ftp://example.test/api/v1", "http:///api/v1", "")) {
      HttpTransport(written, "token-xyz", once()).use { transport ->
        val refused = assertThrows(TransportException::class.java) {
          transport.request("GET", "somewhere", emptyList(), null)
        }

        assertFalse(refused.retryable, "`$written` was reported as worth trying again")
      }
    }
  }

  @Test
  fun aBaseUrlIsExtendedByAPathRatherThanReplacedByIt() {
    // With and without the trailing slash the API URL is written with: a request lands under the
    // base either way, rather than at the root of the host.
    for (written in listOf("/api/v1", "/api/v1/")) {
      FakeApi().use { api ->
        HttpTransport(api.baseUrl() + written, "token-xyz", once()).use { transport ->
          api.willAnswer(FakeApi.Scripted.of(200, emptyMap<String, Any?>()))
          transport.request("GET", "somewhere", emptyList(), null)

          assertEquals("/api/v1/somewhere", api.received()[0].target)
        }
      }
    }
  }

  @Test
  fun aQueryIsAddedToWhateverThePathAlreadyCarried() {
    FakeApi().use { api ->
      HttpTransport(api.baseUrl() + "/api/v1/", "token-xyz", once()).use { transport ->
        api.willAnswer(
          FakeApi.Scripted.of(200, emptyMap<String, Any?>()),
          FakeApi.Scripted.of(200, emptyMap<String, Any?>())
        )

        transport.request(
          "GET",
          "somewhere",
          listOf(QueryParameter("first", "one"), QueryParameter("second", "")),
          null
        )

        // A parameter carrying nothing travels as an empty value rather than as the word `null`.
        assertEquals("/api/v1/somewhere?first=one&second=", api.received()[0].target)

        // A path that already names something is extended rather than given a second `?`.
        transport.request("GET", "somewhere?tenant=acme", listOf(QueryParameter("first", "one")), null)

        assertEquals("/api/v1/somewhere?tenant=acme&first=one", api.received()[1].target)
      }
    }
  }

  @Test
  fun aQueryTheBaseUrlCarriesIsNotOneEveryRequestInherits() {
    // What a base URL's own query string does to the requests issued under it: nothing. A relative
    // path replaces it, which is what RFC 3986 says and not what a caller who put a parameter there
    // would expect — so it is written down rather than left to be discovered.
    FakeApi().use { api ->
      HttpTransport(api.baseUrl() + "/api/v1/?tenant=acme", "token-xyz", once()).use { transport ->
        api.willAnswer(FakeApi.Scripted.of(200, emptyMap<String, Any?>()))
        transport.request("GET", "somewhere", emptyList(), null)

        assertEquals("/api/v1/somewhere", api.received()[0].target)
      }
    }
  }

  companion object {
    private const val INGESTED_ID = "01961234-5678-7abc-8def-0123456789ac"

    private fun once(): Options = Options.defaults()
      .copy(retryPolicy = RetryPolicy.disabled(), requestTimeout = Duration.ofSeconds(5))

    private fun issued(api: FakeApi, options: Options): Answer =
      HttpTransport(api.baseUrl(), "token-xyz", options).use { transport ->
        transport.request("GET", "somewhere", emptyList(), null)
      }

    private fun refused(api: FakeApi, options: Options): TransportException =
      assertThrows(TransportException::class.java) { issued(api, options) }

    /** A head of roughly that many bytes, spread across as many lines as it takes. */
    private fun headOf(bytes: Int, perLine: Int): Map<String, String> {
      val headers = LinkedHashMap<String, String>()
      var written = 0
      var index = 0
      while (written < bytes) {
        val name = "x-pad-$index"
        val value = "y".repeat(perLine)
        headers[name] = value
        written += name.length + value.length + 4
        index++
      }
      return headers
    }

    /**
     * What the runtime bounds a head at, measured rather than looked up.
     *
     * `java.net.http.HttpClient` bounds the whole head of an answer and nothing else: not the number
     * of header lines, not the length of one of them, and not the body at all. The ceiling is
     * settled by `jdk.http.maxHeaderSize`, which is unset by default and falls back to the number
     * this measures — so an application can move it, which is the second reason this client does not
     * lean on it.
     */
    private fun measuredRuntimeHeadBound(): Int {
      // Bisected between a head the runtime is known to read and one it is known to refuse, so that
      // a release changing the number changes what this suite asserts rather than failing on it.
      var accepted = 8 * 1024
      var refused = 4 * 1024 * 1024
      while (refused - accepted > 64 * 1024) {
        val tried = accepted + (refused - accepted) / 2
        if (runtimeReads(tried)) {
          accepted = tried
        } else {
          refused = tried
        }
      }
      return accepted
    }

    private fun runtimeReads(headBytes: Int): Boolean = FakeApi().use { api ->
      api.willAnswer(
        FakeApi.Scripted.of(
          200,
          mapOf("event_id" to INGESTED_ID),
          headOf(headBytes, 1024)
        )
      )
      // Every bound of this client is opened up so that whatever refuses the head is the runtime.
      val unbounded =
        once().copy(
          maxHeadBytes = Int.MAX_VALUE,
          maxResponseHeaders = Int.MAX_VALUE,
          maxHeaderBytes = Int.MAX_VALUE
        )
      try {
        issued(api, unbounded).status == 200
      } catch (refused: TransportException) {
        false
      }
    }
  }
}
