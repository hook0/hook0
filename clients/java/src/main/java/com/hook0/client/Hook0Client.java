package com.hook0.client;

import java.time.Duration;
import java.time.OffsetDateTime;
import java.time.ZoneOffset;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CompletionException;
import java.util.concurrent.ThreadLocalRandom;
import java.util.concurrent.TimeUnit;

/**
 * The Hook0 client, built once and shared wherever an application sends events.
 *
 * <p>Every event is sent under an identifier this client knows: the one set on the {@link Event}, or a UUIDv7 it mints
 * when the event carries none. Passing none does not mean the identifier comes from Hook0 — the value comes from here,
 * travels with the request, and is what a send answers.
 *
 * <p>That is what makes retrying safe. Hook0 keys events on that identifier, so a request repeated after a network
 * failure or a server error ingests the event once rather than twice. It also gives the answer to a retry its meaning:
 * {@code EventAlreadyIngested} in reply to a <em>repeated</em> request says an earlier attempt of that same send
 * reached the API, so the send succeeded. The same answer to a <em>first</em> attempt is a genuine conflict and is
 * reported as one.
 *
 * <p>Only what could end differently is retried: a request that got no answer, a server error, and an instance saying
 * it is being reached faster than it accepts. What the API refuses outright — a quota that is spent, a payload it will
 * not read — is reported as is, since repeating it would only spend the same round trip again. The verdict for every
 * problem the API can report is written down in the conformance corpus committed beside this artefact, which the suite
 * here reads.
 *
 * <p>Both surfaces are here: {@link #sendEvent(Event)} blocks, {@link #sendEventAsync(Event)} hands back a future, and
 * the two differ in when they wait and in nothing else. What an attempt meant, whether to repeat it and how long to
 * wait first are decided by one method that both loops call, so the two cannot come apart.
 */
public final class Hook0Client implements AutoCloseable {

  /** The identifier Hook0 gives the problem it answers when an event identifier is already taken. */
  public static final String ALREADY_INGESTED = "EventAlreadyIngested";

  /**
   * The identifier Hook0 gives the problem it answers when requests are reaching the instance faster than it accepts
   * them.
   *
   * <p>It shares its status with the quota problems, and is the only one of them worth repeating: a quota clears when a
   * plan changes or a day turns, neither of which happens inside the seconds a send is given, while pacing clears on
   * its own and the answer says when.
   */
  public static final String RATE_LIMITED = "RateLimited";

  /** What Hook0 answers when the event identifier a request carries is already taken. */
  public static final int CONFLICT = 409;

  /** What Hook0 answers both when a quota is spent and when requests come in faster than the instance accepts. */
  public static final int PACED = 429;

  /** First status saying the failure is on Hook0's side, and so could clear on its own. */
  public static final int LOWEST_SERVER_ERROR = 500;

  /** What the API names the delay before the request becomes servable in, in whole seconds. */
  public static final String DELAY_HEADER = "retry-after";

  /** Longest value of that header read, and the largest delay it may name. */
  private static final int MAX_DELAY_HEADER_CHARS = 32;

  private static final long MAX_NAMED_DELAY_SECONDS = Integer.MAX_VALUE;

  /** Where an event is ingested, under the API URL. */
  private static final String EVENT_PATH = "event";

  /** Where event types are read and created, under the API URL. */
  private static final String EVENT_TYPES_PATH = "event_types";

  private final String applicationId;
  private final Options options;
  private final Transport transport;
  private final AutoCloseable owned;

  /**
   * Builds a client reaching that API over HTTP, under the bounds given.
   *
   * @param apiUrl base API URL of a Hook0 instance, such as {@code https://app.hook0.com/api/v1}
   * @param applicationId identifier of the Hook0 application events are sent to
   * @param token an authentication token valid for that application
   * @param options the bounds one send is held to
   */
  public Hook0Client(String apiUrl, String applicationId, String token, Options options) {
    this.applicationId = applicationId;
    this.options = options;
    HttpTransport http = new HttpTransport(apiUrl, token, options);
    this.transport = http;
    this.owned = http;
  }

  /**
   * Builds a client reaching that API over HTTP, under the bounds a client applies when none are named.
   *
   * @param apiUrl base API URL of a Hook0 instance
   * @param applicationId identifier of the Hook0 application events are sent to
   * @param token an authentication token valid for that application
   */
  public Hook0Client(String apiUrl, String applicationId, String token) {
    this(apiUrl, applicationId, token, Options.defaults());
  }

  /**
   * Builds a client on a transport the caller owns, which is what lets a suite drive one over anything.
   *
   * @param transport what one request is issued through
   * @param applicationId identifier of the Hook0 application events are sent to
   * @param options the bounds one send is held to
   */
  public Hook0Client(Transport transport, String applicationId, Options options) {
    this.applicationId = applicationId;
    this.options = options;
    this.transport = transport;
    this.owned = null;
  }

  /**
   * What this client issues its requests through, which is also what a generated operation group is built on.
   *
   * @return the transport
   */
  public Transport transport() {
    return transport;
  }

  /**
   * The bounds one send is held to.
   *
   * @return the bounds
   */
  public Options options() {
    return options;
  }

  @Override
  public void close() {
    if (owned != null) {
      try {
        owned.close();
      } catch (Exception ignored) {
        // Closing a transport this client built is best effort: there is nothing a caller could do
        // about a socket that would not shut down, and nothing left to send over it.
      }
    }
  }

  /**
   * Sends an event, and answers the identifier it was sent under.
   *
   * @param event what to send
   * @return the identifier the event was sent under
   * @throws Hook0Exception when the event was not ingested
   */
  public UUID sendEvent(Event event) {
    Send send = prepare(event);

    int issued = 0;
    long waitedMillis = 0;
    while (true) {
      issued++;
      Attempt outcome = attempt();
      try {
        outcome = read(transport.request("POST", EVENT_PATH, List.of(), send.body()));
      } catch (TransportException failure) {
        outcome = failed(failure);
      }

      Decision decision = decide(send, outcome, issued, waitedMillis);
      if (decision.settled()) {
        return decision.answered();
      }
      if (decision.failure() != null) {
        throw decision.failure();
      }
      sleep(decision.waitMillis());
      waitedMillis += decision.waitMillis();
    }
  }

  /**
   * Sends an event, and hands back the identifier it will have been sent under.
   *
   * @param event what to send
   * @return the identifier the event was sent under, once it has been
   */
  public CompletableFuture<UUID> sendEventAsync(Event event) {
    Send send;
    try {
      send = prepare(event);
    } catch (Hook0Exception refused) {
      return CompletableFuture.failedFuture(refused);
    }
    return issue(send, 1, 0);
  }

  private CompletableFuture<UUID> issue(Send send, int issued, long waitedMillis) {
    return transport
        .requestAsync("POST", EVENT_PATH, List.of(), send.body())
        .handle((answer, failure) -> failure == null ? read(answer) : failed(failure))
        .thenCompose(
            outcome -> {
              Decision decision = decide(send, outcome, issued, waitedMillis);
              if (decision.settled()) {
                return CompletableFuture.completedFuture(decision.answered());
              }
              if (decision.failure() != null) {
                return CompletableFuture.<UUID>failedFuture(decision.failure());
              }
              return CompletableFuture.supplyAsync(
                      () -> null,
                      CompletableFuture.delayedExecutor(decision.waitMillis(), TimeUnit.MILLISECONDS))
                  .thenCompose(ignored -> issue(send, issued + 1, waitedMillis + decision.waitMillis()));
            });
  }

  /**
   * Creates the event types the application does not declare yet, and answers those.
   *
   * @param eventTypes the event types the application uses
   * @return the ones this call declared
   * @throws Hook0Exception when the list could not be read or one could not be created
   */
  public List<String> upsertEventTypes(List<String> eventTypes) {
    List<EventType> wanted = parsed(eventTypes);
    if (wanted.isEmpty()) {
      return List.of();
    }

    List<String> declared = declaredEventTypes(readEventTypes());
    List<String> created = new ArrayList<>();
    for (EventType eventType : wanted) {
      if (declared.contains(eventType.toString())) {
        continue;
      }
      Answer answered;
      try {
        answered = transport.request("POST", EVENT_TYPES_PATH, List.of(), eventTypeBody(eventType));
      } catch (TransportException failure) {
        throw ClientException.creatingEventType(eventType.toString(), failure.getMessage());
      }
      refuseCreation(eventType, answered);
      created.add(eventType.toString());
    }
    return List.copyOf(created);
  }

  /**
   * Creates the event types the application does not declare yet, and hands back the ones it created.
   *
   * @param eventTypes the event types the application uses
   * @return the ones this call declared, once it has
   */
  public CompletableFuture<List<String>> upsertEventTypesAsync(List<String> eventTypes) {
    List<EventType> wanted;
    try {
      wanted = parsed(eventTypes);
    } catch (Hook0Exception refused) {
      return CompletableFuture.failedFuture(refused);
    }
    if (wanted.isEmpty()) {
      return CompletableFuture.completedFuture(List.of());
    }

    return transport
        .requestAsync("GET", EVENT_TYPES_PATH, List.of(new QueryParameter("application_id", applicationId)), null)
        .handle(
            (answer, failure) -> {
              if (failure != null) {
                throw new CompletionException(ClientException.availableEventTypes(unwrapped(failure).getMessage()));
              }
              return declaredEventTypes(answer);
            })
        .thenCompose(declared -> createMissing(wanted, declared, 0, new ArrayList<>()));
  }

  private CompletableFuture<List<String>> createMissing(
      List<EventType> wanted, List<String> declared, int at, List<String> created) {
    if (at >= wanted.size()) {
      return CompletableFuture.completedFuture(List.copyOf(created));
    }
    EventType eventType = wanted.get(at);
    if (declared.contains(eventType.toString())) {
      return createMissing(wanted, declared, at + 1, created);
    }

    return transport
        .requestAsync("POST", EVENT_TYPES_PATH, List.of(), eventTypeBody(eventType))
        .handle(
            (answer, failure) -> {
              if (failure != null) {
                throw new CompletionException(
                    ClientException.creatingEventType(eventType.toString(), unwrapped(failure).getMessage()));
              }
              refuseCreation(eventType, answer);
              created.add(eventType.toString());
              return created;
            })
        .thenCompose(ignored -> createMissing(wanted, declared, at + 1, created));
  }

  private static List<EventType> parsed(List<String> eventTypes) {
    List<EventType> wanted = new ArrayList<>();
    if (eventTypes == null) {
      return wanted;
    }
    for (String written : eventTypes) {
      wanted.add(EventType.parse(written));
    }
    return wanted;
  }

  private Answer readEventTypes() {
    try {
      return transport.request(
          "GET", EVENT_TYPES_PATH, List.of(new QueryParameter("application_id", applicationId)), null);
    } catch (TransportException failure) {
      throw ClientException.availableEventTypes(failure.getMessage());
    }
  }

  /** The event types an application already declares, out of what the API answered. */
  private static List<String> declaredEventTypes(Answer answered) {
    if (!answered.successful()) {
      throw ClientException.availableEventTypes(answered.body());
    }
    Object document = document(answered.body());
    if (!(document instanceof List<?> entries)) {
      throw ClientException.availableEventTypes("the API did not answer a list of event types");
    }

    List<String> declared = new ArrayList<>();
    for (Object entry : entries) {
      if (entry instanceof Map<?, ?> members && members.get("event_type_name") instanceof String name) {
        declared.add(name);
      }
    }
    return List.copyOf(declared);
  }

  private Map<String, Object> eventTypeBody(EventType eventType) {
    Map<String, Object> body = new LinkedHashMap<>();
    body.put("application_id", applicationId);
    body.put("service", eventType.service());
    body.put("resource_type", eventType.resourceType());
    body.put("verb", eventType.verb());
    return body;
  }

  private static void refuseCreation(EventType eventType, Answer answered) {
    if (!answered.successful()) {
      throw ClientException.creatingEventType(eventType.toString(), answered.body());
    }
  }

  /** What one send is: the identifier it travels under, the body it sends, and the schedule it keeps. */
  private record Send(UUID eventId, Map<String, Object> body, List<Long> delays) {}

  /** What one attempt at sending an event ended with. */
  private record Attempt(
      UUID ingested, boolean alreadyIngested, String detail, boolean retryable, Long namedDelayMillis) {}

  /** What to do next: answer that identifier, give up, or wait that long and try again. */
  private record Decision(boolean settled, UUID answered, Hook0Exception failure, long waitMillis) {

    static Decision done(UUID answered) {
      return new Decision(true, answered, null, 0);
    }

    static Decision failed(Hook0Exception failure) {
      return new Decision(false, null, failure, 0);
    }

    static Decision waiting(long waitMillis) {
      return new Decision(false, null, null, waitMillis);
    }
  }

  private Send prepare(Event event) {
    UUID eventId = event.eventId() == null ? Uuid7.generate() : event.eventId();

    String payload = event.payload() == null ? "" : event.payload();
    int size = payload.getBytes(java.nio.charset.StandardCharsets.UTF_8).length;
    if (size > options.maxPayloadBytes()) {
      throw ClientException.payloadTooLarge(eventId, size, options.maxPayloadBytes());
    }

    RetryPolicy policy = options.retryPolicy();
    return new Send(eventId, fullEvent(event, eventId), policy.delaysMillis(jitterDraws(policy.attempts() - 1)));
  }

  private Map<String, Object> fullEvent(Event event, UUID eventId) {
    OffsetDateTime occurredAt =
        event.occurredAt() == null ? OffsetDateTime.now(ZoneOffset.UTC) : event.occurredAt();

    Map<String, Object> body = new LinkedHashMap<>();
    body.put("event_id", eventId.toString());
    body.put("application_id", applicationId);
    body.put("event_type", event.eventType());
    body.put("payload", event.payload());
    body.put("payload_content_type", event.payloadContentType());
    body.put("occurred_at", Wire.writeMoment(occurredAt));
    body.put("labels", event.labels() == null ? Map.of() : event.labels());
    if (event.metadata() != null) {
      body.put("metadata", event.metadata());
    }
    return body;
  }

  /** An attempt that has not been made, which is what a loop starts holding before its first request. */
  private static Attempt attempt() {
    return new Attempt(null, false, "", false, null);
  }

  /** What the API answered one attempt, and whether repeating it could end differently. */
  private Attempt read(Answer answered) {
    String body = answered.body() == null ? "" : answered.body();

    if (answered.successful()) {
      UUID ingested = ingestedId(body);
      if (ingested == null) {
        // The API accepted the event but answered something this client cannot read; repeating the
        // request would meet the same answer.
        return new Attempt(null, false, "Hook0 answered " + answered.status() + " without an event id", false, null);
      }
      return new Attempt(ingested, false, "", false, null);
    }

    String problem = problemId(body);
    if (answered.status() == CONFLICT && ALREADY_INGESTED.equals(problem)) {
      return new Attempt(null, true, body, false, null);
    }
    return new Attempt(null, false, body, retryable(answered.status(), problem), namedDelayMillis(answered));
  }

  /** What a request that never produced an answer ended with, read by its cause and not by its type. */
  private static Attempt failed(Throwable failure) {
    Throwable carried = unwrapped(failure);
    if (carried instanceof TransportException reported) {
      return new Attempt(null, false, reported.getMessage(), reported.retryable(), null);
    }
    throw new CompletionException(carried);
  }

  private static Throwable unwrapped(Throwable failure) {
    Throwable walked = failure;
    for (int depth = 0; depth < 8 && walked instanceof CompletionException && walked.getCause() != null; depth++) {
      walked = walked.getCause();
    }
    return walked;
  }

  /**
   * Whether repeating a request the API answered that way could end differently.
   *
   * <p>The status decides on its own everywhere but under the one it answers both a spent quota and a paced instance
   * with: a quota clears when a plan changes or a day turns, and neither is something a send spending seconds can wait
   * for. Only the problem the body names tells the two apart, and a body naming a problem this client has never heard
   * of falls back to what the status says.
   */
  private static boolean retryable(int status, String problem) {
    if (status == PACED) {
      return RATE_LIMITED.equals(problem);
    }
    return status >= LOWEST_SERVER_ERROR;
  }

  /**
   * What to do with what one attempt ended with, which is where both surfaces meet.
   *
   * <p>Everything a send decides is decided here: whether it is over, whether it is worth another request, and how long
   * to wait first. The blocking loop and the one that composes futures differ only in how they carry that out.
   */
  private Decision decide(Send send, Attempt outcome, int issued, long waitedMillis) {
    if (outcome.ingested() != null) {
      // What the API says it keyed the event on, which is the identifier this client sent it under
      // unless the API decided otherwise; either way it is the one a caller has to hold on to.
      return Decision.done(outcome.ingested());
    }
    if (outcome.alreadyIngested() && issued > 1) {
      return Decision.done(send.eventId());
    }
    if (outcome.alreadyIngested()) {
      return Decision.failed(ClientException.eventSending(send.eventId(), outcome.detail()));
    }

    boolean scheduled = outcome.retryable() && issued - 1 < send.delays().size();
    if (!scheduled) {
      return Decision.failed(givenUp(send.eventId(), issued, waitedMillis, outcome.detail()));
    }

    // The delay the API named is preferred over this client's own schedule when it named one, and
    // either way it is cut down to what is left of the budget every delay of one send shares: a
    // number written by the other end cannot stretch a send past what the caller allowed for it.
    long remaining = Math.max(options.retryPolicy().maxTotalDelayMillis() - waitedMillis, 0);
    long wanted =
        outcome.namedDelayMillis() == null
            ? send.delays().get(issued - 1).longValue()
            : outcome.namedDelayMillis().longValue();
    return Decision.waiting(Math.clamp(wanted, 0, remaining));
  }

  private static Hook0Exception givenUp(UUID eventId, int attempts, long waitedMillis, String detail) {
    if (attempts <= 1) {
      return ClientException.eventSending(eventId, detail);
    }
    return ClientException.retriesExhausted(eventId, attempts, waitedMillis, detail);
  }

  /**
   * The delay the API named before the request becomes servable.
   *
   * <p>Only a whole number of seconds is read. The header may also carry a date, which is a clock this client would be
   * comparing against its own, and anything else is a header nobody meant: both leave the client's own schedule in
   * place rather than being guessed at.
   */
  private static Long namedDelayMillis(Answer answered) {
    String written = answered.header(DELAY_HEADER);
    if (written == null) {
      return null;
    }
    written = written.strip();
    if (written.isEmpty() || written.length() > MAX_DELAY_HEADER_CHARS) {
      return null;
    }
    for (int index = 0; index < written.length(); index++) {
      if (written.charAt(index) < '0' || written.charAt(index) > '9') {
        return null;
      }
    }

    long seconds = Long.parseLong(written);
    if (seconds > MAX_NAMED_DELAY_SECONDS) {
      return null;
    }
    return Long.valueOf(Duration.ofSeconds(seconds).toMillis());
  }

  private static UUID ingestedId(String body) {
    if (!(document(body) instanceof Map<?, ?> members)) {
      return null;
    }
    if (!(members.get("event_id") instanceof String written)) {
      return null;
    }
    try {
      return UUID.fromString(written);
    } catch (IllegalArgumentException malformed) {
      return null;
    }
  }

  private static String problemId(String body) {
    if (!(document(body) instanceof Map<?, ?> members)) {
      return null;
    }
    if (members.get("id") instanceof String named) {
      return named;
    }
    return null;
  }

  private static Object document(String body) {
    try {
      return Json.parse(body == null ? "" : body);
    } catch (JsonException unreadable) {
      return null;
    }
  }

  /**
   * The randomness used to jitter the delays of one send.
   *
   * <p>Jitter only has to keep emitters that failed together from coming back together; it does not have to be
   * unpredictable, so the platform's own generator is enough.
   */
  private static double[] jitterDraws(int count) {
    double[] draws = new double[Math.max(count, 0)];
    for (int index = 0; index < draws.length; index++) {
      draws[index] = ThreadLocalRandom.current().nextDouble();
    }
    return draws;
  }

  private static void sleep(long millis) {
    if (millis <= 0) {
      return;
    }
    try {
      Thread.sleep(millis);
    } catch (InterruptedException interrupted) {
      Thread.currentThread().interrupt();
      throw new ClientExceptionInterrupted();
    }
  }

  /** A send a caller interrupted, which is a failure of this client rather than one of the API. */
  private static final class ClientExceptionInterrupted extends Hook0Exception {

    private static final long serialVersionUID = 1L;

    ClientExceptionInterrupted() {
      super("the send was interrupted while waiting to try again");
    }
  }
}
