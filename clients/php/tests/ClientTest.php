<?php

declare(strict_types=1);

namespace Hook0\Tests;

use Hook0\Client;
use Hook0\ClientError;
use Hook0\Options;
use Hook0\RetryPolicy;
use Hook0\Tests\Support\ApiCase;
use Hook0\Tests\Support\ScriptedResponse;

/**
 * What a send does over a real socket.
 *
 * What is asserted is what the API saw — how many requests arrived, and what each of them carried —
 * rather than what the client was asked to do, so a client that reports success without having sent
 * anything fails here.
 */
final class ClientTest extends ApiCase
{
    private const INGESTED_ID = '01961234-5678-7abc-8def-0123456789ab';

    public function testASendThatSucceedsIssuesOneRequest(): void
    {
        $this->api->willAnswer($this->ingested(self::INGESTED_ID));

        $eventId = $this->client()->sendEvent($this->anEvent());

        self::assertSame(self::INGESTED_ID, $eventId);
        self::assertCount(1, $this->api->received());
    }

    public function testAnEventCarryingNoIdIsSentUnderOneTheClientGenerated(): void
    {
        $this->api->willAnswer($this->ingested(self::INGESTED_ID));

        $this->client()->sendEvent($this->anEvent());

        // Without an identifier of its own, a repeated request makes the API mint a second one, and
        // the event is ingested and delivered twice.
        self::assertMatchesRegularExpression(self::UUID_PATTERN, $this->api->eventIdOf(0));
    }

    public function testAnEventCarryingAnIdIsSentUnderIt(): void
    {
        $chosen = '00000000-0000-0000-0000-000000000000';
        $this->api->willAnswer($this->ingested($chosen));

        $eventId = $this->client()->sendEvent($this->anEvent(eventId: $chosen));

        self::assertSame($chosen, $eventId);
        self::assertSame($chosen, $this->api->eventIdOf(0));
    }

    public function testAnAttemptThatRanOutOfTimeIsRepeatedUnderTheSameId(): void
    {
        $this->api->willAnswer(
            $this->ingested(self::INGESTED_ID, heldFor: 1.0),
            $this->ingested(self::INGESTED_ID)
        );

        $eventId = $this->client($this->options(maxAttempts: 3, requestTimeout: 0.2))
            ->sendEvent($this->anEvent());

        self::assertSame(self::INGESTED_ID, $eventId);
        self::assertCount(2, $this->api->received());
        // The retry has to repeat the identifier of the attempt it repeats, or the API ingests twice.
        self::assertSame($this->api->eventIdOf(0), $this->api->eventIdOf(1));
        self::assertMatchesRegularExpression(self::UUID_PATTERN, $this->api->eventIdOf(0));
    }

    public function testRepeatedServerErrorsStopAtTheConfiguredNumberOfAttempts(): void
    {
        $this->api->willAnswer(
            $this->serverError(),
            $this->serverError(),
            $this->serverError(),
            $this->serverError()
        );

        try {
            $this->client($this->options(maxAttempts: 3))->sendEvent($this->anEvent());
            self::fail('a send answered nothing but server errors reported success');
        } catch (ClientError $refused) {
            self::assertStringContainsString('gave up after 3 attempts', $refused->getMessage());
        }

        self::assertCount(3, $this->api->received());
    }

    public function testAnAnswerTheApiWouldRepeatIsNotRetried(): void
    {
        $this->api->willAnswer(
            new ScriptedResponse(429, ['id' => 'TooManyEventsToday', 'status' => 429])
        );

        // A quota that is spent for the day cannot clear itself between two attempts.
        try {
            $this->client($this->options(maxAttempts: 4))->sendEvent($this->anEvent());
            self::fail('a spent quota was reported as a success');
        } catch (ClientError $refused) {
            self::assertStringContainsString('Sending event', $refused->getMessage());
        }

        self::assertCount(1, $this->api->received());
    }

    public function testARetryAnsweredThatTheEventWasAlreadyIngestedReportsSuccess(): void
    {
        $this->api->willAnswer($this->serverError(), $this->alreadyIngested());

        $eventId = $this->client($this->options(maxAttempts: 3))->sendEvent($this->anEvent());

        // The conflict is the mark of the attempt this one repeats having reached the API.
        self::assertSame($this->api->eventIdOf(0), $eventId);
        self::assertCount(2, $this->api->received());
    }

    public function testAFirstAttemptAnsweredThatTheEventWasAlreadyIngestedReportsTheConflict(): void
    {
        $this->api->willAnswer($this->alreadyIngested());

        // Nothing this send did can explain the conflict, so the caller has to hear about it.
        try {
            $this->client($this->options(maxAttempts: 3))->sendEvent($this->anEvent());
            self::fail('a conflict nothing explains was reported as a success');
        } catch (ClientError $refused) {
            self::assertStringContainsString('EventAlreadyIngested', $refused->getMessage());
        }

        self::assertCount(1, $this->api->received());
    }

    public function testAClientThatDoesNotRetryIssuesOneRequest(): void
    {
        $this->api->willAnswer($this->serverError(), $this->serverError(), $this->serverError());

        $this->expectException(ClientError::class);

        try {
            $this->client(new Options(retryPolicy: RetryPolicy::disabled()))
                ->sendEvent($this->anEvent());
        } finally {
            self::assertCount(1, $this->api->received());
        }
    }

    public function testAPayloadAboveTheMaximumIsRefusedBeforeAnyRequest(): void
    {
        $maximum = 16;
        $this->api->willAnswer($this->ingested(self::INGESTED_ID));

        try {
            $this->client($this->options(maxPayloadBytes: $maximum))
                ->sendEvent($this->anEvent(payload: str_repeat('x', $maximum + 1)));
            self::fail('an oversized payload was sent');
        } catch (ClientError $refused) {
            self::assertStringContainsString(
                sprintf('%d bytes this client sends at most', $maximum),
                $refused->getMessage()
            );
        }

        self::assertSame(0, $this->api->count());
    }

    public function testAnAnswerCarryingMoreHeaderLinesThanTheMaximumIsRefusedOnce(): void
    {
        // The HTTP library holds however many header lines a server writes, so the ceiling is this
        // client's to apply. A repetition would read the same oversized head again, which is why it
        // is one.
        $maximum = 8;
        $padding = [];
        for ($index = 1; $index <= $maximum + 1; $index++) {
            $padding[sprintf('x-pad-%d', $index)] = 'v';
        }
        $crowded = new ScriptedResponse(201, $this->ingested(self::INGESTED_ID)->body, 0.0, $padding);
        $this->api->willAnswer($crowded, $crowded, $crowded, $crowded);

        try {
            $this->client($this->options(maxAttempts: 4, maxResponseHeaders: $maximum))
                ->sendEvent($this->anEvent());
            self::fail('a head above the ceiling was read');
        } catch (ClientError $refused) {
            self::assertStringContainsString(
                sprintf('%d header lines read at most', $maximum),
                $refused->getMessage()
            );
        }

        self::assertCount(1, $this->api->received());
    }

    public function testAnAnswerCarryingAHeaderLineAboveTheMaximumIsRefusedOnce(): void
    {
        $maximum = 512;
        $padded = new ScriptedResponse(
            201,
            $this->ingested(self::INGESTED_ID)->body,
            0.0,
            ['x-pad' => str_repeat('v', $maximum + 1)]
        );
        $this->api->willAnswer($padded, $padded, $padded, $padded);

        try {
            $this->client($this->options(maxAttempts: 4, maxHeaderBytes: $maximum))
                ->sendEvent($this->anEvent());
            self::fail('a header line above the ceiling was read');
        } catch (ClientError $refused) {
            self::assertStringContainsString(
                sprintf('`x-pad` header above the %d bytes read at most', $maximum),
                $refused->getMessage()
            );
        }

        self::assertCount(1, $this->api->received());
    }

    public function testAnAnswerCarryingAHeadAboveTheMaximumIsRefusedOnce(): void
    {
        // The count and the length per line multiply, so neither of them bounds what a head costs.
        // This is the one that does, and it is what refuses a head made of lines that are each
        // inside both of the others.
        $maximum = 2048;
        $padding = [];
        for ($index = 1; $index <= 16; $index++) {
            $padding[sprintf('x-pad-%d', $index)] = str_repeat('v', 256);
        }
        $crowded = new ScriptedResponse(201, $this->ingested(self::INGESTED_ID)->body, 0.0, $padding);
        $this->api->willAnswer($crowded, $crowded, $crowded, $crowded);

        try {
            $this->client($this->options(maxAttempts: 4, maxHeadBytes: $maximum))
                ->sendEvent($this->anEvent());
            self::fail('a head above the ceiling was read');
        } catch (ClientError $refused) {
            self::assertStringContainsString(
                sprintf('head above the %d bytes read at most', $maximum),
                $refused->getMessage()
            );
        }

        self::assertCount(1, $this->api->received());
    }

    public function testASendCarriesTheApplicationAndTheCredential(): void
    {
        $this->api->willAnswer($this->ingested(self::INGESTED_ID));

        $this->client()->sendEvent($this->anEvent());

        $request = $this->api->received()[0];

        self::assertSame('POST', $request->verb);
        self::assertStringEndsWith('/event', $request->target);
        self::assertSame('Bearer ' . self::TOKEN, $request->headers['authorization']);
        self::assertSame(self::APPLICATION_ID, $request->json()['application_id']);
        self::assertSame(['environment' => 'production'], $request->json()['labels']);
    }

    public function testAnEventCarryingNoLabelsSendsAnObjectRatherThanAnEmptyList(): void
    {
        // The language spells a list and a map with the one array, so an empty one goes out as `[]`
        // unless the client says otherwise — and an API reading an object refuses that.
        $this->api->willAnswer($this->ingested(self::INGESTED_ID));

        $this->client()->sendEvent($this->anEvent(labels: []));

        self::assertStringContainsString('"labels":{}', $this->api->received()[0]->body);
    }

    public function testEventTypesTheApplicationAlreadyDeclaresAreNotCreated(): void
    {
        $this->api->willAnswer(
            new ScriptedResponse(200, [['event_type_name' => 'auth.user.create']]),
            new ScriptedResponse(201, ['event_type_name' => 'billing.invoice.paid'])
        );

        $created = $this->client()->upsertEventTypes(['auth.user.create', 'billing.invoice.paid']);

        self::assertSame(['billing.invoice.paid'], $created);
        self::assertCount(2, $this->api->received());
    }

    public function testAnEventTypeThatDoesNotReadAsThreePartsIsRefused(): void
    {
        try {
            $this->client()->upsertEventTypes(['not-an-event-type']);
            self::fail('an event type that names nothing was accepted');
        } catch (ClientError $refused) {
            self::assertStringContainsString('does not have a valid syntax', $refused->getMessage());
        }

        self::assertSame(0, $this->api->count());
    }

    public function testTheDefaultScheduleDoublesUpToItsCeiling(): void
    {
        $policy = new RetryPolicy();

        self::assertEqualsWithDelta(0.1, $policy->backoffCeiling(1), 1e-9);
        self::assertEqualsWithDelta(0.2, $policy->backoffCeiling(2), 1e-9);
        self::assertEqualsWithDelta(0.4, $policy->backoffCeiling(3), 1e-9);
        self::assertEqualsWithDelta(2.0, $policy->backoffCeiling(6), 1e-9);
        self::assertEqualsWithDelta(2.0, $policy->backoffCeiling(16), 1e-9);

        // A source of randomness that gives nothing asks for the whole ceiling, which is what makes
        // the budget the thing that cuts the schedule short.
        $delays = $policy->delays([]);

        self::assertCount(3, $delays);
        self::assertLessThanOrEqual($policy->maxTotalDelay, array_sum($delays));
    }

    public function testADisabledPolicyWaitsForNothing(): void
    {
        $policy = RetryPolicy::disabled();

        self::assertSame(1, $policy->attempts());
        self::assertSame([], $policy->delays([1.0, 1.0, 1.0]));
    }

    public function testMintedIdentifiersAreOrderedByTheMomentTheyWereMinted(): void
    {
        // The leading 48 bits are the moment in milliseconds, so identifiers minted in sequence never
        // go back in time — which is what keeps the index they end up in from being written all over.
        // Two minted inside one millisecond differ only in what was drawn, so what is ordered is the
        // moment they carry rather than the whole of them.
        $minted = [];
        for ($index = 0; $index < 64; $index++) {
            $minted[] = Client::generateEventId();
        }

        $moments = array_map(
            static fn (string $identifier): string => substr($identifier, 0, 8) . substr($identifier, 9, 4),
            $minted
        );
        $sorted = $moments;
        sort($sorted);

        self::assertSame($sorted, $moments);
        foreach ($minted as $identifier) {
            self::assertMatchesRegularExpression(self::UUID_PATTERN, $identifier);
            self::assertSame('7', $identifier[14]);
            self::assertContains($identifier[19], ['8', '9', 'a', 'b']);
        }

        // Far enough apart to land in another millisecond, the whole identifier is ordered too.
        $earlier = Client::generateEventId();
        usleep(5000);

        self::assertLessThan(Client::generateEventId(), $earlier);
    }
}
