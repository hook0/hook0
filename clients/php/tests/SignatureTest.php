<?php

declare(strict_types=1);

namespace Hook0\Tests;

use Hook0\ClientError;
use Hook0\Signature;
use PHPUnit\Framework\TestCase;

/**
 * What this client's own verification refuses, beyond what the shared corpus dictates.
 *
 * Every vector the corpus carries — the two schemes, the order the covered headers are named in, a
 * covered header that never arrived, a code that is not whole hexadecimal, and a moment on either
 * side of the window — is run against this client by {@see ConformanceTest}, out of the committed
 * document rather than out of a copy of it. What is here is the rest: the bounds this client sets on
 * a header written by whoever reached the endpoint, and the shapes a signature can take that the
 * shared contract has nothing to say about.
 */
final class SignatureTest extends TestCase
{
    private const SECRET = 'a-subscription-secret';
    private const PAYLOAD = '{"event":"user.created"}';
    private const MOMENT = 1800000000;
    private const TOLERANCE = 300.0;

    /** @var list<array{0: string, 1: string}> */
    private const HEADERS = [
        ['x-event-id', 'evt-1'],
        ['x-delivery-id', 'dlv-1'],
        ['content-type', 'application/json'],
    ];

    public function testASignatureAboveTheAcceptedSizeIsRefusedBeforeItIsSplit(): void
    {
        $header = 't=' . self::MOMENT . ',v0=' . str_repeat('00', Signature::MAX_SIGNATURE_BYTES);

        $this->expectException(ClientError::class);
        $this->expectExceptionMessageMatches('/above the \d+ accepted/');

        Signature::parse($header);
    }

    public function testASignatureCarryingMorePartsThanAcceptedIsRefused(): void
    {
        $header = implode(',', array_fill(0, Signature::MAX_SIGNATURE_PARTS + 1, 'x=1'));

        $this->expectException(ClientError::class);
        $this->expectExceptionMessageMatches('/more than the \d+ parts accepted/');

        Signature::parse($header);
    }

    public function testASignatureCoveringMoreHeadersThanAcceptedIsRefused(): void
    {
        $covered = implode(' ', array_fill(0, Signature::MAX_COVERED_HEADERS + 1, 'x-pad'));

        $this->expectException(ClientError::class);
        $this->expectExceptionMessageMatches('/more than the \d+ headers accepted/');

        Signature::parse(sprintf('t=%d,h=%s,v1=00', self::MOMENT, $covered));
    }

    public function testASignatureNamingNoMomentIsRefused(): void
    {
        $this->expectException(ClientError::class);
        $this->expectExceptionMessageMatches('/carries no `t` part/');

        Signature::parse('v0=00,v1=00');
    }

    public function testASignatureCarryingNeitherCodeIsRefused(): void
    {
        $this->expectException(ClientError::class);
        $this->expectExceptionMessageMatches('/neither a `v0` nor a `v1` code/');

        Signature::parse(sprintf('t=%d,h=x-event-id', self::MOMENT));
    }

    public function testAMomentFurtherFromTheEpochThanAcceptedIsRefusedBeforeItIsRead(): void
    {
        // Written as digits by whoever reached the endpoint, so how many of them there are is settled
        // before any of them reaches the arithmetic that holds the moment against the current time.
        $this->expectException(ClientError::class);
        $this->expectExceptionMessageMatches('/further than \d+ seconds from the epoch/');

        Signature::parse('t=' . str_repeat('9', 40) . ',v0=00');
    }

    public function testACoveredHeaderThatIsNotAHeaderNameIsRefused(): void
    {
        $this->expectException(ClientError::class);
        $this->expectExceptionMessageMatches('/is not a header name/');

        Signature::parse(sprintf('t=%d,h=x-event-id;drop,v1=00', self::MOMENT));
    }

    public function testAHeaderThatIsNotTextIsRefused(): void
    {
        $header = $this->signed(self::MOMENT, ['x-event-id'], self::PAYLOAD);

        $this->expectException(ClientError::class);
        $this->expectExceptionMessageMatches('/not a header value/');

        Signature::verifyWithCurrentTime(
            $header,
            self::PAYLOAD,
            ['x-event-id' => 42],
            self::SECRET,
            self::TOLERANCE,
            new \DateTimeImmutable('@' . self::MOMENT)
        );
    }

    public function testADeliveryCarryingMoreHeadersThanAcceptedIsRefusedBeforeAnyIsRead(): void
    {
        // The headers are whatever reached the endpoint, so how many of them are read at all is this
        // client's to bound rather than the sender's to decide.
        $delivered = [];
        for ($index = 0; $index <= Signature::MAX_DELIVERED_HEADERS; $index++) {
            $delivered['x-header-' . $index] = 'a value';
        }

        $this->expectException(ClientError::class);
        $this->expectExceptionMessageMatches('/more than the \d+ headers accepted/');

        Signature::verifyWithCurrentTime(
            $this->signed(self::MOMENT, ['x-event-id'], self::PAYLOAD),
            self::PAYLOAD,
            $delivered,
            self::SECRET,
            self::TOLERANCE,
            new \DateTimeImmutable('@' . self::MOMENT)
        );
    }

    public function testAHeaderHeldAsPairsThatIsNotANameAndAValueIsRefused(): void
    {
        $this->expectException(ClientError::class);
        $this->expectExceptionMessageMatches('/not a name and a value/');

        Signature::verifyWithCurrentTime(
            $this->signed(self::MOMENT, ['x-event-id'], self::PAYLOAD),
            self::PAYLOAD,
            [['x-event-id', 'evt-1', 'and something else']],
            self::SECRET,
            self::TOLERANCE,
            new \DateTimeImmutable('@' . self::MOMENT)
        );
    }

    public function testAHeaderThatIsNotUtf8IsRefusedRatherThanSigned(): void
    {
        // What is signed is a message built out of these bytes, so a value that is not text at all
        // stops the verification rather than reaching the hash as whatever the language made of it.
        $this->expectException(ClientError::class);
        $this->expectExceptionMessageMatches('/not UTF-8/');

        Signature::verifyWithCurrentTime(
            $this->signed(self::MOMENT, ['x-event-id'], self::PAYLOAD),
            self::PAYLOAD,
            ['x-event-id' => "\xff\xfe"],
            self::SECRET,
            self::TOLERANCE,
            new \DateTimeImmutable('@' . self::MOMENT)
        );
    }

    public function testHeadersReadTheSameWayWhetherTheyAreAMapOrPairs(): void
    {
        $header = $this->signed(self::MOMENT, ['x-event-id', 'x-delivery-id'], self::PAYLOAD);

        $asPairs = self::HEADERS;
        $asAMap = [];
        foreach (self::HEADERS as [$name, $value]) {
            $asAMap[$name] = $value;
        }

        foreach ([$asPairs, $asAMap] as $held) {
            Signature::verifyWithCurrentTime(
                $header,
                self::PAYLOAD,
                $held,
                self::SECRET,
                self::TOLERANCE,
                new \DateTimeImmutable('@' . self::MOMENT)
            );
        }

        self::assertTrue(true, 'a caller holds the headers of a delivery either way');
    }

    public function testTheNameOfACoveredHeaderIsMatchedWithoutRegardToCase(): void
    {
        $header = $this->signed(self::MOMENT, ['x-event-id'], self::PAYLOAD);

        Signature::verifyWithCurrentTime(
            $header,
            self::PAYLOAD,
            ['X-Event-Id' => 'evt-1'],
            self::SECRET,
            self::TOLERANCE,
            new \DateTimeImmutable('@' . self::MOMENT)
        );

        self::assertTrue(true, 'a server writes a header name in whatever case it likes');
    }

    public function testADeliverySignedNowVerifiesAgainstTheCurrentMoment(): void
    {
        // The variant that reads the clock rather than being handed one is what an application calls,
        // and it is the only place the current moment is looked up.
        $now = time();

        Signature::verify(
            $this->signed($now, [], self::PAYLOAD),
            self::PAYLOAD,
            self::HEADERS,
            self::SECRET,
            self::TOLERANCE
        );

        self::assertTrue(true, 'a delivery signed at the current moment is inside any tolerance');
    }

    public function testADeliverySignedLongAgoIsRefusedAgainstTheCurrentMoment(): void
    {
        $this->expectException(ClientError::class);
        $this->expectExceptionMessageMatches('/outside the/');

        Signature::verify(
            $this->signed(self::MOMENT, [], self::PAYLOAD),
            self::PAYLOAD,
            self::HEADERS,
            self::SECRET,
            self::TOLERANCE
        );
    }

    /**
     * A signature over that moment and those headers, computed the way a sender does.
     *
     * @param list<string> $covered
     */
    private function signed(int $moment, array $covered, string $payload): string
    {
        $delivered = [];
        foreach (self::HEADERS as [$name, $value]) {
            $delivered[$name] = $value;
        }

        if ($covered === []) {
            $code = hash_hmac('sha256', $moment . '.' . $payload, self::SECRET);

            return sprintf('t=%d,v0=%s', $moment, $code);
        }

        $values = array_map(static fn (string $name): string => $delivered[$name], $covered);
        $message = $moment . '.' . implode(' ', $covered) . '.' . implode('.', $values) . '.' . $payload;

        return sprintf('t=%d,h=%s,v1=%s', $moment, implode(' ', $covered), hash_hmac('sha256', $message, self::SECRET));
    }
}
