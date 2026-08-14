package com.hook0.kotlin

import java.time.Duration
import java.time.Instant
import java.util.Random
import kotlin.math.max
import kotlin.math.roundToLong
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Assertions.fail
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.Timeout

/**
 * What holds for every input, rather than for the ones a case happened to pick.
 *
 * Five things are checked here. A retry schedule never spends more than the policy that produced it
 * allows, whichever way the randomness fell. Reading a signature header answers with the one failure
 * this client declares, whatever text reached the endpoint, and never with anything else. A value
 * read out of a document the API could answer is written back as the value that was read. The reader
 * this artefact carries instead of a dependency never fails with anything but its own failure, and
 * refuses at its ceilings rather than trimming to them. And identifiers minted in sequence never
 * carry a moment that goes back.
 *
 * There is no property-testing tool on this classpath and this artefact installs nothing at run
 * time, so the search is written here: a fixed seed, a bounded number of draws, and the
 * counter-examples worth keeping committed under `test/resources/regressions` so they run as
 * ordinary cases on every pipeline. A failing draw is one somebody can reproduce by running the
 * suite again rather than one that goes away on a retry.
 */
@Timeout(300)
class PropertyTest {

  @Test
  fun aRetryScheduleStaysWithinEveryBoundOfItsPolicy() {
    val random = Random(SEED)

    for (counterExample in Corpus.regressions("retry_policies")) {
      val written = counterExample as List<*>
      holdsFor(
        RetryPolicy(
          asLong(written[0]).toInt(),
          millis(written[1]),
          millis(written[2]),
          millis(written[3])
        ),
        draws(written[4] as List<*>)
      )
    }

    for (draw in 0 until DRAWS) {
      val policy =
        RetryPolicy(
          random.nextInt(MAX_DRAWN_ATTEMPTS + 5) - 4,
          Duration.ofMillis((random.nextDouble() * MAX_DRAWN_MILLIS).toLong()),
          Duration.ofMillis((random.nextDouble() * MAX_DRAWN_MILLIS).toLong()),
          Duration.ofMillis((random.nextDouble() * MAX_DRAWN_BUDGET).toLong())
        )
      val drawn = DoubleArray(random.nextInt(9)) { random.nextDouble() * 2 - 0.5 }
      holdsFor(policy, drawn)
    }
  }

  @Test
  fun readingASignatureAnswersWithTheOneFailureThisClientDeclares() {
    val random = Random(SEED)
    val headers = ArrayList<String>()
    for (committed in Corpus.regressions("signatures")) {
      headers.add(committed.toString())
    }
    for (draw in 0 until DRAWS) {
      val built = StringBuilder()
      val pieces = random.nextInt(MAX_DRAWN_HEADER + 1)
      for (index in 0 until pieces) {
        built.append(PIECES[random.nextInt(PIECES.size)])
      }
      headers.add(built.toString())
    }

    for (header in headers) {
      try {
        Signature.parse(header)
      } catch (refused: ClientException) {
        continue
      } catch (undeclared: RuntimeException) {
        fail<Unit>("`${preview(header)}` was refused with ${undeclared.javaClass.name}")
      }

      // Parsing answered, so verifying has to answer the same way: a header that reads must not find
      // a way to fail that a caller cannot name.
      try {
        Webhooks.verifyAt(header, "", emptyList(), "secret", Duration.ofSeconds(300), Instant.EPOCH)
      } catch (refused: ClientException) {
        continue
      } catch (undeclared: RuntimeException) {
        fail<Unit>("`${preview(header)}` was verified with ${undeclared.javaClass.name}")
      }
    }
  }

  @Test
  fun aGeneratedValueReadsBackWhatItWrote() {
    val random = Random(SEED)
    val documents = ArrayList<Any?>(Corpus.regressions("documents"))
    for (committed in documents.toList()) {
      for (draw in 0 until 4) {
        documents.add(mutated(committed, random))
      }
    }

    var exercised = 0
    for (declared in Generated.types()) {
      if (!Generated.isAValue(declared)) {
        continue
      }
      for (document in documents) {
        val read = Generated.readOrNothing(declared, document) ?: continue
        val written = Generated.written(read)

        assertEquals(
          read,
          Generated.readOrNothing(declared, written),
          "${declared.simpleName} does not read back"
        )
        exercised++
      }
    }

    assertTrue(
      exercised > 0,
      "no value could be read out of the committed documents, so this checked nothing"
    )
  }

  @Test
  fun theReaderNeverFailsWithAnythingButItsOwnFailure() {
    val random = Random(SEED)

    for (draw in 0 until DRAWS * 4) {
      val built = StringBuilder()
      val pieces = random.nextInt(48)
      for (index in 0 until pieces) {
        built.append(FRAGMENTS[random.nextInt(FRAGMENTS.size)])
      }
      val document = built.toString()

      val read =
        try {
          Json.parse(document)
        } catch (refused: JsonException) {
          continue
        } catch (undeclared: RuntimeException) {
          fail<Any?>("`${preview(document)}` was refused with ${undeclared.javaClass.name}")
        }

      // What was read is written back and read again: a reader that trimmed at one of its ceilings
      // would answer a value the second pass no longer agrees with.
      try {
        assertEquals(read, Json.parse(Json.write(read)), "`${preview(document)}` does not read back")
      } catch (refused: JsonException) {
        fail<Unit>("`${preview(document)}` read as something this writer cannot write")
      }
    }
  }

  @Test
  fun mintedIdentifiersCarryAMomentThatNeverGoesBack() {
    // Identifiers minted inside one millisecond are not ordered — the tail is random — so what holds
    // is that the moment they carry never goes back, which is what the leading bits are for.
    val moments = (0 until DRAWS).map { draw -> prefix(Uuid7.generate(1_800_000_000_000L + draw)) }

    assertEquals(
      moments.sorted(),
      moments,
      "identifiers minted in sequence do not carry moments in sequence"
    )
    assertEquals(7, Uuid7.generate().version())
    assertEquals(2, Uuid7.generate().variant())
  }

  companion object {
    /** What the draws are made from. Fixed, so the suite explores the same inputs everywhere it runs. */
    private const val SEED = 20_260_814L

    /** How many draws each property makes. Bounded, so a pipeline can never be held by one. */
    private const val DRAWS = 200

    /** The bounds a drawn policy is built inside. */
    private const val MAX_DRAWN_ATTEMPTS = 64
    private const val MAX_DRAWN_MILLIS = 10_000.0
    private const val MAX_DRAWN_BUDGET = 60_000.0

    /** Longest header a draw builds. */
    private const val MAX_DRAWN_HEADER = 96

    /** How much of a drawn input a failure prints. */
    private const val MAX_PREVIEW = 120

    /** The pieces a signature header is made of, put together every way a sender that is not Hook0 might. */
    private val PIECES =
      listOf(
        "t", "v0", "v1", "h", "=", ",", "0", "9", "zz", "abc", "x-event-id", "1800000000", "-1",
        "\"", " ", ".", "{", "}"
      )

    /** The pieces a drawn document is made of. */
    private val FRAGMENTS =
      listOf(
        "{", "}", "[", "]", "\"", ":", ",", "a", "1", "-", ".", "e", "true", "false", "null", "\\",
        "\\u0041", " ", "\t"
      )

    private fun holdsFor(policy: RetryPolicy, draws: DoubleArray) {
      val delays = policy.delaysMillis(draws)
      val budget = max(policy.maxTotalDelay.toMillis(), 0)

      assertTrue(policy.attempts() >= 1)
      assertTrue(policy.attempts() <= RetryPolicy.MAX_ATTEMPTS_CAP)
      assertTrue(
        delays.size <= policy.attempts() - 1,
        "a schedule carries more delays than it has retries"
      )

      var spent = 0L
      for ((index, delay) in delays.withIndex()) {
        spent += delay

        assertTrue(delay >= 0, "a delay runs backwards")
        assertTrue(
          delay <= policy.backoffCeilingMillis(index + 1),
          "a delay crosses the ceiling of its retry"
        )
        assertTrue(
          delay <= max(policy.maxBackoff.toMillis(), 0),
          "a delay crosses the ceiling of the policy"
        )
      }
      assertTrue(spent <= budget, "a schedule spends more than the budget its delays share")

      // A schedule never hurries up as it goes: the ceiling of a retry never sits below the one
      // before.
      var previous = -1L
      for (retryNumber in 1..policy.attempts()) {
        val ceiling = policy.backoffCeilingMillis(retryNumber)
        assertTrue(ceiling >= previous, "a later retry is given a lower ceiling than an earlier one")
        previous = ceiling
      }
    }

    private fun mutated(document: Any?, random: Random): Any? {
      if (document !is Map<*, *> || document.isEmpty()) {
        return document
      }

      val names = document.keys.toList()
      val name = names[random.nextInt(names.size)].toString()
      val written = LinkedHashMap<String, Any?>()
      for ((key, held) in document) {
        written[key.toString()] = held
      }

      when (random.nextInt(4)) {
        0 -> written.remove(name)
        1 -> written[name] = random.nextInt(1000).toLong()
        2 -> written[name] = listOf(document[name].toString())
        else -> written[name] = null
      }
      return written
    }

    private fun draws(written: List<*>): DoubleArray = DoubleArray(written.size) { index ->
      when (val one = written[index]) {
        "nan" -> Double.NaN
        "infinity" -> Double.POSITIVE_INFINITY
        "-infinity" -> Double.NEGATIVE_INFINITY
        is Number -> one.toDouble()
        else -> 1.0
      }
    }

    private fun millis(seconds: Any?): Duration =
      Duration.ofMillis(((seconds as Number).toDouble() * 1000).roundToLong())

    private fun asLong(value: Any?): Long = (value as Number).toLong()

    private fun preview(text: String): String = if (text.length <= MAX_PREVIEW) text else text.take(MAX_PREVIEW) + "…"

    private fun prefix(identifier: java.util.UUID): String = identifier.toString().substring(0, 13)
  }
}
