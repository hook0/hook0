import { randomBytes } from 'crypto';
import { URL } from 'node:url';
import { Signature } from './index';

/**
 * Longest one attempt at reaching Hook0 is given before it is abandoned, unless the client is
 * built with another `Hook0ClientOptions`.
 *
 * Ten seconds is far above what ingesting an event takes when the API is healthy, and short enough
 * that a stuck connection does not hold an emitter for a noticeable time.
 */
export const DEFAULT_REQUEST_TIMEOUT_MS: number = 10_000;

/**
 * Largest event payload the client agrees to send, unless it is built with another
 * `Hook0ClientOptions`.
 *
 * The API holds a payload to 699,050 characters, and its body to 2 MiB. Neither is this number,
 * and the difference is worth knowing before changing it. That character limit counts characters
 * rather than bytes, so text that is one byte per character is refused by the API well before this
 * cap is reached, and the cap costs nothing. Text that is not, such as a payload written in a
 * script outside ASCII, reaches a megabyte while still being far short of 699,050 characters, and
 * there this cap is what refuses it. The client rules the event out rather than spending a round
 * trip, and every retry after it, on a request it expects the body limit to reject.
 */
export const DEFAULT_MAX_PAYLOAD_BYTES: number = 1024 * 1024;

/**
 * Largest answer the client reads off the socket, unless it is built with another
 * `Hook0ClientOptions`.
 *
 * The body of an answer is written by the other end: a server that is broken or hostile can
 * otherwise stream into an emitter's memory for as long as the connection lasts, and a client with
 * no ceiling has no answer to that. Eight mebibytes is far above anything Hook0's API replies with,
 * and the read stops there rather than growing with whatever arrives.
 */
export const DEFAULT_MAX_RESPONSE_BYTES: number = 8 * 1024 * 1024;

/**
 * Header lines an answer may carry before this client refuses to read it.
 *
 * The head is written by the other end just like the body, so it is bounded like the body: a client
 * that holds a head of any length has only moved where a broken or hostile server spends its
 * caller's memory. This one and `MAX_HEADER_BYTES` refuse early, on the line that crosses them,
 * rather than at the end of the head.
 */
export const MAX_RESPONSE_HEADERS: number = 64;

/** Longest one header line may be, its name and its value together, in bytes. */
export const MAX_HEADER_BYTES: number = 64 * 1024;

/**
 * Largest whole head an answer may carry, every line counted together, in bytes.
 *
 * This is the one that bounds what a head costs: a line count and a size per line multiply, and
 * `MAX_RESPONSE_HEADERS` lines of `MAX_HEADER_BYTES` each is four mebibytes of head that both of
 * them admit. Sixteen kibibytes is what Node enforces by default, and matching it is the point: a
 * lower ceiling would refuse heads another Hook0 SDK accepts, and a higher one is not reachable
 * from library code at all, since a larger head is refused before this client is consulted.
 */
export const MAX_HEAD_BYTES: number = 16 * 1024;

/**
 * Most attempts a `RetryPolicy` can ever make, whatever `maxAttempts` says.
 *
 * A policy is configuration, and configuration can be wrong; this cap keeps a mistyped
 * `maxAttempts` from turning one send into an unbounded series of requests.
 */
export const MAX_ATTEMPTS_CAP: number = 16;

/** Beyond this many doublings any backoff has long since reached its ceiling. */
const MAX_BACKOFF_DOUBLINGS = 30;

/** The public identifier Hook0 gives the problem it answers when an event ID is already taken. */
const ALREADY_INGESTED = 'EventAlreadyIngested';

/**
 * The public identifier Hook0 gives the problem it answers when requests are reaching the instance
 * faster than it accepts them.
 *
 * It shares its status with the quota problems and is the only one of them worth repeating: a quota
 * clears when a plan changes or a day turns, neither of which happens inside the seconds a send is
 * given, while pacing clears on its own and the answer says when.
 */
const RATE_LIMITED = 'RateLimited';

/** Status Hook0 answers when the event ID a request carries is already taken. */
const CONFLICT = 409;

/**
 * Status Hook0 answers both when a quota is spent and when requests are coming in faster than the
 * instance accepts them. Which of the two it is only the problem the body names can say, which is
 * why this status alone decides nothing.
 */
const PACED = 429;

/** First status that says the failure is on Hook0's side, and so could clear on its own. */
const LOWEST_SERVER_ERROR = 500;

/** What every request says it carries, and what every answer is asked for in. */
const JSON_MEDIA_TYPE = 'application/json';

/** What the API names the delay before the request becomes servable again, in whole seconds. */
const DELAY_HEADER = 'Retry-After';

/** The schemes a request can travel on, and so the ones an API URL may name. */
const REACHABLE_SCHEMES = ['http:', 'https:'];

/**
 * Longest excerpt of a refused answer's body an error message carries, in characters.
 *
 * A body is written by the other end and is only bounded by `maxResponseBytes`, which is megabytes:
 * an error message is not the place to hold however much of it arrived. What is cut is said, so a
 * reader knows there was more.
 */
const MAX_REFUSAL_EXCERPT_CHARS = 512;

/**
 * How Node reports a head above what it agrees to read, which it refuses on the socket before this
 * client is consulted. It is an answer above a bound rather than an answer that never came: the
 * same request draws the same head, and repeating it reads that head again.
 */
const HEAD_OVERFLOW = 'UND_ERR_HEADERS_OVERFLOW';

/**
 * Longest each part this client composes its `User-Agent` out of may be, in characters.
 *
 * The runtime and the operating system are described by the platform rather than by this package,
 * so their length is not this package's to guarantee: they are cut here so that the header cannot
 * grow with whatever the platform feels like saying. Every part is also stripped of anything the
 * grammar of the header uses as punctuation, so a platform cannot forge a shape it does not have.
 */
const MAX_USER_AGENT_PART_CHARS = 64;

/**
 * The version this package is published under, as it reaches the API.
 *
 * Reading it back out of `package.json` would make the manifest an input of the build, and the
 * emit takes its root from the directory every input shares: a manifest sitting above `src` moves
 * the whole emit one directory deeper inside `dist`, which breaks the very paths that manifest
 * points at. It is written here instead, and the conformance suite reads the manifest and holds
 * this header against it, so the two cannot disagree without a case failing.
 */
const VERSION = '2.0.3';

/**
 * One part of the `User-Agent`, with everything the header's own grammar uses taken out of it and
 * cut to `MAX_USER_AGENT_PART_CHARS`.
 * @param part - What the platform, or this package, had to say
 */
function clipped(part: string): string {
  return [...part]
    .filter((character) => character >= ' ' && character <= '~')
    .filter((character) => !'();'.includes(character))
    .slice(0, MAX_USER_AGENT_PART_CHARS)
    .join('');
}

/** Which SDK, at which version, on which runtime and operating system, is talking to the API. */
function userAgent(): string {
  const version = clipped(VERSION);
  // A runtime carrying no `process` at all — a browser, an edge worker — names neither the runtime
  // nor the machine under it rather than guessing at either.
  if (typeof process === 'undefined') {
    return `hook0-client-typescript/${version} (unknown; unknown)`;
  }

  const runtime = clipped(`node ${process.version}`);
  const os = clipped(`${process.platform} ${process.arch}`);
  return `hook0-client-typescript/${version} (${runtime}; ${os})`;
}

/**
 * What every request says it comes from.
 *
 * Composed once: neither the runtime nor the machine under it changes while a process runs, and an
 * instance can otherwise not tell which SDKs, at which versions, are still reaching it.
 */
const USER_AGENT = userAgent();

/**
 * What a cause says, which is never nothing and never the bare word `Error`.
 *
 * Interpolating an `Error` writes its type in front of its message, and writes nothing but that type
 * when it carries no message: `Sending event … failed: Error` is a failure whose message names
 * neither what was reached nor what came back, which is a client that cannot be debugged by whoever
 * installed it.
 * @param error - What the failure was built from
 */
function said(error: Error): string {
  if (error.message.length > 0) {
    return error.message;
  }
  return `${error.name} with no message`;
}

/**
 * Custom error class for Hook0Client
 */
export class Hook0ClientError extends Error {
  /**
   * Error when sending an event fails
   * @param eventId - ID of the event
   * @param error - Error details
   */
  static EventSending(eventId: string | undefined, error: Error): Hook0ClientError {
    return new Hook0ClientError(
      `Sending event${eventId ? ' ' + eventId : ''} failed: ${said(error)}`
    );
  }

  /**
   * Error for invalid event type
   * @param s - Invalid event type string
   */
  static InvalidEventType(s: string): Hook0ClientError {
    return new Hook0ClientError(`Event type ${s} is invalid`);
  }

  /**
   * Error when fetching available event types fails
   * @param error - Error details
   */
  static GetAvailableEventTypes(error: Error): Hook0ClientError {
    return new Hook0ClientError(`Getting available event types failed: ${said(error)}`);
  }

  /**
   * Error when parsing a signature fails
   * @param signature - Invalid signature
   */
  static SignatureParsing(signature: string): Hook0ClientError {
    return new Hook0ClientError(`Could not parse signature: ${signature}`);
  }

  /**
   * Error when parsing a timestamp in a signature fails
   * @param timestamp - Invalid timestamp
   */
  static TimestampParsingInSignature(timestamp: string): Hook0ClientError {
    return new Hook0ClientError(`Could not parse timestamp in signature: ${timestamp}`);
  }

  /**
   * Error when an invalid signature is provided
   * @param signature - Invalid signature
   */
  static InvalidSignature(signature: string): Hook0ClientError {
    return new Hook0ClientError(`Invalid signature: ${signature}`);
  }

  /**
   * Error when a header listed in the signature was not provided with the request
   * @param headerName - Name of the header the signature covers but the request did not carry
   */
  static MissingHeader(headerName: string): Hook0ClientError {
    return new Hook0ClientError(
      `The \`${headerName}\` header present in the webhook's signature was not provided with a value`
    );
  }

  /**
   * Error when a webhook's signature timestamp falls outside the tolerance window
   *
   * This covers both a webhook that was signed too long ago (a replay) and one that was signed too
   * far in the future (a clock that is ahead, or a forged timestamp meant to widen the acceptance
   * window).
   * @param signed_at - Datetime of webhook signature
   * @param tolerance - Maximum difference (in seconds), in either direction, between the signature datetime and the current datetime for the webhook to be considered valid
   * @param current_time - Current time
   */
  static ExpiredWebhook(signed_at: Date, tolerance: number, current_time: Date): Hook0ClientError {
    return new Hook0ClientError(
      `The webhook's signature timestamp is outside the tolerance window (signed_at=${signed_at}, tolerance=${tolerance}, current_time=${current_time})`
    );
  }

  /**
   * Error when an event payload is larger than the client agrees to send
   *
   * Nothing was sent: the event is refused before any request is issued, so neither the round trip
   * nor the retries after it are spent on a request the API would refuse.
   * @param eventId - ID the event would have been sent under
   * @param size - Size of the payload, in bytes
   * @param maximum - Largest payload this client sends, in bytes
   */
  static PayloadTooLarge(eventId: string, size: number, maximum: number): Hook0ClientError {
    return new Hook0ClientError(
      `Sending event ${eventId} failed: event payload is ${size} bytes, which is more than the ${maximum} bytes this client sends at most; nothing was sent`
    );
  }

  /**
   * Error when a send ran out of attempts, or out of the delay budget its attempts share
   * @param eventId - ID the event was sent under, by every attempt
   * @param attempts - Number of attempts that were made
   * @param waitedMs - Time spent waiting between them, in milliseconds
   * @param lastFailure - What the last attempt ran into
   */
  static RetriesExhausted(
    eventId: string,
    attempts: number,
    waitedMs: number,
    lastFailure: string
  ): Hook0ClientError {
    return new Hook0ClientError(
      `Sending event ${eventId} failed: gave up after ${attempts} attempts spread over ${waitedMs}ms of retry delay; last failure: ${lastFailure}`
    );
  }
}

/**
 * How a client spaces out the attempts of a single send.
 *
 * The delay before a retry doubles from `initialBackoffMs` and is capped by `maxBackoffMs`; the
 * actual delay is then drawn anywhere between zero and that ceiling, so that emitters which failed
 * at the same moment do not come back at the same moment. Retrying stops as soon as the delays of
 * the send would add up to more than `maxTotalDelayMs`.
 */
export class RetryPolicy {
  /**
   * Constructor for RetryPolicy
   *
   * The defaults are four attempts spread over at most five seconds: three retries absorb the
   * blips a webhook emitter meets in production (a connection reset, a rolling deployment
   * answering 503) without holding the caller for long, and the five-second budget bounds what the
   * worst send costs whatever the individual delays turn out to be.
   * @param maxAttempts - Attempts a single send makes at most, the first one included. `1` disables retrying.
   * @param initialBackoffMs - Ceiling of the delay before the first retry, in milliseconds
   * @param maxBackoffMs - Ceiling no single delay ever exceeds, in milliseconds
   * @param maxTotalDelayMs - Budget all the delays of one send share, in milliseconds
   */
  constructor(
    public maxAttempts: number = 4,
    public initialBackoffMs: number = 100,
    public maxBackoffMs: number = 2_000,
    public maxTotalDelayMs: number = 5_000
  ) {}

  /** A policy that never retries: one attempt, and the caller hears about whatever it returned. */
  static disabled(): RetryPolicy {
    return new RetryPolicy(1, 0, 0, 0);
  }

  /** Attempts this policy actually makes: `maxAttempts`, brought back inside `1..=MAX_ATTEMPTS_CAP`. */
  attempts(): number {
    const asked = Math.floor(this.maxAttempts);
    if (Number.isNaN(asked)) {
      return 1;
    }
    return Math.min(Math.max(asked, 1), MAX_ATTEMPTS_CAP);
  }

  /**
   * Ceiling of the delay before retry number `retry`, where `1` is the first retry.
   *
   * It doubles from `initialBackoffMs` and never exceeds `maxBackoffMs`, so the ceilings of
   * successive retries never decrease.
   * @param retry - Which retry, counted from one
   */
  backoffCeilingMs(retry: number): number {
    const doublings = Math.min(Math.max(retry - 1, 0), MAX_BACKOFF_DOUBLINGS);
    return Math.min(initialBackoffOf(this) * 2 ** doublings, maxBackoffOf(this));
  }

  /**
   * The delays this policy waits between the attempts of one send, one per retry, given one random
   * draw in `[0, 1)` per retry.
   *
   * Each delay lands between zero and the ceiling of its retry, and the schedule is cut short as
   * soon as the next delay would spend more than `maxTotalDelayMs`. There are therefore at most
   * `attempts() - 1` delays, and they add up to at most `maxTotalDelayMs`.
   *
   * A draw that is missing or is not a finite number is read as `1`, which asks for the whole
   * ceiling: an unusable source of randomness makes the client wait longer, never less.
   * @param draws - One random draw per retry
   */
  delaysMs(draws: number[]): number[] {
    const retries = this.attempts() - 1;
    const budget = maxTotalDelayOf(this);
    const delays: number[] = [];
    let spent = 0;

    for (let retry = 1; retry <= retries; retry += 1) {
      const delay = this.backoffCeilingMs(retry) * drawAt(draws, retry - 1);
      if (spent + delay > budget) {
        break;
      }
      spent += delay;
      delays.push(delay);
    }

    return delays;
  }
}

/** Every bound a client applies to one send. */
export class Hook0ClientOptions {
  /**
   * Constructor for Hook0ClientOptions
   * @param retryPolicy - How the client spaces out the attempts of a send
   * @param requestTimeoutMs - Longest one attempt is given, in milliseconds
   * @param maxPayloadBytes - Largest event payload the client agrees to send, in bytes
   * @param maxResponseBytes - Largest answer the client agrees to read off the socket, in bytes
   */
  constructor(
    public retryPolicy: RetryPolicy = new RetryPolicy(),
    public requestTimeoutMs: number = DEFAULT_REQUEST_TIMEOUT_MS,
    public maxPayloadBytes: number = DEFAULT_MAX_PAYLOAD_BYTES,
    public maxResponseBytes: number = DEFAULT_MAX_RESPONSE_BYTES
  ) {}
}

/**
 * Longest delay the header every request carries can state, in milliseconds.
 *
 * A delay is a `number` here and a whole number of milliseconds on the wire, and the two do not
 * have the same reach: a `number` goes up to where it stops being finite, and one written past
 * `Number.MAX_SAFE_INTEGER` stops being a whole number a reader can hold — it is written in
 * exponent form, which is not the integer the shared contract asks for. This is the largest whole
 * number every runtime reading the header holds exactly, which is what makes it the same ceiling in
 * every SDK rather than one per language.
 */
const MAX_STATED_DELAY_MS = Number.MAX_SAFE_INTEGER;

/**
 * The policy every field falls back to one field at a time, built from the declarations on
 * `RetryPolicy` so that moving a default moves the fallback with it rather than leaving a second
 * copy behind.
 */
const DEFAULT_POLICY = new RetryPolicy();

/**
 * What one duration of a policy is read as, wherever this client reads one.
 *
 * A number that is not finite — infinite either way, or `NaN` — says nothing about how long to
 * wait, so the field's own default is what it is read as. Zero would quietly delete the spacing
 * between attempts and turn a broken policy into a burst; treating it as unbounded is what makes a
 * client wait for ever. The default is neither, and it is the same value in every SDK, so one
 * mistyped field cannot make two clients behave differently.
 *
 * A negative but finite delay is a delay of nothing, which is what it has always been read as.
 * @param value - What the policy holds for this field
 * @param fallback - What that field defaults to
 */
function readDuration(value: number, fallback: number): number {
  if (!Number.isFinite(value)) {
    return fallback;
  }
  return Math.max(value, 0);
}

/** The ceiling of the first retry's delay, as this client reads it. */
function initialBackoffOf(policy: RetryPolicy): number {
  return readDuration(policy.initialBackoffMs, DEFAULT_POLICY.initialBackoffMs);
}

/** The ceiling no single delay exceeds, as this client reads it. */
function maxBackoffOf(policy: RetryPolicy): number {
  return readDuration(policy.maxBackoffMs, DEFAULT_POLICY.maxBackoffMs);
}

/** The budget every delay of one send shares, as this client reads it. */
function maxTotalDelayOf(policy: RetryPolicy): number {
  return readDuration(policy.maxTotalDelayMs, DEFAULT_POLICY.maxTotalDelayMs);
}

/**
 * One delay a policy is read as, in the whole milliseconds the header states it in.
 *
 * Capped at `MAX_STATED_DELAY_MS`: a `number` reaches past where a whole number every reader holds
 * exactly stops, and one written past that range writes itself in exponent form, which is not the
 * integer the shared contract asks for.
 * @param milliseconds - The delay this client reads, already a finite count of milliseconds
 */
function statedDelay(milliseconds: number): number {
  return Math.min(Math.round(milliseconds), MAX_STATED_DELAY_MS);
}

/**
 * The retry policy behind a request, as the header every request carries states it.
 *
 * The four parts are the policy in force, in the order the shared contract fixes, joined the way
 * `X-Hook0-Signature` joins its own: every duration is a count of milliseconds and every part an
 * integer, so an instance reads the value back by cutting each part at its first `=` and nothing
 * here needs a parser of its own. Whole numbers are also what bounds the value without cutting it
 * down to a length: four of them are as long as a number written out and no longer, whatever a
 * caller configures.
 *
 * In force means past this client's own clamps rather than as asked for: a policy that asked for a
 * thousand attempts states the `MAX_ATTEMPTS_CAP` it will make, since a thousand would have a
 * reader watching for a burst that cannot arrive.
 * @param policy - How the client spaces out the attempts of a send
 */
function clientOptions(policy: RetryPolicy): string {
  const attempts = policy.attempts();
  const backoff = statedDelay(initialBackoffOf(policy));
  const ceiling = statedDelay(maxBackoffOf(policy));
  const budget = statedDelay(maxTotalDelayOf(policy));
  return `attempts=${attempts},backoff=${backoff},ceiling=${ceiling},budget=${budget}`;
}

/** The draw for one retry, brought back inside `[0, 1]` whatever the source of randomness gave. */
function drawAt(draws: number[], index: number): number {
  if (index >= draws.length) {
    return 1;
  }
  const draw = draws[index];
  if (!Number.isFinite(draw)) {
    return 1;
  }
  return Math.min(Math.max(draw, 0), 1);
}

/**
 * Draws used to jitter the delays of one send.
 *
 * Jitter only has to keep emitters that failed together from coming back together; it does not
 * have to be unpredictable, so the platform's own generator is enough.
 */
function jitterDraws(count: number): number[] {
  const draws: number[] = [];
  for (let drawn = 0; drawn < count; drawn += 1) {
    draws.push(Math.random());
  }
  return draws;
}

/**
 * A UUIDv7, the same shape of identifier Hook0 mints when it is the one choosing.
 *
 * Its leading 48 bits are the current time in milliseconds, so identifiers generated in sequence
 * are ordered, which is what keeps the index they end up in from being written all over.
 */
function generateEventId(): string {
  /** Bytes of a UUID. */
  const SIZE = 16;
  /** Bytes the millisecond timestamp occupies, big-endian, at the front. */
  const TIMESTAMP_BYTES = 6;

  const bytes = randomBytes(SIZE);
  let milliseconds = Date.now();
  for (let index = TIMESTAMP_BYTES - 1; index >= 0; index -= 1) {
    bytes[index] = milliseconds % 256;
    milliseconds = Math.floor(milliseconds / 256);
  }
  bytes[6] = (bytes[6] & 0x0f) | 0x70;
  bytes[8] = (bytes[8] & 0x3f) | 0x80;

  const hex = bytes.toString('hex');
  return [
    hex.slice(0, 8),
    hex.slice(8, 12),
    hex.slice(12, 16),
    hex.slice(16, 20),
    hex.slice(20, 32),
  ].join('-');
}

/** The ID an event is sent under: the one it carries, or one this client generates for it. */
function eventIdOf(event: Event): string {
  if (typeof event.eventId === 'string' && event.eventId.length > 0) {
    return event.eventId;
  }
  return generateEventId();
}

/** What a body says, when it says JSON. */
type ReadBody = { kind: 'json'; value: unknown } | { kind: 'notJson'; detail: string };

/** Read a body as JSON, without the failure to do so escaping as something else. */
function jsonOf(body: string): ReadBody {
  try {
    return { kind: 'json', value: JSON.parse(body) };
  } catch (cause) {
    return { kind: 'notJson', detail: detailOf(cause) };
  }
}

/** Whether an RFC 9457 problem body names that problem. */
function problemIs(body: string, problem: string): boolean {
  const read = jsonOf(body);
  return (
    read.kind === 'json' &&
    typeof read.value === 'object' &&
    read.value !== null &&
    'id' in read.value &&
    (read.value as { id: unknown }).id === problem
  );
}

/** What a document says at `field`, when it carries one that says anything. */
type Field = { kind: 'says'; text: string } | { kind: 'saysNothing' };

/** Read one string field off a body already read as a JSON object. */
function fieldOf(document: object, field: string): Field {
  if (!(field in document)) {
    return { kind: 'saysNothing' };
  }
  const value = (document as Record<string, unknown>)[field];
  if (typeof value !== 'string' || value.length === 0) {
    return { kind: 'saysNothing' };
  }
  return { kind: 'says', text: value };
}

/** As much of a body as an error message carries, saying what it left out. */
function excerpt(body: string): string {
  if (body.length <= MAX_REFUSAL_EXCERPT_CHARS) {
    return body;
  }
  return `${body.slice(0, MAX_REFUSAL_EXCERPT_CHARS)}… (${body.length} characters in all)`;
}

/**
 * What an answer this client could not act on said, in the one line an error message carries.
 *
 * The status is named whatever else there is, because there may be nothing else: an answer can
 * carry no body at all — a 404 written by a reverse proxy, or by an API URL that misses the path
 * the instance is served under — and a message built out of the body alone then says nothing
 * whatsoever about what went wrong. When the body is one of Hook0's problem documents, its stable
 * identifier is named next: that is what tells one refusal from another without reading prose, and
 * it is what a caller deciding what to do about a failure can match on. Anything else is quoted as
 * it arrived, cut to `MAX_REFUSAL_EXCERPT_CHARS`.
 * @param status - What Hook0 answered
 * @param body - What it answered beside it
 */
function refusalOf(status: number, body: string): string {
  const answered = `the API answered ${status}`;

  const read = jsonOf(body);
  if (read.kind === 'json' && typeof read.value === 'object' && read.value !== null) {
    const problem = fieldOf(read.value, 'id');
    const detail = fieldOf(read.value, 'detail');
    if (problem.kind === 'says' && detail.kind === 'says') {
      return `${answered} ${problem.text}: ${detail.text}`;
    }
    if (problem.kind === 'says') {
      return `${answered} ${problem.text}`;
    }
  }

  if (body.length === 0) {
    return `${answered} with no body`;
  }
  return `${answered} with: ${excerpt(body)}`;
}

/**
 * Whether repeating a request Hook0 answered that way could end differently.
 *
 * The status decides on its own everywhere but under the one Hook0 answers both a spent quota and a
 * paced instance with: a quota clears when a plan changes or a day turns, and neither is something
 * a send spending seconds can wait for. Only the problem the body names tells the two apart, and a
 * body naming a problem this client has never heard of falls back to the status.
 * @param status - What Hook0 answered
 * @param body - What it answered beside it
 */
function isRetryable(status: number, body: string): boolean {
  if (status === PACED) {
    return problemIs(body, RATE_LIMITED);
  }
  // Only the server side of a server error can change between two identical requests.
  return status >= LOWEST_SERVER_ERROR;
}

/** How long Hook0 named before the request becomes servable again, when it named a delay. */
type NamedDelay = { kind: 'noDelayNamed' } | { kind: 'delayNamed'; delayMs: number };

/**
 * The delay Hook0 named beside an answer, when it named one this client can read.
 *
 * Only a whole number of seconds is read. The header may also carry a date, which is a clock this
 * client would be comparing against its own, and anything else is a header nobody meant: both leave
 * the client's own schedule in place rather than being guessed at.
 * @param headers - What the answer carried beside its body
 */
function delayNamedBy(headers: Headers): NamedDelay {
  const written = headers.get(DELAY_HEADER);
  if (typeof written !== 'string' || !/^\d+$/.test(written.trim())) {
    return { kind: 'noDelayNamed' };
  }

  const seconds = Number(written.trim());
  if (!Number.isSafeInteger(seconds)) {
    return { kind: 'noDelayNamed' };
  }
  return { kind: 'delayNamed', delayMs: seconds * 1000 };
}

/** Why the head of an answer is above what this client agrees to hold, when it is. */
type HeadVerdict = { kind: 'headWithinTheBounds' } | { kind: 'headAboveABound'; detail: string };

/**
 * Whether the head of an answer crossed one of the ceilings this client holds a head to.
 *
 * Counted the way a head is written: one line per header, its name and its value together. The line
 * count and the length of one line refuse early, on the line that crosses them; the whole head is
 * what actually bounds the memory a head can cost, since the other two multiply.
 * @param headers - What the answer carried beside its body
 */
function headAboveABound(headers: Headers): HeadVerdict {
  let lines = 0;
  let whole = 0;

  for (const [name, value] of headers) {
    lines += 1;
    if (lines > MAX_RESPONSE_HEADERS) {
      return {
        kind: 'headAboveABound',
        detail: `the API answered more than the ${MAX_RESPONSE_HEADERS} header lines read at most`,
      };
    }

    const line = Buffer.byteLength(name, 'utf8') + Buffer.byteLength(value, 'utf8');
    if (line > MAX_HEADER_BYTES) {
      return {
        kind: 'headAboveABound',
        detail: `the API answered a \`${name}\` header above the ${MAX_HEADER_BYTES} bytes read at most`,
      };
    }

    whole += line;
    if (whole > MAX_HEAD_BYTES) {
      return {
        kind: 'headAboveABound',
        detail: `the API answered a head above the ${MAX_HEAD_BYTES} bytes read at most`,
      };
    }
  }

  return { kind: 'headWithinTheBounds' };
}

/** The body of an answer, read no further than this client agrees to hold. */
type BoundedBody =
  { kind: 'bodyRead'; text: string } | { kind: 'bodyAboveTheBound'; detail: string };

/**
 * Read the body of an answer, stopping at the ceiling rather than at the end of what is written.
 *
 * `fetch` reads a body without a ceiling of its own, so a server that is broken or hostile can
 * stream into an emitter's memory for as long as the connection lasts. Reading it a chunk at a time
 * makes the read stop where the ceiling is, and what has been read up to there is dropped with it.
 * @param response - The answer to read
 * @param maxBytes - Largest body this client agrees to hold
 */
function readBoundedBody(response: Response, maxBytes: number): Promise<BoundedBody> {
  const stream = response.body;
  if (stream === null) {
    return Promise.resolve({ kind: 'bodyRead', text: '' });
  }

  const reader = stream.getReader();
  const chunks: Uint8Array[] = [];
  let held = 0;

  function pump(): Promise<BoundedBody> {
    return reader.read().then((step): Promise<BoundedBody> | BoundedBody => {
      if (step.done) {
        return { kind: 'bodyRead', text: Buffer.concat(chunks).toString('utf8') };
      }

      held += step.value.length;
      if (held > maxBytes) {
        chunks.length = 0;
        return reader.cancel().then((): BoundedBody => ({
          kind: 'bodyAboveTheBound',
          detail: `the API answered more than the ${maxBytes} bytes read at most`,
        }));
      }

      chunks.push(step.value);
      return pump();
    });
  }

  return pump();
}

/** Let go of an answer this client has refused to read, so its connection is not held open. */
function discard(response: Response): Promise<void> {
  const stream = response.body;
  if (stream === null) {
    return Promise.resolve();
  }
  return stream.cancel().then(
    () => undefined,
    () => undefined
  );
}

/** What one attempt at sending an event ended with. */
type Attempt =
  | { kind: 'ingested'; eventId: string }
  | { kind: 'alreadyIngested'; detail: string }
  | { kind: 'failed'; detail: string; retryable: boolean; namedDelay: NamedDelay };

/** The request every attempt of one send repeats, or why nothing could be sent at all. */
type BuildableRequest =
  { kind: 'buildable'; url: string; body: string } | { kind: 'unbuildable'; detail: string };

/** A failed attempt whose answer, if there was one, named no delay. */
function failed(detail: string, retryable: boolean): Attempt {
  return { kind: 'failed', detail, retryable, namedDelay: { kind: 'noDelayNamed' } };
}

/** Resolves after `delayMs`, so that a retry waits before it is issued. */
function wait(delayMs: number): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(resolve, delayMs);
  });
}

/** What a rejected promise says, whatever was thrown. */
function detailOf(cause: unknown): string {
  if (cause instanceof Error) {
    return cause.message;
  }
  return String(cause);
}

/**
 * Whether what `fetch` rejected with is an answer above a ceiling rather than an answer that never
 * came.
 *
 * Node refuses a head above what it reads on the socket, before this client is consulted, and
 * reports it under a code carried somewhere down the chain of causes.
 * @param cause - What the rejected promise carried
 */
function isAboveABound(cause: unknown): boolean {
  /** No chain of causes a runtime builds is anywhere near this long; the bound keeps a cyclic one from being walked forever. */
  const MAX_LINKS = 8;

  // Read by shape rather than by type: a failure raised by the runtime is not built from the same
  // `Error` an embedder holds when the two do not share a realm, and `instanceof` answers no there.
  let walked: unknown = cause;
  for (let link = 0; link < MAX_LINKS; link += 1) {
    if (typeof walked !== 'object' || walked === null) {
      return false;
    }
    if ((walked as { code?: unknown }).code === HEAD_OVERFLOW) {
      return true;
    }
    walked = (walked as { cause?: unknown }).cause;
  }
  return false;
}

/**
 * What a request that never got an answer was, told apart by what caused it rather than by which
 * type carried it.
 *
 * Everything `fetch` rejects with is a `TypeError`, so the type says nothing: a connection that was
 * refused and a head above what Node agrees to read arrive as the same one. An answer that crossed
 * a ceiling draws the same answer the next time, so it is not repeated; everything else here is a
 * request that got no answer, which says nothing about whether Hook0 acted on it and is what the
 * identifier this client chose makes safe to send again.
 * @param cause - What the rejected `fetch` carried
 */
function transportFailure(cause: unknown): Attempt {
  return failed(detailOf(cause), !isAboveABound(cause));
}

/** The identifier Hook0 says it ingested the event under, when its answer carries one. */
function ingestedIdOf(
  body: string
): { kind: 'ingestedId'; eventId: string } | { kind: 'noIngestedId' } {
  const read = jsonOf(body);
  if (
    read.kind === 'json' &&
    typeof read.value === 'object' &&
    read.value !== null &&
    'event_id' in read.value
  ) {
    const ingestedId = (read.value as { event_id: unknown }).event_id;
    if (typeof ingestedId === 'string') {
      return { kind: 'ingestedId', eventId: ingestedId };
    }
  }
  return { kind: 'noIngestedId' };
}

/**
 * Read what Hook0 answered one attempt, and whether repeating it could end differently.
 * @param response - What Hook0 answered
 * @param maxResponseBytes - Largest body this client agrees to hold
 */
function readAttempt(response: Response, maxResponseBytes: number): Promise<Attempt> {
  const namedDelay = delayNamedBy(response.headers);

  // The head is refused before the body is read, so an abusive head costs one pass over what
  // arrived rather than that plus a body on top. An answer that crossed a ceiling this client set
  // for itself draws the same answer the next time, whatever its status says.
  const head = headAboveABound(response.headers);
  if (head.kind === 'headAboveABound') {
    return discard(response).then(() => failed(head.detail, false));
  }

  return readBoundedBody(response, maxResponseBytes).then((body): Attempt => {
    if (body.kind === 'bodyAboveTheBound') {
      return failed(body.detail, false);
    }

    if (response.ok) {
      const ingested = ingestedIdOf(body.text);
      if (ingested.kind === 'ingestedId') {
        return { kind: 'ingested', eventId: ingested.eventId };
      }
      // Hook0 accepted the event but answered something this client cannot read; repeating the
      // request would meet the same answer.
      return failed(`Hook0 answered ${response.status} without an event id`, false);
    }

    if (response.status === CONFLICT && problemIs(body.text, ALREADY_INGESTED)) {
      return { kind: 'alreadyIngested', detail: refusalOf(response.status, body.text) };
    }
    return {
      kind: 'failed',
      detail: refusalOf(response.status, body.text),
      retryable: isRetryable(response.status, body.text),
      namedDelay,
    };
  });
}

/**
 * How long to wait before the next attempt: what Hook0 asked for when it asked for anything, and
 * this client's own schedule otherwise.
 *
 * Either way it is cut down to what is left of the budget the delays of one send share, so a delay
 * written by the other end cannot stretch a send past what its caller allowed for it.
 * @param named - The delay Hook0 named, when it named one
 * @param scheduledMs - What this client's own schedule had in mind
 * @param remainingMs - What is left of the budget every delay of this send shares
 */
function waitBeforeRetry(named: NamedDelay, scheduledMs: number, remainingMs: number): number {
  const wanted = named.kind === 'delayNamed' ? named.delayMs : scheduledMs;
  return Math.max(Math.min(wanted, remainingMs), 0);
}

/**
 * The base every endpoint of this client is resolved against.
 *
 * A relative reference resolves against the *directory* of its base, so `event` against
 * `https://app.hook0.com/api/v1` reaches `/api/event` — the last segment of the base is replaced
 * rather than kept, and the whole API is missed by one path segment. The trailing slash is added
 * here, once, so that the base URL every SDK's README spells without one reaches the same endpoints
 * as the one spelled with it.
 * @param apiUrl - API base URL, with or without a trailing slash
 */
function baseOf(apiUrl: string): URL {
  const base = new URL(apiUrl);
  if (!base.pathname.endsWith('/')) {
    base.pathname = `${base.pathname}/`;
  }
  return base;
}

/**
 * Client class to interact with Hook0 API
 */
export class Hook0Client {
  private headers: { headers: { Authorization: string } };
  private apiUrl: URL;
  private applicationId: string;
  private debug: boolean;
  private options: Hook0ClientOptions;

  /**
   * Constructor for Hook0Client
   * @param apiUrl - API base URL
   * @param applicationId - Application ID
   * @param token - Authorization token
   * @param debug - Whether to log what the client does
   * @param options - Bounds the client applies to one send
   */
  constructor(
    apiUrl: string,
    applicationId: string,
    token: string,
    debug: boolean = false,
    options: Hook0ClientOptions = new Hook0ClientOptions()
  ) {
    this.apiUrl = baseOf(apiUrl);
    this.applicationId = applicationId;
    this.headers = {
      headers: { Authorization: `Bearer ${token}` },
    };
    this.debug = debug;
    this.options = options;
  }

  /**
   * Send an event
   *
   * The event is sent under an ID this client knows: the one set on the event, or a UUIDv7 this
   * client generates when the event carries none. Because Hook0 keys events on that ID, a request
   * that is repeated after a network failure or a server error ingests the event once, not twice —
   * which is what makes retrying safe.
   *
   * A send is bounded on five axes, each of them configurable through `Hook0ClientOptions`: an
   * oversized payload is ruled out before anything is sent, one attempt is bounded by
   * `requestTimeoutMs`, what an answer may cost to read is bounded by `maxResponseBytes`, how many
   * attempts are made is bounded by `RetryPolicy.maxAttempts`, and the time spent waiting between
   * them is bounded by `RetryPolicy.maxTotalDelayMs`.
   *
   * A network failure, a server error and an instance that is pacing its requests are retried;
   * anything Hook0 refuses outright — a spent quota included, which no delay this send can afford
   * would clear — is reported as is. When the answer names how long to wait before the request
   * becomes servable again, that delay is waited out instead of this client's own schedule, cut
   * down to what is left of `RetryPolicy.maxTotalDelayMs`.
   *
   * A retried request that Hook0 answers with `EventAlreadyIngested` resolves: an earlier attempt
   * of this very send reached the API, and the event carries the ID returned here. That answer to
   * a *first* attempt is a genuine conflict and rejects.
   * @param event - Event to be sent
   * @returns Promise resolving to event ID
   */
  sendEvent(event: Event): Promise<string> {
    const eventId = eventIdOf(event);
    const payloadBytes = Buffer.byteLength(event.payload, 'utf8');
    if (payloadBytes > this.options.maxPayloadBytes) {
      return Promise.reject(
        Hook0ClientError.PayloadTooLarge(eventId, payloadBytes, this.options.maxPayloadBytes)
      );
    }

    // The request is built once, before any attempt: a URL nothing can be sent to and a body that
    // cannot be written are the same the second time round, so they are reported here rather than
    // repeated four times over and handed to the caller as a network that would not answer.
    const request = this.eventIngestionRequest(event, eventId);
    if (request.kind === 'unbuildable') {
      return Promise.reject(Hook0ClientError.EventSending(eventId, new Error(request.detail)));
    }

    const policy = this.options.retryPolicy;
    const delays = policy.delaysMs(jitterDraws(policy.attempts() - 1));

    return this.attemptSend(request.url, request.body, eventId, delays, 1, 0);
  }

  /** The request one send repeats, or why it could not be built at all. */
  private eventIngestionRequest(event: Event, eventId: string): BuildableRequest {
    const target = new URL('event', this.apiUrl);
    if (!REACHABLE_SCHEMES.includes(target.protocol)) {
      return {
        kind: 'unbuildable',
        detail: `\`${target.toString()}\` names no scheme a request can travel on; nothing was sent`,
      };
    }

    const fullEvent = FullEvent.fromEvent(event, this.applicationId, eventId);
    try {
      return { kind: 'buildable', url: target.toString(), body: JSON.stringify(fullEvent) };
    } catch (cause) {
      return {
        kind: 'unbuildable',
        detail: `the event cannot be written as JSON: ${detailOf(cause)}; nothing was sent`,
      };
    }
  }

  /**
   * What an answer this client could not act on said, its body read no further than this client
   * agrees to hold.
   * @param response - The answer that was refused
   */
  private async refusal(response: Response): Promise<string> {
    const body = await readBoundedBody(response, this.options.maxResponseBytes);
    if (body.kind === 'bodyAboveTheBound') {
      return `the API answered ${response.status}; ${body.detail}`;
    }
    return refusalOf(response.status, body.text);
  }

  /**
   * The event types the API says it holds, read no further than this client agrees to hold.
   * @param response - What the API answered
   */
  private async readEventTypeNames(response: Response): Promise<string[]> {
    const head = headAboveABound(response.headers);
    if (head.kind === 'headAboveABound') {
      await discard(response);
      throw Hook0ClientError.GetAvailableEventTypes(new Error(head.detail));
    }

    const body = await readBoundedBody(response, this.options.maxResponseBytes);
    if (body.kind === 'bodyAboveTheBound') {
      throw Hook0ClientError.GetAvailableEventTypes(new Error(body.detail));
    }

    const read = jsonOf(body.text);
    if (read.kind === 'notJson' || !Array.isArray(read.value)) {
      throw Hook0ClientError.GetAvailableEventTypes(
        new Error(`the API answered ${response.status} without a list of event types`)
      );
    }
    return read.value.map((et: { event_type_name: string }) => et.event_type_name);
  }

  /** Issue one attempt, then either resolve, wait and issue the next one, or give up. */
  private attemptSend(
    url: string,
    body: string,
    eventId: string,
    delays: number[],
    attempt: number,
    waitedMs: number
  ): Promise<string> {
    return fetch(url, {
      method: 'POST',
      headers: {
        'Content-Type': JSON_MEDIA_TYPE,
        Accept: JSON_MEDIA_TYPE,
        'User-Agent': USER_AGENT,
        // Read per request rather than composed once: the policy is a field a caller can replace
        // on a client already built, and a value settled at construction would state the one it
        // replaced.
        'Hook0-Client-Options': clientOptions(this.options.retryPolicy),
        ...this.headers.headers,
      },
      body,
      signal: AbortSignal.timeout(this.options.requestTimeoutMs),
    })
      .then(
        (response) => readAttempt(response, this.options.maxResponseBytes),
        (cause: unknown): Attempt => transportFailure(cause)
      )
      .then((outcome) => {
        if (outcome.kind === 'ingested') {
          return outcome.eventId;
        }

        if (outcome.kind === 'alreadyIngested') {
          if (attempt > 1) {
            if (this.debug) {
              console.debug(
                `Event ${eventId} was already ingested by a previous attempt of this send`
              );
            }
            return eventId;
          }
          return Promise.reject(Hook0ClientError.EventSending(eventId, new Error(outcome.detail)));
        }

        const retry = attempt - 1;
        if (outcome.retryable && retry < delays.length) {
          const delay = waitBeforeRetry(
            outcome.namedDelay,
            delays[retry],
            this.options.retryPolicy.maxTotalDelayMs - waitedMs
          );
          if (this.debug) {
            console.debug(`Attempt ${attempt} at sending event ${eventId} failed, retrying`);
          }
          return wait(delay).then(() =>
            this.attemptSend(url, body, eventId, delays, attempt + 1, waitedMs + delay)
          );
        }

        if (attempt > 1) {
          return Promise.reject(
            Hook0ClientError.RetriesExhausted(eventId, attempt, waitedMs, outcome.detail)
          );
        }
        return Promise.reject(Hook0ClientError.EventSending(eventId, new Error(outcome.detail)));
      });
  }

  /**
   * Upsert event types
   * @param eventTypes - Array of event type strings (formatted as "service.resource_type.verb")
   * @returns Promise resolving to array of added event types
   */
  async upsertEventTypes(eventTypes: string[]): Promise<string[]> {
    if (eventTypes.length === 0) {
      return [];
    }

    const structuredEventTypes = eventTypes.map((str) => {
      const eventType = EventType.fromString(str);
      if (eventType instanceof Hook0ClientError) {
        throw Hook0ClientError.InvalidEventType(str);
      }
      return eventType;
    });

    if (this.debug) {
      console.debug('Getting the list of available event types');
    }
    const eventTypesUrl = new URL('event_types', this.apiUrl);
    const response = await fetch(
      `${eventTypesUrl.toString()}?application_id=${this.applicationId}`,
      {
        method: 'GET',
        headers: {
          Accept: JSON_MEDIA_TYPE,
          'User-Agent': USER_AGENT,
          'Hook0-Client-Options': clientOptions(this.options.retryPolicy),
          ...this.headers.headers,
        },
      }
    );

    if (!response.ok) {
      throw Hook0ClientError.GetAvailableEventTypes(new Error(await this.refusal(response)));
    }

    const availableEventTypes = new Set(await this.readEventTypeNames(response));

    if (this.debug) {
      console.debug(`There are currently ${availableEventTypes.size} event types`);
    }

    const addedEventTypes: string[] = [];
    for (const eventType of structuredEventTypes) {
      const eventTypeStr = `${eventType.service}.${eventType.resourceType}.${eventType.verb}`;
      if (!availableEventTypes.has(eventTypeStr)) {
        if (this.debug) {
          console.debug(`Creating event type ${eventTypeStr}...`);
        }
        const body = {
          application_id: this.applicationId,
          service: eventType.service,
          resource_type: eventType.resourceType,
          verb: eventType.verb,
        };

        const postResponse = await fetch(eventTypesUrl.toString(), {
          method: 'POST',
          headers: {
            'Content-Type': JSON_MEDIA_TYPE,
            Accept: JSON_MEDIA_TYPE,
            'User-Agent': USER_AGENT,
            'Hook0-Client-Options': clientOptions(this.options.retryPolicy),
            ...this.headers.headers,
          },
          body: JSON.stringify(body),
        });

        if (!postResponse.ok) {
          throw Hook0ClientError.EventSending(
            eventTypeStr,
            new Error(await this.refusal(postResponse))
          );
        }

        addedEventTypes.push(eventTypeStr);
      }
    }
    if (this.debug) {
      console.debug(`${addedEventTypes.length} new event types were created`);
    }
    return addedEventTypes;
  }
}

/**
 * Represents an event
 */
export class Event {
  /**
   * Constructor for Event
   * @param eventType - Event type
   * @param payload - Payload
   * @param payloadContentType - Content type of the payload
   * @param labels - Labels
   * @param metadata - Metadata (Optional)
   * @param occurredAt - Date when the event occurred (Optional)
   * @param eventId - ID of the event (Optional; the client generates a UUIDv7 when none is given,
   * sends it, and returns it — which is what lets it retry a request without risking a second copy
   * of the event being ingested and delivered to every subscriber)
   */
  constructor(
    public eventType: string,
    public payload: string,
    public payloadContentType: string,
    public labels: Record<string, string>,
    public metadata?: Record<string, string>,
    public occurredAt?: Date,
    public eventId?: string
  ) {}
}

/**
 * Represents a full event ready to be sent
 */
class FullEvent {
  public eventId: string;

  /**
   * Constructor for FullEvent
   * @param applicationId - Application ID
   * @param eventType - Event type
   * @param payload - Payload
   * @param payloadContentType - Content type of the payload
   * @param eventId - ID the event is sent under
   * @param metadata - Metadata (Optional)
   * @param occurredAt - Date when the event occurred (Optional)
   * @param labels - Labels (Optional)
   */
  constructor(
    public applicationId: string,
    public eventType: string,
    public payload: string,
    public payloadContentType: string,
    eventId: string,
    public metadata?: Record<string, string>,
    public occurredAt: Date = new Date(),
    public labels: Record<string, string> = {}
  ) {
    this.eventId = eventId;
  }

  /**
   * Create a FullEvent from an Event
   * @param event - Event object
   * @param applicationId - Application ID
   * @param eventId - ID the event is sent under
   * @returns FullEvent instance
   */
  static fromEvent(event: Event, applicationId: string, eventId: string): FullEvent {
    return new FullEvent(
      applicationId,
      event.eventType,
      event.payload,
      event.payloadContentType,
      eventId,
      event.metadata,
      event.occurredAt,
      event.labels
    );
  }

  /**
   * Convert FullEvent to JSON representation
   * @returns JSON object
   */
  toJSON() {
    return {
      event_id: this.eventId,
      application_id: this.applicationId,
      event_type: this.eventType,
      payload: this.payload,
      payload_content_type: this.payloadContentType,
      metadata: this.metadata,
      occurred_at: this.occurredAt,
      labels: this.labels,
    };
  }
}

/**
 * Represents an event type
 */
export class EventType {
  service: string;
  resourceType: string;
  verb: string;

  /**
   * Constructor for EventType
   * @param service - Service name (e.g. "auth")
   * @param resourceType - Resource type (e.g. "user")
   * @param verb - Verb (e.g. "create")
   */
  constructor(service: string, resourceType: string, verb: string) {
    this.service = service;
    this.resourceType = resourceType;
    this.verb = verb;
  }

  /**
   * Create an EventType from a string
   * @param s - String representing the event type (e.g. "auth.user.create")
   * @returns EventType instance or Hook0ClientError
   */
  static fromString(s: string): EventType | Hook0ClientError {
    const regex = /^([A-Z0-9_]+)[.]([A-Z0-9_]+)[.]([A-Z0-9_]+)$/i;
    const captures = s.match(regex);

    if (captures) {
      const [, service, resourceType, verb] = captures;
      return new EventType(service, resourceType, verb);
    } else {
      return Hook0ClientError.InvalidEventType(s);
    }
  }
}

/**
 * Verifies the signature of a webhook.
 * @param signature - The value of the `X-Hook0-Signature` header.
 * @param payload - The raw body of the webhook request.
 * @param subscriptionSecret - The signing secret used to validate the signature.
 * @param tolerance - The maximum allowed time difference for the timestamp, in seconds and in
 * either direction. A timestamp that is too far in the future is rejected just like one that is
 * too far in the past, so that the acceptance window of any given webhook stays bounded.
 * @param currentTime - The current time (used to check the timestamp).
 * @returns `true`, and nothing else. Every reason a webhook is refused is thrown, never returned,
 * so there is no falsy answer to branch on and `true` is the whole of what a caller can be handed.
 */
export function verifyWebhookSignatureWithCurrentTime(
  signature: string,
  payload: Buffer,
  headers: Headers,
  subscriptionSecret: string,
  tolerance: number,
  currentTime: Date
): true {
  const parsedSig = Signature.parse(signature);
  if (!parsedSig) {
    throw Hook0ClientError.SignatureParsing(signature);
  }

  const expectedSignature = parsedSig.verify(payload, headers, subscriptionSecret);
  if (!expectedSignature) {
    throw Hook0ClientError.InvalidSignature(signature);
  }

  if (Math.abs(Math.floor(currentTime.getTime() / 1000) - parsedSig.timestamp) > tolerance) {
    throw Hook0ClientError.ExpiredWebhook(
      new Date(parsedSig.timestamp * 1000),
      tolerance,
      currentTime
    );
  }

  return true;
}

/**
 * Verifies the signature of a webhook.
 * @param signature - The value of the `X-Hook0-Signature` header.
 * @param payload - The raw body of the webhook request.
 * @param subscriptionSecret - The signing secret used to validate the signature.
 * @param tolerance - The maximum allowed time difference for the timestamp, in seconds and in
 * either direction. A timestamp that is too far in the future is rejected just like one that is
 * too far in the past, so that the acceptance window of any given webhook stays bounded.
 * @returns `true`, and nothing else. Every reason a webhook is refused is thrown, never returned,
 * so there is no falsy answer to branch on and `true` is the whole of what a caller can be handed.
 */
export function verifyWebhookSignature(
  signature: string,
  payload: Buffer,
  headers: Headers,
  subscriptionSecret: string,
  tolerance: number
): true {
  return verifyWebhookSignatureWithCurrentTime(
    signature,
    payload,
    headers,
    subscriptionSecret,
    tolerance,
    new Date()
  );
}
