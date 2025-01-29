import { v4 as uuidv4 } from "uuid";
import { URL } from "url";

class Hook0ClientError extends Error {

  static EventSending(eventId: string, error: any): Hook0ClientError {
    return new Hook0ClientError(`Sending event ${eventId} failed: ${error}`);
  }

  static EventTypeInvalid(s: string): Hook0ClientError {
    return new Hook0ClientError(`Event type ${s} is invalid`);
  }

  static GetAvailableEventTypes(error: any): Hook0ClientError {
    return new Hook0ClientError(`Getting available event types failed: ${error}`);
  }
}

class Hook0Client {
  private headers: { headers: { Authorization: string } };
  private apiUrl: URL;
  private applicationId: string;

  constructor(apiUrl: string, applicationId: string, token: string) {
    this.apiUrl = new URL(apiUrl);
    this.applicationId = applicationId;
    this.headers = {
      headers: { Authorization: `Bearer ${token}` },
    };
  }

  async sendEvent(event: Event): Promise<string> {
    const eventIngestionUrl = new URL("event", this.apiUrl);
    console.error(eventIngestionUrl.toString());
    const fullEvent = FullEvent.fromEvent(event, this.applicationId);

    try {
        const response = await fetch(eventIngestionUrl.toString(), {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
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
        throw Hook0ClientError.EventSending(fullEvent.eventId, error instanceof Error ? error : new Error(String(error)));
    }
  }

async upsertEventTypes(eventTypes: string[]): Promise<string[]> {
  const structuredEventTypes = eventTypes.map((str) => {
    const eventType = EventType.fromString(str);
    if (eventType instanceof Hook0ClientError) {
      throw Hook0ClientError.EventTypeInvalid(str);
    }
    return eventType;
  });

  console.debug("Getting the list of available event types");
  const eventTypesUrl = new URL("event_types", this.apiUrl);
  const response = await fetch(`${eventTypesUrl.toString()}?application_id=${this.applicationId}`, {
    method: "GET",
    headers: {
      "Content-Type": "application/json",
      ...this.headers.headers,
    },
  });

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
        method: "POST",
        headers: {
          "Content-Type": "application/json",
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

class Event {
  constructor(
    public eventType: string,
    public payload: string,
    public payloadContentType: string,
    public labels: Record<string, string>,
    public metadata?: Record<string, string>,
    public occurredAt?: Date,
    public eventId?: string,
  ) {}
}

class FullEvent {
  public eventId: string;

  constructor(
    public applicationId: string,
    public eventType: string,
    public payload: string,
    public payloadContentType: string,
    public metadata?: Record<string, string>,
    public occurredAt?: Date,
    public labels: Record<string, string> = {},
    eventId?: string,
  ) {
    this.eventId = eventId || uuidv4();
  }

  static fromEvent(event: Event, applicationId: string): FullEvent {
    return new FullEvent(
      applicationId,
      event.eventType,
      event.payload,
      event.payloadContentType,
      event.metadata,
      event.occurredAt || new Date(),
      event.labels,
      event.eventId,
    );
  }

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

class EventType {
  service: string;
  resourceType: string;
  verb: string;

  constructor(service: string, resourceType: string, verb: string) {
    this.service = service;
    this.resourceType = resourceType;
    this.verb = verb;
  }

  static fromString(s: string): EventType | Hook0ClientError{
    const regex = /^([A-Z0-9_]+)[.]([A-Z0-9_]+)[.]([A-Z0-9_]+)$/i;
    const captures = s.match(regex);

    if (captures) {
      const [, service, resourceType, verb] = captures;
      return new EventType(service, resourceType, verb);
    } else {
      return Hook0ClientError.EventTypeInvalid(s);
    }
  }
}

export { Hook0ClientError, Hook0Client, Event, EventType };
