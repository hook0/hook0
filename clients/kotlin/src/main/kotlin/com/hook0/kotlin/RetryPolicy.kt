package com.hook0.kotlin

import java.time.Duration
import kotlin.math.max
import kotlin.math.min
import kotlin.math.roundToLong

/**
 * How a client spaces out the attempts of a single send.
 *
 * The delay before a retry doubles from [initialBackoff] and is capped by [maxBackoff]; the delay
 * actually waited is then drawn anywhere between zero and that ceiling, so that emitters which
 * failed at the same moment do not come back at the same moment. Retrying stops as soon as the
 * delays of the send would add up to more than [maxTotalDelay].
 *
 * The defaults are four attempts spread over at most five seconds: three retries absorb the blips a
 * webhook emitter meets in production — a connection reset, a rolling deployment answering 503 —
 * without holding the caller for long, and the five-second budget bounds what the worst send costs
 * whatever the individual delays turn out to be.
 *
 * @property maxAttempts attempts a single send makes at most, the first one included; `1` disables
 *     retrying
 * @property initialBackoff ceiling of the delay before the first retry
 * @property maxBackoff ceiling no single delay ever exceeds
 * @property maxTotalDelay budget all the delays of one send share
 */
data class RetryPolicy(
  val maxAttempts: Int,
  val initialBackoff: Duration,
  val maxBackoff: Duration,
  val maxTotalDelay: Duration
) {

  /**
   * Attempts this policy actually makes: [maxAttempts], brought inside `1..MAX_ATTEMPTS_CAP`.
   *
   * @return how many requests one send issues at most
   */
  fun attempts(): Int = maxAttempts.coerceIn(1, MAX_ATTEMPTS_CAP)

  /**
   * The first delay this policy holds, as everything that reads it reads it.
   *
   * One accessor per field, rather than each caller converting the field for itself: the schedule,
   * the budget the retry loop cuts a named delay down to, and the `Hook0-Client-Options` header all
   * state this number, and three conversions that agree today are three that can stop agreeing.
   * Reading the field raw is how the wire came to describe a schedule the client would not keep.
   */
  internal val initialBackoffMillis: Long get() = millis(initialBackoff)

  /** The longest single delay this policy holds, as everything that reads it reads it. */
  internal val maxBackoffMillis: Long get() = millis(maxBackoff)

  /** The budget this policy holds, as everything that reads it reads it. */
  internal val maxTotalDelayMillis: Long get() = millis(maxTotalDelay)

  /**
   * Ceiling of the delay before retry number [retryNumber], where `1` is the first retry.
   *
   * It doubles from [initialBackoff] and never exceeds [maxBackoff], so the ceilings of successive
   * retries never decrease.
   *
   * @param retryNumber which retry, counting from one
   * @return how long that retry may wait at most
   */
  fun backoffCeilingMillis(retryNumber: Int): Long {
    val doublings = (retryNumber.toLong() - 1).coerceIn(0, MAX_BACKOFF_DOUBLINGS.toLong())
    val ceiling = maxBackoffMillis
    var doubled = initialBackoffMillis
    var step = 0L
    while (step < doublings && doubled < ceiling) {
      // Doubling past the ceiling lands on it. Written as a comparison rather than as a doubling
      // that is then capped, because the doubling itself overflows for a ceiling near the largest
      // count there is, and a wrapped one reads as a delay of nothing at all.
      doubled = if (doubled > ceiling / 2) ceiling else doubled * 2
      step++
    }
    return doubled.coerceIn(0, ceiling)
  }

  /**
   * The delays this policy waits between the attempts of one send, one per retry.
   *
   * Each delay lands between zero and the ceiling of its retry, and the schedule is cut short as
   * soon as the next delay would spend more than [maxTotalDelay]. There are therefore at most
   * `attempts() - 1` delays, and they add up to at most `maxTotalDelay`.
   *
   * A draw that is missing or is not a finite number is read as `1`, which asks for the whole
   * ceiling: an unusable source of randomness makes the client wait longer, never less.
   *
   * @param draws one draw in `[0, 1)` per retry
   * @return how long to wait before each retry, in milliseconds
   */
  fun delaysMillis(draws: DoubleArray): List<Long> {
    val budget = maxTotalDelayMillis
    val waits = ArrayList<Long>()
    var spent = 0L

    for (retryNumber in 1..attempts() - 1) {
      val delay = (backoffCeilingMillis(retryNumber) * draw(draws, retryNumber - 1)).roundToLong()
      if (spent + delay > budget) {
        return waits.toList()
      }
      spent += delay
      waits.add(delay)
    }

    return waits.toList()
  }

  companion object {
    /**
     * One delay of a policy, in the whole milliseconds it is counted in.
     *
     * A [Duration] holds more than a count of milliseconds can: `toMillis` raises on anything from
     * about 292 million years up, and a negative one is a number no delay can be waited for.
     * Neither is a duration a caller should write and both are durations a caller can write, so
     * each is brought inside what can be counted — the largest count there is, and zero — rather
     * than raised from the middle of a send.
     *
     * This is the one place a delay of a policy becomes a number, so what the schedule waits and
     * what the client states in `Hook0-Client-Options` cannot come to disagree: they are the same
     * conversion.
     *
     * Nothing here reads a delay that is not a number. The shared rule across the SDKs is that an
     * infinite or not-a-number delay is read as the default of the field holding it; a [Duration] is
     * a count of seconds and a count of nanoseconds, both whole, and no factory on it takes a
     * floating-point number — so there is no value to read that way, and a branch for one would be
     * unreachable.
     *
     * @param held the delay to count
     * @return that delay in whole milliseconds, never negative
     */
    private fun millis(held: Duration): Long {
      if (held.isNegative) {
        return 0
      }
      val seconds = held.seconds
      if (seconds > (Long.MAX_VALUE - 999) / 1000) {
        return Long.MAX_VALUE
      }
      return seconds * 1000 + held.nano / 1_000_000
    }

    /**
     * Most attempts a policy can ever make, whatever [maxAttempts] says.
     *
     * A policy is configuration, and configuration can be wrong; this cap keeps a mistyped
     * [maxAttempts] from turning one send into an unbounded series of requests.
     */
    const val MAX_ATTEMPTS_CAP = 16

    /** Beyond this many doublings any backoff has long since reached its ceiling. */
    const val MAX_BACKOFF_DOUBLINGS = 30

    /**
     * The schedule a client applies when the caller names none.
     *
     * @return four attempts spread over at most five seconds
     */
    fun defaults(): RetryPolicy =
      RetryPolicy(4, Duration.ofMillis(100), Duration.ofMillis(2000), Duration.ofMillis(5000))

    /**
     * A policy that never retries: one attempt, and the caller hears what it answered.
     *
     * @return the schedule of a single attempt
     */
    fun disabled(): RetryPolicy = RetryPolicy(1, Duration.ZERO, Duration.ZERO, Duration.ZERO)

    /**
     * The draw for one retry, brought back inside `[0, 1]` whatever the randomness gave.
     *
     * @param draws what the randomness gave
     * @param index which retry, counting from zero
     * @return a draw inside the unit interval
     */
    fun draw(draws: DoubleArray, index: Int): Double {
      if (index < 0 || index >= draws.size) {
        return 1.0
      }
      val drawn = draws[index]
      if (!drawn.isFinite()) {
        return 1.0
      }
      return drawn.coerceIn(0.0, 1.0)
    }
  }
}
