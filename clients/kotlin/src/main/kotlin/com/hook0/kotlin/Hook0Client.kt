package com.hook0.kotlin

import java.nio.charset.StandardCharsets
import java.time.Duration
import java.time.OffsetDateTime
import java.time.ZoneOffset
import java.util.UUID
import java.util.concurrent.ThreadLocalRandom
import kotlin.math.max

/**
 * The Hook0 client, built once and shared wherever an application sends events.
 *
 * Every event is sent under an identifier this client knows: the one set on the [Event], or a
 * UUIDv7 it mints when the event carries none. Passing none does not mean the identifier comes from
 * Hook0 — the value comes from here, travels with the request, and is what a send answers.
 *
 * That is what makes retrying safe. Hook0 keys events on that identifier, so a request repeated
 * after a network failure or a server error ingests the event once rather than twice. It also gives
 * the answer to a retry its meaning: `EventAlreadyIngested` in reply to a *repeated* request says an
 * earlier attempt of that same send reached the API, so the send succeeded. The same answer to a
 * *first* attempt is a genuine conflict and is reported as one.
 *
 * Only what could end differently is retried: a request that got no answer, a server error, and an
 * instance saying it is being reached faster than it accepts. What the API refuses outright — a
 * quota that is spent, a payload it will not read — is reported as is, since repeating it would only
 * spend the same round trip again. The verdict for every problem the API can report is written down
 * in the conformance corpus committed beside this artefact, which the suite here reads.
 *
 * Both surfaces are here: [sendEvent] blocks, [sendEventSuspending] suspends, and the two differ in
 * how they wait and in nothing else. What an attempt meant, whether to repeat it and how long to
 * wait first are decided by one function both loops call, so the two cannot come apart.
 *
 * @property transport what this client issues its requests through, which is also what a generated
 *     operation group is built on
 * @property options the bounds one send is held to
 */
class Hook0Client private constructor(
  val transport: Transport,
  private val applicationId: String,
  val options: Options,
  private val ownsTransport: Boolean
) : AutoCloseable {

  /**
   * Builds a client on a transport the caller owns, which is what lets a suite drive one over
   * anything.
   *
   * @param transport what one request is issued through
   * @param applicationId identifier of the Hook0 application events are sent to
   * @param options the bounds one send is held to
   */
  constructor(
    transport: Transport,
    applicationId: String,
    options: Options = Options.defaults()
  ) : this(transport, applicationId, options, false)

  /**
   * Builds a client reaching that API over HTTP, under the bounds given.
   *
   * @param apiUrl base API URL of a Hook0 instance, such as `https://app.hook0.com/api/v1`
   * @param applicationId identifier of the Hook0 application events are sent to
   * @param token an authentication token valid for that application
   * @param options the bounds one send is held to
   */
  constructor(
    apiUrl: String,
    applicationId: String,
    token: String,
    options: Options = Options.defaults()
  ) : this(HttpTransport(apiUrl, token, options), applicationId, options, true)

  override fun close() {
    if (!ownsTransport) {
      return
    }
    val closing = transport
    if (closing is AutoCloseable) {
      try {
        closing.close()
      } catch (unclosable: Exception) {
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
  fun sendEvent(event: Event): UUID {
    val send = prepare(event)

    var issued = 0
    var waitedMillis = 0L
    while (true) {
      issued++
      val outcome =
        try {
          read(transport.request("POST", EVENT_PATH, emptyList(), send.body))
        } catch (failure: TransportException) {
          failed(failure)
        }

      when (val decision = decide(send, outcome, issued, waitedMillis)) {
        is Decision.Done -> return decision.answered

        is Decision.Failed -> throw decision.failure

        is Decision.Waiting -> {
          sleep(decision.waitMillis)
          waitedMillis += decision.waitMillis
        }
      }
    }
  }

  /**
   * Sends an event, suspending until it has been, and answers the identifier it was sent under.
   *
   * @param event what to send
   * @return the identifier the event was sent under
   * @throws Hook0Exception when the event was not ingested
   */
  suspend fun sendEventSuspending(event: Event): UUID {
    val send = prepare(event)

    var issued = 0
    var waitedMillis = 0L
    while (true) {
      issued++
      val outcome =
        try {
          read(transport.requestSuspending("POST", EVENT_PATH, emptyList(), send.body))
        } catch (failure: TransportException) {
          failed(failure)
        }

      when (val decision = decide(send, outcome, issued, waitedMillis)) {
        is Decision.Done -> return decision.answered

        is Decision.Failed -> throw decision.failure

        is Decision.Waiting -> {
          Suspending.pausing(decision.waitMillis)
          waitedMillis += decision.waitMillis
        }
      }
    }
  }

  /**
   * Creates the event types the application does not declare yet, and answers those.
   *
   * @param eventTypes the event types the application uses
   * @return the ones this call declared
   * @throws Hook0Exception when the list could not be read or one could not be created
   */
  fun upsertEventTypes(eventTypes: List<String>): List<String> {
    val wanted = eventTypes.map(EventType::parse)
    if (wanted.isEmpty()) {
      return emptyList()
    }

    val listed =
      try {
        transport.request("GET", EVENT_TYPES_PATH, applicationQuery(), null)
      } catch (failure: TransportException) {
        throw ClientException.availableEventTypes(failure.message ?: "")
      }
    val declared = declaredEventTypes(listed)

    val created = ArrayList<String>()
    for (eventType in wanted) {
      if (declared.contains(eventType.toString())) {
        continue
      }
      val answered =
        try {
          transport.request("POST", EVENT_TYPES_PATH, emptyList(), eventTypeBody(eventType))
        } catch (failure: TransportException) {
          throw ClientException.creatingEventType(eventType.toString(), failure.message ?: "")
        }
      refuseCreation(eventType, answered)
      created.add(eventType.toString())
    }
    return created.toList()
  }

  /**
   * Creates the event types the application does not declare yet, suspending until it has, and
   * answers those.
   *
   * @param eventTypes the event types the application uses
   * @return the ones this call declared
   * @throws Hook0Exception when the list could not be read or one could not be created
   */
  suspend fun upsertEventTypesSuspending(eventTypes: List<String>): List<String> {
    val wanted = eventTypes.map(EventType::parse)
    if (wanted.isEmpty()) {
      return emptyList()
    }

    val listed =
      try {
        transport.requestSuspending("GET", EVENT_TYPES_PATH, applicationQuery(), null)
      } catch (failure: TransportException) {
        throw ClientException.availableEventTypes(failure.message ?: "")
      }
    val declared = declaredEventTypes(listed)

    val created = ArrayList<String>()
    for (eventType in wanted) {
      if (declared.contains(eventType.toString())) {
        continue
      }
      val answered =
        try {
          transport.requestSuspending(
            "POST",
            EVENT_TYPES_PATH,
            emptyList(),
            eventTypeBody(eventType)
          )
        } catch (failure: TransportException) {
          throw ClientException.creatingEventType(eventType.toString(), failure.message ?: "")
        }
      refuseCreation(eventType, answered)
      created.add(eventType.toString())
    }
    return created.toList()
  }

  private fun applicationQuery(): List<QueryParameter> = listOf(QueryParameter("application_id", applicationId))

  private fun eventTypeBody(eventType: EventType): Map<String, Any?> {
    val body = LinkedHashMap<String, Any?>()
    body["application_id"] = applicationId
    body["service"] = eventType.service
    body["resource_type"] = eventType.resourceType
    body["verb"] = eventType.verb
    return body
  }

  private fun prepare(event: Event): Send {
    val eventId = event.eventId ?: Uuid7.generate()

    val size = event.payload.toByteArray(StandardCharsets.UTF_8).size
    if (size > options.maxPayloadBytes) {
      throw ClientException.payloadTooLarge(eventId, size, options.maxPayloadBytes)
    }

    val policy = options.retryPolicy
    return Send(eventId, fullEvent(event, eventId), policy.delaysMillis(jitterDraws(policy.attempts() - 1)))
  }

  private fun fullEvent(event: Event, eventId: UUID): Map<String, Any?> {
    val occurredAt = event.occurredAt ?: OffsetDateTime.now(ZoneOffset.UTC)

    val body = LinkedHashMap<String, Any?>()
    body["event_id"] = eventId.toString()
    body["application_id"] = applicationId
    body["event_type"] = event.eventType
    body["payload"] = event.payload
    body["payload_content_type"] = event.payloadContentType
    body["occurred_at"] = Wire.writeMoment(occurredAt)
    body["labels"] = event.labels
    val metadata = event.metadata
    if (metadata != null) {
      body["metadata"] = metadata
    }
    return body
  }

  /**
   * What to do with what one attempt ended with, which is where both surfaces meet.
   *
   * Everything a send decides is decided here: whether it is over, whether it is worth another
   * request, and how long to wait first. The blocking loop and the suspending one differ only in how
   * they carry that out.
   */
  private fun decide(send: Send, outcome: Attempt, issued: Int, waitedMillis: Long): Decision {
    val ingested = outcome.ingested
    if (ingested != null) {
      // What the API says it keyed the event on, which is the identifier this client sent it under
      // unless the API decided otherwise; either way it is the one a caller has to hold on to.
      return Decision.Done(ingested)
    }
    if (outcome.alreadyIngested && issued > 1) {
      return Decision.Done(send.eventId)
    }
    if (outcome.alreadyIngested) {
      return Decision.Failed(ClientException.eventSending(send.eventId, outcome.detail))
    }

    val scheduled = outcome.retryable && issued - 1 < send.delays.size
    if (!scheduled) {
      return Decision.Failed(givenUp(send.eventId, issued, waitedMillis, outcome.detail))
    }

    // The delay the API named is preferred over this client's own schedule when it named one, and
    // either way it is cut down to what is left of the budget every delay of one send shares: a
    // number written by the other end cannot stretch a send past what the caller allowed for it.
    val remaining = max(options.retryPolicy.maxTotalDelay.toMillis() - waitedMillis, 0)
    val wanted = outcome.namedDelayMillis ?: send.delays[issued - 1]
    return Decision.Waiting(wanted.coerceIn(0, remaining))
  }

  /** What the API answered one attempt, and whether repeating it could end differently. */
  private fun read(answered: Answer): Attempt {
    val body = answered.body

    if (answered.successful()) {
      val ingested = ingestedId(body)
      if (ingested == null) {
        // The API accepted the event but answered something this client cannot read; repeating the
        // request would meet the same answer.
        return Attempt(
          null,
          false,
          "Hook0 answered ${answered.status} without an event id",
          false,
          null
        )
      }
      return Attempt(ingested, false, "", false, null)
    }

    val problem = problemId(body)
    if (answered.status == CONFLICT && ALREADY_INGESTED == problem) {
      return Attempt(null, true, body, false, null)
    }
    return Attempt(
      null,
      false,
      body,
      retryable(answered.status, problem),
      namedDelayMillis(answered)
    )
  }

  companion object {
    /** The identifier Hook0 gives the problem it answers when an event identifier is already taken. */
    const val ALREADY_INGESTED = "EventAlreadyIngested"

    /**
     * The identifier Hook0 gives the problem it answers when requests are reaching the instance
     * faster than it accepts them.
     *
     * It shares its status with the quota problems, and is the only one of them worth repeating: a
     * quota clears when a plan changes or a day turns, neither of which happens inside the seconds a
     * send is given, while pacing clears on its own and the answer says when.
     */
    const val RATE_LIMITED = "RateLimited"

    /** What Hook0 answers when the event identifier a request carries is already taken. */
    const val CONFLICT = 409

    /**
     * What Hook0 answers both when a quota is spent and when requests come in faster than the
     * instance accepts.
     */
    const val PACED = 429

    /** First status saying the failure is on Hook0's side, and so could clear on its own. */
    const val LOWEST_SERVER_ERROR = 500

    /** What the API names the delay before the request becomes servable in, in whole seconds. */
    const val DELAY_HEADER = "retry-after"

    /** Longest value of that header read, and the largest delay it may name. */
    private const val MAX_DELAY_HEADER_CHARS = 32

    private const val MAX_NAMED_DELAY_SECONDS = Int.MAX_VALUE.toLong()

    /** Where an event is ingested, under the API URL. */
    private const val EVENT_PATH = "event"

    /** Where event types are read and created, under the API URL. */
    private const val EVENT_TYPES_PATH = "event_types"

    /** What a request that never produced an answer ended with, read by its cause and not its type. */
    private fun failed(failure: TransportException): Attempt =
      Attempt(null, false, failure.message ?: "", failure.retryable, null)

    /**
     * Whether repeating a request the API answered that way could end differently.
     *
     * The status decides on its own everywhere but under the one it answers both a spent quota and a
     * paced instance with: a quota clears when a plan changes or a day turns, and neither is
     * something a send spending seconds can wait for. Only the problem the body names tells the two
     * apart, and a body naming a problem this client has never heard of falls back to what the
     * status says.
     */
    private fun retryable(status: Int, problem: String?): Boolean {
      if (status == PACED) {
        return RATE_LIMITED == problem
      }
      return status >= LOWEST_SERVER_ERROR
    }

    private fun givenUp(eventId: UUID, attempts: Int, waitedMillis: Long, detail: String): Hook0Exception {
      if (attempts <= 1) {
        return ClientException.eventSending(eventId, detail)
      }
      return ClientException.retriesExhausted(eventId, attempts, waitedMillis, detail)
    }

    /**
     * The delay the API named before the request becomes servable.
     *
     * Only a whole number of seconds is read. The header may also carry a date, which is a clock
     * this client would be comparing against its own, and anything else is a header nobody meant:
     * both leave the client's own schedule in place rather than being guessed at.
     */
    private fun namedDelayMillis(answered: Answer): Long? {
      val written = answered.header(DELAY_HEADER)?.trim() ?: return null
      if (written.isEmpty() || written.length > MAX_DELAY_HEADER_CHARS) {
        return null
      }
      for (character in written) {
        if (character < '0' || character > '9') {
          return null
        }
      }

      val seconds = written.toLongOrNull() ?: return null
      if (seconds > MAX_NAMED_DELAY_SECONDS) {
        return null
      }
      return Duration.ofSeconds(seconds).toMillis()
    }

    /** The event types an application already declares, out of what the API answered. */
    private fun declaredEventTypes(answered: Answer): List<String> {
      if (!answered.successful()) {
        throw ClientException.availableEventTypes(answered.body)
      }
      val document = document(answered.body)
      if (document !is List<*>) {
        throw ClientException.availableEventTypes("the API did not answer a list of event types")
      }

      val declared = ArrayList<String>()
      for (entry in document) {
        if (entry is Map<*, *>) {
          val name = entry["event_type_name"]
          if (name is String) {
            declared.add(name)
          }
        }
      }
      return declared.toList()
    }

    private fun refuseCreation(eventType: EventType, answered: Answer) {
      if (!answered.successful()) {
        throw ClientException.creatingEventType(eventType.toString(), answered.body)
      }
    }

    private fun ingestedId(body: String): UUID? {
      val members = document(body) as? Map<*, *> ?: return null
      val written = members["event_id"] as? String ?: return null
      return try {
        UUID.fromString(written)
      } catch (malformed: IllegalArgumentException) {
        null
      }
    }

    private fun problemId(body: String): String? {
      val members = document(body) as? Map<*, *> ?: return null
      return members["id"] as? String
    }

    private fun document(body: String): Any? = try {
      Json.parse(body)
    } catch (unreadable: JsonException) {
      null
    }

    /**
     * The randomness used to jitter the delays of one send.
     *
     * Jitter only has to keep emitters that failed together from coming back together; it does not
     * have to be unpredictable, so the platform's own generator is enough.
     */
    private fun jitterDraws(count: Int): DoubleArray =
      DoubleArray(max(count, 0)) { ThreadLocalRandom.current().nextDouble() }

    private fun sleep(millis: Long) {
      if (millis <= 0) {
        return
      }
      try {
        Thread.sleep(millis)
      } catch (interrupted: InterruptedException) {
        Thread.currentThread().interrupt()
        throw ClientException.refusedDelivery(
          "the send was interrupted while waiting to try again: ${interrupted.message}"
        )
      }
    }
  }

  /** What one send is: the identifier it travels under, the body it sends, and the schedule it keeps. */
  private data class Send(val eventId: UUID, val body: Map<String, Any?>, val delays: List<Long>)

  /** What one attempt at sending an event ended with. */
  private data class Attempt(
    val ingested: UUID?,
    val alreadyIngested: Boolean,
    val detail: String,
    val retryable: Boolean,
    val namedDelayMillis: Long?
  )

  /** What to do next: answer that identifier, give up, or wait that long and try again. */
  private sealed interface Decision {
    data class Done(val answered: UUID) : Decision

    data class Failed(val failure: Hook0Exception) : Decision

    data class Waiting(val waitMillis: Long) : Decision
  }
}
