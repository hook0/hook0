package com.hook0.client;

import java.time.OffsetDateTime;
import java.util.Map;
import java.util.UUID;

/**
 * An event to send to Hook0.
 *
 * <p>{@code eventId} is the caller's to set when it already has one to key the event on. Left as {@code null}, the
 * client mints a UUIDv7, sends it and answers it — which is what lets it repeat a request without risking a second copy
 * of the event being ingested and delivered to every subscriber.
 *
 * @param eventType the type of the event, as the application declares it
 * @param payload what the event carries
 * @param payloadContentType how to read the payload
 * @param labels what Hook0 routes the event by
 * @param metadata anything else worth carrying, or {@code null}
 * @param occurredAt when the event happened, or {@code null} for the current moment
 * @param eventId what to key the event on, or {@code null} to let the client choose
 */
public record Event(
    String eventType,
    String payload,
    String payloadContentType,
    Map<String, String> labels,
    Map<String, String> metadata,
    OffsetDateTime occurredAt,
    UUID eventId) {

  /**
   * The event, with nothing beyond what the API requires.
   *
   * @param eventType the type of the event
   * @param payload what the event carries
   * @param payloadContentType how to read the payload
   * @param labels what Hook0 routes the event by
   * @return the event to send
   */
  public static Event of(
      String eventType, String payload, String payloadContentType, Map<String, String> labels) {
    return new Event(eventType, payload, payloadContentType, labels, null, null, null);
  }

  /**
   * The same event, carrying that metadata.
   *
   * @param chosen what else to carry
   * @return the event with it
   */
  public Event withMetadata(Map<String, String> chosen) {
    return new Event(eventType, payload, payloadContentType, labels, chosen, occurredAt, eventId);
  }

  /**
   * The same event, saying it happened then.
   *
   * @param chosen when the event happened
   * @return the event with that moment
   */
  public Event withOccurredAt(OffsetDateTime chosen) {
    return new Event(eventType, payload, payloadContentType, labels, metadata, chosen, eventId);
  }

  /**
   * The same event, keyed on that identifier.
   *
   * @param chosen what to key the event on
   * @return the event with that identifier
   */
  public Event withEventId(UUID chosen) {
    return new Event(eventType, payload, payloadContentType, labels, metadata, occurredAt, chosen);
  }
}
