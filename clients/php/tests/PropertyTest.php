<?php

declare(strict_types=1);

namespace Hook0\Tests;

use Hook0\Client;
use Hook0\ClientError;
use Hook0\DecodeError;
use Hook0\RetryPolicy;
use Hook0\Runtime;
use Hook0\Signature;
use Hook0\Tests\Support\Contract;
use Hook0\Tests\Support\GeneratedSurface;
use PHPUnit\Framework\TestCase;

/**
 * What holds for every input, rather than for the ones a case happened to pick.
 *
 * Four things are checked here. A retry schedule never spends more than the policy that produced it
 * allows, whichever way the randomness fell. Reading a signature header answers with the one failure
 * this package declares, whatever text reached the endpoint, and never with anything else. A value
 * read out of a document the API could answer is written back as the value that was read. And
 * identifiers minted in sequence never go back in time.
 *
 * There is no property-testing tool in the language's own library, and this package installs nothing
 * at runtime, so the search is written here: a fixed seed, a bounded number of draws, and the
 * counter-examples worth keeping committed under `regressions/` so they run as ordinary cases on
 * every pipeline. A failing draw is one somebody can reproduce by running the suite again rather
 * than one that goes away on a retry.
 */
final class PropertyTest extends TestCase
{
    /** What the draws are made from. Fixed, so the suite explores the same inputs everywhere it runs. */
    private const SEED = 20260814;

    /** How many draws each property makes. Bounded, so a pipeline can never be held by one. */
    private const DRAWS = 200;

    /**
     * How far two sums of the same floats may sit apart before the difference is a defect rather than
     * the order they were added in.
     */
    private const ROUNDING = 1e-9;

    /** The bounds a drawn policy is built inside. */
    private const MAX_DRAWN_ATTEMPTS = 64;
    private const MAX_DRAWN_SECONDS = 10.0;
    private const MAX_DRAWN_BUDGET = 60.0;

    /** Longest header a draw builds, counted in the pieces a signature is made of. */
    private const MAX_DRAWN_HEADER = 96;

    /** How many mutations of one committed document a draw makes. */
    private const MUTATIONS = 4;

    protected function setUp(): void
    {
        mt_srand(self::SEED);
    }

    public function testARetryScheduleStaysWithinEveryBoundOfItsPolicy(): void
    {
        $cases = Contract::regressions('retry_policies');
        for ($index = 0; $index < self::DRAWS; $index++) {
            $cases[] = $this->drawnPolicy();
        }

        foreach ($cases as $case) {
            [$maxAttempts, $initialBackoff, $maxBackoff, $maxTotalDelay, $draws] = $case;
            $policy = new RetryPolicy(
                maxAttempts: (int) $maxAttempts,
                initialBackoff: (float) $initialBackoff,
                maxBackoff: (float) $maxBackoff,
                maxTotalDelay: (float) $maxTotalDelay
            );

            $this->holdsFor($policy, array_map($this->unusable(...), $draws));
        }
    }

    public function testReadingASignatureAnswersWithTheOneFailureThisPackageDeclares(): void
    {
        $headers = Contract::regressions('signatures');
        for ($index = 0; $index < self::DRAWS; $index++) {
            $headers[] = $this->drawnHeader();
        }

        foreach ($headers as $header) {
            self::assertIsString($header);

            try {
                Signature::parse($header);
            } catch (ClientError) {
                continue;
            } catch (\Throwable $unexpected) {
                self::fail(sprintf(
                    'reading `%s` answered %s: %s',
                    $header,
                    $unexpected::class,
                    $unexpected->getMessage()
                ));
            }

            // Parsing answered, so verifying has to answer the same way: a header that reads must not
            // find a way to fail that a caller cannot name.
            try {
                Signature::verifyWithCurrentTime(
                    $header,
                    '',
                    [],
                    'secret',
                    300.0,
                    new \DateTimeImmutable('@0')
                );
            } catch (ClientError) {
                continue;
            } catch (\Throwable $unexpected) {
                self::fail(sprintf(
                    'verifying `%s` answered %s: %s',
                    $header,
                    $unexpected::class,
                    $unexpected->getMessage()
                ));
            }
        }
    }

    public function testAGeneratedTypeReadsBackWhatItWrote(): void
    {
        $documents = $this->documents();

        foreach (GeneratedSurface::models() as $model) {
            foreach ($documents as $document) {
                try {
                    $read = $model::fromJson($document);
                } catch (DecodeError) {
                    continue;
                }

                $written = $read->toArray();

                self::assertTrue(
                    $read->equals($model::fromJson($written)),
                    sprintf('a %s read out of %s does not read back', $model, Runtime::encode($document))
                );
                self::assertSame(
                    Runtime::encode($written),
                    Runtime::encode($model::fromJson($written)->toArray()),
                    sprintf('a %s written back twice is not written back the same way', $model)
                );
            }
        }
    }

    public function testReadingADocumentAnswersWithTheOneFailureTheRuntimeDeclares(): void
    {
        $documents = $this->documents();
        $exercised = 0;

        foreach (GeneratedSurface::models() as $model) {
            foreach ($documents as $document) {
                $exercised++;

                try {
                    $model::fromJson($document);
                } catch (DecodeError) {
                    continue;
                } catch (\Throwable $unexpected) {
                    self::fail(sprintf(
                        'reading %s as a %s answered %s: %s',
                        Runtime::encode($document),
                        $model,
                        $unexpected::class,
                        $unexpected->getMessage()
                    ));
                }
            }
        }

        // A search that read nothing would pass on every document there is.
        self::assertGreaterThan(0, $exercised);
    }

    public function testMintedIdentifiersCarryAMomentThatNeverGoesBack(): void
    {
        $moments = [];
        for ($index = 0; $index < self::DRAWS; $index++) {
            $minted = Client::generateEventId();
            $moments[] = substr($minted, 0, 8) . substr($minted, 9, 4);
        }

        $sorted = $moments;
        sort($sorted);

        self::assertSame($sorted, $moments);
    }

    /**
     * @param list<float> $draws
     */
    private function holdsFor(RetryPolicy $policy, array $draws): void
    {
        $delays = $policy->delays($draws);
        $budget = max($policy->maxTotalDelay, 0.0);

        self::assertGreaterThanOrEqual(1, $policy->attempts());
        self::assertLessThanOrEqual(RetryPolicy::MAX_ATTEMPTS_CAP, $policy->attempts());
        self::assertLessThanOrEqual($policy->attempts() - 1, count($delays));
        self::assertLessThanOrEqual($budget + self::ROUNDING, array_sum($delays));

        foreach ($delays as $index => $delay) {
            self::assertGreaterThanOrEqual(0.0, $delay);
            self::assertLessThanOrEqual($policy->backoffCeiling($index + 1) + self::ROUNDING, $delay);
            self::assertLessThanOrEqual(max($policy->maxBackoff, 0.0) + self::ROUNDING, $delay);
        }

        // A schedule never hurries up as it goes: the ceiling of a retry never sits below the one
        // before it.
        $ceilings = [];
        for ($retryNumber = 1; $retryNumber <= $policy->attempts(); $retryNumber++) {
            $ceilings[] = $policy->backoffCeiling($retryNumber);
        }
        $sorted = $ceilings;
        sort($sorted);

        self::assertSame($sorted, $ceilings);
    }

    /** A draw that is no draw at all, which has to make the client wait longer rather than less. */
    private function unusable(mixed $drawn): mixed
    {
        return match ($drawn) {
            'nan' => NAN,
            'infinity' => INF,
            '-infinity' => -INF,
            default => $drawn,
        };
    }

    /**
     * @return array{0: int, 1: float, 2: float, 3: float, 4: list<float>}
     */
    private function drawnPolicy(): array
    {
        $draws = [];
        for ($index = 0, $count = mt_rand(0, 8); $index < $count; $index++) {
            $draws[] = ($this->unitDraw() * 2) - 0.5;
        }

        return [
            mt_rand(-4, self::MAX_DRAWN_ATTEMPTS),
            $this->unitDraw() * self::MAX_DRAWN_SECONDS,
            $this->unitDraw() * self::MAX_DRAWN_SECONDS,
            $this->unitDraw() * self::MAX_DRAWN_BUDGET,
            $draws,
        ];
    }

    /**
     * A header built out of the pieces a signature is made of, put together every way a sender that
     * is not Hook0 might put them together.
     */
    private function drawnHeader(): string
    {
        $pieces = ['t', 'v0', 'v1', 'h', '=', ',', '0', '9', 'zz', 'abc', 'x-event-id',
            '1800000000', '-1', '"', ' ', '.', '{', '}'];

        $header = '';
        for ($index = 0, $count = mt_rand(0, self::MAX_DRAWN_HEADER); $index < $count; $index++) {
            $header .= $pieces[mt_rand(0, count($pieces) - 1)];
        }

        return $header;
    }

    /**
     * The committed documents and a few mutations of each: a member taken away, replaced by something
     * of another type, buried inside something else, or answered as nothing.
     *
     * @return list<mixed>
     */
    private function documents(): array
    {
        $documents = Contract::regressions('documents');

        $mutated = [];
        foreach ($documents as $document) {
            for ($index = 0; $index < self::MUTATIONS; $index++) {
                $mutated[] = $this->mutated($document);
            }
        }

        return array_merge($documents, $mutated);
    }

    private function mutated(mixed $document): mixed
    {
        if (!is_array($document) || $document === []) {
            return $document;
        }

        $keys = array_keys($document);
        $key = $keys[mt_rand(0, count($keys) - 1)];

        switch (mt_rand(0, 3)) {
            case 0:
                unset($document[$key]);

                return $document;
            case 1:
                $document[$key] = mt_rand(0, 1000);

                return $document;
            case 2:
                $document[$key] = [$document[$key]];

                return $document;
            default:
                $document[$key] = null;

                return $document;
        }
    }

    /** One draw in `[0, 1]`, from the generator the seed above settled. */
    private function unitDraw(): float
    {
        return mt_rand() / mt_getrandmax();
    }
}
