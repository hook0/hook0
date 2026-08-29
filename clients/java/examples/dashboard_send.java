/*
 * What the dashboard shows under "Send an event", for Java.
 *
 * This file exists so that the snippet is compiled against the real client. A renamed method, a
 * changed signature or a dropped field turns `clients.java.check` red on the day it happens, which
 * is the whole reason the snippet lives here rather than in the dashboard: one written by hand over
 * there is backed by nothing and drifts in silence.
 *
 * Two pairs of markers say how it is read. `hook0:snippet` delimits what a reader is shown, so that
 * anything this file needs only in order to compile stays out of it. `hook0:label` delimits the one
 * rendering of a label, which the dashboard repeats once per label the form carries and joins with
 * the separator its manifest declares — the region carries no trailing separator of its own, and
 * sits inside its container, so no label at all leaves a valid empty one.
 *
 * The `__HOOK0_*__` words are string literals, which is what lets a file full of them compile. They
 * never resolve to anything: this example is built, never run.
 */

// hook0:snippet:begin
import com.hook0.client.Event;
import com.hook0.client.Hook0Client;
import java.util.LinkedHashMap;
import java.util.Map;
import java.util.UUID;

class SendAnEvent {
  public static void main(String[] args) {
    // `Hook0Client` is `AutoCloseable`, and one of them is meant to live as long as the application
    // rather than as long as a send.
    try (Hook0Client client =
        new Hook0Client("__HOOK0_API_URL__", "__HOOK0_APPLICATION_ID__", "__HOOK0_TOKEN__")) {
      // What Hook0 routes the event by. A map filled a line at a time, which puts no ceiling on how
      // many labels an event carries and sends the last value given for a repeated key, the way the
      // other ten clients do.
      Map<String, String> labels = new LinkedHashMap<>();
      // hook0:label:begin
      labels.put("__HOOK0_LABEL_KEY__", "__HOOK0_LABEL_VALUE__"); // hook0:label:end
      UUID sent =
          client.sendEvent(
              Event.of("__HOOK0_EVENT_TYPE__", "__HOOK0_PAYLOAD__", "application/json", labels));
      System.out.println("ingested as " + sent);
    }
  }
}
// hook0:snippet:end
