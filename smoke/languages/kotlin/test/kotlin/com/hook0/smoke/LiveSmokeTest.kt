package com.hook0.smoke

import com.hook0.kotlin.ClientException
import com.hook0.kotlin.Event
import com.hook0.kotlin.Hook0Client
import com.hook0.kotlin.Webhooks
import java.nio.file.Files
import java.nio.file.Path
import java.time.Duration
import java.util.UUID
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test

/**
 * The Kotlin client against a Hook0 that is really running.
 *
 * Three things the loopback suite cannot ask: whether an application secret the API minted is
 * accepted, whether a second send under an identifier already ingested is reported as the conflict
 * it is, and whether a signature the output worker computed verifies. Everything else about this
 * client is settled by `clients/kotlin/test`.
 */
class LiveSmokeTest {

  /** The conflict the API answers a duplicated ingestion with. */
  private val alreadyIngested = "EventAlreadyIngested"

  @Test
  fun `the client talks to a real instance`() {
    val eventType = setting("HOOK0_EVENT_TYPE")

    Hook0Client(setting("HOOK0_API_URL"), setting("HOOK0_APPLICATION_ID"), setting("HOOK0_TOKEN")).use { client ->
      val sent = client.sendEvent(event(eventType, null))
      println("ingested $sent")

      val refused =
        try {
          client.sendEvent(event(eventType, sent))
          throw AssertionError("sending the same event twice was accepted twice")
        } catch (reported: ClientException) {
          reported
        }
      assertTrue(
        refused.message.orEmpty().contains(alreadyIngested),
        "the second send failed without naming $alreadyIngested: ${refused.message}"
      )
      println("the second send reported $alreadyIngested")
    }

    verify(Path.of(setting("HOOK0_DELIVERY")))
    println("the signature the instance produced verifies")
  }

  /** The event both sends carry, under the identifier the caller names. */
  private fun event(eventType: String, eventId: UUID?) =
    Event(
      eventType = eventType,
      payload = """{"from":"the kotlin smoke"}""",
      payloadContentType = "application/json",
      labels = mapOf("language" to "kotlin"),
      eventId = eventId
    )

  /** Verifies what the output worker really delivered, with this client's own verification. */
  private fun verify(delivery: Path) {
    val headers =
      Files.readString(delivery.resolve("headers"))
        .split("\n")
        .mapNotNull { line ->
          val at = line.indexOf(": ")
          if (at > 0) line.substring(0, at) to line.substring(at + 2) else null
        }

    Webhooks.verify(
      Files.readString(delivery.resolve("signature")).trim(),
      Files.readString(delivery.resolve("body")),
      headers,
      Files.readString(delivery.resolve("secret")).trim(),
      Duration.ofSeconds(Files.readString(delivery.resolve("tolerance")).trim().toLong())
    )
  }

  /** A setting the harness passes, or a refusal naming it. */
  private fun setting(name: String): String =
    System.getenv(name).orEmpty().ifEmpty { error("$name is not set") }
}
