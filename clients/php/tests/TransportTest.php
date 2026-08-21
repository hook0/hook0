<?php

declare(strict_types=1);

namespace Hook0\Tests;

use Hook0\Tests\Support\ApiCase;
use Hook0\Tests\Support\ScriptedResponse;
use Hook0\Transport;
use Hook0\TransportError;

/**
 * Where a request lands, which is decided before a socket is opened.
 *
 * What a base URL and a path add up to is the one thing about a request this client settles on its
 * own: everything else is either what the caller asked for or what the API answered. A base that
 * names nowhere is therefore refused rather than turned into a message accusing the network, and a
 * path carrying a scheme of its own goes where it says rather than under the base.
 */
final class TransportTest extends ApiCase
{
    public function testAPathCarryingASchemeOfItsOwnGoesThereRatherThanUnderTheBase(): void
    {
        // A caller handed an absolute URL — a link the API answered with, say — is sending to that
        // URL. Pasting it under the base would build a target neither of them names.
        $this->api->willAnswer(new ScriptedResponse(200, ['reached' => true]));

        $transport = new Transport('http://nowhere.invalid/api/v1', self::TOKEN);
        [$status, $payload] = $transport->request('GET', $this->api->baseUrl() . '/somewhere-else');

        self::assertSame(200, $status);
        self::assertSame('{"reached":true}', $payload);
        self::assertSame('/somewhere-else', $this->api->received()[0]->target);
    }

    public function testABaseUrlNoPathCanBeResolvedAgainstIsRefusedBeforeAnythingIsSent(): void
    {
        $transport = new Transport('not a base url', self::TOKEN);

        try {
            $transport->request('GET', '/event');
            self::fail('a request was built on a base URL that names nowhere');
        } catch (TransportError $refused) {
            self::assertSame('unusable_api_url', $refused->causeName);
            self::assertFalse($refused->retryable, 'building the same unusable request again cannot end differently');
            self::assertStringContainsString('not a base url', $refused->getMessage());
        }
    }

    public function testATargetNamingNoSchemeAtAllIsRefusedBeforeAnythingIsSent(): void
    {
        // A relative path extends the base, so a base that is not a URL leaves a target that names
        // no scheme and no host — nowhere to send anything to.
        $transport = new Transport('nowhere', self::TOKEN);

        try {
            $transport->request('GET', 'event');
            self::fail('a request was sent somewhere that names neither a scheme nor a host');
        } catch (TransportError $refused) {
            self::assertSame('unusable_api_url', $refused->causeName);
            self::assertStringContainsString('nowhere/event', $refused->getMessage());
        }
    }

    public function testARootedPathReplacesTheBasePathAndItsQueryTravelsEscaped(): void
    {
        // The generated half writes whole paths — `/api/v1/…` — so a base carrying a path of its own
        // must not have it pasted in front of them a second time.
        $this->api->willAnswer(new ScriptedResponse(200, []), new ScriptedResponse(200, []));

        $transport = new Transport($this->api->baseUrl() . '/somewhere', self::TOKEN);
        $transport->request('GET', '/api/v1/event_types', [['a name', 'a value/with a space'], ['and', 'another']]);
        $transport->request('GET', 'beside-the-base');

        self::assertSame(
            '/api/v1/event_types?a%20name=a%20value%2Fwith%20a%20space&and=another',
            $this->api->received()[0]->target
        );
        // A relative one extends it instead, which is the other half of the same rule.
        self::assertSame('/somewhere/beside-the-base', $this->api->received()[1]->target);
    }
}
