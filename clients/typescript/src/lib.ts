import { URL } from 'url';
import { Signature } from './index';

/**
 * Maximum number of pages `paginatedFetch` will follow before throwing.
 * Guards against accidental infinite loops if a server returns a self-referential
 * `Link: rel="next"` header.
 */
const PAGINATED_FETCH_MAX_PAGES = 100;

/**
 * Parse the URL of the `rel="next"` link from an RFC 5988 Link header value.
 * Returns null when no `next` relation is present.
 *
 * Accepts the conventional comma-separated form, e.g.:
 *   `<https://api.example.com/event_types?application_id=X&pagination_cursor=ABC>; rel="next", <…>; rel="prev"`
 */
function parseNextLink(linkHeader: string | null): string | null {
  if (!linkHeader) {
    return null;
  }

  // Split on commas that are NOT inside <…>. Link header URIs cannot contain `>`
  // (per RFC 3986 / RFC 5988), so a simple split on `, ` between segments is safe
  // when we first match each `<URI>; params` group.
  const segmentRegex = /<([^>]+)>\s*;\s*([^,]+)/g;
  let match: RegExpExecArray | null;
  while ((match = segmentRegex.exec(linkHeader)) !== null) {
    const url = match[1];
    const params = match[2];
    if (/rel\s*=\s*"?next"?/i.test(params)) {
      return url;
    }
  }
  return null;
}

/**
 * Follow cursor-paginated `Link: rel="next"` chains until exhausted and
 * return the merged flat array.
 *
 * Hook0 list endpoints (`/event_types`, `/subscriptions`, `/request_attempts`)
 * cap responses at 100 items and signal more pages via an RFC 5988 `Link` header.
 * Single-page responses (no `Link` header) work transparently.
 *
 * Hard-caps at {@link PAGINATED_FETCH_MAX_PAGES} pages to prevent infinite
 * loops on broken/buggy servers; throws above that.
 *
 * @param url - Initial URL to fetch
 * @param headers - Request headers (forwarded as-is on every page)
 * @returns Merged array of items across all pages
 */
export function paginatedFetch<T>(url: string, headers: Record<string, string>): Promise<T[]> {
  const collected: T[] = [];

  const fetchPage = (pageUrl: string, pageCount: number): Promise<T[]> => {
    if (pageCount > PAGINATED_FETCH_MAX_PAGES) {
      return Promise.reject(
        new Error(
          `paginatedFetch exceeded ${PAGINATED_FETCH_MAX_PAGES} pages while following Link: rel="next" — aborting to prevent an infinite loop. Last URL: ${pageUrl}`
        )
      );
    }

    return fetch(pageUrl, { method: 'GET', headers }).then((response) => {
      if (!response.ok) {
        return Promise.reject(
          new Error(`HTTP ${response.status} ${response.statusText} on ${pageUrl}`)
        );
      }

      return response.json().then((body) => {
        if (!Array.isArray(body)) {
          return Promise.reject(
            new Error(
              `paginatedFetch expected an array body but received ${typeof body} on ${pageUrl}`
            )
          );
        }
        for (const item of body) {
          collected.push(item as T);
        }

        // Headers API normalizes header names case-insensitively.
        const linkHeader = response.headers.get('link');
        const nextUrl = parseNextLink(linkHeader);
        if (nextUrl === null) {
          return collected;
        }
        return fetchPage(nextUrl, pageCount + 1);
      });
    });
  };

  return fetchPage(url, 1);
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
   * Error when a webhook has expired because it was sent too long ago
   * @param signed_at - Datetime of webhook signature
   * @param tolerance - Maximum difference (in seconds) between the signature datetime and the current datetime for the webhook to be considered valid
   * @param current_time - Current time
   */
  static ExpiredWebhook(signed_at: Date, tolerance: number, current_time: Date): Hook0ClientError {
    return new Hook0ClientError(
      `The webhook has expired because it was sent too long ago (signed_at=${signed_at}, tolerance=${tolerance}, current_time=${current_time})`
    );
  }
}

/**
 * Client class to interact with Hook0 API
 */
export class Hook0Client {
  private headers: { headers: { Authorization: string } };
  private apiUrl: URL;
  private applicationId: string;
  private debug: boolean;

  /**
   * Constructor for Hook0Client
   * @param apiUrl - API base URL
   * @param applicationId - Application ID
   * @param token - Authorization token
   */
  constructor(apiUrl: string, applicationId: string, token: string, debug: boolean = false) {
    this.apiUrl = new URL(apiUrl);
    this.applicationId = applicationId;
    this.headers = {
      headers: { Authorization: `Bearer ${token}` },
    };
    this.debug = debug;
  }

  /**
   * Send an event
   * @param event - Event to be sent
   * @returns Promise resolving to event ID
   */
  async sendEvent(event: Event): Promise<string> {
    const eventIngestionUrl = new URL('event', this.apiUrl);
    const fullEvent = FullEvent.fromEvent(event, this.applicationId);

    try {
      const response = await fetch(eventIngestionUrl.toString(), {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          ...this.headers.headers,
        },
        body: JSON.stringify(fullEvent),
      });

      if (!response.ok) {
        const body = await response.text();
        throw Hook0ClientError.EventSending(fullEvent.eventId, new Error(body));
      }

      const body = (await response.json()) as { event_id: string };
      return body.event_id;
    } catch (error) {
      throw Hook0ClientError.EventSending(
        fullEvent.eventId,
        error instanceof Error ? error : new Error(String(error))
      );
    }
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
    const listUrl = `${eventTypesUrl.toString()}?application_id=${this.applicationId}`;
    const listHeaders = {
      'Content-Type': 'application/json',
      ...this.headers.headers,
    };

    // Server caps each page at 100 items and emits a `Link: rel="next"` header
    // when more pages exist. Follow the cursor chain until exhausted so we
    // never silently miss event types beyond the first page.
    const availableEventTypesVec = await paginatedFetch<{ event_type_name: string }>(
      listUrl,
      listHeaders
    ).catch((error: unknown) => {
      throw Hook0ClientError.GetAvailableEventTypes(
        error instanceof Error ? error : new Error(String(error))
      );
    });
    const availableEventTypes = new Set(availableEventTypesVec.map((et) => et.event_type_name));

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
   * @param eventId - ID of the event (Optional)
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
  public eventId: string | undefined;

  /**
   * Constructor for FullEvent
   * @param applicationId - Application ID
   * @param eventType - Event type
   * @param payload - Payload
   * @param payloadContentType - Content type of the payload
   * @param metadata - Metadata (Optional)
   * @param occurredAt - Date when the event occurred (Optional)
   * @param labels - Labels (Optional)
   * @param eventId - ID of the event (Optional)
   */
  constructor(
    public applicationId: string,
    public eventType: string,
    public payload: string,
    public payloadContentType: string,
    public metadata?: Record<string, string>,
    public occurredAt: Date = new Date(),
    public labels: Record<string, string> = {},
    eventId?: string
  ) {
    this.eventId = eventId;
  }

  /**
   * Create a FullEvent from an Event
   * @param event - Event object
   * @param applicationId - Application ID
   * @returns FullEvent instance
   */
  static fromEvent(event: Event, applicationId: string): FullEvent {
    return new FullEvent(
      applicationId,
      event.eventType,
      event.payload,
      event.payloadContentType,
      event.metadata,
      event.occurredAt,
      event.labels,
      event.eventId
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
 * @param tolerance - The maximum allowed time difference for the timestamp (in seconds).
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
    throw Hook0ClientError.ExpiredWebhook(new Date(parsedSig.timestamp), tolerance, currentTime);
  }

  return true;
}

/**
 * Verifies the signature of a webhook.
 * @param signature - The value of the `X-Hook0-Signature` header.
 * @param payload - The raw body of the webhook request.
 * @param subscriptionSecret - The signing secret used to validate the signature.
 * @param tolerance - The maximum allowed time difference for the timestamp (in seconds).
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
