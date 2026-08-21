<?php

declare(strict_types=1);

namespace Hook0;

/**
 * How a client spaces out the attempts of a single send.
 *
 * The delay before a retry doubles from `$initialBackoff` and is capped by `$maxBackoff`; the delay
 * actually waited is then drawn anywhere between zero and that ceiling, so that emitters which
 * failed at the same moment do not come back at the same moment. Retrying stops as soon as the
 * delays of the send would add up to more than `$maxTotalDelay`.
 *
 * The defaults are four attempts spread over at most five seconds: three retries absorb the blips a
 * webhook emitter meets in production — a connection reset, a rolling deployment answering 503 —
 * without holding the caller for long, and the five-second budget bounds what the worst send costs
 * whatever the individual delays turn out to be.
 */
final class RetryPolicy
{
    /**
     * Most attempts a policy can ever make, whatever `$maxAttempts` says.
     *
     * A policy is configuration, and configuration can be wrong; this cap keeps a mistyped
     * `maxAttempts` from turning one send into an unbounded series of requests.
     */
    public const MAX_ATTEMPTS_CAP = 16;

    /** Beyond this many doublings any backoff has long since reached its ceiling. */
    private const MAX_BACKOFF_DOUBLINGS = 30;

    /**
     * What each duration of a policy is where a caller named none, in seconds.
     *
     * Declared here rather than written into the signature below, because they are also what a
     * duration falls back to when a caller names one no schedule could be built on: a fallback
     * spelled out a second time is one that will disagree with the default the first time either
     * moves.
     */
    public const DEFAULT_INITIAL_BACKOFF = 0.1;
    public const DEFAULT_MAX_BACKOFF = 2.0;
    public const DEFAULT_MAX_TOTAL_DELAY = 5.0;

    /**
     * @param int $maxAttempts attempts a single send makes at most, the first one included; `1`
     *   disables retrying
     * @param float $initialBackoff ceiling of the delay before the first retry, in seconds
     * @param float $maxBackoff ceiling no single delay ever exceeds, in seconds
     * @param float $maxTotalDelay budget all the delays of one send share, in seconds
     */
    public function __construct(
        public readonly int $maxAttempts = 4,
        public readonly float $initialBackoff = self::DEFAULT_INITIAL_BACKOFF,
        public readonly float $maxBackoff = self::DEFAULT_MAX_BACKOFF,
        public readonly float $maxTotalDelay = self::DEFAULT_MAX_TOTAL_DELAY,
    ) {
    }

    /**
     * The delay before the first retry this policy is in force with, in seconds.
     *
     * What a send waits and what a request states are both read from here, so the two cannot come
     * to describe different policies.
     */
    public function initialBackoffInForce(): float
    {
        return self::inForce($this->initialBackoff, self::DEFAULT_INITIAL_BACKOFF);
    }

    /** The ceiling no single delay of this policy exceeds, in seconds. */
    public function maxBackoffInForce(): float
    {
        return self::inForce($this->maxBackoff, self::DEFAULT_MAX_BACKOFF);
    }

    /** The budget all the delays of one send share, in seconds. */
    public function maxTotalDelayInForce(): float
    {
        return self::inForce($this->maxTotalDelay, self::DEFAULT_MAX_TOTAL_DELAY);
    }

    /** A policy that never retries: one attempt, and the caller hears what it answered. */
    public static function disabled(): self
    {
        return new self(
            maxAttempts: 1,
            initialBackoff: 0.0,
            maxBackoff: 0.0,
            maxTotalDelay: 0.0
        );
    }

    /** Attempts this policy actually makes: `$maxAttempts`, brought inside `1..MAX_ATTEMPTS_CAP`. */
    public function attempts(): int
    {
        return min(max($this->maxAttempts, 1), self::MAX_ATTEMPTS_CAP);
    }

    /**
     * Ceiling of the delay before retry number `$retryNumber`, where `1` is the first retry.
     *
     * It doubles from `$initialBackoff` and never exceeds `$maxBackoff`, so the ceilings of
     * successive retries never decrease.
     */
    public function backoffCeiling(int $retryNumber): float
    {
        $doublings = min(max($retryNumber - 1, 0), self::MAX_BACKOFF_DOUBLINGS);
        $ceiling = $this->maxBackoffInForce();
        $wanted = $this->initialBackoffInForce() * (2 ** $doublings);

        return min($wanted, $ceiling);
    }

    /**
     * The delays this policy waits between the attempts of one send, one per retry.
     *
     * Each delay lands between zero and the ceiling of its retry, and the schedule is cut short as
     * soon as the next delay would spend more than `$maxTotalDelay`. There are therefore at most
     * `attempts() - 1` delays, and they add up to at most `$maxTotalDelay`.
     *
     * A draw that is missing or is not a finite number is read as `1`, which asks for the whole
     * ceiling: an unusable source of randomness makes the client wait longer, never less.
     *
     * @param list<float> $draws one draw in `[0, 1)` per retry
     * @return list<float>
     */
    public function delays(array $draws): array
    {
        $budget = $this->maxTotalDelayInForce();
        $waits = [];
        $spent = 0.0;

        for ($retryNumber = 1; $retryNumber < $this->attempts(); $retryNumber++) {
            $delay = $this->backoffCeiling($retryNumber) * self::draw($draws, $retryNumber - 1);
            if ($spent + $delay > $budget) {
                break;
            }

            $spent += $delay;
            $waits[] = $delay;
        }

        return $waits;
    }

    /**
     * The draw for one retry, brought back inside `[0, 1]` whatever the randomness gave.
     *
     * @param list<float> $draws
     */
    private static function draw(array $draws, int $index): float
    {
        $drawn = $draws[$index] ?? null;
        if (!is_float($drawn) && !is_int($drawn)) {
            return 1.0;
        }
        if (!is_finite((float) $drawn)) {
            return 1.0;
        }

        return min(max((float) $drawn, 0.0), 1.0);
    }

    /**
     * A number of seconds a caller set, brought back to something a schedule can be built on.
     *
     * A value that is not a finite number names no duration at all, and it is read as the one an
     * unconfigured policy holds. Nothing is the tempting reading and the wrong one: a policy whose
     * delays collapse to zero fires its whole schedule back to back, which is the burst a client
     * states its policy so that an instance could recognise — it would manufacture the very traffic
     * the header exists to explain. Unbounded is worse: a send that never comes back. The default
     * is bounded, is what every client falls back to, and leaves the client behaving the way an
     * unconfigured one does, which is what an unusable value should buy.
     *
     * A negative number is a real duration somebody wrote rather than an unusable one, and keeps
     * being read as nothing.
     */
    private static function inForce(float $seconds, float $default): float
    {
        if (!is_finite($seconds)) {
            return $default;
        }

        return max($seconds, 0.0);
    }
}
