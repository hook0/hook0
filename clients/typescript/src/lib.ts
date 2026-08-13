import { randomBytes } from 'crypto';
import { URL } from 'url';
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
 * Hook0's API refuses request bodies above 2 MiB, so a payload above 1 MiB is already at risk of
 * being refused once the JSON envelope around it (metadata, labels, identifiers) is counted. The
 * client rules such an event out rather than spending a round trip, and every retry after it, on a
 * request that cannot be accepted.
 */
export const DEFAULT_MAX_PAYLOAD_BYTES: number = 1024 * 1024;

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

/** Status Hook0 answers when the event ID a request carries is already taken. */
const CONFLICT = 409;

/** First status that says the failure is on Hook0's side, and so could clear on its own. */
const LOWEST_SERVER_ERROR = 500;

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
    return new Hook0ClientError(`Sending event${eventId ? ' ' + eventId : ''} failed: ${error}`);
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
    return new Hook0ClientError(`Getting available event types failed: ${error}`);
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
    const initial = Math.max(this.initialBackoffMs, 0);
    const maximum = Math.max(this.maxBackoffMs, 0);
    return Math.min(initial * 2 ** doublings, maximum);
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
    const budget = Math.max(this.maxTotalDelayMs, 0);
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
   */
  constructor(
    public retryPolicy: RetryPolicy = new RetryPolicy(),
    public requestTimeoutMs: number = DEFAULT_REQUEST_TIMEOUT_MS,
    public maxPayloadBytes: number = DEFAULT_MAX_PAYLOAD_BYTES
  ) {}
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

/** Whether an RFC 9457 problem body is the one Hook0 answers when an event ID is already taken. */
function isAlreadyIngested(body: string): boolean {
  try {
    const problem: unknown = JSON.parse(body);
    return (
      typeof problem === 'object' &&
      problem !== null &&
      'id' in problem &&
      (problem as { id: unknown }).id === ALREADY_INGESTED
    );
  } catch {
    return false;
  }
}

/** What one attempt at sending an event ended with. */
type Attempt =
  | { kind: 'ingested'; eventId: string }
  | { kind: 'alreadyIngested'; detail: string }
  | { kind: 'failed'; detail: string; retryable: boolean };

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

/** Read what Hook0 answered one attempt, and whether repeating it could end differently. */
function readAttempt(response: Response): Promise<Attempt> {
  if (response.ok) {
    return response.json().then(
      (body: unknown): Attempt => {
        if (typeof body === 'object' && body !== null && 'event_id' in body) {
          const ingestedId = (body as { event_id: unknown }).event_id;
          if (typeof ingestedId === 'string') {
            return { kind: 'ingested', eventId: ingestedId };
          }
        }
        // Hook0 accepted the event but answered something this client cannot read; repeating the
        // request would meet the same answer.
        return {
          kind: 'failed',
          detail: `Hook0 answered ${response.status} without an event id`,
          retryable: false,
        };
      },
      (cause: unknown): Attempt => ({
        kind: 'failed',
        detail: detailOf(cause),
        retryable: false,
      })
    );
  }

  return response.text().then(
    (body): Attempt => {
      if (response.status === CONFLICT && isAlreadyIngested(body)) {
        return { kind: 'alreadyIngested', detail: body };
      }
      return {
        kind: 'failed',
        detail: body,
        // Only the server side of a server error can change between two identical requests.
        retryable: response.status >= LOWEST_SERVER_ERROR,
      };
    },
    (cause: unknown): Attempt => ({
      kind: 'failed',
      detail: detailOf(cause),
      retryable: true,
    })
  );
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
    this.apiUrl = new URL(apiUrl);
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
   * A send is bounded on four axes, each of them configurable through `Hook0ClientOptions`: an
   * oversized payload is ruled out before anything is sent, one attempt is bounded by
   * `requestTimeoutMs`, how many attempts are made is bounded by `RetryPolicy.maxAttempts`, and
   * the time spent waiting between them is bounded by `RetryPolicy.maxTotalDelayMs`. Only a
   * network failure or a server error is retried; anything Hook0 refuses outright is reported as
   * is.
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

    const eventIngestionUrl = new URL('event', this.apiUrl).toString();
    const fullEvent = FullEvent.fromEvent(event, this.applicationId, eventId);
    const body = JSON.stringify(fullEvent);
    const policy = this.options.retryPolicy;
    const delays = policy.delaysMs(jitterDraws(policy.attempts() - 1));

    return this.attemptSend(eventIngestionUrl, body, eventId, delays, 1, 0);
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
        'Content-Type': 'application/json',
        ...this.headers.headers,
      },
      body,
      signal: AbortSignal.timeout(this.options.requestTimeoutMs),
    })
      .then(
        (response) => readAttempt(response),
        // `fetch` only rejects when the request never got an answer: a refused connection, a reset
        // one, an attempt that ran out of time. None of them says whether Hook0 ingested the event,
        // which is precisely why the client sends an ID it chose itself.
        (cause: unknown): Attempt => ({
          kind: 'failed',
          detail: detailOf(cause),
          retryable: true,
        })
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
          const delay = delays[retry];
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
          'Content-Type': 'application/json',
          ...this.headers.headers,
        },
      }
    );

    if (!response.ok) {
      throw Hook0ClientError.GetAvailableEventTypes(new Error(response.statusText));
    }

    const availableEventTypesVec = await response.json();
    const availableEventTypes = new Set(
      availableEventTypesVec.map((et: { event_type_name: string }) => et.event_type_name)
    );

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
            'Content-Type': 'application/json',
            ...this.headers.headers,
          },
          body: JSON.stringify(body),
        });

        if (!postResponse.ok) {
          throw Hook0ClientError.EventSending(eventTypeStr, new Error(postResponse.statusText));
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
 * @returns Resolves if the signature is valid, otherwise throws an error.
 */
export function verifyWebhookSignatureWithCurrentTime(
  signature: string,
  payload: Buffer,
  headers: Headers,
  subscriptionSecret: string,
  tolerance: number,
  currentTime: Date
): boolean | Hook0ClientError {
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
 * @returns Resolves if the signature is valid, otherwise throws an error.
 */
export function verifyWebhookSignature(
  signature: string,
  payload: Buffer,
  headers: Headers,
  subscriptionSecret: string,
  tolerance: number
): boolean | Hook0ClientError {
  return verifyWebhookSignatureWithCurrentTime(
    signature,
    payload,
    headers,
    subscriptionSecret,
    tolerance,
    new Date()
  );
}
