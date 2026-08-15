<?php

declare(strict_types=1);

namespace Hook0;

/**
 * How a request reaches the API, and what a server on the other end is not allowed to cost.
 *
 * The transport answers the status and the bytes and knows nothing of what the API declares: reading
 * those bytes is the generated half's job, and deciding whether to send them again is the client's.
 * That is what lets one HTTP implementation serve both the hand-written event path and every
 * generated method — a generated group calls whatever object it is handed, and this is the one this
 * package ships.
 *
 * Nothing here reaches for a third-party HTTP library. What the bundled one bounds on its own is
 * little: it refuses to follow a redirect unless asked, and it holds one response header line to a
 * hundred kilobytes. It bounds neither how many header lines it accepts, nor the head as a whole,
 * nor the body, and its own per-line ceiling sits above the one the shared corpus names. So every
 * ceiling that matters is applied here, in the callbacks the exchange is read through, which is also
 * what lets a head past the count or a line past its length be refused before the body is read at
 * all.
 */
final class Transport
{
    /**
     * Longest one attempt at reaching the API is given before it is abandoned, in seconds.
     *
     * Ten seconds is far above what ingesting an event takes when the API is healthy, and short
     * enough that a stuck connection does not hold a caller for a noticeable time.
     */
    public const DEFAULT_REQUEST_TIMEOUT = 10.0;

    /** Largest response body read off a socket, in bytes. */
    public const DEFAULT_MAX_RESPONSE_BYTES = 8 * 1024 * 1024;

    /**
     * How many header lines an answer may carry before it is refused.
     *
     * Sixty-four is well above what the API sends, and the point of the bound is to refuse early:
     * the line that crosses it stops the exchange before the body is read.
     */
    public const DEFAULT_MAX_RESPONSE_HEADERS = 64;

    /** Longest one header line may be, name and value together, in bytes. */
    public const DEFAULT_MAX_HEADER_BYTES = 64 * 1024;

    /**
     * Largest whole head an answer may carry, every line counted together, in bytes.
     *
     * This is the one that bounds what a head costs. A line count and a size per line multiply:
     * sixty-four lines of sixty-four kilobytes each is four megabytes of head, and both of the
     * bounds above admit it. They earn their place by refusing early, on the line that crosses them
     * rather than at the end of the head; this one sets the ceiling.
     */
    public const DEFAULT_MAX_HEAD_BYTES = 16 * 1024;

    /** What a request body says it carries, and what an answer is asked for in. */
    private const JSON_MEDIA_TYPE = 'application/json';

    /**
     * Longest each part this client composes its `User-Agent` out of may be, in characters.
     *
     * The runtime and the operating system are described by the platform rather than by this package,
     * so their length is not this package's to guarantee: they are cut here so that the header cannot
     * grow with whatever the platform feels like saying. Every part is also stripped of anything the
     * grammar of the header uses as punctuation, so a platform cannot forge a shape it does not have.
     */
    private const MAX_USER_AGENT_PART_CHARS = 64;

    /** What this package is installed under, which is what its version is asked for by. */
    private const PACKAGE_NAME = 'hook0/client';

    /** What a version reads as where nothing that knows one is there to be asked. */
    private const UNKNOWN_VERSION = 'unknown';

    /** The schemes this transport reaches. */
    private const SCHEMES = ['http', 'https'];

    /** What a URL carrying a scheme of its own looks like. */
    private const ABSOLUTE_URL = '#^[A-Za-z][A-Za-z0-9+.\-]*://#';

    /** Shortest timeout the underlying library reads as a timeout rather than as none at all. */
    private const MIN_TIMEOUT_MS = 1;

    /**
     * @param string $baseUrl where the API lives, such as https://app.hook0.com/api/v1
     * @param string $token an authentication token valid for that API
     * @param float $timeout how long one attempt is given, in seconds
     * @param int $maxResponseBytes the largest answer read off a socket
     * @param int $maxResponseHeaders how many header lines an answer may carry
     * @param int $maxHeaderBytes the longest one header line may be
     * @param int $maxHeadBytes the largest whole head, every line counted together
     */
    public function __construct(
        private readonly string $baseUrl,
        private readonly string $token,
        private readonly float $timeout = self::DEFAULT_REQUEST_TIMEOUT,
        private readonly int $maxResponseBytes = self::DEFAULT_MAX_RESPONSE_BYTES,
        private readonly int $maxResponseHeaders = self::DEFAULT_MAX_RESPONSE_HEADERS,
        private readonly int $maxHeaderBytes = self::DEFAULT_MAX_HEADER_BYTES,
        private readonly int $maxHeadBytes = self::DEFAULT_MAX_HEAD_BYTES,
    ) {
    }

    /**
     * What the API answered, whether or not it answered a success.
     *
     * This is the shape the generated half of this package reads, which is the status and the bytes.
     * A caller that also needs what the answer carried beside its body — the delay a paced instance
     * names is one — asks {@see self::deliver()} for it.
     *
     * @param list<array{0: string, 1: string}> $query the name and value pairs of the query string
     * @param mixed $body what to send as a JSON document, or nothing at all
     * @return array{0: int, 1: string} the status and the body
     */
    public function request(string $method, string $path, array $query = [], mixed $body = null): array
    {
        [$status, , $payload] = $this->deliver($method, $path, $query, $body);

        return [$status, $payload];
    }

    /**
     * What the API answered, headers included, whether or not it answered a success.
     *
     * Header names are lowercased and a later value wins over an earlier one under the same name, so
     * a caller reads a header without knowing which case the server wrote it in.
     *
     * @param list<array{0: string, 1: string}> $query the name and value pairs of the query string
     * @param mixed $body what to send as a JSON document, or nothing at all
     * @return array{0: int, 1: array<string, string>, 2: string} the status, the headers and the body
     */
    public function deliver(string $method, string $path, array $query = [], mixed $body = null): array
    {
        return $this->exchange($method, $this->resolved($path, $query), $body);
    }

    /**
     * Where a request lands: a path of its own replaces the base's, a relative one extends it.
     */
    private function resolved(string $path, array $query): string
    {
        $target = $this->joined($path);

        $parts = parse_url($target);
        if ($parts === false || !isset($parts['scheme'], $parts['host'])) {
            throw TransportError::unusableApiUrl(
                sprintf('`%s` is not somewhere this transport can send a request', $target)
            );
        }
        if (!in_array(strtolower($parts['scheme']), self::SCHEMES, true) || $parts['host'] === '') {
            throw TransportError::unusableApiUrl(
                sprintf('`%s` is not somewhere this transport can send a request', $target)
            );
        }
        if (count($query) === 0) {
            return $target;
        }

        $written = [];
        foreach ($query as [$name, $value]) {
            $written[] = rawurlencode($name) . '=' . rawurlencode($value);
        }
        $separator = str_contains($target, '?') ? '&' : '?';

        return $target . $separator . implode('&', $written);
    }

    /** The base URL and the path an operation names, put together the way a browser would. */
    private function joined(string $path): string
    {
        if (preg_match(self::ABSOLUTE_URL, $path) === 1) {
            return $path;
        }

        $base = rtrim($this->baseUrl, '/') . '/';
        if (!str_starts_with($path, '/')) {
            return $base . $path;
        }

        $parts = parse_url($base);
        if ($parts === false || !isset($parts['scheme'], $parts['host'])) {
            throw TransportError::unusableApiUrl(
                sprintf('`%s` is not a base URL a path can be resolved against', $this->baseUrl)
            );
        }
        $port = isset($parts['port']) ? ':' . $parts['port'] : '';

        return $parts['scheme'] . '://' . $parts['host'] . $port . $path;
    }

    /**
     * One exchange, bounded on every axis a server controls.
     *
     * @return array{0: int, 1: array<string, string>, 2: string}
     */
    private function exchange(string $method, string $target, mixed $body): array
    {
        $headers = [];
        $payload = '';
        $head = 0;
        $lines = 0;
        $refused = null;

        $handle = curl_init();
        curl_setopt_array($handle, $this->options($method, $target, $body));
        curl_setopt(
            $handle,
            CURLOPT_HEADERFUNCTION,
            function (\CurlHandle $handle, string $line) use (&$headers, &$head, &$lines, &$refused): int {
                $read = strlen($line);
                $refused ??= $this->refusedHead($line, $head + $read, $lines + 1);
                if ($refused !== null) {
                    return 0;
                }

                $head += $read;
                $name = strstr($line, ':', true);
                if ($name !== false) {
                    $lines++;
                    $headers[strtolower(trim($name))] = trim(substr($line, strlen($name) + 1));
                }

                return $read;
            }
        );
        curl_setopt(
            $handle,
            CURLOPT_WRITEFUNCTION,
            function (\CurlHandle $connection, string $chunk) use (&$payload, &$refused): int {
                if (strlen($payload) + strlen($chunk) > $this->maxResponseBytes) {
                    $refused ??= sprintf(
                        'the API answered more than the %d bytes read at most',
                        $this->maxResponseBytes
                    );

                    return 0;
                }
                $payload .= $chunk;

                return strlen($chunk);
            }
        );

        $answered = curl_exec($handle);
        $failure = curl_error($handle);
        $status = (int) curl_getinfo($handle, CURLINFO_RESPONSE_CODE);
        unset($handle);

        if ($refused !== null) {
            throw TransportError::answerAboveABound($refused);
        }
        if ($answered === false) {
            throw TransportError::noAnswer($failure === '' ? 'the API answered nothing' : $failure);
        }

        return [$status, $headers, $payload];
    }

    /**
     * Why a head this long, or this many lines in, is one this client will not read, when it is.
     *
     * The status line carries no name and is not counted as a header, but its bytes are: it is part
     * of what the head costs.
     */
    private function refusedHead(string $line, int $head, int $lines): ?string
    {
        $named = strstr($line, ':', true) !== false;

        if ($named && strlen(rtrim($line, "\r\n")) > $this->maxHeaderBytes) {
            $name = strtolower(trim((string) strstr($line, ':', true)));

            return sprintf(
                'the API answered a `%s` header above the %d bytes read at most',
                $name,
                $this->maxHeaderBytes
            );
        }
        if ($named && $lines > $this->maxResponseHeaders) {
            return sprintf(
                'the API answered more than the %d header lines read at most',
                $this->maxResponseHeaders
            );
        }
        if ($head > $this->maxHeadBytes) {
            return sprintf(
                'the API answered a head above the %d bytes read at most',
                $this->maxHeadBytes
            );
        }

        return null;
    }

    /**
     * Everything one request is issued under, beyond the callbacks that bound what it may cost.
     *
     * @return array<int, mixed>
     */
    private function options(string $method, string $target, mixed $body): array
    {
        $milliseconds = max(self::MIN_TIMEOUT_MS, (int) round($this->timeout * 1000));

        $lines = [
            'Authorization: Bearer ' . $this->token,
            'Accept: ' . self::JSON_MEDIA_TYPE,
            'User-Agent: ' . self::userAgent(),
            // The library asks the server for permission before sending a body of any size, and
            // waits a second for an answer that a plain HTTP/1.1 server never sends. Nothing here
            // needs that handshake, so it is turned off rather than waited out.
            'Expect:',
        ];

        $options = [
            CURLOPT_URL => $target,
            CURLOPT_CUSTOMREQUEST => strtoupper($method),
            CURLOPT_RETURNTRANSFER => true,
            // A redirect is an answer, not a place to follow: chasing one would send the credential
            // to wherever the answer pointed, as many times as it kept pointing.
            CURLOPT_FOLLOWLOCATION => false,
            CURLOPT_TIMEOUT_MS => $milliseconds,
            CURLOPT_CONNECTTIMEOUT_MS => $milliseconds,
            // Timeouts under a second are only honoured when the library is told not to reach for
            // the alarm signal it otherwise uses to interrupt name resolution.
            CURLOPT_NOSIGNAL => true,
        ];

        if ($body !== null) {
            $lines[] = 'Content-Type: ' . self::JSON_MEDIA_TYPE;
            $options[CURLOPT_POSTFIELDS] = Runtime::encode($body);
        }
        $options[CURLOPT_HTTPHEADER] = $lines;

        return $options;
    }

    /**
     * Which SDK, at which version, on which runtime and operating system, is talking to the API.
     *
     * A Composer package carries no version of its own: the registry reads one off the tag that
     * publishes it, which is why `composer.json` declares none and whatever installed this package is
     * the only thing that can be asked. A source checkout that went through no installer has nothing
     * to ask, and says so rather than naming a version it cannot stand behind. So does a project that
     * has Composer but copied these sources in rather than requiring them: asking after a package
     * nothing installed raises rather than answering, and a header is no place to find that out.
     */
    private static function userAgent(): string
    {
        $installed = class_exists(\Composer\InstalledVersions::class)
            && \Composer\InstalledVersions::isInstalled(self::PACKAGE_NAME)
            ? \Composer\InstalledVersions::getPrettyVersion(self::PACKAGE_NAME)
            : null;

        return sprintf(
            'hook0-client-php/%s (%s; %s)',
            self::clipped($installed ?? self::UNKNOWN_VERSION),
            self::clipped('php ' . PHP_VERSION),
            self::clipped(PHP_OS_FAMILY . ' ' . php_uname('m'))
        );
    }

    /**
     * One part of the `User-Agent`, with everything the header's own grammar uses taken out of it and
     * cut to {@see self::MAX_USER_AGENT_PART_CHARS}.
     */
    private static function clipped(string $part): string
    {
        $printable = (string) preg_replace('/[^\x20-\x7E]/', '', $part);

        return substr(str_replace(['(', ')', ';'], '', $printable), 0, self::MAX_USER_AGENT_PART_CHARS);
    }
}
