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
use Hook0\Tests\Support\ApiDocument;
use Hook0\Tests\Support\GeneratedSurface;
use Hook0\Tests\Support\ScriptedResponse;
use Hook0\Tests\Support\Synthesis;
use Hook0\Transport;
use PHPUnit\Framework\Attributes\DataProvider;

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

    /**
     * Whether a case asks for everything an operation or a value may carry, or only what it must.
     *
     * @return iterable<string, array{0: bool}>
     */
    public static function optionality(): iterable
    {
        yield 'with everything it may carry' => [true];
        yield 'with only what it requires' => [false];
    }

    /**
     * Every operation the document declares issues the request it is declared as, and reads it back.
     *
     * Run twice: once giving every argument the operation may be asked with, once giving only the
     * ones it requires, which is what says an argument left out leaves the query it would have
     * filled empty. The last assertion is the one that grows: an operation the API adds fails this
     * case until something drives it.
     */
    #[DataProvider('optionality')]
    public function testEveryOperationTheDocumentDeclaresIsReachedTheWayItDeclaresIt(bool $optionals): void
    {
        $reached = [];

        foreach (GeneratedSurface::groups() as $group) {
            $reaching = new $group(new Transport($this->api->baseUrl(), self::TOKEN));
            foreach (GeneratedSurface::operationsOf($group) as $operation) {
                $named = sprintf('%s::%s', $group, $operation->getName());
                $given = Synthesis::arguments($operation, $optionals);
                [$answered, $expected] = $this->answerFor($operation, $optionals);
                $this->api->willAnswer(new ScriptedResponse(200, $answered));

                $read = $operation->invokeArgs($reaching, $given);

                $received = $this->api->received();
                $request = $received[count($received) - 1];
                $declared = ApiDocument::reached($request);

                self::assertSame(
                    Synthesis::written($expected),
                    Synthesis::written($read),
                    $named . ' did not read back what the API answered'
                );
                self::assertSame('Bearer ' . self::TOKEN, $request->headers['authorization'], $named);
                self::assertSame('application/json', $request->headers['accept'], $named);
                $this->assertPathFilledIn($declared, $request, $named);
                $this->assertQueryAssembled($declared, $request, $operation, $given, $optionals, $named);
                $this->assertBodySent($request, $given, $named);

                $reached[] = $declared->named();
            }
        }

        $wanted = array_map(
            static fn (\Hook0\Tests\Support\DeclaredOperation $one): string => $one->named(),
            ApiDocument::operations()
        );
        sort($wanted);
        $reached = array_values(array_unique($reached));
        sort($reached);

        self::assertSame($wanted, $reached, 'the generated groups reach other operations than the document declares');
    }

    /**
     * A value written out and read back in is the value it started as, member for member.
     *
     * Run once with every member the schema may leave out set and once with none of them, which is
     * what tells a member that was read apart from one that was defaulted to the same thing.
     */
    #[DataProvider('optionality')]
    public function testEverySchemaTheDocumentDeclaresReadsBackWhatItWrote(bool $optionals): void
    {
        $walked = 0;
        foreach (GeneratedSurface::models() as $declared) {
            $held = Synthesis::model($declared, $optionals);
            $written = $held->toArray();

            $read = $declared::fromJson($written);
            self::assertTrue($read->equals($held), $declared . ' does not read back what it wrote');
            self::assertSame($written, $read->toArray(), $declared . ' does not write back what it read');

            foreach ($this->unset($declared, $held) as $absent) {
                self::assertArrayNotHasKey(
                    $absent,
                    $written,
                    sprintf('%s wrote `%s` out although it holds nothing', $declared, $absent)
                );
            }
            $walked++;
        }

        self::assertGreaterThan(0, $walked, 'the generator wrote no schema at all');
    }

    public function testASchemaRefusesADocumentThatIsNotTheObjectItDeclares(): void
    {
        foreach (GeneratedSurface::models() as $declared) {
            foreach ([1, 'text', true, [1, 2], null] as $answered) {
                try {
                    $declared::fromJson($answered);
                    self::fail(sprintf('%s read `%s` as a document', $declared, get_debug_type($answered)));
                } catch (\Hook0\DecodeError $refused) {
                    self::assertStringContainsString(
                        (new \ReflectionClass($declared))->getShortName(),
                        $refused->getMessage(),
                        $declared . ' did not say what it was that could not be read'
                    );
                }
            }
        }
    }

    public function testASchemaRefusesAMemberTheDocumentDoesNotDeclareItAs(): void
    {
        // Everything a schema describes is refused when it arrives as something else, and says which
        // member it was. What it leaves undescribed is kept as it arrived and so is refused by
        // nothing, and there are exactly as many members that accept anything as it leaves undescribed.
        foreach (GeneratedSurface::models() as $declared) {
            $written = Synthesis::model($declared, true)->toArray();
            $accepted = [];

            foreach (array_keys($written) as $name) {
                // Neither an object nor a scalar any of the readers accept, whichever the member is.
                $wrong = $written;
                $wrong[$name] = [['neither' => 'a scalar']];
                try {
                    $declared::fromJson($wrong);
                    $accepted[] = $name;
                } catch (\Hook0\DecodeError $refused) {
                    self::assertStringContainsString(
                        $name,
                        $refused->getMessage(),
                        $declared . ' did not say which member it could not read'
                    );
                }
            }

            self::assertCount(
                $this->opaque($declared),
                $accepted,
                sprintf('%s read %s although it describes what they hold', $declared, implode(', ', $accepted))
            );
        }
    }

    public function testAMemberThatCarriesAListRefusesADocumentThatIsNotOne(): void
    {
        // Which members carry a list is read off what each value wrote rather than named here, so a
        // schema that grows one is held to this the moment it does.
        $walked = 0;
        foreach (GeneratedSurface::models() as $declared) {
            $written = Synthesis::model($declared, true)->toArray();
            foreach ($written as $name => $value) {
                if (!is_array($value) || !array_is_list($value)) {
                    continue;
                }

                try {
                    $declared::fromJson([$name => 'not a list at all'] + $written);
                    self::fail(sprintf('%s read a string as the list `%s` carries', $declared, $name));
                } catch (\Hook0\DecodeError $refused) {
                    self::assertStringContainsString('expected an array', $refused->getMessage());
                    self::assertStringContainsString((string) $name, $refused->getMessage());
                }
                $walked++;
            }
        }

        self::assertGreaterThan(0, $walked, 'no value the API declares carries a list');
    }

    public function testAMomentIsRefusedWhenItNamesNoMomentAndWhenItNamesNoDay(): void
    {
        // Which members carry a moment is not named here: it is read off what each value wrote, so a
        // schema that grows one is held to this the moment it does.
        $walked = 0;
        foreach (GeneratedSurface::models() as $declared) {
            $written = Synthesis::model($declared, true)->toArray();
            foreach ($written as $name => $value) {
                $whole = is_string($value) && str_contains($value, 'T');
                $day = is_string($value) && preg_match('/^\d{4}-\d{2}-\d{2}$/', (string) $value) === 1;
                if (!$whole && !$day) {
                    continue;
                }

                foreach (['not a moment at all', $whole ? '2026-02-31T00:00:00+00:00' : '2026-02-31'] as $wrong) {
                    try {
                        $declared::fromJson([$name => $wrong] + $written);
                        self::fail(sprintf('%s read `%s` as a moment under `%s`', $declared, $wrong, $name));
                    } catch (\Hook0\DecodeError $refused) {
                        self::assertStringContainsString((string) $name, $refused->getMessage());
                    }
                    $walked++;
                }
            }
        }

        self::assertGreaterThan(0, $walked, 'no value the API declares carries a moment');
    }

    public function testEveryProblemTheCatalogueNamesIsRaisedAsTheFailureItIs(): void
    {
        /** @var array<string, class-string<ProblemError>> $catalogue */
        $catalogue = (new \ReflectionClass(ProblemError::class))->getConstant('PROBLEMS');
        self::assertNotSame([], $catalogue, 'the generator mapped no problem at all');

        foreach ($catalogue as $problem => $expected) {
            $this->api->willAnswer(new ScriptedResponse(400, [
                'id' => $problem,
                'status' => 400,
                'title' => 'refused',
                'detail' => 'what this case scripted',
                'type' => 'https://hook0.com/documentation/errors/' . $problem,
            ]));

            try {
                $this->applications()->get(self::APPLICATION_UUID);
                self::fail($problem . ' was read as a success');
            } catch (ProblemError $reported) {
                self::assertInstanceOf($expected, $reported, $problem . ' was not raised as the failure it names');
                self::assertSame(400, $reported->status);
                self::assertSame($problem, $reported->problem?->id->value);
                self::assertSame('what this case scripted', $reported->problem?->detail);
            }
        }
    }

    public function testAFailureTooLongToReportWholeIsCutWithoutBreakingACharacterInTwo(): void
    {
        // What a failure carries is written by a server this client does not control and is cut to a
        // budget in bytes, which lands wherever it lands inside a character. What is reported is held
        // to being text all the same, rather than travelling half a character into what a caller logs.
        $answered = str_repeat('e', Runtime::MAX_PREVIEW_BYTES - 1) . 'éclat, and not a problem document';
        $this->api->willAnswer(new ScriptedResponse(500, $answered));

        try {
            $this->applications()->get(self::APPLICATION_UUID);
            self::fail('a body that is not a problem document was read as a success');
        } catch (ProblemError $reported) {
            self::assertSame(500, $reported->status);
            self::assertNull($reported->problem);
            self::assertSame(1, preg_match('//u', $reported->getMessage()), 'the report is not text');
            self::assertStringContainsString('…', $reported->getMessage(), 'the report was not cut');
            self::assertStringNotContainsString('éclat', $reported->getMessage());
        }
    }

    /**
     * What the API answers one operation, and the value that operation is expected to read out of it.
     *
     * @return array{0: mixed, 1: mixed}
     */
    private function answerFor(\ReflectionMethod $operation, bool $optionals): array
    {
        $answers = Synthesis::answered($operation);
        if ($answers === null) {
            return [null, null];
        }

        [$type, $listed] = $answers;
        $held = Synthesis::valueFor($type, $optionals);

        return $listed
            ? [[Synthesis::written($held)], [$held]]
            : [Synthesis::written($held), $held];
    }

    /**
     * @param array<string, mixed> $given
     */
    private function assertPathFilledIn(
        \Hook0\Tests\Support\DeclaredOperation $declared,
        \Hook0\Tests\Support\ReceivedRequest $request,
        string $named
    ): void {
        $sent = $declared->segmentsOf($request);
        foreach (explode('/', $declared->template) as $index => $segment) {
            if (!str_starts_with($segment, '{')) {
                continue;
            }
            // The value lands in the path escaped, so that nothing in it can name a segment the
            // operation never had.
            self::assertSame(
                rawurlencode(Synthesis::ARGUMENT_TEXT),
                $sent[$index],
                sprintf('%s left `%s` unescaped', $named, $segment)
            );
        }
    }

    /**
     * @param array<string, mixed> $given
     */
    private function assertQueryAssembled(
        \Hook0\Tests\Support\DeclaredOperation $declared,
        \Hook0\Tests\Support\ReceivedRequest $request,
        \ReflectionMethod $operation,
        array $given,
        bool $optionals,
        string $named
    ): void {
        $carried = \Hook0\Tests\Support\DeclaredOperation::queryOf($request);
        $names = array_keys($carried);
        sort($names);

        self::assertSame(
            $declared->queryNames($optionals),
            $names,
            $named . ' assembled a query the document does not declare'
        );

        foreach (Synthesis::wireNames($operation) as $argument => $wire) {
            if (!isset($carried[$wire]) || !array_key_exists($argument, $given)) {
                continue;
            }
            self::assertSame([$given[$argument]], $carried[$wire], sprintf('%s carried `%s` altered', $named, $wire));
        }
    }

    /**
     * @param array<string, mixed> $given
     */
    private function assertBodySent(
        \Hook0\Tests\Support\ReceivedRequest $request,
        array $given,
        string $named
    ): void {
        foreach ($given as $value) {
            if (!is_object($value) || !method_exists($value, 'toArray')) {
                continue;
            }
            self::assertSame(
                Runtime::encode($value->toArray()),
                Runtime::encode($request->json()),
                $named . ' sent a body the API cannot read back'
            );
        }
    }

    /**
     * Every member of a value that holds nothing, which is every one the case left out.
     *
     * @return list<string>
     */
    private function unset(string $declared, object $held): array
    {
        $absent = [];
        foreach (Synthesis::wireNames((new \ReflectionClass($declared))->getConstructor()) as $name => $wire) {
            if ($held->{$name} === null) {
                $absent[] = $wire;
            }
        }

        return $absent;
    }

    /** How many members of a value the document describes nothing about. */
    private function opaque(string $declared): int
    {
        $constructor = (new \ReflectionClass($declared))->getConstructor();
        $opaque = 0;
        foreach (Synthesis::declaredTypes($constructor) as $type) {
            if (Synthesis::peeled($type) === 'mixed') {
                $opaque++;
            }
        }

        return $opaque;
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
