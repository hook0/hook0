package com.hook0.kotlin

import java.time.OffsetDateTime
import java.util.UUID

/**
 * An event to send to Hook0.
 *
 * [eventId] is the caller's to set when it already has one to key the event on. Left out, the client
 * mints a UUIDv7, sends it and answers it — which is what lets it repeat a request without risking a
 * second copy of the event being ingested and delivered to every subscriber.
 *
 * @property eventType the type of the event, as the application declares it
 * @property payload what the event carries
 * @property payloadContentType how to read the payload
 * @property labels what Hook0 routes the event by
 * @property metadata anything else worth carrying
 * @property occurredAt when the event happened, or nothing for the current moment
 * @property eventId what to key the event on, or nothing to let the client choose
 */
data class Event(
  val eventType: String,
  val payload: String,
  val payloadContentType: String,
  val labels: Map<String, String> = emptyMap(),
  val metadata: Map<String, String>? = null,
  val occurredAt: OffsetDateTime? = null,
  val eventId: UUID? = null
)
