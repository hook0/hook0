<?php

declare(strict_types=1);

namespace Hook0\Tests\Support;

/**
 * A Hook0 API on a loopback port, for the lifetime of one case.
 *
 * It runs in a process of its own rather than a thread, because the language has neither: a case has
 * to be able to hold one answer past the client's timeout and still serve the attempt that follows
 * it, which is exactly what a client repeating a timed-out request walks into. Connections are
 * therefore multiplexed here, each one carrying the moment it is to be answered at, so a held answer
 * holds that exchange and no other.
 *
 * Everything is bounded: how long the process lives, how many connections it holds, how much of a
 * request it reads, and how long the head of one may be. It speaks as much HTTP/1.1 as one exchange
 * needs and nothing more.
 *
 * It is told what to answer through a file of scripted responses, and it answers what it received
 * through a file of one JSON document per line, so that a case reads back exactly what arrived.
 */
final class FakeApiServer
{
    /** Most connections held at once. */
    private const MAX_CONNECTIONS = 64;

    /** Most bytes of one request read, head and body together. */
    private const MAX_REQUEST_BYTES = 512 * 1024;

    /** Most bytes of the head of one request read before the body is looked for. */
    private const MAX_HEAD_BYTES = 64 * 1024;

    /** How long one pass through the loop waits for something to happen, in microseconds. */
    private const TICK_MICROSECONDS = 2000;

    /** How much is read off one ready socket at a time. */
    private const CHUNK_BYTES = 8192;

    /** What separates the head of a request from its body. */
    private const HEAD_SEPARATOR = "\r\n\r\n";

    /** @var list<array{socket: resource, in: string, out: string|null, at: float}> */
    private array $connections = [];

    private int $answered = 0;

    private function __construct(
        private readonly string $scriptPath,
        private readonly string $logPath,
        private readonly float $deadline,
    ) {
    }

    /**
     * Serves until the deadline the caller named, and answers what to exit with.
     *
     * @param list<string> $arguments the script, the log, and how long to live for
     */
    public static function run(array $arguments): int
    {
        if (count($arguments) < 3) {
            fwrite(STDERR, "usage: serve.php <script> <log> <lifetime-seconds>\n");

            return 1;
        }

        $listener = stream_socket_server('tcp://127.0.0.1:0', $errorNumber, $errorText);
        if ($listener === false) {
            fwrite(STDERR, sprintf("listening failed: %s (%d)\n", $errorText, $errorNumber));

            return 1;
        }
        stream_set_blocking($listener, false);

        $address = (string) stream_socket_get_name($listener, false);
        fwrite(STDOUT, substr($address, (int) strrpos($address, ':') + 1) . "\n");
        fflush(STDOUT);

        $server = new self(
            $arguments[0],
            $arguments[1],
            microtime(true) + (float) $arguments[2]
        );
        $server->serve($listener);

        return 0;
    }

    /**
     * @param resource $listener
     */
    private function serve($listener): void
    {
        while (microtime(true) < $this->deadline) {
            $readable = [$listener];
            foreach ($this->connections as $connection) {
                if ($connection['out'] === null) {
                    $readable[] = $connection['socket'];
                }
            }
            $writable = [];
            $failed = [];
            @stream_select($readable, $writable, $failed, 0, self::TICK_MICROSECONDS);

            if (in_array($listener, $readable, true)) {
                $this->accept($listener);
            }
            $this->read($readable);
            $this->answer();
        }

        foreach ($this->connections as $connection) {
            fclose($connection['socket']);
        }
        fclose($listener);
    }

    /**
     * @param resource $listener
     */
    private function accept($listener): void
    {
        $accepted = @stream_socket_accept($listener, 0);
        if ($accepted === false) {
            return;
        }
        if (count($this->connections) >= self::MAX_CONNECTIONS) {
            fclose($accepted);

            return;
        }

        stream_set_blocking($accepted, false);
        $this->connections[] = ['socket' => $accepted, 'in' => '', 'out' => null, 'at' => 0.0];
    }

    /**
     * @param list<resource> $readable
     */
    private function read(array $readable): void
    {
        foreach ($this->connections as $index => $connection) {
            if ($connection['out'] !== null || !in_array($connection['socket'], $readable, true)) {
                continue;
            }

            $chunk = @fread($connection['socket'], self::CHUNK_BYTES);
            if ($chunk === false || $chunk === '') {
                if (feof($connection['socket'])) {
                    fclose($connection['socket']);
                    unset($this->connections[$index]);
                }
                continue;
            }

            $read = $connection['in'] . $chunk;
            $head = strpos($read, self::HEAD_SEPARATOR);
            if (strlen($read) > self::MAX_REQUEST_BYTES || ($head === false && strlen($read) > self::MAX_HEAD_BYTES)) {
                fclose($connection['socket']);
                unset($this->connections[$index]);
                continue;
            }
            $this->connections[$index]['in'] = $read;

            $request = self::complete($read);
            if ($request === null) {
                continue;
            }

            file_put_contents(
                $this->logPath,
                json_encode($request, JSON_INVALID_UTF8_SUBSTITUTE) . "\n",
                FILE_APPEND
            );

            $scripted = $this->scripted($this->answered);
            $this->answered++;
            $this->connections[$index]['out'] = self::written($scripted);
            $this->connections[$index]['at'] = microtime(true) + $scripted['held_for'];
        }
    }

    private function answer(): void
    {
        foreach ($this->connections as $index => $connection) {
            if ($connection['out'] === null || microtime(true) < $connection['at']) {
                continue;
            }

            @fwrite($connection['socket'], $connection['out']);
            fclose($connection['socket']);
            unset($this->connections[$index]);
        }
    }

    /**
     * The answer scripted for request number `$index`, or the one that says none was.
     *
     * The file is read afresh every time, so a case that queues an answer after the process started
     * still has it served.
     *
     * @return array{status: int, body: mixed, held_for: float, headers: array<string, string>}
     */
    private function scripted(int $index): array
    {
        $written = @file_get_contents($this->scriptPath);
        $queued = $written === false ? [] : json_decode($written, true, 32);
        if (!is_array($queued) || !isset($queued[$index]) || !is_array($queued[$index])) {
            return [
                'status' => 500,
                'body' => ['error' => 'the case scripted no answer for this request'],
                'held_for' => 0.0,
                'headers' => [],
            ];
        }

        $answer = $queued[$index];

        return [
            'status' => (int) ($answer['status'] ?? 500),
            'body' => $answer['body'] ?? null,
            'held_for' => (float) ($answer['held_for'] ?? 0.0),
            'headers' => (array) ($answer['headers'] ?? []),
        ];
    }

    /**
     * One answer, as the bytes that go back down the socket.
     *
     * @param array{status: int, body: mixed, held_for: float, headers: array<string, string>} $answer
     */
    private static function written(array $answer): string
    {
        $body = is_string($answer['body'])
            ? $answer['body']
            : (string) json_encode($answer['body'], JSON_UNESCAPED_SLASHES);

        $head = sprintf("HTTP/1.1 %d Answer\r\n", $answer['status'])
            . "Content-Type: application/json\r\n"
            . sprintf("Content-Length: %d\r\n", strlen($body));
        foreach ($answer['headers'] as $name => $value) {
            $head .= sprintf("%s: %s\r\n", $name, $value);
        }

        return $head . "Connection: close\r\n\r\n" . $body;
    }

    /**
     * The request a connection has read, once it has read all of it.
     *
     * @return array{verb: string, target: string, headers: array<string, string>, body: string}|null
     */
    private static function complete(string $read): ?array
    {
        $end = strpos($read, self::HEAD_SEPARATOR);
        if ($end === false) {
            return null;
        }

        $lines = explode("\r\n", substr($read, 0, $end));
        $requestLine = explode(' ', (string) array_shift($lines));

        $headers = [];
        foreach ($lines as $line) {
            $name = strstr($line, ':', true);
            if ($name !== false) {
                $headers[strtolower(trim($name))] = trim(substr($line, strlen($name) + 1));
            }
        }

        $length = (int) ($headers['content-length'] ?? '0');
        $body = substr($read, $end + strlen(self::HEAD_SEPARATOR));
        if (strlen($body) < $length) {
            return null;
        }

        return [
            'verb' => $requestLine[0] ?? '',
            'target' => $requestLine[1] ?? '',
            'headers' => $headers,
            'body' => substr($body, 0, $length),
        ];
    }
}
