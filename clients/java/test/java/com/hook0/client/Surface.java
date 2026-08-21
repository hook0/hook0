package com.hook0.client;

import java.util.List;
import java.util.UUID;
import java.util.concurrent.CompletionException;

/**
 * The two surfaces this client offers, as one thing a case can be written against.
 *
 * <p>Every case that sends an event runs twice, once through each. That is the whole point of the type: a defect fixed
 * in one surface and left standing in the other is exactly what having two costs, and the only way to find one is to
 * put the same case through both.
 *
 * <p>The future is joined and its wrapper unwrapped, so a case sees the same failure whichever surface raised it —
 * which is itself an assertion, since a surface that reported another type would fail every case that names one.
 */
enum Surface {
  BLOCKING {
    @Override
    UUID send(Hook0Client client, Event event) {
      return client.sendEvent(event);
    }

    @Override
    List<String> upsertEventTypes(Hook0Client client, List<String> eventTypes) {
      return client.upsertEventTypes(eventTypes);
    }
  },
  FUTURE {
    @Override
    UUID send(Hook0Client client, Event event) {
      return joined(() -> client.sendEventAsync(event).join());
    }

    @Override
    List<String> upsertEventTypes(Hook0Client client, List<String> eventTypes) {
      return joined(() -> client.upsertEventTypesAsync(eventTypes).join());
    }
  };

  abstract UUID send(Hook0Client client, Event event);

  abstract List<String> upsertEventTypes(Hook0Client client, List<String> eventTypes);

  static <T> T joined(java.util.function.Supplier<T> waiting) {
    try {
      return waiting.get();
    } catch (CompletionException wrapped) {
      Throwable carried = wrapped.getCause();
      if (carried instanceof RuntimeException reported) {
        throw reported;
      }
      throw wrapped;
    }
  }
}
