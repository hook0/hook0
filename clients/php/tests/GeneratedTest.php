<?php

declare(strict_types=1);

namespace Hook0\Tests;

use Hook0\Generated\ApplicationInfo;
use Hook0\Generated\ApplicationsApi;
use Hook0\Generated\NotFoundError;
use Hook0\Generated\Problem;
use Hook0\Generated\ProblemError;
use Hook0\Generated\ProblemId;
use Hook0\Generated\RequestAttemptStatus;
use Hook0\Generated\SubscriptionTarget;
use Hook0\Runtime;
use Hook0\Tests\Support\ApiCase;
use Hook0\Tests\Support\GeneratedSurface;
use Hook0\Tests\Support\ScriptedResponse;
use Hook0\Transport;

/**
 * What the generated request layer puts on the wire, and what it does with what comes back.
 *
 * The generated half is handed a transport and nothing else, so these cases hand it the real one and
 * watch a real API answer: the path it filled in, the query it assembled, the credential it carried,
 * the type it read back, and the exception it threw when the answer was a problem.
 *
 * Nothing here names an operation the API does not declare. What is exercised is reached through the
 * generated namespace, so an operation that stops being declared takes its case with it rather than
 * leaving one that runs against nothing.
 */
final class GeneratedTest extends ApiCase
{
    private const APPLICATION_UUID = '3f2504e0-4f89-41d3-9a0c-0305e82c3301';
    private const ORGANIZATION_UUID = '3f2504e0-4f89-41d3-9a0c-0305e82c3302';

    public function testAGeneratedMethodReadsBackTheTypeTheApiDeclares(): void
    {
        $this->api->willAnswer(new ScriptedResponse(200, $this->anApplication()));

        $application = $this->applications()->get(self::APPLICATION_UUID);

        self::assertInstanceOf(ApplicationInfo::class, $application);
        self::assertSame(self::APPLICATION_UUID, $application->applicationId);
        self::assertSame(100, $application->quotas->eventsPerDayLimit);
        self::assertSame('Done', $application->onboardingSteps->event->value);
    }

    public function testAGeneratedMethodFillsThePathAndCarriesTheCredential(): void
    {
        $this->api->willAnswer(new ScriptedResponse(200, $this->anApplication()));

        $this->applications()->get(self::APPLICATION_UUID);

        $request = $this->api->received()[0];

        self::assertSame('GET', $request->verb);
        // The identifier lands in the path rather than staying as the placeholder that named it.
        self::assertSame('/api/v1/applications/' . self::APPLICATION_UUID, $request->target);
        self::assertSame('Bearer ' . self::TOKEN, $request->headers['authorization']);
    }

    public function testAGeneratedMethodAssemblesTheQueryTheOperationDeclares(): void
    {
        $this->api->willAnswer(new ScriptedResponse(200, []));

        self::assertSame([], $this->applications()->list(self::ORGANIZATION_UUID));
        self::assertSame(
            '/api/v1/applications/?organization_id=' . self::ORGANIZATION_UUID,
            $this->api->received()[0]->target
        );
    }

    public function testAGeneratedMethodReadsAListOfTheTypeTheApiDeclares(): void
    {
        $this->api->willAnswer(new ScriptedResponse(200, [[
            'application_id' => self::APPLICATION_UUID,
            'name' => 'an application',
            'organization_id' => self::ORGANIZATION_UUID,
        ]]));

        $listed = $this->applications()->list(self::ORGANIZATION_UUID);

        self::assertCount(1, $listed);
        self::assertSame('an application', $listed[0]->name);
    }

    public function testAProblemTheApiReportsIsThrownAsTheExceptionItNames(): void
    {
        $this->api->willAnswer(new ScriptedResponse(404, $this->aProblem()));

        try {
            $this->applications()->get(self::APPLICATION_UUID);
            self::fail('a problem the API reported was read as a success');
        } catch (NotFoundError $reported) {
            self::assertSame(404, $reported->status);
            self::assertSame('This application does not exist.', $reported->problem?->detail);
            // Every problem is a kind of the one exception a caller may catch instead of naming each.
            self::assertInstanceOf(ProblemError::class, $reported);
        }
    }

    public function testAFailureThatIsNotAProblemDocumentIsStillReported(): void
    {
        $this->api->willAnswer(
            new ScriptedResponse(502, 'a gateway wrote this, and it is not a problem document')
        );

        try {
            $this->applications()->get(self::APPLICATION_UUID);
            self::fail('a failure with no problem document was read as a success');
        } catch (ProblemError $reported) {
            self::assertSame(502, $reported->status);
            self::assertNull($reported->problem);
        }
    }

    public function testAMemberTheDocumentDoesNotRequireIsAbsentRatherThanWrittenBackAsNothing(): void
    {
        $problem = Problem::fromJson($this->aProblem());

        self::assertArrayNotHasKey('validation', $problem->toArray());
        self::assertSame($this->aProblem(), $problem->toArray());
    }

    public function testADocumentThatDoesNotSayWhatItDeclaresStopsTheRead(): void
    {
        $unreadable = ['name' => 42] + $this->anApplication();

        $this->expectException(\Hook0\DecodeError::class);

        ApplicationInfo::fromJson($unreadable);
    }

    public function testAValueOutsideAClosedListIsRefusedRatherThanCarried(): void
    {
        $unreadable = $this->anApplication();
        $unreadable['onboarding_steps']['event'] = 'Maybe';

        $this->expectException(\Hook0\DecodeError::class);

        ApplicationInfo::fromJson($unreadable);
    }

    public function testTheGeneratedSurfaceIsReachableUnderTheNamesTheApiSpells(): void
    {
        // PHP reads what follows `->`, `::` and `$` as a name rather than as a keyword, so nothing
        // the API spells has to be moved out of the way here: a member called `until`, `method` or
        // `type` is a member called exactly that, and the wire name and the PHP name are one.
        self::assertTrue(property_exists(RequestAttemptStatus::class, 'until'));
        self::assertFalse(property_exists(RequestAttemptStatus::class, 'until_'));
        self::assertTrue(property_exists(SubscriptionTarget::class, 'method'));
        self::assertFalse(property_exists(SubscriptionTarget::class, 'method_'));
        self::assertTrue(property_exists(Problem::class, 'id'));
        self::assertTrue(property_exists(Problem::class, 'type'));

        $written = RequestAttemptStatus::fromJson([
            'type' => 'waiting',
            'since' => '2026-08-14T12:00:00Z',
            'until' => '2026-08-14T12:05:00Z',
        ])->toArray();

        self::assertArrayHasKey('until', $written);
        self::assertArrayNotHasKey('until_', $written);
    }

    public function testNothingTheGeneratorDeclaredIsSpelledLikeAMagicMethod(): void
    {
        // A declaration carrying one of the names every class already answers to changes what happens
        // a long way from where it was declared: a `__toString` decides what the value reads as the
        // next time anything prints it. `__construct` is the exception, since a class is what it
        // declares one for.
        $deliberate = ['__construct'];

        foreach (GeneratedSurface::declared() as $declared) {
            $reflection = new \ReflectionClass($declared);
            foreach ($reflection->getMethods() as $method) {
                $inherited = $method->getDeclaringClass()->getName() !== $declared;
                if ($inherited || !str_starts_with($method->getName(), '__')) {
                    continue;
                }
                self::assertContains(
                    $method->getName(),
                    $deliberate,
                    sprintf('%s declares `%s`, which every class already answers to', $declared, $method->getName())
                );
            }
        }
    }

    public function testEveryProblemTheCatalogueNamesIsAnExceptionOfItsOwn(): void
    {
        /** @var array<string, class-string<ProblemError>> $catalogue */
        $catalogue = (new \ReflectionClass(ProblemError::class))->getConstant('PROBLEMS');

        $named = array_map(static fn (ProblemId $case): string => $case->value, ProblemId::cases());
        sort($named);
        $keys = array_keys($catalogue);
        sort($keys);

        self::assertSame($named, $keys);
        self::assertCount(count($catalogue), array_unique(array_values($catalogue)));
        foreach ($catalogue as $declared) {
            self::assertTrue(is_subclass_of($declared, ProblemError::class), $declared);
        }
    }

    public function testEveryValueTheApiDeclaresWritesBackWhatItRead(): void
    {
        // Every closed list is a real enumeration, so a value outside it cannot be built at all and a
        // value inside it travels as the string the API answered.
        foreach (GeneratedSurface::enumerations() as $declared) {
            foreach ($declared::cases() as $case) {
                self::assertSame($case, $declared::from($case->value));
                self::assertNull($declared::tryFrom($case->value . '-no-such-value'));
            }
        }
    }

    public function testAnOpenKeyedObjectCarryingNothingIsWrittenBackAsAnObject(): void
    {
        $written = Runtime::encode(\Hook0\Generated\EventPost::fromJson([
            'application_id' => self::APPLICATION_UUID,
            'event_type' => 'auth.user.create',
            'labels' => [],
            'occurred_at' => '2026-08-14T12:00:00Z',
            'payload' => '{}',
            'payload_content_type' => 'application/json',
        ])->toArray());

        self::assertStringContainsString('"labels":{}', $written);
    }

    private function applications(): ApplicationsApi
    {
        return new ApplicationsApi(new Transport($this->api->baseUrl(), self::TOKEN));
    }

    /**
     * @return array<string, mixed>
     */
    private function anApplication(): array
    {
        return [
            'application_id' => self::APPLICATION_UUID,
            'name' => 'an application',
            'organization_id' => self::ORGANIZATION_UUID,
            'consumption' => ['events_per_day' => 12],
            'onboarding_steps' => ['event' => 'Done', 'event_type' => 'ToDo', 'subscription' => 'ToDo'],
            'quotas' => ['days_of_events_retention_limit' => 7, 'events_per_day_limit' => 100],
        ];
    }

    /**
     * @return array<string, mixed>
     */
    private function aProblem(): array
    {
        return [
            'detail' => 'This application does not exist.',
            'id' => 'NotFound',
            'status' => 404,
            'title' => 'Not found',
            'type' => 'https://documentation.hook0.com/problems',
        ];
    }
}
