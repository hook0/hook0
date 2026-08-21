package com.hook0.client;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.hook0.client.generated.ApplicationsApi;
import java.time.Duration;
import java.time.OffsetDateTime;
import java.util.List;
import java.util.Map;
import java.util.UUID;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.Timeout;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.EnumSource;

/** What a send does over a real socket, put through both surfaces. */
@Timeout(60)
final class ClientTest {

  private static final String INGESTED_ID = "01961234-5678-7abc-8def-0123456789ac";

  private static Options options(int maxAttempts) {
    return Options.defaults()
        .withRetryPolicy(
            new RetryPolicy(maxAttempts, Duration.ofMillis(5), Duration.ofMillis(5), Duration.ofSeconds(1)))
        .withRequestTimeout(Duration.ofSeconds(5));
  }

  private static Hook0Client client(FakeApi api, Options options) {
    return new Hook0Client(api.baseUrl(), "app-123", "token-xyz", options);
  }

  @ParameterizedTest
  @EnumSource(Surface.class)
  void aBudgetTooLargeToCountStillCutsDownTheDelayTheApiNames(Surface surface) {
    // The retry loop reads the budget a third time, to cut an API-named delay down to what is left
    // of it. That read is the one furthest from the schedule and the header, and a budget too large
    // to count raised there rather than anywhere a caller would look — the send died between two
    // attempts, on the answer that asked it to come back.
    Options absurd =
        options(2)
            .withRetryPolicy(
                new RetryPolicy(2, Duration.ofMillis(5), Duration.ofMillis(5), Duration.ofSeconds(Long.MAX_VALUE)));

    try (FakeApi api = new FakeApi();
        Hook0Client client = client(api, absurd)) {
      api.willAnswer(
          FakeApi.Scripted.of(
              503,
              Map.of(
                  "id", "ServiceUnavailable",
                  "status", Long.valueOf(503),
                  "title", "unavailable",
                  "detail", "come back shortly",
                  "type", "https://hook0.com/documentation/errors/ServiceUnavailable"),
              Map.of("Retry-After", "0")),
          ingested(INGESTED_ID));

      assertEquals(INGESTED_ID, surface.send(client, anEvent()).toString());
      assertEquals(2, api.received().size(), "the named delay was not honoured and retried");
    }
  }

  private static Event anEvent() {
    return Event.of(
        "auth.user.create", "{\"email\": \"test@example.com\"}", "application/json", Map.of("environment",
            "production"));
  }

  private static FakeApi.Scripted ingested(String eventId) {
    return FakeApi.Scripted.of(
        201,
        Map.of("application_id", "app-123", "event_id", eventId, "received_at", "2026-01-01"));
  }

  private static FakeApi.Scripted alreadyIngested() {
    return FakeApi.Scripted.of(
        409,
        Map.of(
            "id", "EventAlreadyIngested",
            "title", "Event already Ingested",
            "detail", "This event was previously ingested and recorded inside Hook0 service.",
            "status", Long.valueOf(409),
            "type", "https://documentation.hook0.com/problems"));
  }

  private static FakeApi.Scripted serverError() {
    return FakeApi.Scripted.of(500, Map.of("id", "InternalServerError", "status", Long.valueOf(500)));
  }

  private static FakeApi.Scripted refused(int status, String problem) {
    return FakeApi.Scripted.of(
        status,
        Map.of("id", problem, "status", Long.valueOf(status), "title", "refused", "detail", "scripted"));
  }

  @ParameterizedTest
  @EnumSource(Surface.class)
  void aSendThatSucceedsIssuesOneRequest(Surface surface) {
    try (FakeApi api = new FakeApi(); Hook0Client client = client(api, options(4))) {
      api.willAnswer(ingested(INGESTED_ID));

      assertEquals(UUID.fromString(INGESTED_ID), surface.send(client, anEvent()));
      assertEquals(1, api.received().size());
    }
  }

  @ParameterizedTest
  @EnumSource(Surface.class)
  void anEventCarryingNoIdIsSentUnderOneTheClientMinted(Surface surface) {
    try (FakeApi api = new FakeApi(); Hook0Client client = client(api, options(4))) {
      api.willAnswer(ingested(INGESTED_ID));
      surface.send(client, anEvent());

      Object sent = api.received().get(0).json();
      String carried = (String) ((Map<?, ?>) sent).get("event_id");

      assertEquals(7, UUID.fromString(carried).version());
    }
  }

  @ParameterizedTest
  @EnumSource(Surface.class)
  void anEventCarryingAnIdIsSentUnderIt(Surface surface) {
    UUID chosen = UUID.fromString("01961234-5678-7abc-8def-0123456789ab");
    try (FakeApi api = new FakeApi(); Hook0Client client = client(api, options(4))) {
      api.willAnswer(ingested(chosen.toString()));
      surface.send(client, anEvent().withEventId(chosen));

      assertEquals(chosen.toString(), ((Map<?, ?>) api.received().get(0).json()).get("event_id"));
    }
  }

  @ParameterizedTest
  @EnumSource(Surface.class)
  void anAttemptThatRanOutOfTimeIsRepeatedUnderTheSameId(Surface surface) {
    Options impatient = options(4).withRequestTimeout(Duration.ofMillis(200));
    try (FakeApi api = new FakeApi(); Hook0Client client = client(api, impatient)) {
      api.willAnswer(
          new FakeApi.Scripted(201, Json.write(Map.of("event_id", INGESTED_ID)), Duration.ofSeconds(1), Map.of()),
          ingested(INGESTED_ID));

      assertEquals(UUID.fromString(INGESTED_ID), surface.send(client, anEvent()));
      assertEquals(2, api.received().size());
      assertEquals(
          ((Map<?, ?>) api.received().get(0).json()).get("event_id"),
          ((Map<?, ?>) api.received().get(1).json()).get("event_id"),
          "a repeated attempt sent the event under another identifier, which is how one event becomes two");
    }
  }

  @ParameterizedTest
  @EnumSource(Surface.class)
  void repeatedServerErrorsStopAtTheAttemptBound(Surface surface) {
    try (FakeApi api = new FakeApi(); Hook0Client client = client(api, options(3))) {
      api.willAnswer(serverError(), serverError(), serverError(), serverError());

      ClientException refused = assertThrows(ClientException.class, () -> surface.send(client, anEvent()));

      assertEquals(3, api.received().size());
      assertTrue(refused.getMessage().contains("gave up after 3 attempts"), refused.getMessage());
    }
  }

  @ParameterizedTest
  @EnumSource(Surface.class)
  void anAnswerTheApiWouldRepeatIsNotRetried(Surface surface) {
    try (FakeApi api = new FakeApi(); Hook0Client client = client(api, options(4))) {
      api.willAnswer(refused(400, "EventInvalidJsonPayload"), ingested(INGESTED_ID));

      assertThrows(ClientException.class, () -> surface.send(client, anEvent()));
      assertEquals(1, api.received().size());
    }
  }

  @ParameterizedTest
  @EnumSource(Surface.class)
  void aRetryAnsweredThatTheEventWasAlreadyIngestedReportsSuccess(Surface surface) {
    try (FakeApi api = new FakeApi(); Hook0Client client = client(api, options(4))) {
      api.willAnswer(serverError(), alreadyIngested());

      assertEquals(7, surface.send(client, anEvent()).version());
      assertEquals(2, api.received().size());
    }
  }

  @ParameterizedTest
  @EnumSource(Surface.class)
  void aFirstAttemptAnsweredThatTheEventWasAlreadyIngestedReportsTheConflict(Surface surface) {
    try (FakeApi api = new FakeApi(); Hook0Client client = client(api, options(4))) {
      api.willAnswer(alreadyIngested());

      ClientException refused = assertThrows(ClientException.class, () -> surface.send(client, anEvent()));

      assertEquals(1, api.received().size());
      assertTrue(refused.getMessage().contains("EventAlreadyIngested"), refused.getMessage());
    }
  }

  @ParameterizedTest
  @EnumSource(Surface.class)
  void aClientThatDoesNotRetryIssuesOneRequest(Surface surface) {
    Options once = Options.defaults().withRetryPolicy(RetryPolicy.disabled());
    try (FakeApi api = new FakeApi(); Hook0Client client = client(api, once)) {
      api.willAnswer(serverError(), ingested(INGESTED_ID));

      assertThrows(ClientException.class, () -> surface.send(client, anEvent()));
      assertEquals(1, api.received().size());
    }
  }

  @ParameterizedTest
  @EnumSource(Surface.class)
  void aPayloadAboveTheMaximumIsRefusedBeforeAnyRequest(Surface surface) {
    Options small = options(4).withMaxPayloadBytes(64);
    try (FakeApi api = new FakeApi(); Hook0Client client = client(api, small)) {
      api.willAnswer(ingested(INGESTED_ID));
      Event oversized =
          Event.of("auth.user.create", "x".repeat(128), "text/plain", Map.of());

      ClientException refused = assertThrows(ClientException.class, () -> surface.send(client, oversized));

      assertEquals(0, api.received().size());
      assertTrue(refused.getMessage().contains("nothing was sent"), refused.getMessage());
    }
  }

  @ParameterizedTest
  @EnumSource(Surface.class)
  void aSendCarriesTheApplicationAndTheCredential(Surface surface) {
    try (FakeApi api = new FakeApi(); Hook0Client client = client(api, options(4))) {
      api.willAnswer(ingested(INGESTED_ID));
      surface.send(client, anEvent());

      FakeApi.Received sent = api.received().get(0);

      assertEquals("Bearer token-xyz", sent.headers().get("authorization"));
      assertEquals("app-123", ((Map<?, ?>) sent.json()).get("application_id"));
    }
  }

  @ParameterizedTest
  @EnumSource(Surface.class)
  void eventTypesTheApplicationAlreadyDeclaresAreNotCreated(Surface surface) {
    try (FakeApi api = new FakeApi(); Hook0Client client = client(api, options(4))) {
      api.willAnswer(
          FakeApi.Scripted.of(200, List.of(Map.of("event_type_name", "auth.user.create"))),
          FakeApi.Scripted.of(201, Map.of("event_type_name", "billing.invoice.paid")));

      List<String> created =
          surface.upsertEventTypes(client, List.of("auth.user.create", "billing.invoice.paid"));

      assertEquals(List.of("billing.invoice.paid"), created);
      assertEquals(2, api.received().size());
    }
  }

  @ParameterizedTest
  @EnumSource(Surface.class)
  void anEventTypeThatDoesNotReadAsThreePartsIsRefused(Surface surface) {
    try (FakeApi api = new FakeApi(); Hook0Client client = client(api, options(4))) {
      assertThrows(ClientException.class, () -> surface.upsertEventTypes(client, List.of("nope")));
      assertEquals(0, api.received().size());
    }
  }

  @Test
  void anEventTypeNobodyCouldHaveMeantIsRefusedRatherThanSpelledOut() {
    // Nothing, nothing at all, and a segment carrying a character an event type is not written with.
    assertThrows(ClientException.class, () -> EventType.parse(null));
    assertThrows(ClientException.class, () -> EventType.parse(""));
    assertThrows(ClientException.class, () -> EventType.parse("auth.user!.create"));
    assertEquals("auth.user.create", EventType.parse("auth.user.create").toString());
  }

  @ParameterizedTest
  @EnumSource(Surface.class)
  void anEventSaysWhatElseItCarriesAndWhenItHappened(Surface surface) {
    OffsetDateTime happened = OffsetDateTime.parse("2026-01-02T03:04:05Z");
    try (FakeApi api = new FakeApi(); Hook0Client client = client(api, options(4))) {
      api.willAnswer(ingested(INGESTED_ID));
      surface.send(client, anEvent().withMetadata(Map.of("tenant", "acme")).withOccurredAt(happened));

      Map<?, ?> sent = (Map<?, ?>) api.received().get(0).json();

      assertEquals(Map.of("tenant", "acme"), sent.get("metadata"));
      // The moment the caller named, rather than the one the send was made at.
      assertEquals("2026-01-02T03:04:05Z", sent.get("occurred_at"));
    }
  }

  @Test
  void aClientBuiltWithoutBoundsIsHeldToTheDefaultOnes() {
    try (FakeApi api = new FakeApi();
        Hook0Client client = new Hook0Client(api.baseUrl(), "app-123", "token-xyz")) {
      api.willAnswer(ingested(INGESTED_ID));

      assertEquals(Options.defaults(), client.options());
      assertEquals(UUID.fromString(INGESTED_ID), client.sendEvent(anEvent()));
    }
  }

  @Test
  void aGeneratedGroupIssuesItsRequestsThroughTheTransportTheClientHolds() {
    // What `transport()` is for: the same credential and the same bounds a send is held to are what
    // a generated operation group is built on, rather than a second connection to the same API.
    try (FakeApi api = new FakeApi();
        Hook0Client client = new Hook0Client(api.baseUrl(), "app-123", "token-xyz")) {
      api.willAnswer(FakeApi.Scripted.of(200, List.of()));

      assertTrue(new ApplicationsApi(client.transport()).list("an-organization").isEmpty());
      assertEquals("Bearer token-xyz", api.received().get(0).headers().get("authorization"));
    }
  }

  @Test
  void aTransportTheCallerOwnsOutlivesTheClientBuiltOnIt() {
    try (FakeApi api = new FakeApi();
        HttpTransport owned = new HttpTransport(api.baseUrl(), "token-xyz", options(4))) {
      api.willAnswer(ingested(INGESTED_ID), ingested(INGESTED_ID));

      try (Hook0Client first = new Hook0Client(owned, "app-123", options(4))) {
        assertEquals(UUID.fromString(INGESTED_ID), first.sendEvent(anEvent()));
      }

      // The client did not build the transport, so closing the client did not close it: it is the
      // caller's, and a second client built on it still reaches the API.
      try (Hook0Client second = new Hook0Client(owned, "app-123", options(4))) {
        assertEquals(UUID.fromString(INGESTED_ID), second.sendEvent(anEvent()));
      }

      assertEquals(2, api.received().size());
    }
  }

  @ParameterizedTest
  @EnumSource(Surface.class)
  void aListOfEventTypesTheApiRefusesToAnswerIsReported(Surface surface) {
    try (FakeApi api = new FakeApi(); Hook0Client client = client(api, options(4))) {
      api.willAnswer(refused(500, "InternalServerError"));

      ClientException reported =
          assertThrows(ClientException.class, () -> surface.upsertEventTypes(client, List.of("auth.user.create")));

      assertTrue(reported.getMessage().startsWith("Getting available event types failed"), reported.getMessage());
      // Nothing was created, since what is already declared was never learnt.
      assertEquals(1, api.received().size());
    }
  }

  @ParameterizedTest
  @EnumSource(Surface.class)
  void anAnswerThatIsNotAListOfEventTypesIsReported(Surface surface) {
    try (FakeApi api = new FakeApi(); Hook0Client client = client(api, options(4))) {
      api.willAnswer(FakeApi.Scripted.of(200, Map.of("event_type_name", "auth.user.create")));

      ClientException reported =
          assertThrows(ClientException.class, () -> surface.upsertEventTypes(client, List.of("auth.user.create")));

      assertTrue(reported.getMessage().contains("did not answer a list of event types"), reported.getMessage());
      assertEquals(1, api.received().size());
    }
  }

  @ParameterizedTest
  @EnumSource(Surface.class)
  void askingForNoEventTypesDeclaresNoneAndIssuesNoRequest(Surface surface) {
    try (FakeApi api = new FakeApi(); Hook0Client client = client(api, options(4))) {
      assertEquals(List.of(), surface.upsertEventTypes(client, List.of()));
      assertEquals(0, api.received().size());
    }
  }

  @ParameterizedTest
  @EnumSource(Surface.class)
  void aListOfEventTypesTheApiNeverAnswersIsReported(Surface surface) {
    // The request got no answer at all rather than one that could not be read, and a caller has to
    // hear about it as the same failure whichever way the call was made.
    Options impatient = options(4).withRequestTimeout(Duration.ofMillis(200));
    try (FakeApi api = new FakeApi(); Hook0Client client = client(api, impatient)) {
      api.willAnswer(FakeApi.Scripted.of(200, List.of(), Duration.ofSeconds(1)));

      ClientException reported =
          assertThrows(ClientException.class, () -> surface.upsertEventTypes(client, List.of("auth.user.create")));

      assertTrue(reported.getMessage().startsWith("Getting available event types failed"), reported.getMessage());
    }
  }

  @ParameterizedTest
  @EnumSource(Surface.class)
  void anEventTypeTheApiNeverAnswersAboutIsReported(Surface surface) {
    Options impatient = options(4).withRequestTimeout(Duration.ofMillis(200));
    try (FakeApi api = new FakeApi(); Hook0Client client = client(api, impatient)) {
      api.willAnswer(
          FakeApi.Scripted.of(200, List.of()),
          FakeApi.Scripted.of(201, Map.of("event_type_name", "auth.user.create"), Duration.ofSeconds(1)));

      ClientException reported =
          assertThrows(ClientException.class, () -> surface.upsertEventTypes(client, List.of("auth.user.create")));

      assertTrue(
          reported.getMessage().startsWith("Creating event type 'auth.user.create' failed"), reported.getMessage());
    }
  }

  @ParameterizedTest
  @EnumSource(Surface.class)
  void anEventTypeTheApiRefusesToCreateIsReported(Surface surface) {
    try (FakeApi api = new FakeApi(); Hook0Client client = client(api, options(4))) {
      api.willAnswer(FakeApi.Scripted.of(200, List.of()), refused(409, "EventTypeAlreadyExist"));

      ClientException reported =
          assertThrows(ClientException.class, () -> surface.upsertEventTypes(client, List.of("auth.user.create")));

      assertTrue(
          reported.getMessage().startsWith("Creating event type 'auth.user.create' failed"), reported.getMessage());
      assertEquals(2, api.received().size());
    }
  }

  @ParameterizedTest
  @EnumSource(Surface.class)
  void anAnswerTakingTheEventWithoutSayingWhichIsNotReadAsASuccess(Surface surface) {
    try (FakeApi api = new FakeApi(); Hook0Client client = client(api, options(4))) {
      api.willAnswer(FakeApi.Scripted.of(201, Map.of("application_id", "app-123")), ingested(INGESTED_ID));

      ClientException reported = assertThrows(ClientException.class, () -> surface.send(client, anEvent()));

      assertTrue(reported.getMessage().contains("without an event id"), reported.getMessage());
      // Repeating the request would meet the same answer, so the send stops rather than spending
      // every attempt it had left on it.
      assertEquals(1, api.received().size());
    }
  }

  @ParameterizedTest
  @EnumSource(Surface.class)
  void anAnswerCarryingNoDocumentAtAllIsStillReported(Surface surface) {
    Options once = options(4).withRetryPolicy(RetryPolicy.disabled());
    try (FakeApi api = new FakeApi(); Hook0Client client = client(api, once)) {
      api.willAnswer(new FakeApi.Scripted(500, "<html>a proxy answered this</html>", Duration.ZERO, Map.of()));

      ClientException reported = assertThrows(ClientException.class, () -> surface.send(client, anEvent()));

      assertTrue(reported.getMessage().contains("a proxy answered this"), reported.getMessage());
    }
  }

  @ParameterizedTest
  @EnumSource(Surface.class)
  void aDelayTheApiNamesInAnythingButSecondsLeavesTheScheduleInPlace(Surface surface) {
    // A `Retry-After` carrying a date is a clock this client would be comparing against its own; one
    // carrying nothing, or more than a delay is ever written in, is a header nobody meant. None of
    // them stops the send: it comes back on its own schedule rather than on one it read out of a
    // spelling it does not understand.
    for (String written : List.of("", "Wed, 21 Oct 2026 07:28:00 GMT", "0".repeat(64), "9999999999")) {
      try (FakeApi api = new FakeApi(); Hook0Client client = client(api, options(2))) {
        api.willAnswer(
            FakeApi.Scripted.of(
                503,
                Map.of("id", "ServiceUnavailable", "status", Long.valueOf(503)),
                Map.of("Retry-After", written)),
            ingested(INGESTED_ID));

        assertEquals(
            UUID.fromString(INGESTED_ID),
            surface.send(client, anEvent()),
            "a send stopped on a `Retry-After: " + written + "` rather than keeping its own schedule");
        assertEquals(2, api.received().size());
      }
    }
  }

  @Test
  void theDefaultScheduleDoublesUpToItsCeiling() {
    RetryPolicy policy = RetryPolicy.defaults();

    assertEquals(100, policy.backoffCeilingMillis(1));
    assertEquals(200, policy.backoffCeilingMillis(2));
    assertEquals(400, policy.backoffCeilingMillis(3));
    assertEquals(2000, policy.backoffCeilingMillis(20));
  }

  @Test
  void aDisabledPolicyWaitsForNothing() {
    assertEquals(1, RetryPolicy.disabled().attempts());
    assertEquals(List.of(), RetryPolicy.disabled().delaysMillis(new double[] {1.0, 1.0, 1.0}));
  }

  @Test
  void mintedIdentifiersCarryAMomentThatNeverGoesBack() throws InterruptedException {
    // Two identifiers minted inside one millisecond are not ordered — the tail is random — so what
    // is asserted is the millisecond prefix, plus a strictly ordered pair separated by a real wait.
    //
    // Two consecutive calls are not guaranteed to land in the same millisecond: they straddle a
    // boundary whenever the clock turns over between them, which is rare and therefore exactly the
    // kind of failure that arrives once a week in CI and is blamed on the runner. What holds of any
    // two calls is that the moment never goes back.
    UUID first = Uuid7.generate();
    UUID next = Uuid7.generate();

    assertTrue(
        prefix(next).compareTo(prefix(first)) >= 0,
        "an identifier minted after another carries an earlier moment");

    Thread.sleep(5);
    UUID later = Uuid7.generate();

    assertTrue(
        prefix(later).compareTo(prefix(first)) > 0,
        "an identifier minted a moment later carries an earlier moment");
    assertNotEquals(first, next, "two identifiers minted one after the other are the same identifier");
  }

  private static String prefix(UUID identifier) {
    return identifier.toString().substring(0, 13).replace("-", "");
  }
}
