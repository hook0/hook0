import { v4 as uuidv4 } from 'uuid';
import { URL } from 'url';

/**
 * Custom error class for Hook0Client
 */
class Hook0ClientError extends Error {
  /**
   * Error when sending an event fails
   * @param eventId - ID of the event
   * @param error - Error details
   */
  static EventSending(eventId: string, error: any): Hook0ClientError {
    return new Hook0ClientError(`Sending event ${eventId} failed: ${error}`);
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
  static GetAvailableEventTypes(error: any): Hook0ClientError {
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
}

/**
 * Client class to interact with Hook0 API
 */
class Hook0Client {
  private headers: { headers: { Authorization: string } };
  private apiUrl: URL;
  private applicationId: string;

  /**
   * Constructor for Hook0Client
   * @param apiUrl - API base URL
   * @param applicationId - Application ID
   * @param token - Authorization token
   */
  constructor(apiUrl: string, applicationId: string, token: string) {
    this.apiUrl = new URL(apiUrl);
    this.applicationId = applicationId;
    this.headers = {
      headers: { Authorization: `Bearer ${token}` },
    };
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
        throw Hook0ClientError.EventSending(fullEvent.eventId, body);
      }

      return fullEvent.eventId;
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

    console.debug('Getting the list of available event types');
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

    console.debug(`There are currently ${availableEventTypes.size} event types`);

    const addedEventTypes: string[] = [];
    for (const eventType of structuredEventTypes) {
      const eventTypeStr = `${eventType.service}.${eventType.resourceType}.${eventType.verb}`;
      if (!availableEventTypes.has(eventTypeStr)) {
        console.log(`Creating event type ${eventTypeStr}...`);
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

    console.debug(`${addedEventTypes.length} new event types were created`);
    return addedEventTypes;
  }
}

/**
 * Represents an event
 */
class Event {
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
  public eventId: string;

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
    public occurredAt?: Date,
    public labels: Record<string, string> = {},
    eventId?: string
  ) {
    this.eventId = eventId || uuidv4();
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
      event.occurredAt || new Date(),
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
      occurred_at: this.occurredAt ? this.occurredAt.toISOString() : new Date().toISOString(),
      labels: this.labels,
    };
  }
}

/**
 * Represents an event type
 */
class EventType {
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

export { Hook0ClientError, Hook0Client, Event, EventType };
