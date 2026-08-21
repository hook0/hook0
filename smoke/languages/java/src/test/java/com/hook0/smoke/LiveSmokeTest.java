package com.hook0.smoke;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.hook0.client.Answer;
import com.hook0.client.ClientException;
import com.hook0.client.Event;
import com.hook0.client.Hook0Client;
import com.hook0.client.HttpTransport;
import com.hook0.client.Options;
import com.hook0.client.QueryParameter;
import com.hook0.client.Transport;
import com.hook0.client.Uuid7;
import com.hook0.client.Webhooks;
import com.hook0.client.generated.ApplicationInfo;
import com.hook0.client.generated.ApplicationPost;
import com.hook0.client.generated.ApplicationSecret;
import com.hook0.client.generated.ApplicationSecretPost;
import com.hook0.client.generated.ApplicationSecretsApi;
import com.hook0.client.generated.ApplicationsApi;
import com.hook0.client.generated.ErrorsApi;
import com.hook0.client.generated.EventPost;
import com.hook0.client.generated.EventType;
import com.hook0.client.generated.EventTypePost;
import com.hook0.client.generated.EventTypesApi;
import com.hook0.client.generated.EventWithPayload;
import com.hook0.client.generated.EventsApi;
import com.hook0.client.generated.EventsPerDayApi;
import com.hook0.client.generated.EventsPerDayEntry;
import com.hook0.client.generated.IngestedEvent;
import com.hook0.client.generated.InstanceApi;
import com.hook0.client.generated.InstanceConfig;
import com.hook0.client.generated.PayloadContentTypesApi;
import com.hook0.client.generated.Problem;
import com.hook0.client.generated.ProblemException;
import com.hook0.client.generated.QuotasApi;
import com.hook0.client.generated.QuotasResponse;
import com.hook0.client.generated.ReplayEvent;
import com.hook0.client.generated.RequestAttempt;
import com.hook0.client.generated.RequestAttemptsApi;
import com.hook0.client.generated.ResponseApi;
import com.hook0.client.generated.ServiceToken;
import com.hook0.client.generated.ServiceTokenApi;
import com.hook0.client.generated.ServiceTokenPost;
import com.hook0.client.generated.Subscription;
import com.hook0.client.generated.SubscriptionPost;
import com.hook0.client.generated.SubscriptionPostTarget;
import com.hook0.client.generated.SubscriptionsApi;
import java.io.IOException;
import java.net.URI;
import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Duration;
import java.time.OffsetDateTime;
import java.time.ZoneOffset;
import java.util.AbstractMap;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import java.util.function.Supplier;
import org.junit.jupiter.api.Test;

/**
 * The Java client against a Hook0 that is really running.
 *
 * <p>Two things happen here, and the second is the reason the first is worth having.
 *
 * <p>The control: whether an application secret the API minted is accepted, whether a second send under an identifier
 * already ingested is reported as the conflict it is, and whether a signature the output worker computed verifies.
 * Those are the three questions no loopback suite can ask itself, because a suite that signs and verifies with the
 * same artefact only proves the artefact agrees with itself.
 *
 * <p>The surface: every operation the API document declares, driven through the generated layer against the same
 * instance, and every model type it decodes out of a real answer. {@code clients/java/test} already drives all of
 * them — against an API the suite itself writes, out of the same document the client was generated from. That proves
 * the client matches the document. It cannot prove the document matches Hook0, and a field the API really answers
 * under another name passes there and fails on a consumer's first call.
 */
final class LiveSmokeTest {

  /** The conflict the API answers a duplicated ingestion with. */
  private static final String ALREADY_INGESTED = "EventAlreadyIngested";

  /**
   * What this smoke labels everything it creates with, so that the subscription it makes and the event it sends find
   * each other.
   */
  private static final String LANGUAGE = "java";

  /**
   * Where the subscription this smoke creates points. Nothing listens there, deliberately: what a delivery proves is
   * proved once, by the webhook the harness catches and every language verifies.
   */
  private static final String NOWHERE = "http://127.0.0.1:1/";

  /** What a paced instance answers. */
  private static final int TOO_MANY_REQUESTS = 429;

  /** The most times one request is sent again after that answer. */
  private static final int PACED_AGAIN = 8;

  /** The shortest this waits between two tries. */
  private static final Duration SHORTEST_PAUSE = Duration.ofMillis(200);

  /** The longest it waits, whatever the answer asked for. */
  private static final Duration LONGEST_PAUSE = Duration.ofSeconds(10);

  /**
   * The most digits a {@code Retry-After} may carry before it is read as the ceiling below rather than as a number.
   * A header is written by a server this smoke does not control, and {@link Long#parseLong} refuses what does not fit.
   */
  private static final int MOST_DIGITS = 9;

  @Test
  void theClientTalksToARealInstance() throws Exception {
    sendTwice();
    surface();

    // Last, and on purpose: it needs no instance at all, so it still answers after the flow above has deleted the
    // application it was run against.
    verify(Path.of(setting("HOOK0_DELIVERY")));
    System.out.println("the signature the instance produced verifies");
  }

  /** The same event, twice, under the identifier the API minted for the first of them. */
  private static void sendTwice() {
    String eventType = setting("HOOK0_EVENT_TYPE");

    try (Hook0Client client =
        new Hook0Client(setting("HOOK0_API_URL"), setting("HOOK0_APPLICATION_ID"), setting("HOOK0_TOKEN"))) {
      UUID sent = client.sendEvent(event(eventType, null));
      System.out.println("ingested " + sent);

      ClientException refused =
          assertThrows(
              ClientException.class,
              () -> client.sendEvent(event(eventType, sent)),
              "sending the same event twice was accepted twice");
      assertTrue(
          String.valueOf(refused.getMessage()).contains(ALREADY_INGESTED),
          () -> "the second send failed without naming " + ALREADY_INGESTED + ": " + refused.getMessage());
      System.out.println("the second send reported " + ALREADY_INGESTED);
    }
  }

  /**
   * Every operation the API document declares, driven against the instance in the order a consumer would: what it
   * needs is created, read and listed, updated, and destroyed last.
   *
   * <p>Two credentials, because the API takes two and one of them cannot do everything. An application secret is
   * scoped to the application it belongs to; what belongs to the organization — listing its applications, everything
   * about service tokens, its per-day counts — needs the organization-scoped token beside it.
   */
  private static void surface() {
    String origin = originOf(setting("HOOK0_API_URL"));
    String application = setting("HOOK0_APPLICATION_ID");
    UUID owning = UUID.fromString(application);
    UUID organization = UUID.fromString(setting("HOOK0_ORGANIZATION_ID"));
    String seeded = setting("HOOK0_SEEDED_APPLICATION_ID");
    String attempt = setting("HOOK0_REQUEST_ATTEMPT_ID");
    String response = setting("HOOK0_RESPONSE_ID");
    Map<String, String> labels = Map.of("language", LANGUAGE);

    try (Paced held = new Paced(new HttpTransport(origin, setting("HOOK0_TOKEN"), Options.defaults()));
        Paced organizationWide =
            new Paced(new HttpTransport(origin, setting("HOOK0_SERVICE_TOKEN"), Options.defaults()))) {

      ApplicationsApi applications = new ApplicationsApi(held);
      ApplicationSecretsApi secrets = new ApplicationSecretsApi(held);
      EventTypesApi eventTypes = new EventTypesApi(held);
      SubscriptionsApi subscriptions = new SubscriptionsApi(held);
      EventsApi events = new EventsApi(held);
      EventsPerDayApi eventsPerDay = new EventsPerDayApi(held);
      InstanceApi instance = new InstanceApi(held);
      QuotasApi quotas = new QuotasApi(held);
      PayloadContentTypesApi payloadContentTypes = new PayloadContentTypesApi(held);
      ErrorsApi errorCatalogue = new ErrorsApi(held);

      ApplicationsApi organizationApplications = new ApplicationsApi(organizationWide);
      EventsPerDayApi organizationEventsPerDay = new EventsPerDayApi(organizationWide);
      RequestAttemptsApi requestAttempts = new RequestAttemptsApi(organizationWide);
      ResponseApi responses = new ResponseApi(organizationWide);
      ServiceTokenApi serviceTokens = new ServiceTokenApi(organizationWide);

      // What the instance says about itself, which is what an application asks before it has anything of its own: how
      // it is configured, what it will let this account do, what a payload may be, and every problem it can report.
      InstanceConfig configured = read("instance.get", instance::get);
      decoded("InstanceConfig", configured);

      QuotasResponse allowed = read("quotas.get", quotas::get);
      decoded("QuotasResponseLimits", allowed.limits());
      decoded("QuotasResponse", allowed);

      exercised("payload_content_types.list", payloadContentTypes::list);

      List<Problem> catalogue = read("errors.list", errorCatalogue::list);
      assertFalse(catalogue.isEmpty(), "the instance published an empty catalogue of the problems it can report");
      decoded("ProblemId", catalogue.get(0).id());
      decoded("Problem", catalogue.get(0));

      // The application this smoke owns. One per language, so that the three deletions at the end of this flow are
      // real deletions rather than something eleven other smokes have to live with.
      ApplicationInfo info = read("applications.get", () -> applications.get(application));
      decoded("ApplicationInfoConsumption", info.consumption());
      decoded("ApplicationInfoQuotas", info.quotas());
      decoded("ApplicationInfoOnboardingStepsEvent", info.onboardingSteps().event());
      decoded("ApplicationInfoOnboardingStepsEventType", info.onboardingSteps().eventType());
      decoded("ApplicationInfoOnboardingStepsSubscription", info.onboardingSteps().subscription());
      decoded("ApplicationInfoOnboardingSteps", info.onboardingSteps());
      decoded("ApplicationInfo", info);

      decoded(
          "Application",
          read(
              "applications.update",
              () ->
                  applications.update(
                      application, new ApplicationPost("the application the java smoke drives", organization))));

      // The organization's, so the organization credential. Listing what an account has is the first thing a console
      // does.
      exercised("applications.list", () -> organizationApplications.list(organization.toString()));

      // This one is driven with the *application* secret on purpose, and it is the flow's one refusal. Creating an
      // application is the organization's business and an application secret is not the organization's, so the
      // instance answers a problem document and this client reads it — which is the half of the client that nothing
      // else here would exercise.
      exercised(
          "applications.create",
          () ->
              applications.create(
                  new ApplicationPost(
                      "an application the java smoke's application secret may not create", organization)));

      // A second secret, so that the one this smoke is authenticating with is never the one it revokes. Deleting that
      // one succeeds and then locks the flow out of everything below.
      ApplicationSecret minted =
          read(
              "applicationSecrets.create",
              () -> secrets.create(new ApplicationSecretPost(owning, "a secret the java smoke minted")));
      decoded("ApplicationSecret", minted);
      String mintedToken = minted.token().toString();

      exercised("applicationSecrets.read", () -> secrets.read(application));
      exercised(
          "applicationSecrets.update",
          () -> secrets.update(mintedToken, new ApplicationSecretPost(owning, "a secret the java smoke renamed")));
      exercised("applicationSecrets.delete", () -> secrets.delete(mintedToken, application));

      // An event type of this smoke's own, rather than the one the harness declared: what is created here is what is
      // subscribed to, sent, replayed and deleted below.
      EventType declared =
          read("eventTypes.create", () -> eventTypes.create(new EventTypePost(owning, "smoke", LANGUAGE, "ran")));
      decoded("EventType", declared);

      exercised("eventTypes.get", () -> eventTypes.get(declared.eventTypeName(), application));
      exercised("eventTypes.list", () -> eventTypes.list(application));

      SubscriptionPostTarget target = new SubscriptionPostTarget(Map.of(), "POST", "http", NOWHERE);
      Subscription subscription =
          read(
              "subscriptions.create",
              () ->
                  subscriptions.create(
                      new SubscriptionPost(
                          owning,
                          List.of(declared.eventTypeName()),
                          true,
                          target,
                          null,
                          "what the java smoke subscribes to its own events with",
                          null,
                          null,
                          labels,
                          null)));
      decoded("SubscriptionTarget", subscription.target());
      decoded("Subscription", subscription);
      String subscribed = subscription.subscriptionId().toString();

      exercised("subscriptions.get", () -> subscriptions.get(subscribed));
      exercised("subscriptions.list", () -> subscriptions.list(application));
      exercised(
          "subscriptions.update",
          () ->
              subscriptions.update(
                  subscribed,
                  new SubscriptionPost(
                      owning,
                      List.of(declared.eventTypeName()),
                      true,
                      target,
                      null,
                      "what the java smoke renamed it to",
                      null,
                      null,
                      labels,
                      null)));

      // The event the subscription above selects, sent through the generated layer rather than through sendEvent: the
      // hand-written half has its own three questions above, and this is the operation the document declares.
      IngestedEvent ingested =
          read(
              "events.ingest",
              () ->
                  events.ingest(
                      new EventPost(
                          owning,
                          declared.eventTypeName(),
                          labels,
                          OffsetDateTime.now(ZoneOffset.UTC),
                          "{\"from\":\"the java smoke\"}",
                          "application/json",
                          Uuid7.generate(),
                          null)));
      decoded("IngestedEvent", ingested);
      String sent = ingested.eventId().toString();

      EventWithPayload whole = read("events.get", () -> events.get(sent, application));
      decoded("EventWithPayload", whole);

      List<com.hook0.client.generated.Event> listed = read("events.list", () -> events.list(application));
      assertFalse(listed.isEmpty(), "the instance ingested an event and then listed none");
      decoded("Event", listed.get(0));

      exercised("events.replay", () -> events.replay(sent, new ReplayEvent(owning)));

      // This application was created a moment ago and the counts come out of a view the instance refreshes on a cycle
      // of its own, so this answers a list with nothing in it — which is an answer, and one a client has to be able to
      // read.
      exercised(
          "events_per_day.list_for_application", () -> eventsPerDay.listForApplication(application, null, null));

      // The organization's counts do have something in them: the harness waited for the instance to refresh them
      // before running any of this, precisely so that the type they are answered with is one a client decodes rather
      // than one nothing ever produces.
      List<EventsPerDayEntry> perDay =
          read(
              "events_per_day.list_for_organization",
              () -> organizationEventsPerDay.listForOrganization(organization.toString(), null, null));
      assertFalse(perDay.isEmpty(), "the organization has ingested events and its per-day counts are empty");
      decoded("EventsPerDayEntry", perDay.get(0));

      // An attempt and a response exist only once the output worker has finished a delivery. The harness waited for
      // one, in the application it caught the shared delivery from, and handed the ids on — so this reads them back
      // with the organization credential rather than waiting again.
      exercised(
          "requestAttempts.read", () -> requestAttempts.read(seeded, null, null, null, null, null, null));

      RequestAttempt attempted = read("requestAttempts.get", () -> requestAttempts.get(attempt, seeded));
      decoded("RequestAttemptEvent", attempted.event());
      decoded("RequestAttemptSubscription", attempted.subscription());
      decoded("RequestAttemptStatusType", attempted.status().type());
      decoded("RequestAttemptStatus", attempted.status());
      decoded("RequestAttempt", attempted);

      decoded("Response", read("response.get", () -> responses.get(response, seeded)));

      // Service tokens belong to the organization, so they are minted, read and revoked with the organization
      // credential. The one revoked below is the one minted here — never the one this half of the flow is
      // authenticating with.
      ServiceToken issued =
          read(
              "serviceToken.create",
              () -> serviceTokens.create(new ServiceTokenPost("a token the java smoke minted", organization)));
      decoded("ServiceToken", issued);
      String issuedId = issued.tokenId().toString();

      exercised("serviceToken.list", () -> serviceTokens.list(organization.toString()));
      exercised("serviceToken.get", () -> serviceTokens.get(issuedId, organization.toString()));
      exercised(
          "serviceToken.edit",
          () ->
              serviceTokens.edit(
                  issuedId, new ServiceTokenPost("a token the java smoke renamed", organization)));
      exercised("serviceToken.delete", () -> serviceTokens.delete(issuedId, organization.toString()));

      // Destroyed in the order the instance can accept: the subscription that references the event type, then the
      // event type, then the application — which is last because the secret this whole flow authenticates with stops
      // authenticating the moment its application is gone.
      exercised("subscriptions.delete", () -> subscriptions.delete(subscribed, application));
      exercised("eventTypes.delete", () -> eventTypes.delete(declared.eventTypeName(), application));
      exercised("applications.delete", () -> applications.delete(application));
    }
  }

  /** Reports one operation the flow goes on to use the answer of, which has to be a success. */
  private static <T> T read(String operation, Supplier<T> asking) {
    T answered;
    try {
      answered = asking.get();
    } catch (RuntimeException failed) {
      throw new AssertionError(operation + ": the flow needs what it answers, and it answered " + failed, failed);
    }
    System.out.println("exercised " + operation + " accepted");
    return answered;
  }

  /**
   * Reports one operation driven for its own sake, whichever way the instance answered it.
   *
   * <p>A success and a problem are both complete round trips through the generated layer: the request was composed,
   * the instance answered, and this client read the answer. What is neither — the API not reached, a body this client
   * cannot read, a problem it does not know — stops the smoke, because none of those say the client and the instance
   * agree on anything.
   */
  private static void exercised(String operation, Runnable asking) {
    try {
      asking.run();
    } catch (ProblemException refused) {
      Problem problem = refused.problem();
      if (problem == null) {
        throw new AssertionError(
            operation + ": what came back names no problem this client knows: " + refused, refused);
      }
      System.out.println("exercised " + operation + " refused:" + problem.id().wireValue());
      return;
    } catch (RuntimeException failed) {
      throw new AssertionError(operation + ": " + failed, failed);
    }
    System.out.println("exercised " + operation + " accepted");
  }

  /**
   * Reports one generated model type as decoded out of a real answer.
   *
   * <p>The value is taken rather than only named, so the line cannot outlive what it is about: a field that stops
   * being part of an answer stops compiling here.
   */
  private static <T> void decoded(String model, T value) {
    System.out.println("decoded " + model);
  }

  /**
   * The instance without the path the hand-written half is built with.
   *
   * <p>The generated half composes paths that already carry {@code /api/v1}, since the API document's own server URL
   * is the bare origin. Handing this transport the whole of {@code HOOK0_API_URL} happens to reach the same request:
   * {@link URI#resolve} lets an absolute path replace the base's, as RFC 3986 says, and what the base carried is
   * discarded whichever of the two it was given. That is how one language joins two URLs rather than a contract — the
   * TypeScript client resolves with {@code new URL} and was posting to {@code /api/event} until the first live run
   * found it — so this points at the origin, which is what the contract says.
   */
  private static String originOf(String apiUrl) {
    URI parsed = URI.create(apiUrl);
    return parsed.getScheme() + "://" + parsed.getAuthority();
  }

  /** The event both sends carry, under the identifier the caller names. */
  private static Event event(String eventType, UUID eventId) {
    Event built =
        Event.of(eventType, "{\"from\":\"the java smoke\"}", "application/json", Map.of("language", LANGUAGE));
    return eventId == null ? built : built.withEventId(eventId);
  }

  /** Verifies what the output worker really delivered, with this client's own verification. */
  private static void verify(Path delivery) throws IOException {
    List<Map.Entry<String, String>> headers = new ArrayList<>();
    for (String line : Files.readString(delivery.resolve("headers")).split("\n")) {
      int at = line.indexOf(": ");
      if (at > 0) {
        headers.add(new AbstractMap.SimpleEntry<>(line.substring(0, at), line.substring(at + 2)));
      }
    }

    Webhooks.verify(
        Files.readString(delivery.resolve("signature")).trim(),
        Files.readString(delivery.resolve("body")),
        headers,
        Files.readString(delivery.resolve("secret")).trim(),
        Duration.ofSeconds(Long.parseLong(Files.readString(delivery.resolve("tolerance")).trim())));
  }

  /**
   * A setting the harness passes, or a refusal naming it: a smoke that ran without one would report a failure of the
   * client for something the harness never handed it.
   */
  private static String setting(String name) {
    String value = System.getenv(name);
    if (value == null || value.isEmpty()) {
      throw new IllegalStateException(name + " is not set");
    }
    return value;
  }

  /**
   * What every generated method is issued through, waiting out a paced instance.
   *
   * <p>Hook0 paces callers per credential, and a flow driving three dozen operations one after another is exactly what
   * that is for. The answer says the request was not processed and is safe to send again after the delay it names, so
   * this waits and sends it again rather than handing the caller a problem that says nothing about the operation it
   * was asking about.
   *
   * <p>It wraps the transport the client ships rather than replacing it, and it wraps it at the one place both
   * generated surfaces pass through: an {@link Answer} carries the headers beside the body, which is where the delay
   * is written.
   */
  private static final class Paced implements Transport, AutoCloseable {

    private final HttpTransport inner;

    Paced(HttpTransport inner) {
      this.inner = inner;
    }

    @Override
    public Answer request(String method, String path, List<QueryParameter> query, Object body) {
      for (int sent = 1; ; sent++) {
        Answer answered = inner.request(method, path, query, body);
        if (answered.status() != TOO_MANY_REQUESTS || sent > PACED_AGAIN) {
          return answered;
        }
        waited(pause(answered));
      }
    }

    @Override
    public CompletableFuture<Answer> requestAsync(
        String method, String path, List<QueryParameter> query, Object body) {
      // The same policy, since a transport that paced only the half a caller happened to use would be a trap for
      // whoever reads this as an example. This smoke drives the blocking half.
      return CompletableFuture.supplyAsync(() -> request(method, path, query, body));
    }

    @Override
    public void close() {
      inner.close();
    }
  }

  /**
   * How long the answer says to wait, held between a floor and a ceiling of this smoke's own.
   *
   * <p>The floor is there because the header counts in whole seconds and the delay being waited out is a fraction of
   * one, so a truthful {@code Retry-After: 0} would otherwise mean sending the same request again immediately,
   * forever. The ceiling is there because a header is written by a server this smoke does not control.
   */
  private static Duration pause(Answer answered) {
    Duration asked =
        Optional.ofNullable(answered.header("Retry-After"))
            .map(String::trim)
            .filter(written -> !written.isEmpty() && written.length() <= MOST_DIGITS)
            .filter(written -> written.chars().allMatch(Character::isDigit))
            .map(written -> Duration.ofSeconds(Long.parseLong(written)))
            .orElse(SHORTEST_PAUSE);

    Duration held = asked.compareTo(SHORTEST_PAUSE) < 0 ? SHORTEST_PAUSE : asked;
    return held.compareTo(LONGEST_PAUSE) > 0 ? LONGEST_PAUSE : held;
  }

  /** Waits that long, and reports an interruption as what it is rather than sending the request again at once. */
  private static void waited(Duration pause) {
    try {
      Thread.sleep(pause.toMillis());
    } catch (InterruptedException interrupted) {
      Thread.currentThread().interrupt();
      throw new IllegalStateException("the wait between two tries was interrupted", interrupted);
    }
  }
}
