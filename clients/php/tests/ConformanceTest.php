<?php

declare(strict_types=1);

namespace Hook0\Tests;

use Hook0\Client;
use Hook0\ClientError;
use Hook0\Options;
use Hook0\RetryPolicy;
use Hook0\Signature;
use Hook0\Tests\Support\ApiCase;
use Hook0\Tests\Support\Contract;
use Hook0\Tests\Support\ReceivedRequest;
use Hook0\Tests\Support\ScriptedResponse;

/**
 * The cases the shared conformance corpus dictates, run against this client.
 *
 * Nothing below writes down a verdict, a bound or a signature of its own: they are read out of the
 * committed documents and this client is driven against them over a real socket. A case added to the
 * corpus is therefore exercised here without this file being touched, and a verdict changed there
 * fails here until this client agrees with it again.
 */
final class ConformanceTest extends ApiCase
{
    private const INGESTED_ID = '01961234-5678-7abc-8def-0123456789ac';

    /** The schedule a case that is not about waiting spends between attempts, in seconds. */
    private const PROMPT_BACKOFF = 0.005;

    /**
     * The budget the delay cases share. A delay the API names above it is expected to be cut down to
     * it, so this also bounds what those cases cost.
     */
    private const DELAY_BUDGET = 1.1;

    /**
     * What a wait may overshoot by before it is read as more than what was asked for: a loopback
     * round trip, a timer and a scheduler all sit inside it.
     */
    private const DELAY_SLACK = 0.6;

    /**
     * How a refusal the corpus names reads in this client's own words. Every name the corpus declares
     * is looked up here, so one added there stops this suite until it is mapped rather than passing
     * under whatever the client happened to say.
     */
    private const REFUSALS = [
        'code_not_hexadecimal' => 'not hexadecimal',
        'header_not_delivered' => 'was not delivered',
        'code_mismatch' => 'does not match',
        'outside_tolerance' => 'outside the',
    ];

    public function testTheCorpusSaysWhatEveryProblemDoesToASend(): void
    {
        // The status is not what decides: the corpus carries problems answering the same status with
        // opposite verdicts, and a client reading the status alone fails half of them.
        foreach (Contract::of('retry.json')['problems'] as $rule) {
            [$issued, $ingested] = $this->issuedFor($this->refusal($rule['status'], $rule['problem']));
            $expected = $rule['retryable'] ? 2 : 1;

            self::assertSame($expected, $issued, sprintf(
                '`%s` under %d issued %d requests where the corpus expects %d: %s',
                $rule['problem'],
                $rule['status'],
                $issued,
                $expected,
                $rule['reason']
            ));
            self::assertSame($rule['retryable'], $ingested);
        }
    }

    public function testTheCorpusSaysWhatEveryStatusDoesToASend(): void
    {
        // A body naming no problem this client could read is also what an older client meets when the
        // API names a problem it has never heard of.
        foreach (Contract::of('retry.json')['statuses'] as $rule) {
            $answer = $this->refusal($rule['status'], 'AProblemThisClientHasNeverHeardOf');
            [$issued] = $this->issuedFor($answer);
            $expected = $rule['retryable'] ? 2 : 1;

            self::assertSame($expected, $issued, sprintf(
                'a status of %d issued %d requests where the corpus expects %d: %s',
                $rule['status'],
                $issued,
                $expected,
                $rule['reason']
            ));
        }
    }

    public function testTheCorpusSaysWhatARequestTheApiNeverAnsweredDoes(): void
    {
        // Every cause the corpus names is provoked for real rather than reported: a server that sits
        // on an answer past the timeout, an answer above a ceiling this client set for itself, and a
        // URL nothing can be sent to.
        foreach (Contract::of('retry.json')['transport']['causes'] as $rule) {
            [$issued, $survived] = $this->provoked($rule['cause']);
            $expected = $rule['retryable'] ? 2 : 1;

            self::assertSame($expected, $issued, sprintf(
                '`%s` issued %d requests where the corpus expects %d: %s',
                $rule['cause'],
                $issued,
                $expected,
                $rule['reason']
            ));
            self::assertSame($rule['retryable'], $survived);
        }
    }

    public function testTheDelayTheApiNamesIsHonouredAndBounded(): void
    {
        // The header is written by the other end, so honouring it whole would hand a stranger the
        // length of this client's send. What the corpus asks for is that a delay be waited out when
        // the budget can afford it and cut down to what is left of the budget when it cannot.
        $retryAfter = Contract::of('retry.json')['retry_after'];

        foreach ($retryAfter['cases'] as $delay) {
            $waited = $this->waitedFor($retryAfter['header'], $delay['header']);
            $expected = $delay['honoured'] ? min((float) $delay['seconds'], self::DELAY_BUDGET) : 0.0;

            self::assertGreaterThanOrEqual($expected, $waited, sprintf(
                '`%s: %s` was retried after %.3fs, sooner than the %.3fs it asked for',
                $retryAfter['header'],
                $delay['header'],
                $waited,
                $expected
            ));
            self::assertLessThanOrEqual($expected + self::DELAY_SLACK, $waited, sprintf(
                '`%s: %s` held the send for %.3fs, above the %.3fs it is bounded to',
                $retryAfter['header'],
                $delay['header'],
                $waited,
                $expected
            ));
        }
    }

    public function testTheBoundsAreTheOnesTheCorpusNames(): void
    {
        // This client's defaults, held against the one place the numbers are written down. What is
        // asserted is read from the corpus rather than listed here, so a bound added there and left
        // unapplied fails instead of passing unnoticed.
        $built = new Client('http://127.0.0.1:1', self::APPLICATION_ID, self::TOKEN);
        $policy = $built->options->retryPolicy;
        $applied = [
            'max_attempts' => (float) $policy->maxAttempts,
            'max_attempts_cap' => (float) RetryPolicy::MAX_ATTEMPTS_CAP,
            'initial_backoff_ms' => $policy->initialBackoff * 1000,
            'max_backoff_ms' => $policy->maxBackoff * 1000,
            'max_total_delay_ms' => $policy->maxTotalDelay * 1000,
            'request_timeout_ms' => $built->options->requestTimeout * 1000,
            'max_payload_bytes' => (float) $built->options->maxPayloadBytes,
            'max_response_bytes' => (float) $built->options->maxResponseBytes,
            'max_head_bytes' => (float) $built->options->maxHeadBytes,
            'max_response_headers' => (float) $built->options->maxResponseHeaders,
            'max_header_bytes' => (float) $built->options->maxHeaderBytes,
        ];

        $bounds = Contract::of('bounds.json')['bounds'];
        $unapplied = array_values(array_diff(array_keys($bounds), array_keys($applied)));

        self::assertSame([], $unapplied, sprintf(
            'the corpus names %s, which this client does not apply: a bound added to the contract '
            . 'and left unread here keeps the suite green over half a contract',
            implode(', ', array_map(static fn (string $name): string => '`' . $name . '`', $unapplied))
        ));
        foreach ($bounds as $name => $wanted) {
            self::assertEqualsWithDelta((float) $wanted, $applied[$name], 0.001, $name);
        }
    }

    public function testEveryHeaderTheCorpusNamesTravels(): void
    {
        $contract = Contract::of('request.json');

        $this->api->willAnswer(
            $this->ingested(self::INGESTED_ID),
            new ScriptedResponse(200, [['event_type_name' => 'auth.user.create']])
        );

        $client = $this->client();
        $client->sendEvent($this->anEvent());
        $client->upsertEventTypes(['auth.user.create']);

        $received = $this->api->received();
        self::assertCount(2, $received);
        [$carryingABody, $carryingNone] = $received;

        $composedAtMost = (int) $contract['max_composed_bytes'];
        $bound = ['token' => self::TOKEN, 'language' => 'php'];

        foreach ($contract['headers'] as $header) {
            self::assertContains($header['when'], $contract['occasions'], sprintf(
                'the corpus names an occasion `%s` this suite does not know how to build',
                $header['when']
            ));

            $name = strtolower($header['name']);
            $chunks = self::templateChunks($header['value'], $bound);

            $this->carried($carryingABody, $header, $chunks, $composedAtMost);
            if ($header['when'] === 'every request') {
                $this->carried($carryingNone, $header, $chunks, $composedAtMost);
                continue;
            }
            self::assertArrayNotHasKey($name, $carryingNone->headers, $header['reason']);
        }
    }

    public function testEveryRefusalTheCorpusDeclaresReadsAsOneOfThisClients(): void
    {
        // A refusal named in the corpus and mapped to nothing here would pass under any wording.
        self::assertSame(
            [],
            array_values(array_diff(Contract::of('signature.json')['refusals'], array_keys(self::REFUSALS)))
        );
    }

    public function testEveryDeliveryOfTheCorpusIsVerifiedAsItSays(): void
    {
        // A refused delivery has to be refused for the reason the corpus names: a client that computed
        // a code over a header that never arrived and reported a mismatch would otherwise look right.
        foreach (Contract::of('signature.json')['vectors'] as $vector) {
            if ($vector['verdict'] === 'accepted') {
                $this->verified($vector);
                self::assertTrue(true, $vector['reason']);
                continue;
            }

            try {
                $this->verified($vector);
                self::fail(sprintf('`%s` was accepted: %s', $vector['name'], $vector['reason']));
            } catch (ClientError $refused) {
                self::assertStringContainsString(
                    self::REFUSALS[$vector['refusal']],
                    $refused->getMessage(),
                    sprintf(
                        'a delivery the corpus refuses as `%s` was answered `%s`: %s',
                        $vector['refusal'],
                        $refused->getMessage(),
                        $vector['reason']
                    )
                );
            }
        }
    }

    /**
     * Holds one header of one request that reached the socket against what the corpus says of it.
     *
     * @param array<string, mixed> $header the entry the corpus declares that header under
     * @param list<string> $chunks the literal text of its value, with the holes this suite fills in
     */
    private function carried(
        ReceivedRequest $request,
        array $header,
        array $chunks,
        int $composedAtMost
    ): void {
        $name = strtolower($header['name']);
        $written = $request->headers[$name] ?? '';

        self::assertTrue(self::matchesChunks($chunks, $written), sprintf(
            'the request carried `%s: %s` where the corpus says `%s`: %s',
            $name,
            $written,
            $header['value'],
            $header['reason']
        ));

        // A value with a hole this suite cannot fill is one the client composed out of what the
        // platform told it, and what the platform says is as long as it feels like.
        if (count($chunks) > 1) {
            self::assertLessThanOrEqual($composedAtMost, strlen($written), sprintf(
                'the request carried %d bytes of `%s`, above the %d the corpus cuts a composed '
                . 'value to',
                strlen($written),
                $name,
                $composedAtMost
            ));
        }
    }

    /**
     * What a value of the request document is made of, once the holes this suite can speak for are
     * filled in.
     *
     * A value is a template: `${name}` is a hole and everything around it is literal. A hole named in
     * `$bound` becomes part of the literal text around it; one that is not is a hole no suite can
     * fill without reimplementing the client it is testing, and it separates two chunks. A template
     * whose holes are all bound is therefore one chunk, and the whole value is that chunk.
     *
     * @param array<string, string> $bound what each hole this suite can speak for carries
     * @return list<string>
     */
    private static function templateChunks(string $template, array $bound): array
    {
        $chunks = [''];
        $rest = $template;

        while (($opened = strpos($rest, '${')) !== false) {
            $closed = strpos($rest, '}', $opened);
            if ($closed === false) {
                break;
            }

            $last = count($chunks) - 1;
            $chunks[$last] .= substr($rest, 0, $opened);
            $name = substr($rest, $opened + 2, $closed - $opened - 2);

            if (array_key_exists($name, $bound)) {
                $chunks[$last] .= $bound[$name];
            } else {
                $chunks[] = '';
            }
            $rest = substr($rest, $closed + 1);
        }

        $chunks[count($chunks) - 1] .= $rest;

        return $chunks;
    }

    /**
     * Whether what arrived is what those chunks describe: the literal text in order, anchored at both
     * ends, with something non-empty standing in every hole between them.
     *
     * @param list<string> $chunks
     */
    private static function matchesChunks(array $chunks, string $carried): bool
    {
        if (count($chunks) === 1) {
            return $carried === $chunks[0];
        }
        if (!str_starts_with($carried, $chunks[0])) {
            return false;
        }

        $rest = substr($carried, strlen($chunks[0]));
        foreach (array_slice($chunks, 1, -1) as $chunk) {
            // A hole stands before this chunk, and nothing is not something, so the search starts
            // past whatever fills it.
            $found = $rest === '' ? false : strpos($rest, $chunk, 1);
            if ($found === false) {
                return false;
            }
            $rest = substr($rest, $found + strlen($chunk));
        }

        $last = $chunks[count($chunks) - 1];

        return strlen($rest) > strlen($last) && str_ends_with($rest, $last);
    }

    /**
     * One cause of a request the API never answered, provoked over a real socket: how many requests
     * the send issued, and whether it ended up ingesting the event.
     *
     * @return array{0: int, 1: bool}
     */
    private function provoked(string $cause): array
    {
        return match ($cause) {
            'no_answer' => $this->provokedByNoAnswer(),
            'answer_above_a_bound' => $this->provokedByAnAnswerAboveABound(),
            'unusable_api_url' => $this->provokedByAnUnusableApiUrl(),
            default => throw new \RuntimeException(sprintf(
                'the corpus names a cause `%s` this suite does not know how to provoke',
                $cause
            )),
        };
    }

    /**
     * An attempt that runs out of time before the API writes anything.
     *
     * @return array{0: int, 1: bool}
     */
    private function provokedByNoAnswer(): array
    {
        $this->restarted();
        $this->api->willAnswer(
            $this->ingested(self::INGESTED_ID, heldFor: 1.0),
            $this->ingested(self::INGESTED_ID)
        );

        return $this->issuedBy(
            $this->client($this->options(maxAttempts: 4, requestTimeout: 0.2))
        );
    }

    /**
     * An answer larger than what this client agreed to read off the socket.
     *
     * @return array{0: int, 1: bool}
     */
    private function provokedByAnAnswerAboveABound(): array
    {
        $this->restarted();
        $oversized = new ScriptedResponse(
            201,
            ['event_id' => self::INGESTED_ID, 'padding' => str_repeat('x', 2048)]
        );
        $this->api->willAnswer($oversized, $this->ingested(self::INGESTED_ID));

        return $this->issuedBy(
            $this->client($this->options(maxAttempts: 4, maxResponseBytes: 256))
        );
    }

    /**
     * A base URL nothing can be sent to, which means nothing is ever sent.
     *
     * @return array{0: int, 1: bool}
     */
    private function provokedByAnUnusableApiUrl(): array
    {
        $this->restarted();
        $this->api->willAnswer($this->ingested(self::INGESTED_ID));

        return $this->issuedBy(new Client(
            'gopher://nowhere.invalid',
            self::APPLICATION_ID,
            self::TOKEN,
            $this->options(maxAttempts: 4)
        ));
    }

    /**
     * How many attempts a send made, and whether it ended up ingesting the event.
     *
     * A send that reached a server is counted by what that server received. One that never reached
     * anything — an API URL nothing can be sent to is the corpus's own example — is counted by what
     * the client says it did, which is also the message a caller is left holding: a misconfiguration
     * retried four times reads as a network that would not answer.
     *
     * @return array{0: int, 1: bool}
     */
    private function issuedBy(Client $built): array
    {
        try {
            $built->sendEvent($this->anEvent());

            return [$this->api->count(), true];
        } catch (ClientError $refused) {
            return [max($this->api->count(), $this->attemptsOf($refused->getMessage())), false];
        }
    }

    /** What a send says it did, out of the message it failed with. */
    private function attemptsOf(string $message): int
    {
        if (preg_match('/gave up after (\d+) attempts/', $message, $named) !== 1) {
            return 1;
        }

        return (int) $named[1];
    }

    /**
     * How many requests a send made when the API answered that way and then took the event, and
     * whether the send ended up ingesting it.
     *
     * @return array{0: int, 1: bool}
     */
    private function issuedFor(ScriptedResponse $answer): array
    {
        $this->restarted();
        $this->api->willAnswer($answer, $this->ingested(self::INGESTED_ID));

        try {
            $this->client($this->options(maxAttempts: 4))->sendEvent($this->anEvent());

            return [$this->api->count(), true];
        } catch (ClientError) {
            return [$this->api->count(), false];
        }
    }

    /** How long a send spent waiting when the API named that delay beside a paced answer. */
    private function waitedFor(string $header, string $written): float
    {
        [$paced] = $this->pacedPair();

        $this->restarted();
        $this->api->willAnswer(
            $this->refusal($paced['status'], $paced['problem'], [$header => $written]),
            $this->ingested(self::INGESTED_ID)
        );

        $chosen = new Options(
            retryPolicy: new RetryPolicy(
                maxAttempts: 4,
                initialBackoff: self::PROMPT_BACKOFF,
                maxBackoff: self::PROMPT_BACKOFF,
                maxTotalDelay: self::DELAY_BUDGET
            ),
            requestTimeout: 5.0
        );

        $started = hrtime(true);
        $this->client($chosen)->sendEvent($this->anEvent());
        $waited = (hrtime(true) - $started) / 1_000_000_000;

        self::assertCount(2, $this->api->received(), 'a paced answer was not retried');

        return $waited;
    }

    /**
     * Two problems answering the same status, one worth repeating and one not.
     *
     * That pair is the whole reason the corpus classifies problems rather than statuses, and the
     * retryable one is the answer the API names a delay beside.
     *
     * @return array{0: array<string, mixed>, 1: array<string, mixed>}
     */
    private function pacedPair(): array
    {
        $problems = Contract::of('retry.json')['problems'];
        foreach ($problems as $rule) {
            if (!$rule['retryable']) {
                continue;
            }
            foreach ($problems as $candidate) {
                if ($candidate['status'] === $rule['status'] && !$candidate['retryable']) {
                    return [$rule, $candidate];
                }
            }
        }

        throw new \RuntimeException('no status of the corpus carries opposite verdicts');
    }

    /**
     * What the API says when it refuses a request, in the shape every failure of it takes.
     *
     * @param array<string, string> $headers
     */
    private function refusal(int $status, string $problem, array $headers = []): ScriptedResponse
    {
        return new ScriptedResponse(
            $status,
            [
                'id' => $problem,
                'status' => $status,
                'title' => 'refused',
                'detail' => 'what the corpus scripted',
                'type' => 'https://hook0.com/documentation/errors/' . $problem,
            ],
            0.0,
            $headers
        );
    }

    /**
     * @param array<string, mixed> $vector
     */
    private function verified(array $vector): void
    {
        Signature::verifyWithCurrentTime(
            $vector['signature'],
            $vector['payload'],
            $vector['headers'],
            $vector['secret'],
            (float) $vector['tolerance_seconds'],
            new \DateTimeImmutable('@' . $vector['current_time'])
        );
    }
}
