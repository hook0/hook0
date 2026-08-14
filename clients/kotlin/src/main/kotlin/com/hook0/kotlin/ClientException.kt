package com.hook0.kotlin

import java.util.Locale
import java.util.UUID

/**
 * What this client refuses to do, or gave up on doing.
 *
 * Sending an event, upserting an event type and verifying a webhook all raise this, so a caller has
 * one thing to catch whatever it asked the client to do. The failures the *API* reports are a
 * different matter: those are the problems it names in its own error contract, and the generated
 * half raises one exception per problem.
 *
 * The factories below are what the client raises through; each one exists so that the same
 * situation always reads the same way, whichever call it came out of.
 *
 * @param detail what to say about the failure
 */
class ClientException private constructor(detail: String) : Hook0Exception(detail) {

  companion object {
    /**
     * A send the API refused for a reason repeating it would not change.
     *
     * @param eventId what the event was sent under
     * @param detail what the API said about it
     * @return the failure to raise
     */
    fun eventSending(eventId: UUID, detail: String): ClientException =
      ClientException("Sending event $eventId failed: $detail")

    /**
     * A send that ran out of attempts, or out of the delay budget its attempts share.
     *
     * A send that gave up and a single refused request are otherwise indistinguishable to a caller,
     * which is the difference between a transient outage and a request that will never be accepted.
     *
     * @param eventId what the event was sent under
     * @param attempts how many requests the send issued
     * @param waitedMillis how long it spent waiting between them
     * @param detail what the last failure said
     * @return the failure to raise
     */
    fun retriesExhausted(eventId: UUID, attempts: Int, waitedMillis: Long, detail: String): ClientException {
      val spent = String.format(Locale.ROOT, "%.3f", waitedMillis / 1000.0)
      return ClientException(
        "Sending event $eventId failed: gave up after $attempts attempts spread over ${spent}s " +
          "of retry delay; last failure: $detail"
      )
    }

    /**
     * A payload above what the client agrees to send, refused before a socket is opened.
     *
     * @param eventId what the event would have been sent under
     * @param size how large the payload is
     * @param maximum how large one may be
     * @return the failure to raise
     */
    fun payloadTooLarge(eventId: UUID, size: Int, maximum: Int): ClientException = ClientException(
      "Sending event $eventId failed: event payload is $size bytes, which is more than the " +
        "$maximum bytes this client sends at most; nothing was sent"
    )

    /**
     * An event type that does not read as `service.resource_type.verb`.
     *
     * @param eventType what was handed in
     * @return the failure to raise
     */
    fun invalidEventType(eventType: String): ClientException = ClientException(
      "Provided event type '$eventType' does not have a valid syntax (service.resource_type.verb)"
    )

    /**
     * The list of event types the application already declares could not be read.
     *
     * @param detail what stopped it
     * @return the failure to raise
     */
    fun availableEventTypes(detail: String): ClientException =
      ClientException("Getting available event types failed: $detail")

    /**
     * An event type that could not be created.
     *
     * @param eventType which one
     * @param detail what stopped it
     * @return the failure to raise
     */
    fun creatingEventType(eventType: String, detail: String): ClientException =
      ClientException("Creating event type '$eventType' failed: $detail")

    /**
     * A webhook this client refuses to accept.
     *
     * @param detail why it is refused
     * @return the failure to raise
     */
    fun refusedDelivery(detail: String): ClientException = ClientException(detail)
  }
}
