package com.hook0.client;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.junit.jupiter.api.Assertions.fail;

import java.time.Duration;
import java.time.Instant;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Random;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.Timeout;

/**
 * What holds for every input, rather than for the ones a case happened to pick.
 *
 * <p>Five things are checked here. A retry schedule never spends more than the policy that produced it allows,
 * whichever way the randomness fell. Reading a signature header answers with the one failure this client declares,
 * whatever text reached the endpoint, and never with anything else. A value read out of a document the API could answer
 * is written back as the value that was read. The reader this artefact carries instead of a dependency never fails with
 * anything but its own failure, and refuses at its ceilings rather than trimming to them. And identifiers minted in
 * sequence never carry a moment that goes back.
 *
 * <p>There is no property-testing tool in the standard library and this artefact installs nothing at run time, so the
 * search is written here: a fixed seed, a bounded number of draws, and the counter-examples worth keeping committed
 * under {@code test/resources/regressions} so they run as ordinary cases on every pipeline. A failing draw is one
 * somebody can reproduce by running the suite again rather than one that goes away on a retry.
 */
@Timeout(180)
final class PropertyTest {

  /** What the draws are made from. Fixed, so the suite explores the same inputs everywhere it runs. */
  private static final long SEED = 20_260_814L;

  /** How many draws each property makes. Bounded, so a pipeline can never be held by one. */
  private static final int DRAWS = 200;

  /** The bounds a drawn policy is built inside. */
  private static final int MAX_DRAWN_ATTEMPTS = 64;
  private static final long MAX_DRAWN_MILLIS = 10_000;
  private static final long MAX_DRAWN_BUDGET = 60_000;

  /** Longest header a draw builds. */
  private static final int MAX_DRAWN_HEADER = 96;

  /** The pieces a signature header is made of, put together every way a sender that is not Hook0 might. */
  private static final List<String> PIECES =
      List.of("t", "v0", "v1", "h", "=", ",", "0", "9", "zz", "abc", "x-event-id", "1800000000", "-1", "\"", " ", ".",
          "{", "}");

  /** The pieces a drawn document is made of. */
  private static final List<String> FRAGMENTS =
      List.of("{", "}", "[", "]", "\"", ":", ",", "a", "1", "-", ".", "e", "true", "false", "null", "\\", "\\u0041",
          " ", "\t");

  @Test
  void aRetryScheduleStaysWithinEveryBoundOfItsPolicy() {
    Random random = new Random(SEED);
    List<Object> committed = Corpus.regressions("retry_policies");

    for (Object counterExample : committed) {
      List<?> written = (List<?>) counterExample;
      holdsFor(
          new RetryPolicy(
              (int) asLong(written.get(0)),
              millis(written.get(1)),
              millis(written.get(2)),
              millis(written.get(3))),
          draws((List<?>) written.get(4)));
    }

    for (int draw = 0; draw < DRAWS; draw++) {
      RetryPolicy policy =
          new RetryPolicy(
              random.nextInt(MAX_DRAWN_ATTEMPTS + 5) - 4,
              Duration.ofMillis((long) (random.nextDouble() * MAX_DRAWN_MILLIS)),
              Duration.ofMillis((long) (random.nextDouble() * MAX_DRAWN_MILLIS)),
              Duration.ofMillis((long) (random.nextDouble() * MAX_DRAWN_BUDGET)));
      double[] drawn = new double[random.nextInt(9)];
      for (int index = 0; index < drawn.length; index++) {
        drawn[index] = random.nextDouble() * 2 - 0.5;
      }
      holdsFor(policy, drawn);
    }
  }

  @Test
  void readingASignatureAnswersWithTheOneFailureThisClientDeclares() {
    Random random = new Random(SEED);
    List<String> headers = new ArrayList<>();
    for (Object committed : Corpus.regressions("signatures")) {
      headers.add(String.valueOf(committed));
    }
    for (int draw = 0; draw < DRAWS; draw++) {
      StringBuilder built = new StringBuilder();
      int pieces = random.nextInt(MAX_DRAWN_HEADER + 1);
      for (int index = 0; index < pieces; index++) {
        built.append(PIECES.get(random.nextInt(PIECES.size())));
      }
      headers.add(built.toString());
    }

    for (String header : headers) {
      try {
        Signature.parse(header);
      } catch (ClientException refused) {
        continue;
      } catch (RuntimeException undeclared) {
        fail("`" + preview(header) + "` was refused with " + undeclared.getClass().getName());
      }

      // Parsing answered, so verifying has to answer the same way: a header that reads must not find
      // a way to fail that a caller cannot name.
      try {
        Webhooks.verifyAt(header, "", List.of(), "secret", Duration.ofSeconds(300), Instant.EPOCH);
      } catch (ClientException refused) {
        continue;
      } catch (RuntimeException undeclared) {
        fail("`" + preview(header) + "` was verified with " + undeclared.getClass().getName());
      }
    }
  }

  @Test
  void aGeneratedValueReadsBackWhatItWrote() {
    Random random = new Random(SEED);
    List<Object> documents = new ArrayList<>(Corpus.regressions("documents"));
    for (Object committed : List.copyOf(documents)) {
      for (int draw = 0; draw < 4; draw++) {
        documents.add(mutated(committed, random));
      }
    }

    int exercised = 0;
    for (Class<?> declared : Generated.types()) {
      if (!declared.isRecord()) {
        continue;
      }
      for (Object document : documents) {
        Object read = Generated.readOrNothing(declared, document);
        if (read == null) {
          continue;
        }
        Object written = Generated.written(read);

        assertEquals(
            read,
            Generated.readOrNothing(declared, written),
            declared.getSimpleName() + " does not read back");
        exercised++;
      }
    }

    assertTrue(exercised > 0, "no value could be read out of the committed documents, so this checked nothing");
  }

  @Test
  void theReaderNeverFailsWithAnythingButItsOwnFailure() {
    Random random = new Random(SEED);

    for (int draw = 0; draw < DRAWS * 4; draw++) {
      StringBuilder built = new StringBuilder();
      int pieces = random.nextInt(48);
      for (int index = 0; index < pieces; index++) {
        built.append(FRAGMENTS.get(random.nextInt(FRAGMENTS.size())));
      }
      String document = built.toString();

      Object read;
      try {
        read = Json.parse(document);
      } catch (JsonException refused) {
        continue;
      } catch (RuntimeException undeclared) {
        fail("`" + preview(document) + "` was refused with " + undeclared.getClass().getName());
        continue;
      }

      // What was read is written back and read again: a reader that trimmed at one of its ceilings
      // would answer a value the second pass no longer agrees with.
      try {
        assertEquals(read, Json.parse(Json.write(read)), "`" + preview(document) + "` does not read back");
      } catch (JsonException refused) {
        fail("`" + preview(document) + "` read as something this writer cannot write");
      }
    }
  }

  @Test
  void mintedIdentifiersCarryAMomentThatNeverGoesBack() {
    // Identifiers minted inside one millisecond are not ordered — the tail is random — so what holds
    // is that the moment they carry never goes back, which is what the leading bits are for.
    List<String> moments = new ArrayList<>();
    for (int draw = 0; draw < DRAWS; draw++) {
      moments.add(Uuid7.generate(1_800_000_000_000L + draw).toString().substring(0, 13));
    }

    List<String> sorted = new ArrayList<>(moments);
    sorted.sort(String::compareTo);

    assertEquals(sorted, moments, "identifiers minted in sequence do not carry moments in sequence");
    assertNotNull(Uuid7.generate());
    assertEquals(7, Uuid7.generate().version());
    assertEquals(2, Uuid7.generate().variant());
  }

  private static void holdsFor(RetryPolicy policy, double[] draws) {
    List<Long> delays = policy.delaysMillis(draws);
    long budget = Math.max(policy.maxTotalDelay().toMillis(), 0);

    assertTrue(policy.attempts() >= 1);
    assertTrue(policy.attempts() <= RetryPolicy.MAX_ATTEMPTS_CAP);
    assertTrue(delays.size() <= policy.attempts() - 1, "a schedule carries more delays than it has retries");

    long spent = 0;
    for (int index = 0; index < delays.size(); index++) {
      long delay = delays.get(index).longValue();
      spent += delay;

      assertTrue(delay >= 0, "a delay runs backwards");
      assertTrue(delay <= policy.backoffCeilingMillis(index + 1), "a delay crosses the ceiling of its retry");
      assertTrue(delay <= Math.max(policy.maxBackoff().toMillis(), 0), "a delay crosses the ceiling of the policy");
    }
    assertTrue(spent <= budget, "a schedule spends more than the budget its delays share");

    // A schedule never hurries up as it goes: the ceiling of a retry never sits below the one before.
    long previous = -1;
    for (int retryNumber = 1; retryNumber <= policy.attempts(); retryNumber++) {
      long ceiling = policy.backoffCeilingMillis(retryNumber);
      assertTrue(ceiling >= previous, "a later retry is given a lower ceiling than an earlier one");
      previous = ceiling;
    }
  }

  private static Object mutated(Object document, Random random) {
    if (!(document instanceof Map<?, ?> members) || members.isEmpty()) {
      return document;
    }

    List<?> names = List.copyOf(members.keySet());
    Object name = names.get(random.nextInt(names.size()));
    Map<String, Object> written = new LinkedHashMap<>();
    for (Map.Entry<?, ?> member : members.entrySet()) {
      written.put(String.valueOf(member.getKey()), member.getValue());
    }

    switch (random.nextInt(4)) {
      case 0 -> written.remove(name);
      case 1 -> written.put(String.valueOf(name), Long.valueOf(random.nextInt(1000)));
      case 2 -> written.put(String.valueOf(name), List.of(String.valueOf(members.get(name))));
      default -> written.put(String.valueOf(name), null);
    }
    return written;
  }

  private static double[] draws(List<?> written) {
    double[] drawn = new double[written.size()];
    for (int index = 0; index < drawn.length; index++) {
      Object one = written.get(index);
      drawn[index] =
          switch (String.valueOf(one)) {
            case "nan" -> Double.NaN;
            case "infinity" -> Double.POSITIVE_INFINITY;
            case "-infinity" -> Double.NEGATIVE_INFINITY;
            default -> one instanceof Number number ? number.doubleValue() : 1.0;
          };
    }
    return drawn;
  }

  private static Duration millis(Object seconds) {
    return Duration.ofMillis((long) (((Number) seconds).doubleValue() * 1000));
  }

  private static long asLong(Object value) {
    return ((Number) value).longValue();
  }

  private static String preview(String text) {
    return text.length() <= 120 ? text : text.substring(0, 120) + "…";
  }
}
