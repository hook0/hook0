package com.hook0.smoke

import com.hook0.kotlin.Answer
import com.hook0.kotlin.ClientException
import com.hook0.kotlin.Event
import com.hook0.kotlin.Hook0Client
import com.hook0.kotlin.HttpTransport
import com.hook0.kotlin.QueryParameter
import com.hook0.kotlin.Transport
import com.hook0.kotlin.Uuid7
import com.hook0.kotlin.Webhooks
import com.hook0.kotlin.generated.ApplicationPost
import com.hook0.kotlin.generated.ApplicationSecretPost
import com.hook0.kotlin.generated.ApplicationSecretsApi
import com.hook0.kotlin.generated.ApplicationsApi
import com.hook0.kotlin.generated.ErrorsApi
import com.hook0.kotlin.generated.EventPost
import com.hook0.kotlin.generated.EventTypePost
import com.hook0.kotlin.generated.EventTypesApi
import com.hook0.kotlin.generated.EventsApi
import com.hook0.kotlin.generated.EventsPerDayApi
import com.hook0.kotlin.generated.InstanceApi
import com.hook0.kotlin.generated.PayloadContentTypesApi
import com.hook0.kotlin.generated.ProblemException
import com.hook0.kotlin.generated.QuotasApi
import com.hook0.kotlin.generated.ReplayEvent
import com.hook0.kotlin.generated.RequestAttemptsApi
import com.hook0.kotlin.generated.ResponseApi
import com.hook0.kotlin.generated.ServiceTokenApi
import com.hook0.kotlin.generated.ServiceTokenPost
import com.hook0.kotlin.generated.SubscriptionPost
import com.hook0.kotlin.generated.SubscriptionPostTarget
import com.hook0.kotlin.generated.SubscriptionsApi
import java.net.URI
import java.nio.file.Files
import java.nio.file.Path
import java.time.Duration
import java.time.OffsetDateTime
import java.time.ZoneOffset
import java.util.UUID
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test

/** The conflict the API answers a duplicated ingestion with. */
private const val ALREADY_INGESTED = "EventAlreadyIngested"

/**
 * What this smoke labels everything it creates with, so that the subscription it makes and the event
 * it sends find each other.
 */
private const val LANGUAGE = "kotlin"

/**
 * Where the subscription this smoke creates points. Nothing listens there, deliberately: what a
 * delivery proves is proved once, by the webhook the harness catches and every language verifies.
 */
private const val NOWHERE = "http://127.0.0.1:1/"

/** What a paced instance answers. */
private const val TOO_MANY_REQUESTS = 429

/** The most times one request is sent again after that answer. */
private const val PACED_AGAIN = 8

/** The shortest this waits between two tries. */
private val SHORTEST_PAUSE: Duration = Duration.ofMillis(200)

/** The longest it waits, whatever the answer asked for. */
private val LONGEST_PAUSE: Duration = Duration.ofSeconds(10)

/**
 * The most digits a `Retry-After` may carry before it is read as the ceiling above rather than as a
 * number. A header is written by a server this smoke does not control, and `toLongOrNull` answers
 * nothing for what does not fit — this only keeps the arithmetic off a thousand-digit string.
 */
private const val MOST_DIGITS = 9

/**
 * The Kotlin client against a Hook0 that is really running.
 *
 * Two things happen here, and the second is the reason the first is worth having.
 *
 * The control: whether an application secret the API minted is accepted, whether a second send under
 * an identifier already ingested is reported as the conflict it is, and whether a signature the
 * output worker computed verifies. Those are the three questions no loopback suite can ask itself,
 * because a suite that signs and verifies with the same artefact only proves the artefact agrees
 * with itself.
 *
 * The surface: every operation the API document declares, driven through the generated layer against
 * the same instance, and every model type it decodes out of a real answer. `clients/kotlin/test`
 * already drives all of them — against an API the suite itself writes, out of the same document the
 * client was generated from. That proves the client matches the document. It cannot prove the
 * document matches Hook0, and a field the API really answers under another name passes there and
 * fails on a consumer's first call.
 */
class LiveSmokeTest {

  @Test
  fun `the client talks to a real instance`() {
    sendTwice()
    surface()

    // Last, and on purpose: it needs no instance at all, so it still answers after the flow above
    // has deleted the application it was run against.
    verify(Path.of(setting("HOOK0_DELIVERY")))
    println("the signature the instance produced verifies")
  }

  /** The same event, twice, under the identifier the API minted for the first of them. */
  private fun sendTwice() {
    val eventType = setting("HOOK0_EVENT_TYPE")

    Hook0Client(setting("HOOK0_API_URL"), setting("HOOK0_APPLICATION_ID"), setting("HOOK0_TOKEN")).use { client ->
      val sent = client.sendEvent(event(eventType, null))
      println("ingested $sent")

      val refused =
        try {
          client.sendEvent(event(eventType, sent))
          throw AssertionError("sending the same event twice was accepted twice")
        } catch (reported: ClientException) {
          reported
        }
      assertTrue(
        refused.message.orEmpty().contains(ALREADY_INGESTED),
        "the second send failed without naming $ALREADY_INGESTED: ${refused.message}"
      )
      println("the second send reported $ALREADY_INGESTED")
    }
  }

  /**
   * Two credentials, because the API takes two and one of them cannot do everything. An application
   * secret is scoped to the application it belongs to; what belongs to the organization — listing
   * its applications, everything about service tokens, its per-day counts — needs the
   * organization-scoped token beside it.
   */
  private fun surface() {
    val origin = originOf(setting("HOOK0_API_URL"))
    Paced(HttpTransport(origin, setting("HOOK0_TOKEN"))).use { held ->
      Paced(HttpTransport(origin, setting("HOOK0_SERVICE_TOKEN"))).use { organizationWide ->
        drive(held, organizationWide)
      }
    }
  }

  /**
   * Every operation the API document declares, driven against the instance in the order a consumer
   * would: what it needs is created, read and listed, updated, and destroyed last.
   */
  private fun drive(held: Transport, organizationWide: Transport) {
    val application = setting("HOOK0_APPLICATION_ID")
    val owning = UUID.fromString(application)
    val organization = setting("HOOK0_ORGANIZATION_ID")
    val owner = UUID.fromString(organization)
    val seeded = setting("HOOK0_SEEDED_APPLICATION_ID")
    val labels = mapOf("language" to LANGUAGE)

    val applications = ApplicationsApi(held)
    val secrets = ApplicationSecretsApi(held)
    val eventTypes = EventTypesApi(held)
    val subscriptions = SubscriptionsApi(held)
    val events = EventsApi(held)
    val eventsPerDay = EventsPerDayApi(held)
    val instance = InstanceApi(held)
    val quotas = QuotasApi(held)
    val payloadContentTypes = PayloadContentTypesApi(held)
    val errorCatalogue = ErrorsApi(held)

    val organizationApplications = ApplicationsApi(organizationWide)
    val organizationEventsPerDay = EventsPerDayApi(organizationWide)
    val requestAttempts = RequestAttemptsApi(organizationWide)
    val responses = ResponseApi(organizationWide)
    val serviceTokens = ServiceTokenApi(organizationWide)

    // What the instance says about itself, which is what an application asks before it has anything
    // of its own: how it is configured, what it will let this account do, what a payload may be, and
    // every problem it can report.
    decoded("InstanceConfig", read("instance.get") { instance.get() })

    val allowed = read("quotas.get") { quotas.get() }
    decoded("QuotasResponseLimits", allowed.limits)
    decoded("QuotasResponse", allowed)

    exercised("payload_content_types.list") { payloadContentTypes.list() }

    val catalogue = read("errors.list") { errorCatalogue.list() }
    assertTrue(
      catalogue.isNotEmpty(),
      "the instance published an empty catalogue of the problems it can report"
    )
    decoded("ProblemId", catalogue.first().id)
    decoded("Problem", catalogue.first())

    // The application this smoke owns. One per language, so that the three deletions at the end of
    // this flow are real deletions rather than something eleven other smokes have to live with.
    val info = read("applications.get") { applications.get(application) }
    decoded("ApplicationInfoConsumption", info.consumption)
    decoded("ApplicationInfoQuotas", info.quotas)
    decoded("ApplicationInfoOnboardingStepsEvent", info.onboardingSteps.event)
    decoded("ApplicationInfoOnboardingStepsEventType", info.onboardingSteps.eventType)
    decoded("ApplicationInfoOnboardingStepsSubscription", info.onboardingSteps.subscription)
    decoded("ApplicationInfoOnboardingSteps", info.onboardingSteps)
    decoded("ApplicationInfo", info)

    decoded(
      "Application",
      read("applications.update") {
        applications.update(application, ApplicationPost("the application the kotlin smoke drives", owner))
      }
    )

    // The organization's, so the organization credential. Listing what an account has is the first
    // thing a console does.
    exercised("applications.list") { organizationApplications.list(organization) }

    // This one is driven with the *application* secret on purpose, and it is the flow's one refusal.
    // Creating an application is the organization's business and an application secret is not the
    // organization's, so the instance answers a problem document and this client reads it — which is
    // the half of the client that nothing else here would exercise.
    exercised("applications.create") {
      applications.create(
        ApplicationPost("an application the kotlin smoke's application secret may not create", owner)
      )
    }

    // A second secret, so that the one this smoke is authenticating with is never the one it
    // revokes. Deleting that one succeeds and then locks the flow out of everything below.
    val minted =
      read("applicationSecrets.create") {
        secrets.create(ApplicationSecretPost(owning, "a secret the kotlin smoke minted"))
      }
    decoded("ApplicationSecret", minted)
    val mintedToken = minted.token.toString()

    exercised("applicationSecrets.list") { secrets.list(application) }
    exercised("applicationSecrets.update") {
      secrets.update(mintedToken, ApplicationSecretPost(owning, "a secret the kotlin smoke renamed"))
    }
    exercised("applicationSecrets.delete") { secrets.delete(mintedToken, application) }

    // An event type of this smoke's own, rather than the one the harness declared: what is created
    // here is what is subscribed to, sent, replayed and deleted below.
    val declared =
      read("eventTypes.create") {
        eventTypes.create(EventTypePost(owning, resourceType = "smoke", service = LANGUAGE, verb = "ran"))
      }
    decoded("EventType", declared)

    exercised("eventTypes.get") { eventTypes.get(declared.eventTypeName, application) }
    exercised("eventTypes.list") { eventTypes.list(application) }

    val target =
      SubscriptionPostTarget(headers = emptyMap<String, String>(), method = "POST", type = "http", url = NOWHERE)
    val subscription =
      read("subscriptions.create") {
        subscriptions.create(
          SubscriptionPost(
            applicationId = owning,
            eventTypes = listOf(declared.eventTypeName),
            isEnabled = true,
            target = target,
            description = "what the kotlin smoke subscribes to its own events with",
            labels = labels
          )
        )
      }
    decoded("SubscriptionTarget", subscription.target)
    decoded("Subscription", subscription)
    val subscribed = subscription.subscriptionId.toString()

    exercised("subscriptions.get") { subscriptions.get(subscribed) }
    exercised("subscriptions.list") { subscriptions.list(application) }
    exercised("subscriptions.update") {
      subscriptions.update(
        subscribed,
        SubscriptionPost(
          applicationId = owning,
          eventTypes = listOf(declared.eventTypeName),
          isEnabled = true,
          target = target,
          description = "what the kotlin smoke renamed it to",
          labels = labels
        )
      )
    }

    // The event the subscription above selects, sent through the generated layer rather than through
    // sendEvent: the hand-written half has its own three questions above, and this is the operation
    // the document declares.
    val ingested =
      read("events.ingest") {
        events.ingest(
          EventPost(
            applicationId = owning,
            eventType = declared.eventTypeName,
            labels = labels,
            occurredAt = OffsetDateTime.now(ZoneOffset.UTC),
            payload = """{"from":"the kotlin smoke"}""",
            payloadContentType = "application/json",
            eventId = Uuid7.generate()
          )
        )
      }
    decoded("IngestedEvent", ingested)
    val sent = ingested.eventId.toString()

    decoded("EventWithPayload", read("events.get") { events.get(sent, application) })

    val listed = read("events.list") { events.list(application) }
    assertTrue(listed.isNotEmpty(), "the instance ingested an event and then listed none")
    decoded("Event", listed.first())

    exercised("events.replay") { events.replay(sent, ReplayEvent(owning)) }

    // This application was created a moment ago and the counts come out of a view the instance
    // refreshes on a cycle of its own, so this answers a list with nothing in it — which is an
    // answer, and one a client has to be able to read.
    exercised("events_per_day.list_for_application") { eventsPerDay.listForApplication(application) }

    // The organization's counts do have something in them: the harness waited for the instance to
    // refresh them before running any of this, precisely so that the type they are answered with is
    // one a client decodes rather than one nothing ever produces.
    val perDay =
      read("events_per_day.list_for_organization") { organizationEventsPerDay.listForOrganization(organization) }
    assertTrue(
      perDay.isNotEmpty(),
      "the organization has ingested events and its per-day counts are empty"
    )
    decoded("EventsPerDayEntry", perDay.first())

    // An attempt and a response exist only once the output worker has finished a delivery. The
    // harness waited for one, in the application it caught the shared delivery from, and handed the
    // ids on — so this reads them back with the organization credential rather than waiting again.
    exercised("requestAttempts.list") { requestAttempts.list(seeded) }

    val attempted = read("requestAttempts.get") { requestAttempts.get(setting("HOOK0_REQUEST_ATTEMPT_ID"), seeded) }
    decoded("RequestAttemptEvent", attempted.event)
    decoded("RequestAttemptSubscription", attempted.subscription)
    decoded("RequestAttemptStatusType", attempted.status.type)
    decoded("RequestAttemptStatus", attempted.status)
    decoded("RequestAttempt", attempted)

    decoded("Response", read("response.get") { responses.get(setting("HOOK0_RESPONSE_ID"), seeded) })

    // Service tokens belong to the organization, so they are minted, read and revoked with the
    // organization credential. The one revoked below is the one minted here — never the one this
    // half of the flow is authenticating with.
    val issued =
      read("serviceToken.create") {
        serviceTokens.create(ServiceTokenPost("a token the kotlin smoke minted", owner))
      }
    decoded("ServiceToken", issued)
    val issuedId = issued.tokenId.toString()

    exercised("serviceToken.list") { serviceTokens.list(organization) }
    exercised("serviceToken.get") { serviceTokens.get(issuedId, organization) }
    exercised("serviceToken.update") {
      serviceTokens.update(issuedId, ServiceTokenPost("a token the kotlin smoke renamed", owner))
    }
    exercised("serviceToken.delete") { serviceTokens.delete(issuedId, organization) }

    // Destroyed in the order the instance can accept: the subscription that references the event
    // type, then the event type, then the application — which is last because the secret this whole
    // flow authenticates with stops authenticating the moment its application is gone.
    exercised("subscriptions.delete") { subscriptions.delete(subscribed, application) }
    exercised("eventTypes.delete") { eventTypes.delete(declared.eventTypeName, application) }
    exercised("applications.delete") { applications.delete(application) }
  }

  /** The event both sends carry, under the identifier the caller names. */
  private fun event(eventType: String, eventId: UUID?) =
    Event(
      eventType = eventType,
      payload = """{"from":"the kotlin smoke"}""",
      payloadContentType = "application/json",
      labels = mapOf("language" to LANGUAGE),
      eventId = eventId
    )

  /** Verifies what the output worker really delivered, with this client's own verification. */
  private fun verify(delivery: Path) {
    val headers =
      Files.readString(delivery.resolve("headers"))
        .split("\n")
        .mapNotNull { line ->
          val at = line.indexOf(": ")
          if (at > 0) line.substring(0, at) to line.substring(at + 2) else null
        }

    Webhooks.verify(
      Files.readString(delivery.resolve("signature")).trim(),
      Files.readString(delivery.resolve("body")),
      headers,
      Files.readString(delivery.resolve("secret")).trim(),
      Duration.ofSeconds(Files.readString(delivery.resolve("tolerance")).trim().toLong())
    )
  }
}

/** Reports one operation the flow goes on to use the answer of, which has to be a success. */
private fun <T> read(operation: String, asking: () -> T): T {
  val answered =
    try {
      asking()
    } catch (failed: RuntimeException) {
      throw AssertionError("$operation: the flow needs what it answers, and it answered $failed", failed)
    }
  println("exercised $operation accepted")
  return answered
}

/**
 * Reports one operation driven for its own sake, whichever way the instance answered it.
 *
 * A success and a problem are both complete round trips through the generated layer: the request was
 * composed, the instance answered, and this client read the answer. What is neither — the API not
 * reached, a body this client cannot read, a problem it does not know — stops the smoke, because
 * none of those say the client and the instance agree on anything.
 */
private fun exercised(operation: String, asking: () -> Unit) {
  try {
    asking()
  } catch (refused: ProblemException) {
    val problem =
      refused.problem
        ?: throw AssertionError("$operation: what came back names no problem this client knows: $refused", refused)
    println("exercised $operation refused:${problem.id.wireValue}")
    return
  } catch (failed: RuntimeException) {
    throw AssertionError("$operation: $failed", failed)
  }
  println("exercised $operation accepted")
}

/**
 * Reports one generated model type as decoded out of a real answer.
 *
 * The value is taken rather than only named, so the line cannot outlive what it is about: a field
 * that stops being part of an answer stops compiling here.
 */
private fun <T> decoded(model: String, @Suppress("UNUSED_PARAMETER") value: T) {
  println("decoded $model")
}

/**
 * The instance without the path the hand-written half is built with.
 *
 * The generated half composes paths that already carry `/api/v1`, since the API document's own
 * server URL is the bare origin. Handing this transport the whole of `HOOK0_API_URL` happens to
 * reach the same request: `URI.resolve` lets an absolute path replace the base's, as RFC 3986 says,
 * and what the base carried is discarded whichever of the two it was given. That is how one language
 * joins two URLs rather than a contract — the TypeScript client resolves with `new URL` and was
 * posting to `/api/event` until the first live run found it — so this points at the origin, which is
 * what the contract says.
 */
private fun originOf(apiUrl: String): String = URI.create(apiUrl).let { "${it.scheme}://${it.authority}" }

/**
 * A setting the harness passes, or a refusal naming it: a smoke that ran without one would report a
 * failure of the client for something the harness never handed it.
 */
private fun setting(name: String): String = System.getenv(name).orEmpty().ifEmpty { error("$name is not set") }

/**
 * What every generated method is issued through, waiting out a paced instance.
 *
 * Hook0 paces callers per credential, and a flow driving three dozen operations one after another is
 * exactly what that is for. The answer says the request was not processed and is safe to send again
 * after the delay it names, so this waits and sends it again rather than handing the caller a
 * problem that says nothing about the operation it was asking about.
 *
 * It wraps the transport the client ships rather than replacing it, and it wraps it at the one place
 * both generated surfaces pass through: an [Answer] carries the headers beside the body, which is
 * where the delay is written.
 */
private class Paced(private val inner: HttpTransport) : Transport, AutoCloseable {

  override fun request(method: String, path: String, query: List<QueryParameter>, body: Any?): Answer {
    var sent = 1
    while (true) {
      val answered = inner.request(method, path, query, body)
      if (answered.status != TOO_MANY_REQUESTS || sent > PACED_AGAIN) {
        return answered
      }
      Thread.sleep(pause(answered).toMillis())
      sent++
    }
  }

  /**
   * The same policy, since a transport that paced only the half a caller happened to use would be a
   * trap for whoever reads this as an example. This smoke drives the blocking half; a caller with
   * `kotlinx.coroutines` on its classpath would write `delay` where this holds the thread, which is
   * the one thing an SDK depending on `kotlin-stdlib` alone cannot write for it.
   */
  override suspend fun requestSuspending(
    method: String,
    path: String,
    query: List<QueryParameter>,
    body: Any?
  ): Answer = request(method, path, query, body)

  override fun close() = inner.close()
}

/**
 * How long the answer says to wait, held between a floor and a ceiling of this smoke's own.
 *
 * The floor is there because the header counts in whole seconds and the delay being waited out is a
 * fraction of one, so a truthful `Retry-After: 0` would otherwise mean sending the same request
 * again immediately, forever. The ceiling is there because a header is written by a server this
 * smoke does not control.
 */
private fun pause(answered: Answer): Duration {
  val asked =
    answered
      .header("Retry-After")
      ?.trim()
      ?.takeIf { it.length in 1..MOST_DIGITS && it.all(Char::isDigit) }
      ?.let { Duration.ofSeconds(it.toLong()) }
      ?: SHORTEST_PAUSE

  return asked.coerceIn(SHORTEST_PAUSE, LONGEST_PAUSE)
}
