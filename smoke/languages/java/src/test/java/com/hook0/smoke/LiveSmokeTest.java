package com.hook0.smoke;

import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.hook0.client.ClientException;
import com.hook0.client.Event;
import com.hook0.client.Hook0Client;
import com.hook0.client.Webhooks;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Duration;
import java.util.AbstractMap;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.UUID;
import org.junit.jupiter.api.Test;

/**
 * The Java client against a Hook0 that is really running.
 *
 * <p>Three things the loopback suite cannot ask: whether an application secret the API minted is accepted, whether a
 * second send under an identifier already ingested is reported as the conflict it is, and whether a signature the
 * output worker computed verifies. Everything else about this client is settled by {@code clients/java/test}.
 */
final class LiveSmokeTest {

  /** The conflict the API answers a duplicated ingestion with. */
  private static final String ALREADY_INGESTED = "EventAlreadyIngested";

  @Test
  void theClientTalksToARealInstance() throws Exception {
    String eventType = setting("HOOK0_EVENT_TYPE");

    try (Hook0Client client =
        new Hook0Client(setting("HOOK0_API_URL"), setting("HOOK0_APPLICATION_ID"), setting("HOOK0_TOKEN"))) {
      UUID sent = client.sendEvent(event(eventType, null));
      System.out.println("ingested " + sent);

      ClientException refused =
          assertThrows(
              ClientException.class,
              () -> client.sendEvent(event(eventType, sent)),
              "sending the same event twice was accepted twice");
      assertTrue(
          String.valueOf(refused.getMessage()).contains(ALREADY_INGESTED),
          () -> "the second send failed without naming " + ALREADY_INGESTED + ": " + refused.getMessage());
      System.out.println("the second send reported " + ALREADY_INGESTED);
    }

    verify(Path.of(setting("HOOK0_DELIVERY")));
    System.out.println("the signature the instance produced verifies");
  }

  /** The event both sends carry, under the identifier the caller names. */
  private static Event event(String eventType, UUID eventId) {
    Event built =
        Event.of(eventType, "{\"from\":\"the java smoke\"}", "application/json", Map.of("language", "java"));
    return eventId == null ? built : built.withEventId(eventId);
  }

  /** Verifies what the output worker really delivered, with this client's own verification. */
  private static void verify(Path delivery) throws IOException {
    List<Map.Entry<String, String>> headers = new ArrayList<>();
    for (String line : Files.readString(delivery.resolve("headers")).split("\n")) {
      int at = line.indexOf(": ");
      if (at > 0) {
        headers.add(new AbstractMap.SimpleEntry<>(line.substring(0, at), line.substring(at + 2)));
      }
    }

    Webhooks.verify(
        Files.readString(delivery.resolve("signature")).trim(),
        Files.readString(delivery.resolve("body")),
        headers,
        Files.readString(delivery.resolve("secret")).trim(),
        Duration.ofSeconds(Long.parseLong(Files.readString(delivery.resolve("tolerance")).trim())));
  }

  /** A setting the harness passes, or a refusal naming it. */
  private static String setting(String name) {
    String value = System.getenv(name);
    if (value == null || value.isEmpty()) {
      throw new IllegalStateException(name + " is not set");
    }
    return value;
  }
}
