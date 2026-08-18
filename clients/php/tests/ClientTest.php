<?php

declare(strict_types=1);

namespace Hook0\Tests;

use Hook0\Client;
use Hook0\ClientError;
use Hook0\Options;
use Hook0\RetryPolicy;
use Hook0\Tests\Support\ApiCase;
use Hook0\Tests\Support\ScriptedResponse;
use Hook0\Transport;

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

    public function testAnAnswerCarryingAHeadAboveACeilingTheCallerLoweredIsRefusedOnce(): void
    {
        // The count and the length per line multiply, so neither of them bounds what a head costs.
        // This is the one that does, and it is what refuses a head made of lines that are each
        // inside both of the others. Here the ceiling is one a caller chose, which is how the
        // option is shown to reach the transport at all rather than only its default being right.
        $maximum = 2048;
        $crowded = new ScriptedResponse(
            201,
            $this->ingested(self::INGESTED_ID)->body,
            0.0,
            $this->paddingOf(16, 256)
        );
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

    public function testAnAnswerCarryingAHeadAboveTheCeilingThisClientAppliesIsRefusedOnce(): void
    {
        // The ceiling the contract names, exercised as it ships rather than lowered to suit the
        // case: a client whose default were wrong passes the case above and fails this one.
        //
        // Sixty lines of a kilobyte, and that shape is the point. The two component bounds must not
        // be what refuses this head, or the case would report the wrong ceiling while looking green:
        // lines long enough that the total is crossed inside the first score of them, and few enough
        // that the line count is never approached — the API's own `Content-Type`, `Content-Length`
        // and `Connection` are counted too, so a head built from sixty-four padding lines is refused
        // for being too many lines, which proves nothing about the total.
        $crowded = new ScriptedResponse(
            201,
            $this->ingested(self::INGESTED_ID)->body,
            0.0,
            $this->paddingOf(60, 1024)
        );
        $this->api->willAnswer($crowded, $crowded, $crowded, $crowded);

        try {
            $this->client()->sendEvent($this->anEvent());
            self::fail('a head above the ceiling was read');
        } catch (ClientError $refused) {
            self::assertStringContainsString(
                sprintf('head above the %d bytes read at most', Transport::DEFAULT_MAX_HEAD_BYTES),
                $refused->getMessage()
            );
        }

        self::assertCount(1, $this->api->received());
    }

    public function testAnAnswerCarryingAHeadWellUnderTheCeilingIsRead(): void
    {
        // Eight kilobytes: comfortably under the ceiling rather than just under it. Refusal above a
        // safety ceiling is a property of this client; acceptance at the rim is a property of
        // whichever HTTP stack is underneath, which settles a head before this client is consulted,
        // so a case built at the rim would report the runtime of the day instead.
        $this->api->willAnswer(new ScriptedResponse(
            201,
            $this->ingested(self::INGESTED_ID)->body,
            0.0,
            $this->paddingOf(8, 1024)
        ));

        $eventId = $this->client()->sendEvent($this->anEvent());

        self::assertSame(self::INGESTED_ID, $eventId);
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

    public function testAPayloadThatIsNotTextStopsTheSendRatherThanReachingTheWireMangled(): void
    {
        // A payload is bytes the caller handed over, and a request body is a JSON document: bytes
        // that are not text cannot be written as one, and what must not happen is a send that puts
        // whatever the language made of them on the wire.
        $this->expectException(\Hook0\DecodeError::class);

        try {
            $this->client()->sendEvent($this->anEvent(payload: "\xff\xfe not a document"));
        } finally {
            self::assertSame(0, $this->api->count(), 'a payload that is not text reached the API');
        }
    }

    public function testUpsertingNoEventTypeAtAllReachesTheApiForNothing(): void
    {
        self::assertSame([], $this->client()->upsertEventTypes([]));
        self::assertSame(0, $this->api->count());
    }

    public function testAnEventCarryingMetadataSendsItBesideTheLabels(): void
    {
        $this->api->willAnswer($this->ingested(self::INGESTED_ID));

        $this->client()->sendEvent($this->anEvent(metadata: ['tenant' => 'acme']));

        self::assertSame(['tenant' => 'acme'], $this->api->received()[0]->json()['metadata']);
    }

    public function testAnAcceptedEventTheApiNamedNoIdentifierForIsReportedRatherThanRepeated(): void
    {
        // The event was ingested, so repeating the request would meet the same answer; what the
        // caller cannot be given is the identifier it was ingested under.
        $this->api->willAnswer(new ScriptedResponse(201, ['received_at' => '2026-01-01']));

        try {
            $this->client()->sendEvent($this->anEvent());
            self::fail('an answer naming no event id was read as a send that succeeded');
        } catch (ClientError $refused) {
            self::assertStringContainsString('without an event id', $refused->getMessage());
        }

        self::assertSame(1, $this->api->count());
    }

    public function testARefusalTheApiWroteNoProblemDocumentForFallsBackToWhatItsStatusSays(): void
    {
        // The one status that says both a spent quota and a paced instance is told apart by the
        // problem the body names. A body naming none — here, one that is not a document at all —
        // leaves the status to decide, and that status is one a repeat cannot clear.
        $this->api->willAnswer(new ScriptedResponse(429, 'a gateway wrote this, and it is not JSON'));

        try {
            $this->client()->sendEvent($this->anEvent());
            self::fail('a refusal with no problem document was read as a send that succeeded');
        } catch (ClientError $refused) {
            self::assertStringContainsString('a gateway wrote this', $refused->getMessage());
        }

        self::assertSame(1, $this->api->count());
    }

    public function testEventTypesTheApiCouldNotBeAskedForAreReportedRatherThanTakenAsNone(): void
    {
        // Nothing is created off a list that was never read: taking a failure for an empty list
        // would have this client declare every event type the application already has.
        $this->api->willAnswer(new ScriptedResponse(503, ['id' => 'ServiceUnavailable', 'status' => 503]));

        try {
            $this->client()->upsertEventTypes(['auth.user.create']);
            self::fail('a list of event types that was refused was read as a list');
        } catch (ClientError $refused) {
            self::assertStringContainsString('Getting available event types failed', $refused->getMessage());
        }

        self::assertSame(1, $this->api->count());
    }

    public function testAListOfEventTypesTheApiCouldNotAnswerAtAllIsReportedAsTheAskItWas(): void
    {
        // The answer crosses a ceiling this client set for itself, so no list ever arrives. What the
        // caller is told is what was being asked for, rather than the transport's own words.
        $this->api->willAnswer(new ScriptedResponse(200, [['event_type_name' => str_repeat('a', 2048)]]));

        try {
            $this->client($this->options(maxResponseBytes: 256))->upsertEventTypes(['auth.user.create']);
            self::fail('a list of event types that never arrived was read as a list');
        } catch (ClientError $refused) {
            self::assertStringContainsString('Getting available event types failed', $refused->getMessage());
            self::assertStringContainsString('more than the 256 bytes read at most', $refused->getMessage());
        }
    }

    public function testAnEventTypeTheApiNeverAnsweredForIsReportedUnderTheNameItWasAskedFor(): void
    {
        $this->api->willAnswer(
            new ScriptedResponse(200, []),
            new ScriptedResponse(201, ['padding' => str_repeat('a', 2048)])
        );

        try {
            $this->client($this->options(maxResponseBytes: 256))->upsertEventTypes(['auth.user.create']);
            self::fail('an event type whose creation never answered was read as created');
        } catch (ClientError $refused) {
            self::assertStringContainsString("Creating event type 'auth.user.create' failed", $refused->getMessage());
            self::assertStringContainsString('more than the 256 bytes read at most', $refused->getMessage());
        }
    }

    public function testAListOfEventTypesThatIsNotAListIsReportedRatherThanWalked(): void
    {
        $this->api->willAnswer(new ScriptedResponse(200, ['event_type_name' => 'auth.user.create']));

        try {
            $this->client()->upsertEventTypes(['auth.user.create']);
            self::fail('an answer that is not a list of event types was walked as one');
        } catch (ClientError $refused) {
            self::assertStringContainsString('did not answer a list of event types', $refused->getMessage());
        }
    }

    public function testAnEntryOfTheListNamingNoEventTypeIsPassedOverRatherThanStoppingTheRead(): void
    {
        // The application declares one of the two, under an entry beside one this client cannot
        // read: the one it can read is still matched, and the other is still created.
        $this->api->willAnswer(
            new ScriptedResponse(200, [['no name here' => true], ['event_type_name' => 'auth.user.create']]),
            new ScriptedResponse(201, ['event_type_name' => 'billing.invoice.paid'])
        );

        $created = $this->client()->upsertEventTypes(['auth.user.create', 'billing.invoice.paid']);

        self::assertSame(['billing.invoice.paid'], $created);
    }

    public function testAnEventTypeTheApiRefusedToCreateIsReportedUnderTheNameItWasAskedFor(): void
    {
        $this->api->willAnswer(
            new ScriptedResponse(200, []),
            new ScriptedResponse(409, ['id' => 'EventTypeAlreadyExist', 'status' => 409])
        );

        try {
            $this->client()->upsertEventTypes(['auth.user.create']);
            self::fail('an event type the API refused was read as created');
        } catch (ClientError $refused) {
            self::assertStringContainsString("Creating event type 'auth.user.create' failed", $refused->getMessage());
            self::assertStringContainsString('EventTypeAlreadyExist', $refused->getMessage());
        }
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

    public function testADurationNoScheduleCouldBeBuiltOnIsTheDefaultBothOnTheWireAndInTheWait(): void
    {
        // A value that is not a finite number names no duration, and a policy holding one is in
        // force with the default of that field. Both halves are held to it: what the request states
        // and what the send would actually wait, because a header that named a policy the client
        // does not run would be worse than no header at all.
        $unusable = ['+INF' => INF, '-INF' => -INF, 'NaN' => NAN];
        $held = ['initialBackoff', 'maxBackoff', 'maxTotalDelay'];

        $default = new RetryPolicy();
        $draws = [0.5, 0.5, 0.5];

        foreach ($unusable as $named => $seconds) {
            foreach ($held as $index => $field) {
                $written = [4, 0.1, 2.0, 5.0];
                $written[$index + 1] = $seconds;
                $policy = new RetryPolicy(...$written);

                $this->restarted();
                $this->api->willAnswer($this->ingested(self::INGESTED_ID));
                $this->client(new Options(retryPolicy: $policy, requestTimeout: 5.0))
                    ->sendEvent($this->anEvent());

                $stated = $this->api->received()[0]->headers['hook0-client-options'] ?? '';
                $because = sprintf('a policy holding `%s: %s`', $field, $named);

                self::assertSame('attempts=4,backoff=100,ceiling=2000,budget=5000', $stated, $because);
                self::assertSame($default->delays($draws), $policy->delays($draws), $because);
            }
        }
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

    /**
     * Header lines an answer is padded with, each the same length, named apart so none replaces another.
     *
     * @return array<string, string>
     */
    private function paddingOf(int $lines, int $bytes): array
    {
        $padding = [];
        for ($index = 1; $index <= $lines; $index++) {
            $padding[sprintf('x-pad-%d', $index)] = str_repeat('v', $bytes);
        }

        return $padding;
    }
}
