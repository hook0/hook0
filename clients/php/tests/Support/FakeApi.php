<?php

declare(strict_types=1);

namespace Hook0\Tests\Support;

/**
 * A Hook0 API listening on a loopback port for the lifetime of one case.
 *
 * Every case goes over a real socket: the request the client builds, the headers it sets, the way it
 * reads an answer and the way it gives up on one are all the real ones. Nothing here stands in for a
 * part of the client, so a case that passes says the client works rather than that it was called.
 *
 * The server itself is the process this starts; what is here is the case's side of it — where to
 * reach it, what to queue for it to answer, and what it received.
 */
final class FakeApi
{
    /** How long the server process lives before it stops on its own, in seconds. */
    private const LIFETIME_SECONDS = 60.0;

    /** How long a case waits for the server to say which port it took, in seconds. */
    private const START_TIMEOUT_SECONDS = 10.0;

    /** How long a case waits for the server to stop once it has been asked to, in seconds. */
    private const STOP_TIMEOUT_SECONDS = 5.0;

    /** How long one pass of either wait sleeps, in microseconds. */
    private const POLL_MICROSECONDS = 2000;

    /** Most requests read back out of the log, which is far above what any case makes. */
    private const MAX_LOGGED_REQUESTS = 1024;

    /** @var resource */
    private $process;

    /** @var array<int, resource> */
    private array $pipes = [];

    private readonly string $directory;
    private readonly string $scriptPath;
    private readonly string $logPath;
    private readonly int $port;

    /** @var list<ScriptedResponse> */
    private array $scripted = [];

    public function __construct()
    {
        $this->directory = self::temporaryDirectory();
        $this->scriptPath = $this->directory . '/script.json';
        $this->logPath = $this->directory . '/requests.jsonl';
        file_put_contents($this->scriptPath, '[]');
        file_put_contents($this->logPath, '');

        $descriptors = [1 => ['pipe', 'w'], 2 => ['pipe', 'w']];
        $process = proc_open(
            [
                PHP_BINARY,
                __DIR__ . '/serve.php',
                $this->scriptPath,
                $this->logPath,
                (string) self::LIFETIME_SECONDS,
            ],
            $descriptors,
            $pipes
        );
        if ($process === false) {
            throw new \RuntimeException('the fake API could not be started');
        }

        $this->process = $process;
        $this->pipes = $pipes;
        $this->port = $this->announcedPort();
    }

    /** Where the client reaches this API. */
    public function baseUrl(): string
    {
        return sprintf('http://127.0.0.1:%d', $this->port);
    }

    /** Queues the answers the case expects the client to draw, in order. */
    public function willAnswer(ScriptedResponse ...$responses): void
    {
        foreach ($responses as $response) {
            $this->scripted[] = $response;
        }

        $written = array_map(
            static fn (ScriptedResponse $response): array => $response->scripted(),
            $this->scripted
        );
        file_put_contents(
            $this->scriptPath,
            json_encode($written, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES)
        );
    }

    /**
     * Every request this API received, in the order it received them.
     *
     * @return list<ReceivedRequest>
     */
    public function received(): array
    {
        $written = (string) file_get_contents($this->logPath);
        $lines = array_filter(explode("\n", $written), static fn (string $line): bool => $line !== '');
        if (count($lines) > self::MAX_LOGGED_REQUESTS) {
            throw new \RuntimeException(sprintf(
                'a case made more than the %d requests read back',
                self::MAX_LOGGED_REQUESTS
            ));
        }

        $received = [];
        foreach ($lines as $line) {
            $read = json_decode($line, true, 32, JSON_THROW_ON_ERROR);
            $received[] = new ReceivedRequest(
                (string) $read['verb'],
                (string) $read['target'],
                (array) $read['headers'],
                (string) $read['body']
            );
        }

        return $received;
    }

    /** How many requests this API has received so far. */
    public function count(): int
    {
        return count($this->received());
    }

    /** The event identifier request number `$index` carried, as the API read it. */
    public function eventIdOf(int $index): string
    {
        $received = $this->received();
        if (!isset($received[$index])) {
            throw new \RuntimeException(sprintf(
                'expected at least %d requests, got %d',
                $index + 1,
                count($received)
            ));
        }

        return (string) $received[$index]->json()['event_id'];
    }

    public function close(): void
    {
        proc_terminate($this->process);

        $deadline = microtime(true) + self::STOP_TIMEOUT_SECONDS;
        while (microtime(true) < $deadline && proc_get_status($this->process)['running']) {
            usleep(self::POLL_MICROSECONDS);
        }

        foreach ($this->pipes as $pipe) {
            if (is_resource($pipe)) {
                fclose($pipe);
            }
        }
        proc_close($this->process);

        foreach (['script.json', 'requests.jsonl'] as $name) {
            @unlink($this->directory . '/' . $name);
        }
        @rmdir($this->directory);
    }

    /** The port the server took, read off the one line it writes before it starts serving. */
    private function announcedPort(): int
    {
        stream_set_blocking($this->pipes[1], false);

        $deadline = microtime(true) + self::START_TIMEOUT_SECONDS;
        $announced = '';
        while (microtime(true) < $deadline && !str_contains($announced, "\n")) {
            $chunk = fread($this->pipes[1], 32);
            if ($chunk === false || $chunk === '') {
                usleep(self::POLL_MICROSECONDS);
                continue;
            }
            $announced .= $chunk;
        }

        $port = (int) trim($announced);
        if ($port <= 0) {
            throw new \RuntimeException(sprintf(
                'the fake API named no port: %s',
                trim((string) stream_get_contents($this->pipes[2]))
            ));
        }

        return $port;
    }

    private static function temporaryDirectory(): string
    {
        $directory = sys_get_temp_dir() . '/hook0-php-' . bin2hex(random_bytes(8));
        if (!mkdir($directory, 0o700) && !is_dir($directory)) {
            throw new \RuntimeException(sprintf('%s could not be made', $directory));
        }

        return $directory;
    }
}
