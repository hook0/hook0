package com.hook0.client;

import java.time.Duration;
import java.util.ArrayList;
import java.util.List;

/**
 * How a client spaces out the attempts of a single send.
 *
 * <p>The delay before a retry doubles from {@link #initialBackoff()} and is capped by {@link #maxBackoff()}; the delay
 * actually waited is then drawn anywhere between zero and that ceiling, so that emitters which failed at the same
 * moment do not come back at the same moment. Retrying stops as soon as the delays of the send would add up to more
 * than {@link #maxTotalDelay()}.
 *
 * <p>The defaults are four attempts spread over at most five seconds: three retries absorb the blips a webhook emitter
 * meets in production — a connection reset, a rolling deployment answering 503 — without holding the caller for long,
 * and the five-second budget bounds what the worst send costs whatever the individual delays turn out to be.
 *
 * @param maxAttempts attempts a single send makes at most, the first one included; {@code 1} disables retrying
 * @param initialBackoff ceiling of the delay before the first retry
 * @param maxBackoff ceiling no single delay ever exceeds
 * @param maxTotalDelay budget all the delays of one send share
 */
public record RetryPolicy(int maxAttempts, Duration initialBackoff, Duration maxBackoff, Duration maxTotalDelay) {

  /**
   * Most attempts a policy can ever make, whatever {@link #maxAttempts()} says.
   *
   * <p>A policy is configuration, and configuration can be wrong; this cap keeps a mistyped {@code maxAttempts} from
   * turning one send into an unbounded series of requests.
   */
  public static final int MAX_ATTEMPTS_CAP = 16;

  /** Beyond this many doublings any backoff has long since reached its ceiling. */
  public static final int MAX_BACKOFF_DOUBLINGS = 30;

  /**
   * The schedule a client applies when the caller names none.
   *
   * @return four attempts spread over at most five seconds
   */
  public static RetryPolicy defaults() {
    return new RetryPolicy(4, Duration.ofMillis(100), Duration.ofMillis(2000), Duration.ofMillis(5000));
  }

  /**
   * A policy that never retries: one attempt, and the caller hears what it answered.
   *
   * @return the schedule of a single attempt
   */
  public static RetryPolicy disabled() {
    return new RetryPolicy(1, Duration.ZERO, Duration.ZERO, Duration.ZERO);
  }

  /**
   * Attempts this policy actually makes: {@link #maxAttempts()}, brought inside {@code 1..MAX_ATTEMPTS_CAP}.
   *
   * @return how many requests one send issues at most
   */
  public int attempts() {
    return Math.clamp(maxAttempts, 1, MAX_ATTEMPTS_CAP);
  }

  /**
   * Ceiling of the delay before retry number {@code retryNumber}, where {@code 1} is the first retry.
   *
   * <p>It doubles from {@link #initialBackoff()} and never exceeds {@link #maxBackoff()}, so the ceilings of successive
   * retries never decrease.
   *
   * @param retryNumber which retry, counting from one
   * @return how long that retry may wait at most
   */
  public long backoffCeilingMillis(int retryNumber) {
    int doublings = Math.clamp((long) retryNumber - 1, 0, MAX_BACKOFF_DOUBLINGS);
    long ceiling = millis(maxBackoff);
    long doubled = millis(initialBackoff);
    for (int step = 0; step < doublings && doubled < ceiling; step++) {
      // Doubling past the ceiling lands on it. Written as a comparison rather than as a doubling
      // that is then capped, because the doubling itself overflows for a ceiling near the largest
      // count there is, and a wrapped one reads as a delay of nothing at all.
      doubled = doubled > ceiling / 2 ? ceiling : doubled * 2;
    }
    return Math.clamp(doubled, 0, ceiling);
  }

  /**
   * One delay of this policy, in the whole milliseconds it is counted in.
   *
   * <p>A {@link Duration} holds more than a count of milliseconds can: {@link Duration#toMillis()} raises on anything
   * from about 292 million years up, and a negative one is a number no delay can be waited for. Neither is a duration
   * a caller should write and both are durations a caller can write, so each is brought inside what can be counted —
   * the largest count there is, and zero — rather than raised from the middle of a send.
   *
   * <p>This is the one place a delay of this policy becomes a number, so what the schedule waits and what the client
   * states in {@code Hook0-Client-Options} cannot come to disagree: they are the same conversion.
   *
   * <p>Nothing here reads a delay that is not a number. The shared rule across the SDKs is that an infinite or
   * not-a-number delay is read as the default of the field holding it; a {@link Duration} is a count of seconds and a
   * count of nanoseconds, both whole, and no factory on it takes a floating-point number — so there is no value to
   * read that way, and a branch for one would be unreachable.
   *
   * @param held the delay to count
   * @return that delay in whole milliseconds, never negative
   */
  static long millis(Duration held) {
    if (held.isNegative()) {
      return 0;
    }
    long seconds = held.getSeconds();
    if (seconds > (Long.MAX_VALUE - 999) / 1000) {
      return Long.MAX_VALUE;
    }
    return seconds * 1000 + held.getNano() / 1_000_000;
  }

  /**
   * The delays this policy waits between the attempts of one send, one per retry.
   *
   * <p>Each delay lands between zero and the ceiling of its retry, and the schedule is cut short as soon as the next
   * delay would spend more than {@link #maxTotalDelay()}. There are therefore at most {@code attempts() - 1} delays,
   * and they add up to at most {@code maxTotalDelay}.
   *
   * <p>A draw that is missing or is not a finite number is read as {@code 1}, which asks for the whole ceiling: an
   * unusable source of randomness makes the client wait longer, never less.
   *
   * @param draws one draw in {@code [0, 1)} per retry
   * @return how long to wait before each retry, in milliseconds
   */
  public List<Long> delaysMillis(double[] draws) {
    long budget = millis(maxTotalDelay);
    List<Long> waits = new ArrayList<>();
    long spent = 0;

    for (int retryNumber = 1; retryNumber <= attempts() - 1; retryNumber++) {
      long delay = Math.round(backoffCeilingMillis(retryNumber) * draw(draws, retryNumber - 1));
      if (spent + delay > budget) {
        return List.copyOf(waits);
      }
      spent += delay;
      waits.add(Long.valueOf(delay));
    }

    return List.copyOf(waits);
  }

  /**
   * The draw for one retry, brought back inside {@code [0, 1]} whatever the randomness gave.
   *
   * @param draws what the randomness gave
   * @param index which retry, counting from zero
   * @return a draw inside the unit interval
   */
  public static double draw(double[] draws, int index) {
    if (draws == null || index < 0 || index >= draws.length) {
      return 1.0;
    }
    double drawn = draws[index];
    if (!Double.isFinite(drawn)) {
      return 1.0;
    }
    return Math.clamp(drawn, 0.0, 1.0);
  }
}
