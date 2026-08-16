package com.hook0.client;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.junit.jupiter.api.Assertions.fail;

import java.io.IOException;
import java.io.UncheckedIOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Duration;
import java.time.Instant;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.TreeSet;
import java.util.regex.Matcher;
import java.util.regex.Pattern;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.Timeout;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.EnumSource;

/**
 * The cases the shared conformance corpus dictates, run against this client through both surfaces.
 *
 * <p>Nothing below writes down a verdict, a bound, a header or a signature of its own. Everything is read out of the
 * committed documents at {@code clients/conformance} and this client is driven against them over a real socket, so a
 * case added there is exercised here without this file being touched.
 */
@Timeout(180)
final class ConformanceTest {

  private static final Map<String, Object> RETRY = Corpus.document("retry.json");
  private static final Map<String, Object> REQUEST = Corpus.document("request.json");
  private static final Map<String, Object> SIGNATURE = Corpus.document("signature.json");

  private static final String INGESTED_ID = "01961234-5678-7abc-8def-0123456789ac";
  private static final String TOKEN = "token-xyz";

  /** The schedule a case that is not about waiting spends between attempts. */
  private static final Duration PROMPT_BACKOFF = Duration.ofMillis(5);

  /**
   * The budget the delay cases share. A delay the API names above it is expected to be cut down to it, so this also
   * bounds what those cases cost.
   */
  private static final Duration DELAY_BUDGET = Duration.ofMillis(1100);

  /**
   * What a wait may overshoot by before it is read as more than what was asked for: a loopback round trip, a timer and
   * a scheduler all sit inside it.
   */
  private static final Duration DELAY_SLACK = Duration.ofMillis(900);

  /**
   * What a wait may come back early by before it is read as a wait that did not happen.
   *
   * <p>A delay is scheduled on one clock and measured on another — the wait goes through a timer, and what this suite
   * reads is {@code System.nanoTime()} — so the two disagree by a fraction of a millisecond in either direction, and
   * the millisecond this suite counts in is truncated rather than rounded. A send that waited its whole delay can
   * therefore read one millisecond short of it, which is not a defect and which no client can fix.
   *
   * <p>It is deliberately a small fraction of the shortest delay any case asserts, which is a second: a client that
   * shortened a delay, skipped it, or ignored the header still reads hundreds of milliseconds below what it asked
   * for, and still fails. Measured rather than assumed, on the shortest case, against a client mutated to wait nine
   * tenths of what it was told: eleven settled runs came back between 89ms and 93ms short, so this floor sits some
   * 39ms clear of catching that. What eats into the margin is the round trips inside the measured window, which give
   * back what the mutation removes — so a run cold enough to widen them is the one to distrust, not a run that
   * waited.
   */
  private static final Duration CLOCK_SLACK = Duration.ofMillis(50);

  /** What a send says it did, out of the message it failed with. */
  private static final Pattern GAVE_UP = Pattern.compile("gave up after (\\d+) attempts");

  /** What the build file publishes this artefact at, read beside the coordinates it publishes it under. */
  private static final Pattern PUBLISHED =
      Pattern.compile("<artifactId>hook0-client</artifactId>\\s*<version>([^<]+)</version>");

  /**
   * How a refusal the corpus names reads in this client's own words.
   *
   * <p>Every name the corpus declares is looked up here, so one added there stops this suite until it is mapped rather
   * than passing under whatever the client happened to say.
   */
  private static final Map<String, String> REFUSALS =
      Map.of(
          "code_not_hexadecimal", "not hexadecimal",
          "header_not_delivered", "was not delivered",
          "code_mismatch", "does not match",
          "outside_tolerance", "outside the");

  @ParameterizedTest
  @EnumSource(Surface.class)
  void theCorpusSaysWhatEveryProblemDoesToASend(Surface surface) {
    // The status is not what decides: the corpus carries problems answering the same status with
    // opposite verdicts, and a client reading the status alone fails half of them.
    for (Map<String, Object> rule : Corpus.entries(RETRY, "problems")) {
      boolean retryable = ((Boolean) rule.get("retryable")).booleanValue();
      int expected = retryable ? 2 : 1;
      Outcome outcome =
          issuedFor(surface, refusal(status(rule), (String) rule.get("problem"), Map.of()));

      assertEquals(
          expected,
          outcome.issued(),
          "`"
              + rule.get("problem")
              + "` under "
              + status(rule)
              + " issued "
              + outcome.issued()
              + " requests where the corpus expects "
              + expected
              + ": "
              + rule.get("reason"));
      assertEquals(retryable, outcome.ingested());
    }
  }

  @ParameterizedTest
  @EnumSource(Surface.class)
  void theCorpusSaysWhatEveryStatusDoesToASend(Surface surface) {
    // A body naming no problem this client could read is also what an older client meets when the
    // API names a problem it has never heard of.
    for (Map<String, Object> rule : Corpus.entries(RETRY, "statuses")) {
      boolean retryable = ((Boolean) rule.get("retryable")).booleanValue();
      int expected = retryable ? 2 : 1;
      Outcome outcome =
          issuedFor(surface, refusal(status(rule), "AProblemThisClientHasNeverHeardOf", Map.of()));

      assertEquals(
          expected,
          outcome.issued(),
          "a status of "
              + status(rule)
              + " issued "
              + outcome.issued()
              + " requests where the corpus expects "
              + expected
              + ": "
              + rule.get("reason"));
    }
  }

  @ParameterizedTest
  @EnumSource(Surface.class)
  void theCorpusSaysWhatARequestTheApiNeverAnsweredDoes(Surface surface) {
    // Every cause the corpus names is provoked for real rather than reported: a server that sits on
    // an answer past the timeout, an answer above a ceiling this client set for itself, and a URL
    // nothing can be sent to.
    @SuppressWarnings("unchecked")
    Map<String, Object> transport = (Map<String, Object>) RETRY.get("transport");
    for (Map<String, Object> rule : Corpus.entries(transport, "causes")) {
      String cause = (String) rule.get("cause");
      boolean retryable = ((Boolean) rule.get("retryable")).booleanValue();
      int expected = retryable ? 2 : 1;
      Outcome outcome = provoked(surface, cause);

      assertEquals(
          expected,
          outcome.issued(),
          "`"
              + cause
              + "` issued "
              + outcome.issued()
              + " requests where the corpus expects "
              + expected
              + ": "
              + rule.get("reason"));
      assertEquals(retryable, outcome.ingested());
    }
  }

  @ParameterizedTest
  @EnumSource(Surface.class)
  void theDelayTheApiNamesIsHonouredAndBounded(Surface surface) {
    // The header is written by the other end, so honouring it whole would hand a stranger the length
    // of this client's send. What the corpus asks for is that a delay be waited out when the budget
    // can afford it and cut down to what is left of the budget when it cannot.
    @SuppressWarnings("unchecked")
    Map<String, Object> retryAfter = (Map<String, Object>) RETRY.get("retry_after");
    String header = (String) retryAfter.get("header");

    for (Map<String, Object> delay : Corpus.entries(retryAfter, "cases")) {
      boolean honoured = ((Boolean) delay.get("honoured")).booleanValue();
      long asked = honoured ? ((Long) delay.get("seconds")).longValue() * 1000 : 0;
      long expected = Math.min(asked, DELAY_BUDGET.toMillis());
      long waited = waitedFor(surface, header, (String) delay.get("header"));

      assertTrue(
          waited >= expected - CLOCK_SLACK.toMillis(),
          "`" + header + ": " + delay.get("header") + "` was retried after " + waited
              + "ms, sooner than the " + expected + "ms it asked for");
      assertTrue(
          waited <= expected + DELAY_SLACK.toMillis(),
          "`" + header + ": " + delay.get("header") + "` held the send for " + waited
              + "ms, above the " + expected + "ms it is bounded to");
    }
  }

  @Test
  void theBoundsAreTheOnesTheCorpusNames() {
    // This client's defaults, held against the one place the numbers are written down. What is
    // asserted is read from the corpus rather than listed here, so a bound added there and left
    // unapplied fails instead of passing unnoticed.
    Options built = Options.defaults();
    RetryPolicy policy = built.retryPolicy();

    Map<String, Long> applied = new LinkedHashMap<>();
    applied.put("max_attempts", Long.valueOf(policy.maxAttempts()));
    applied.put("max_attempts_cap", Long.valueOf(RetryPolicy.MAX_ATTEMPTS_CAP));
    applied.put("initial_backoff_ms", Long.valueOf(policy.initialBackoff().toMillis()));
    applied.put("max_backoff_ms", Long.valueOf(policy.maxBackoff().toMillis()));
    applied.put("max_total_delay_ms", Long.valueOf(policy.maxTotalDelay().toMillis()));
    applied.put("request_timeout_ms", Long.valueOf(built.requestTimeout().toMillis()));
    applied.put("max_payload_bytes", Long.valueOf(built.maxPayloadBytes()));
    applied.put("max_response_bytes", Long.valueOf(built.maxResponseBytes()));
    applied.put("max_head_bytes", Long.valueOf(built.maxHeadBytes()));
    applied.put("max_response_headers", Long.valueOf(built.maxResponseHeaders()));
    applied.put("max_header_bytes", Long.valueOf(built.maxHeaderBytes()));

    Map<String, Object> document = Corpus.document("bounds.json");
    @SuppressWarnings("unchecked")
    Map<String, Object> bounds = (Map<String, Object>) document.get("bounds");

    TreeSet<String> unapplied = new TreeSet<>(bounds.keySet());
    unapplied.removeAll(applied.keySet());

    assertTrue(unapplied.isEmpty(), "the corpus names bounds this client does not apply: " + unapplied);

    String text = Corpus.text("bounds.json");
    for (Map.Entry<String, Object> bound : bounds.entrySet()) {
      assertEquals(
          ((Long) bound.getValue()).longValue(),
          applied.get(bound.getKey()).longValue(),
          bound.getKey());
      // Read once more straight out of the committed text, so the number this suite asserts is the
      // number somebody wrote down rather than whatever the reader made of it.
      assertTrue(
          text.contains("\"" + bound.getKey() + "\": " + bound.getValue()),
          "`" + bound.getKey() + "` was not read out of the corpus as it is written there");
    }
  }

  @ParameterizedTest
  @EnumSource(Surface.class)
  void everyRequestCarriesWhatTheCorpusPins(Surface surface) {
    List<Map<String, Object>> headers = Corpus.entries(REQUEST, "headers");
    List<Object> occasions = Corpus.values(REQUEST, "occasions");

    TreeSet<String> unknown = new TreeSet<>();
    for (Map<String, Object> header : headers) {
      if (!occasions.contains(header.get("when"))) {
        unknown.add((String) header.get("when"));
      }
    }
    assertTrue(unknown.isEmpty(), "the corpus pins a header for an occasion it does not declare: " + unknown);

    Options built = options(1);
    try (FakeApi api = new FakeApi();
        Hook0Client client = new Hook0Client(api.baseUrl(), "app-123", TOKEN, built)) {
      api.willAnswer(FakeApi.Scripted.of(201, Map.of("event_id", INGESTED_ID)));
      surface.send(client, anEvent());

      // A send carries a body, so every occasion the corpus declares applies to this one request.
      long composedAtMost = ((Long) REQUEST.get("max_composed_bytes")).longValue();
      // The schedule is the one this client was built with a few lines up, so what the corpus writes
      // about the retry policy is an exact string rather than a shape with something in it.
      RetryPolicy policy = built.retryPolicy();
      Map<String, String> bound =
          Map.of(
              "token", TOKEN,
              "language", "java",
              "version", publishedVersion(),
              "attempts", String.valueOf(policy.attempts()),
              "backoff_ms", String.valueOf(policy.initialBackoff().toMillis()),
              "ceiling_ms", String.valueOf(policy.maxBackoff().toMillis()),
              "budget_ms", String.valueOf(policy.maxTotalDelay().toMillis()));

      FakeApi.Received sent = api.received().get(0);
      for (Map<String, Object> header : headers) {
        String name = ((String) header.get("name")).toLowerCase(Locale.ROOT);
        String template = (String) header.get("value");
        String written = sent.headers().getOrDefault(name, "");
        List<String> chunks = templateChunks(template, bound);

        assertTrue(
            matchesChunks(chunks, written),
            "the request carried `" + name + ": " + written + "` where the shared contract says `" + template + "`: "
                + header.get("reason"));

        // A value with a hole this suite cannot fill is one the client composed out of what the
        // platform told it, and what the platform says is as long as it feels like.
        if (chunks.size() > 1) {
          int spent = written.getBytes(StandardCharsets.UTF_8).length;
          assertTrue(
              spent <= composedAtMost,
              "the request carried " + spent + " bytes of `" + name + "`, above the " + composedAtMost
                  + " the shared contract cuts a composed value to");
        }
      }
    }
  }

  /**
   * What a value of the request document is made of, once the holes this suite can speak for are filled in.
   *
   * <p>A value is a template: `${name}` is a hole and everything around it is literal. A hole named in
   * {@code bound} becomes part of the literal text around it; one that is not is a hole no suite can fill without
   * reimplementing the client it is testing, and it separates two chunks. A template whose holes are all bound is
   * therefore one chunk, and the whole value is that chunk.
   */
  private static List<String> templateChunks(String template, Map<String, String> bound) {
    List<String> chunks = new ArrayList<>();
    chunks.add("");
    String rest = template;

    for (int opened = rest.indexOf("${"); opened >= 0; opened = rest.indexOf("${")) {
      int closed = rest.indexOf('}', opened);
      if (closed < 0) {
        break;
      }
      int last = chunks.size() - 1;
      chunks.set(last, chunks.get(last) + rest.substring(0, opened));

      String name = rest.substring(opened + 2, closed);
      if (bound.containsKey(name)) {
        chunks.set(last, chunks.get(last) + bound.get(name));
      } else {
        chunks.add("");
      }
      rest = rest.substring(closed + 1);
    }

    int last = chunks.size() - 1;
    chunks.set(last, chunks.get(last) + rest);
    return List.copyOf(chunks);
  }

  /**
   * Whether what arrived is what those chunks describe: the literal text in order, anchored at both ends, with
   * something non-empty standing in every hole between them.
   */
  private static boolean matchesChunks(List<String> chunks, String carried) {
    String first = chunks.get(0);
    if (chunks.size() == 1) {
      return carried.equals(first);
    }
    if (!carried.startsWith(first)) {
      return false;
    }

    String rest = carried.substring(first.length());
    for (String chunk : chunks.subList(1, chunks.size() - 1)) {
      // A hole stands before this chunk, and nothing is not something, so the search starts past
      // whatever fills it.
      int found = rest.indexOf(chunk, 1);
      if (found < 0) {
        return false;
      }
      rest = rest.substring(found + chunk.length());
    }

    String last = chunks.get(chunks.size() - 1);
    return rest.length() > last.length() && rest.endsWith(last);
  }

  /**
   * The version this artefact is published at, which is the one hole of the {@code User-Agent} this suite can speak
   * for.
   *
   * <p>A jar carries no build file to read it back out of at runtime, so the client writes it down beside the
   * transport; reading it here out of {@code pom.xml} is what keeps the two from drifting apart.
   */
  private static String publishedVersion() {
    Path pom = Path.of("pom.xml").toAbsolutePath();
    String text;
    try {
      text = Files.readString(pom, StandardCharsets.UTF_8);
    } catch (IOException unreadable) {
      throw new UncheckedIOException("the build file is not where this suite looks for it: " + pom, unreadable);
    }

    Matcher published = PUBLISHED.matcher(text);
    assertTrue(published.find(), "the build file publishes this artefact at no version: " + pom);
    return published.group(1).strip();
  }

  @Test
  void everyRefusalTheCorpusDeclaresReadsAsOneOfThisClients() {
    // A refusal named in the corpus and mapped to nothing here would pass under any wording.
    TreeSet<String> unmapped = new TreeSet<>();
    for (Object refusal : Corpus.values(SIGNATURE, "refusals")) {
      if (!REFUSALS.containsKey(refusal)) {
        unmapped.add(String.valueOf(refusal));
      }
    }
    assertTrue(unmapped.isEmpty(), "the corpus declares refusals this suite maps to nothing: " + unmapped);
  }

  @Test
  void everyDeliveryOfTheCorpusIsVerifiedAsItSays() {
    // A refused delivery has to be refused for the reason the corpus names: a client that computed a
    // code over a header that never arrived and reported a mismatch would otherwise look right.
    for (Map<String, Object> vector : Corpus.entries(SIGNATURE, "vectors")) {
      if ("accepted".equals(vector.get("verdict"))) {
        verified(vector);
        continue;
      }

      ClientException refused =
          assertThrows(ClientException.class, () -> verified(vector), (String) vector.get("name"));
      String wanted = REFUSALS.get(vector.get("refusal"));

      assertTrue(
          refused.getMessage().contains(wanted),
          "a delivery the corpus refuses as `"
              + vector.get("refusal")
              + "` was answered `"
              + refused.getMessage()
              + "`: "
              + vector.get("reason"));
    }
  }

  private static void verified(Map<String, Object> vector) {
    @SuppressWarnings("unchecked")
    List<List<Object>> delivered = (List<List<Object>>) vector.get("headers");
    List<Map.Entry<String, String>> headers = new ArrayList<>();
    for (List<Object> header : delivered) {
      headers.add(Map.entry((String) header.get(0), (String) header.get(1)));
    }

    Webhooks.verifyAt(
        (String) vector.get("signature"),
        (String) vector.get("payload"),
        headers,
        (String) vector.get("secret"),
        Duration.ofSeconds(((Long) vector.get("tolerance_seconds")).longValue()),
        Instant.ofEpochSecond(((Long) vector.get("current_time")).longValue()));
  }

  /** How many requests a send issued, and whether it ended up ingesting the event. */
  private record Outcome(int issued, boolean ingested) {}

  private static int status(Map<String, Object> rule) {
    return (int) ((Long) rule.get("status")).longValue();
  }

  private static Options options(int maxAttempts) {
    return Options.defaults()
        .withRetryPolicy(new RetryPolicy(maxAttempts, PROMPT_BACKOFF, PROMPT_BACKOFF, Duration.ofSeconds(1)))
        .withRequestTimeout(Duration.ofSeconds(5));
  }

  private static Event anEvent() {
    return Event.of("auth.user.create", "{\"email\": \"test@example.com\"}", "application/json", Map.of());
  }

  private static FakeApi.Scripted ingested() {
    return FakeApi.Scripted.of(201, Map.of("application_id", "app-123", "event_id", INGESTED_ID));
  }

  /** What the API says when it refuses a request, in the shape every Hook0 failure takes. */
  private static FakeApi.Scripted refusal(int status, String problem, Map<String, String> headers) {
    return FakeApi.Scripted.of(
        status,
        Map.of(
            "id", problem,
            "status", Long.valueOf(status),
            "title", "refused",
            "detail", "what the corpus scripted",
            "type", "https://hook0.com/documentation/errors/" + problem),
        headers);
  }

  /**
   * How many requests a send made when the API answered that way and then took the event.
   *
   * <p>One API per case rather than one per suite: what is counted is what this send issued, and a count carried over
   * from the case before it would say nothing.
   */
  private static Outcome issuedFor(Surface surface, FakeApi.Scripted answer) {
    try (FakeApi api = new FakeApi();
        Hook0Client client = new Hook0Client(api.baseUrl(), "app-123", TOKEN, options(4))) {
      api.willAnswer(answer, ingested());
      return counted(api, () -> surface.send(client, anEvent()));
    }
  }

  /** One cause of a request the API never answered, provoked over a real socket. */
  private static Outcome provoked(Surface surface, String cause) {
    return switch (cause) {
      case "no_answer" -> provokedByNoAnswer(surface);
      case "answer_above_a_bound" -> provokedByAnAnswerAboveABound(surface);
      case "unusable_api_url" -> provokedByAnUnusableApiUrl(surface);
      default -> throw new IllegalStateException(
          "the corpus names a cause `" + cause + "` this suite does not know how to provoke");
    };
  }

  /** An attempt that runs out of time before the API writes anything. */
  private static Outcome provokedByNoAnswer(Surface surface) {
    Options impatient = options(4).withRequestTimeout(Duration.ofMillis(200));
    try (FakeApi api = new FakeApi();
        Hook0Client client = new Hook0Client(api.baseUrl(), "app-123", TOKEN, impatient)) {
      api.willAnswer(
          new FakeApi.Scripted(201, Json.write(Map.of("event_id", INGESTED_ID)), Duration.ofSeconds(1), Map.of()),
          ingested());
      return counted(api, () -> surface.send(client, anEvent()));
    }
  }

  /** An answer larger than what this client agreed to read off the socket. */
  private static Outcome provokedByAnAnswerAboveABound(Surface surface) {
    Options small = options(4).withMaxResponseBytes(256);
    try (FakeApi api = new FakeApi();
        Hook0Client client = new Hook0Client(api.baseUrl(), "app-123", TOKEN, small)) {
      api.willAnswer(
          FakeApi.Scripted.of(201, Map.of("event_id", INGESTED_ID, "padding", "x".repeat(2048))), ingested());
      return counted(api, () -> surface.send(client, anEvent()));
    }
  }

  /** A base URL nothing can be sent to, which means nothing is ever sent. */
  private static Outcome provokedByAnUnusableApiUrl(Surface surface) {
    try (FakeApi api = new FakeApi();
        Hook0Client client = new Hook0Client("gopher://nowhere.invalid", "app-123", TOKEN, options(4))) {
      api.willAnswer(ingested());
      return counted(api, () -> surface.send(client, anEvent()));
    }
  }

  /**
   * How many attempts a send made, and whether it ended up ingesting the event.
   *
   * <p>A send that reached a server is counted by what that server received. One that never reached anything — an API
   * URL nothing can be sent to is the corpus's own example — is counted by what the client says it did, which is also
   * the message a caller is left holding: a misconfiguration retried four times reads as a network that would not
   * answer.
   */
  private static Outcome counted(FakeApi api, Runnable send) {
    try {
      send.run();
      return new Outcome(api.received().size(), true);
    } catch (Hook0Exception refused) {
      return new Outcome(Math.max(api.received().size(), attemptsOf(refused.getMessage())), false);
    }
  }

  private static int attemptsOf(String message) {
    Matcher named = GAVE_UP.matcher(message == null ? "" : message);
    return named.find() ? Integer.parseInt(named.group(1)) : 1;
  }

  /** How long a send spent waiting when the API named that delay beside a paced answer. */
  private static long waitedFor(Surface surface, String header, String written) {
    Map<String, Object> paced = pacedProblem();
    Options budgeted =
        Options.defaults()
            .withRetryPolicy(new RetryPolicy(4, PROMPT_BACKOFF, PROMPT_BACKOFF, DELAY_BUDGET))
            .withRequestTimeout(Duration.ofSeconds(5));

    try (FakeApi api = new FakeApi();
        Hook0Client client = new Hook0Client(api.baseUrl(), "app-123", TOKEN, budgeted)) {
      api.willAnswer(
          refusal(status(paced), (String) paced.get("problem"), Map.of(header, written)), ingested());

      long started = System.nanoTime();
      surface.send(client, anEvent());
      long waited = (System.nanoTime() - started) / 1_000_000;

      assertEquals(2, api.received().size(), "a paced answer was not retried");
      return waited;
    }
  }

  /**
   * A problem the corpus says is worth repeating, sharing its status with one it says is not.
   *
   * <p>That pair is the whole reason the corpus classifies problems rather than statuses, and the retryable one is the
   * answer the API names a delay beside.
   */
  private static Map<String, Object> pacedProblem() {
    List<Map<String, Object>> problems = Corpus.entries(RETRY, "problems");
    for (Map<String, Object> rule : problems) {
      if (!((Boolean) rule.get("retryable")).booleanValue()) {
        continue;
      }
      for (Map<String, Object> other : problems) {
        if (status(other) == status(rule) && !((Boolean) other.get("retryable")).booleanValue()) {
          return rule;
        }
      }
    }
    return fail("no status of the corpus carries opposite verdicts");
  }
}
