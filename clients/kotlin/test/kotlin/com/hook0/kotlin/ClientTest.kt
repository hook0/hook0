package com.hook0.kotlin

import java.time.Duration
import java.util.UUID
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertNotEquals
import org.junit.jupiter.api.Assertions.assertNotNull
import org.junit.jupiter.api.Assertions.assertThrows
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.Timeout
import org.junit.jupiter.params.ParameterizedTest
import org.junit.jupiter.params.provider.EnumSource

/** What a send does over a real socket, put through both surfaces. */
@Timeout(120)
class ClientTest {

  @ParameterizedTest
  @EnumSource(Surface::class)
  fun aBudgetTooLargeToCountStillCutsDownTheDelayTheApiNames(surface: Surface) {
    // The retry loop reads the budget a third time, to cut an API-named delay down to what is left
    // of it. That read is the one furthest from the schedule and the header, and a budget too large
    // to count raised there rather than anywhere a caller would look — the send died between two
    // attempts, on the answer that asked it to come back.
    val absurd = options(2).copy(
      retryPolicy = RetryPolicy(
        2,
        Duration.ofMillis(5),
        Duration.ofMillis(5),
        Duration.ofSeconds(Long.MAX_VALUE)
      )
    )

    FakeApi().use { api ->
      client(api, absurd).use { client ->
        api.willAnswer(
          FakeApi.Scripted.of(
            503,
            mapOf(
              "id" to "ServiceUnavailable",
              "status" to 503L,
              "title" to "unavailable",
              "detail" to "come back shortly",
              "type" to "https://hook0.com/documentation/errors/ServiceUnavailable"
            ),
            mapOf("Retry-After" to "0")
          ),
          ingested(INGESTED_ID)
        )

        assertEquals(UUID.fromString(INGESTED_ID), surface.send(client, anEvent()))
        assertEquals(2, api.received().size, "the named delay was not honoured and retried")
      }
    }
  }

  @ParameterizedTest
  @EnumSource(Surface::class)
  fun aSendThatSucceedsIssuesOneRequest(surface: Surface) {
    FakeApi().use { api ->
      client(api, options(4)).use { client ->
        api.willAnswer(ingested(INGESTED_ID))

        assertEquals(UUID.fromString(INGESTED_ID), surface.send(client, anEvent()))
        assertEquals(1, api.received().size)
      }
    }
  }

  @ParameterizedTest
  @EnumSource(Surface::class)
  fun anEventCarryingNoIdIsSentUnderOneTheClientMinted(surface: Surface) {
    FakeApi().use { api ->
      client(api, options(4)).use { client ->
        api.willAnswer(ingested(INGESTED_ID))
        surface.send(client, anEvent())

        val sent = api.received()[0].json() as Map<*, *>
        assertEquals(7, UUID.fromString(sent["event_id"] as String).version())
      }
    }
  }

  @ParameterizedTest
  @EnumSource(Surface::class)
  fun anEventCarryingAnIdIsSentUnderIt(surface: Surface) {
    val chosen = UUID.fromString("01961234-5678-7abc-8def-0123456789ab")
    FakeApi().use { api ->
      client(api, options(4)).use { client ->
        api.willAnswer(ingested(chosen.toString()))
        surface.send(client, anEvent().copy(eventId = chosen))

        assertEquals(chosen.toString(), (api.received()[0].json() as Map<*, *>)["event_id"])
      }
    }
  }

  @ParameterizedTest
  @EnumSource(Surface::class)
  fun anAttemptThatRanOutOfTimeIsRepeatedUnderTheSameId(surface: Surface) {
    val impatient = options(4).copy(requestTimeout = Duration.ofMillis(200))
    FakeApi().use { api ->
      client(api, impatient).use { client ->
        api.willAnswer(
          FakeApi.Scripted(
            201,
            Json.write(mapOf("event_id" to INGESTED_ID)),
            Duration.ofSeconds(1)
          ),
          ingested(INGESTED_ID)
        )

        assertEquals(UUID.fromString(INGESTED_ID), surface.send(client, anEvent()))
        assertEquals(2, api.received().size)

        val first = (api.received()[0].json() as Map<*, *>)["event_id"]
        assertNotNull(
          first,
          "the event travelled under no identifier of its own, so what the API ingests twice is " +
            "whatever it minted for each attempt"
        )
        assertEquals(
          first,
          (api.received()[1].json() as Map<*, *>)["event_id"],
          "a repeated attempt sent the event under another identifier, which is how one event " +
            "becomes two"
        )
      }
    }
  }

  @ParameterizedTest
  @EnumSource(Surface::class)
  fun repeatedServerErrorsStopAtTheAttemptBound(surface: Surface) {
    FakeApi().use { api ->
      client(api, options(3)).use { client ->
        api.willAnswer(serverError(), serverError(), serverError(), serverError())

        val refused =
          assertThrows(ClientException::class.java) { surface.send(client, anEvent()) }

        assertEquals(3, api.received().size)
        assertTrue(
          refused.message?.contains("gave up after 3 attempts") == true,
          refused.message
        )
      }
    }
  }

  @ParameterizedTest
  @EnumSource(Surface::class)
  fun anAnswerTheApiWouldRepeatIsNotRetried(surface: Surface) {
    FakeApi().use { api ->
      client(api, options(4)).use { client ->
        api.willAnswer(refused(400, "EventInvalidJsonPayload"), ingested(INGESTED_ID))

        assertThrows(ClientException::class.java) { surface.send(client, anEvent()) }
        assertEquals(1, api.received().size)
      }
    }
  }

  @ParameterizedTest
  @EnumSource(Surface::class)
  fun aRetryAnsweredThatTheEventWasAlreadyIngestedReportsSuccess(surface: Surface) {
    FakeApi().use { api ->
      client(api, options(4)).use { client ->
        api.willAnswer(serverError(), alreadyIngested())

        assertEquals(7, surface.send(client, anEvent()).version())
        assertEquals(2, api.received().size)
      }
    }
  }

  @ParameterizedTest
  @EnumSource(Surface::class)
  fun aFirstAttemptAnsweredThatTheEventWasAlreadyIngestedReportsTheConflict(surface: Surface) {
    FakeApi().use { api ->
      client(api, options(4)).use { client ->
        api.willAnswer(alreadyIngested())

        val refused =
          assertThrows(ClientException::class.java) { surface.send(client, anEvent()) }

        assertEquals(1, api.received().size)
        assertTrue(refused.message?.contains("EventAlreadyIngested") == true, refused.message)
      }
    }
  }

  @ParameterizedTest
  @EnumSource(Surface::class)
  fun aClientThatDoesNotRetryIssuesOneRequest(surface: Surface) {
    val once = Options.defaults().copy(retryPolicy = RetryPolicy.disabled())
    FakeApi().use { api ->
      client(api, once).use { client ->
        api.willAnswer(serverError(), ingested(INGESTED_ID))

        assertThrows(ClientException::class.java) { surface.send(client, anEvent()) }
        assertEquals(1, api.received().size)
      }
    }
  }

  @ParameterizedTest
  @EnumSource(Surface::class)
  fun aPayloadAboveTheMaximumIsRefusedBeforeAnyRequest(surface: Surface) {
    val small = options(4).copy(maxPayloadBytes = 64)
    FakeApi().use { api ->
      client(api, small).use { client ->
        api.willAnswer(ingested(INGESTED_ID))
        val oversized = Event("auth.user.create", "x".repeat(128), "text/plain")

        val refused =
          assertThrows(ClientException::class.java) { surface.send(client, oversized) }

        assertEquals(0, api.received().size)
        assertTrue(refused.message?.contains("nothing was sent") == true, refused.message)
      }
    }
  }

  @ParameterizedTest
  @EnumSource(Surface::class)
  fun aSendCarriesTheApplicationAndTheCredential(surface: Surface) {
    FakeApi().use { api ->
      client(api, options(4)).use { client ->
        api.willAnswer(ingested(INGESTED_ID))
        surface.send(client, anEvent())

        val sent = api.received()[0]

        assertEquals("Bearer token-xyz", sent.headers["authorization"])
        assertEquals("app-123", (sent.json() as Map<*, *>)["application_id"])
      }
    }
  }

  @ParameterizedTest
  @EnumSource(Surface::class)
  fun eventTypesTheApplicationAlreadyDeclaresAreNotCreated(surface: Surface) {
    FakeApi().use { api ->
      client(api, options(4)).use { client ->
        api.willAnswer(
          FakeApi.Scripted.of(200, listOf(mapOf("event_type_name" to "auth.user.create"))),
          FakeApi.Scripted.of(201, mapOf("event_type_name" to "billing.invoice.paid"))
        )

        val created =
          surface.upsertEventTypes(client, listOf("auth.user.create", "billing.invoice.paid"))

        assertEquals(listOf("billing.invoice.paid"), created)
        assertEquals(2, api.received().size)
      }
    }
  }

  @ParameterizedTest
  @EnumSource(Surface::class)
  fun anEventTypeThatDoesNotReadAsThreePartsIsRefused(surface: Surface) {
    FakeApi().use { api ->
      client(api, options(4)).use { client ->
        assertThrows(ClientException::class.java) {
          surface.upsertEventTypes(client, listOf("nope"))
        }
        assertEquals(0, api.received().size)
      }
    }
  }

  @Test
  fun theDefaultScheduleDoublesUpToItsCeiling() {
    val policy = RetryPolicy.defaults()

    assertEquals(100, policy.backoffCeilingMillis(1))
    assertEquals(200, policy.backoffCeilingMillis(2))
    assertEquals(400, policy.backoffCeilingMillis(3))
    assertEquals(2000, policy.backoffCeilingMillis(20))
  }

  @Test
  fun aDisabledPolicyWaitsForNothing() {
    assertEquals(1, RetryPolicy.disabled().attempts())
    assertEquals(
      emptyList<Long>(),
      RetryPolicy.disabled().delaysMillis(doubleArrayOf(1.0, 1.0, 1.0))
    )
  }

  @Test
  fun mintedIdentifiersCarryAMomentThatNeverGoesBack() {
    // Two identifiers minted inside one millisecond are not ordered — the tail is random — so what
    // is asserted is the millisecond prefix, plus a strictly ordered pair separated by a real wait.
    //
    // Two consecutive calls are not guaranteed to land in the same millisecond: they straddle a
    // boundary whenever the clock turns over between them, which is rare and therefore exactly the
    // kind of failure that arrives once a week in CI and is blamed on the runner. What holds of any
    // two calls is that the moment never goes back.
    val first = Uuid7.generate()
    val next = Uuid7.generate()

    assertTrue(
      prefix(next) >= prefix(first),
      "an identifier minted after another carries an earlier moment"
    )

    Thread.sleep(5)
    val later = Uuid7.generate()

    assertTrue(
      prefix(later) > prefix(first),
      "an identifier minted a moment later carries an earlier moment"
    )
    assertNotEquals(first, next, "two identifiers minted one after the other are the same identifier")
  }

  companion object {
    private const val INGESTED_ID = "01961234-5678-7abc-8def-0123456789ac"

    private fun options(maxAttempts: Int): Options = Options.defaults()
      .copy(
        retryPolicy =
        RetryPolicy(
          maxAttempts,
          Duration.ofMillis(5),
          Duration.ofMillis(5),
          Duration.ofSeconds(1)
        ),
        requestTimeout = Duration.ofSeconds(5)
      )

    private fun client(api: FakeApi, options: Options): Hook0Client =
      Hook0Client(api.baseUrl(), "app-123", "token-xyz", options)

    private fun anEvent(): Event = Event(
      "auth.user.create",
      "{\"email\": \"test@example.com\"}",
      "application/json",
      mapOf("environment" to "production")
    )

    private fun ingested(eventId: String): FakeApi.Scripted = FakeApi.Scripted.of(
      201,
      mapOf(
        "application_id" to "app-123",
        "event_id" to eventId,
        "received_at" to "2026-01-01"
      )
    )

    private fun alreadyIngested(): FakeApi.Scripted = FakeApi.Scripted.of(
      409,
      mapOf(
        "id" to "EventAlreadyIngested",
        "title" to "Event already Ingested",
        "detail" to "This event was previously ingested and recorded inside Hook0 service.",
        "status" to 409L,
        "type" to "https://documentation.hook0.com/problems"
      )
    )

    private fun serverError(): FakeApi.Scripted =
      FakeApi.Scripted.of(500, mapOf("id" to "InternalServerError", "status" to 500L))

    private fun refused(status: Int, problem: String): FakeApi.Scripted = FakeApi.Scripted.of(
      status,
      mapOf(
        "id" to problem,
        "status" to status.toLong(),
        "title" to "refused",
        "detail" to "scripted"
      )
    )

    private fun prefix(identifier: UUID): String = identifier.toString().substring(0, 13).replace("-", "")
  }
}
