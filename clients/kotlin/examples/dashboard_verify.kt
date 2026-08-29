/*
 * What the dashboard shows under "Verify a webhook", for Kotlin.
 *
 * Sending is only half of what a reader has come to do, and it is the easier half. This is the one
 * the SDK reference calls the half most often got wrong by hand, so the dashboard shows it beside
 * the send rather than leaving it to be found later.
 *
 * The secret is read from the environment on purpose. The dashboard cannot know which subscription
 * a reader means — outside the onboarding it loads none, and an application may have several — so
 * it points at the subscription instead of guessing one, and no second secret is put on screen.
 *
 * Read the markers as in `dashboard_send.kt`: `hook0:snippet` is what is displayed, everything
 * outside it is what makes the file compile.
 */

// hook0:snippet:begin
import com.hook0.kotlin.ClientException
import com.hook0.kotlin.Webhooks
import java.time.Duration

// Verify against the *raw* body: one that has been parsed and serialised again no longer hashes to
// what was signed. The tolerance is bilateral, so a delivery dated too far ahead is refused exactly
// like one dated too far behind, and `verify` names no default: pass one.
fun accept(signature: String, body: String, headers: Map<String, String>): Boolean {
  // The secret of the subscription being verified, which the dashboard links to rather than prints:
  // it cannot know which subscription a reader means, and an application may have several. Read
  // before the `try` and allowed to throw. A variable nobody exported and one exported empty are
  // the same defect: verification hashes the delivery against whatever key it is handed, so either
  // one refuses every genuine delivery as forged while saying nothing at all.
  val secret = System.getenv("HOOK0_SUBSCRIPTION_SECRET")
  check(!secret.isNullOrEmpty()) { "HOOK0_SUBSCRIPTION_SECRET is not set" }

  return try {
    Webhooks.verify(signature, body, headers, secret, Duration.ofMinutes(5))
    true
  } catch (refused: ClientException) {
    false
  }
}
// hook0:snippet:end

fun main() {
  // Nothing here is ever run: this file exists to be compiled against the real client.
  println("accepted: " + accept("", "", mapOf("x-hook0-signature" to "")))
}
