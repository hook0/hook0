// The rest of the file, for every Java example of the SDK reference.
//
// A snippet on a page is written for a reader: it leaves out the imports, it assumes a client is
// already built, and it names an application id, a token or a secret without saying where it came
// from. Each region below is the file that snippet would live in, with a hole where it goes. The
// page points at one by name on the fence, so what a snippet is standing on is one word away from
// the snippet itself.
//
// Every region becomes its own file, compiled on its own, which is why every one of them wraps its
// hole in a class named `{{Name}}`: Java requires a public top-level class to sit in a file named
// after it, and `{{Name}}` is what this project names that file. What the snippet was handed arrives
// as a parameter of the method the hole sits in, so a page never needs to say where a value like
// `applicationId` or `token` came from — the method signature already says what type it is.

// HARNESS send
import com.hook0.client.Event;
import com.hook0.client.Hook0Client;
import java.util.Map;
import java.util.UUID;

/** What sending this event was handed before it started. */
public final class {{Name}} {
  static void send(String applicationId, String token) {
    EXAMPLE
  }
}
// END HARNESS

// HARNESS send_async
import com.hook0.client.Event;
import com.hook0.client.Hook0Client;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;

/** The client and the event an unwaited send was handed before it started. */
public final class {{Name}} {
  static void send(Hook0Client client, Event event) {
    EXAMPLE
  }
}
// END HARNESS

// HARNESS event_builder
import com.hook0.client.Event;
import java.time.OffsetDateTime;
import java.time.ZoneOffset;
import java.util.Map;
import java.util.UUID;

/** What building this event was handed before it started. */
public final class {{Name}} {
  static void build(String payload, String knownId) {
    EXAMPLE
  }
}
// END HARNESS

// HARNESS bounds
import com.hook0.client.Hook0Client;
import com.hook0.client.Options;
import com.hook0.client.RetryPolicy;
import java.time.Duration;

/** What configuring the client was handed before it started. */
public final class {{Name}} {
  static void configure(String applicationId, String token) {
    EXAMPLE
  }
}
// END HARNESS

// HARNESS verify
import com.hook0.client.Webhooks;
import java.time.Duration;
import java.util.Map;

/** A stand-in for whichever HTTP framework's request type a caller actually has. */
interface IncomingRequest {
  String getHeader(String name);
}

/** What verifying this delivery was handed before it started. */
public final class {{Name}} {
  static void verify(
      IncomingRequest request,
      String rawBody,
      Map<String, String> headersAsTheyArrived,
      String subscriptionSecret) {
    EXAMPLE
  }
}
// END HARNESS

// HARNESS spring
import com.hook0.client.ClientException;
import com.hook0.client.Webhooks;
import java.time.Duration;
import java.util.List;
import java.util.Map;
import org.springframework.http.HttpHeaders;
import org.springframework.http.MediaType;
import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestHeader;
import org.springframework.web.bind.annotation.RestController;

/** The controller the page shows one handler of. */
@RestController
public final class {{Name}} {

  private final String subscriptionSecret = "a-subscription-secret";

  private void handleDelivery(String rawBody) {
    // act on the delivery
  }

  EXAMPLE
}
// END HARNESS

// HARNESS upsert
import com.hook0.client.Hook0Client;
import java.util.List;

/** What declaring these event types was handed before it started. */
public final class {{Name}} {
  static void declare(Hook0Client client) {
    EXAMPLE
  }
}
// END HARNESS

// HARNESS rest_api
import com.hook0.client.Hook0Client;
import com.hook0.client.generated.ApplicationInfo;
import com.hook0.client.generated.ApplicationsApi;
import com.hook0.client.generated.ApplicationsAsyncApi;
import java.util.concurrent.CompletableFuture;

/** What reaching the rest of the API was handed before it started. */
public final class {{Name}} {
  static void read(Hook0Client client, String applicationId) {
    EXAMPLE
  }
}
// END HARNESS
