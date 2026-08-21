package com.hook0.client;

import java.util.UUID;

/**
 * What this client refuses to do, or gave up on doing.
 *
 * <p>Sending an event, upserting an event type and verifying a webhook all raise this, so a caller has one thing to
 * catch whatever it asked the client to do. The failures the <em>API</em> reports are a different matter: those are the
 * problems it names in its own error contract, and the generated half raises one exception per problem.
 *
 * <p>The factories below are what the client raises through; each one exists so that the same situation always reads
 * the same way, whichever call it came out of.
 */
public final class ClientException extends Hook0Exception {

  private static final long serialVersionUID = 1L;

  private ClientException(String detail) {
    super(detail);
  }

  /**
   * A send the API refused for a reason repeating it would not change.
   *
   * @param eventId what the event was sent under
   * @param detail what the API said about it
   * @return the failure to raise
   */
  public static ClientException eventSending(UUID eventId, String detail) {
    return new ClientException("Sending event " + eventId + " failed: " + detail);
  }

  /**
   * A send that ran out of attempts, or out of the delay budget its attempts share.
   *
   * <p>A send that gave up and a single refused request are otherwise indistinguishable to a caller, which is the
   * difference between a transient outage and a request that will never be accepted.
   *
   * @param eventId what the event was sent under
   * @param attempts how many requests the send issued
   * @param waitedMillis how long it spent waiting between them
   * @param detail what the last failure said
   * @return the failure to raise
   */
  public static ClientException retriesExhausted(UUID eventId, int attempts, long waitedMillis, String detail) {
    return new ClientException(
        "Sending event "
            + eventId
            + " failed: gave up after "
            + attempts
            + " attempts spread over "
            + String.format(java.util.Locale.ROOT, "%.3f", waitedMillis / 1000.0)
            + "s of retry delay; last failure: "
            + detail);
  }

  /**
   * A payload above what the client agrees to send, refused before a socket is opened.
   *
   * @param eventId what the event would have been sent under
   * @param size how large the payload is
   * @param maximum how large one may be
   * @return the failure to raise
   */
  public static ClientException payloadTooLarge(UUID eventId, int size, int maximum) {
    return new ClientException(
        "Sending event "
            + eventId
            + " failed: event payload is "
            + size
            + " bytes, which is more than the "
            + maximum
            + " bytes this client sends at most; nothing was sent");
  }

  /**
   * An event type that does not read as {@code service.resource_type.verb}.
   *
   * @param eventType what was handed in
   * @return the failure to raise
   */
  public static ClientException invalidEventType(String eventType) {
    return new ClientException(
        "Provided event type '" + eventType + "' does not have a valid syntax (service.resource_type.verb)");
  }

  /**
   * The list of event types the application already declares could not be read.
   *
   * @param detail what stopped it
   * @return the failure to raise
   */
  public static ClientException availableEventTypes(String detail) {
    return new ClientException("Getting available event types failed: " + detail);
  }

  /**
   * An event type that could not be created.
   *
   * @param eventType which one
   * @param detail what stopped it
   * @return the failure to raise
   */
  public static ClientException creatingEventType(String eventType, String detail) {
    return new ClientException("Creating event type '" + eventType + "' failed: " + detail);
  }

  /**
   * A webhook this client refuses to accept.
   *
   * @param detail why it is refused
   * @return the failure to raise
   */
  public static ClientException refusedDelivery(String detail) {
    return new ClientException(detail);
  }
}
