// The rest of the file, for every Kotlin example of the SDK reference.
//
// A snippet on a page is written for a reader: it leaves out the imports, it assumes a client is
// already built, and it names an application id, a token or a secret without saying where it came
// from. Each region below is the file that snippet would live in, with a hole where it goes. The
// page points at one by name on the fence, so what a snippet is standing on is one word away from
// the snippet itself.
//
// Kotlin allows a top-level function outside any class, so — unlike the Java harness beside this
// one — no region here wraps its hole in a wrapper type. What the snippet was handed arrives as a
// parameter of the function the hole sits in, so a page never needs to say where a value like
// `applicationId` or `subscriptionSecret` came from — the function signature already says what type
// it is.

// HARNESS send
import com.hook0.kotlin.Event
import com.hook0.kotlin.Hook0Client
import java.util.UUID

/** What sending this event was handed before it started. */
fun send(applicationId: String, token: String) {
  EXAMPLE
}
// END HARNESS

// HARNESS send_suspending
import com.hook0.kotlin.Event
import com.hook0.kotlin.Hook0Client
import java.util.UUID

/** The client and the event an unwaited send was handed before it started. */
suspend fun sendSuspending(client: Hook0Client, event: Event) {
  EXAMPLE
}
// END HARNESS

// HARNESS event_builder
import com.hook0.kotlin.Event
import java.time.OffsetDateTime
import java.time.ZoneOffset

/** Building this event needs nothing beyond the language and the client itself. */
fun build() {
  EXAMPLE
}
// END HARNESS

// HARNESS bounds
import com.hook0.kotlin.Hook0Client
import com.hook0.kotlin.Options
import com.hook0.kotlin.RetryPolicy
import java.time.Duration

/** What configuring the client was handed before it started. */
fun configure(applicationId: String, token: String) {
  EXAMPLE
}
// END HARNESS

// HARNESS verify
import com.hook0.kotlin.Webhooks
import java.time.Duration

/** A stand-in for whichever HTTP framework's request type a caller actually has. */
interface IncomingRequest {
  fun getHeader(name: String): String
}

/** What verifying this delivery was handed before it started. */
fun verify(
  request: IncomingRequest,
  rawBody: String,
  headersAsTheyArrived: Map<String, String>,
  subscriptionSecret: String
) {
  EXAMPLE
}
// END HARNESS

// HARNESS ktor
import com.hook0.kotlin.ClientException
import com.hook0.kotlin.Webhooks
import io.ktor.http.HttpStatusCode
import io.ktor.server.application.Application
import io.ktor.server.request.receiveText
import io.ktor.server.response.respond
import io.ktor.server.routing.post
import io.ktor.server.routing.routing
import java.time.Duration

private const val subscriptionSecret = "a-subscription-secret"

private fun handleDelivery(rawBody: String) {
  // act on the delivery
}

/** The module the page shows one route of. */
fun Application.module() {
  EXAMPLE
}
// END HARNESS

// HARNESS upsert
import com.hook0.kotlin.Hook0Client

/** What declaring these event types was handed before it started. */
fun declare(client: Hook0Client) {
  EXAMPLE
}
// END HARNESS

// HARNESS rest_api
import com.hook0.kotlin.Hook0Client
import com.hook0.kotlin.generated.ApplicationsApi
import com.hook0.kotlin.generated.ApplicationsSuspendingApi

/** What reaching the rest of the API was handed before it started. */
suspend fun read(client: Hook0Client, applicationId: String) {
  EXAMPLE
}
// END HARNESS
