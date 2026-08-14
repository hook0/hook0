package com.hook0.kotlin

import java.nio.charset.StandardCharsets
import java.time.Duration
import java.time.Instant
import java.util.Locale
import javax.crypto.Mac
import javax.crypto.spec.SecretKeySpec
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertThrows
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.Timeout

/**
 * What a signature header has to be before it is one, beyond the vectors the shared corpus carries.
 *
 * The corpus owns the codes: it holds every accepted and refused delivery, computed outside every
 * implementation, and `ConformanceTest` drives them. What is here is the reading — the shapes a
 * header can arrive in that never reach a code at all — and the two ways a caller can hand the
 * headers over.
 */
@Timeout(60)
class SignatureTest {

  @Test
  fun aSignatureCarryingNoMomentIsRefused() {
    val refused =
      assertThrows(ClientException::class.java) {
        verified("v0=$BODY_CODE,x=1", delivered(), Instant.ofEpochSecond(MOMENT))
      }

    assertTrue(refused.message?.contains("no `t` part") == true, refused.message)
  }

  @Test
  fun aSignatureCarryingNoCodeIsRefused() {
    val refused =
      assertThrows(ClientException::class.java) {
        verified("t=$MOMENT,h=x-event-id", delivered(), Instant.ofEpochSecond(MOMENT))
      }

    assertTrue(refused.message?.contains("nor a `v1` code") == true, refused.message)
  }

  @Test
  fun aMomentNoClockCouldHoldIsRefused() {
    val refused =
      assertThrows(ClientException::class.java) {
        verified("t=99999999999999,v0=$BODY_CODE", delivered(), Instant.ofEpochSecond(MOMENT))
      }

    assertTrue(refused.message?.contains("from the epoch") == true, refused.message)
  }

  @Test
  fun aMomentThatIsNotANumberOfSecondsIsRefusedRatherThanRead() {
    for (written in listOf("1_0", "1.0", "0x10", " ", "+1", "9".repeat(40))) {
      assertThrows(
        ClientException::class.java,
        { verified("t=$written,v0=$BODY_CODE", delivered(), Instant.ofEpochSecond(MOMENT)) },
        "`$written` was read as a number of seconds"
      )
    }
  }

  @Test
  fun thePartsOfASignatureAreReadAroundTheSpacesTheyArrivedWith() {
    verified(" t = $MOMENT , v0 = $BODY_CODE ", delivered(), Instant.ofEpochSecond(MOMENT))
  }

  @Test
  fun onlyTheFirstAssignatorOfAPartSeparatesItsNameFromItsValue() {
    // A value carrying a further one is a value, not a third part: splitting on all of them would
    // silently drop everything past the second and verify against a truncated code.
    val refused =
      assertThrows(ClientException::class.java) {
        verified("t=$MOMENT,v0=$BODY_CODE=extra", delivered(), Instant.ofEpochSecond(MOMENT))
      }

    assertTrue(refused.message?.contains("not hexadecimal") == true, refused.message)
  }

  @Test
  fun headersGivenAsAMappingAreReadLikeHeadersGivenAsPairs() {
    Webhooks.verifyAt(
      HEADER_SIGNATURE,
      PAYLOAD,
      delivered(),
      SECRET,
      TOLERANCE,
      Instant.ofEpochSecond(MOMENT)
    )
    Webhooks.verify(
      HEADER_SIGNATURE,
      PAYLOAD,
      mapOf(
        "X-Event-Id" to "evt-1",
        "X-Delivery-Id" to "dlv-1",
        "Content-Type" to "application/json"
      ),
      SECRET,
      Duration.ofDays(365 * 100)
    )
  }

  @Test
  fun theDeliveredHeadersAreFoundWhateverCaseTheyArrivedIn() {
    verified(
      HEADER_SIGNATURE,
      listOf("X-EVENT-ID" to "evt-1", "X-Delivery-Id" to "dlv-1"),
      Instant.ofEpochSecond(MOMENT)
    )
  }

  @Test
  fun aBodyThatChangedAfterItWasSignedIsRefused() {
    val refused =
      assertThrows(ClientException::class.java) {
        Webhooks.verifyAt(
          "t=$MOMENT,v0=$BODY_CODE",
          "$PAYLOAD ",
          delivered(),
          SECRET,
          TOLERANCE,
          Instant.ofEpochSecond(MOMENT)
        )
      }

    assertTrue(refused.message?.contains("does not match") == true, refused.message)
  }

  @Test
  fun aSignatureMadeUnderAnotherSecretIsRefused() {
    val refused =
      assertThrows(ClientException::class.java) {
        Webhooks.verifyAt(
          "t=$MOMENT,v0=$BODY_CODE",
          PAYLOAD,
          delivered(),
          "another-secret",
          TOLERANCE,
          Instant.ofEpochSecond(MOMENT)
        )
      }

    assertTrue(refused.message?.contains("does not match") == true, refused.message)
  }

  @Test
  fun aSignatureCoveringMoreHeadersThanTheBoundIsRefused() {
    val covered = List(Signature.MAX_COVERED_HEADERS + 1) { "x-pad" }.joinToString(" ")

    val refused =
      assertThrows(ClientException::class.java) {
        verified(
          "t=$MOMENT,h=$covered,v0=$BODY_CODE",
          delivered(),
          Instant.ofEpochSecond(MOMENT)
        )
      }

    assertTrue(refused.message?.contains("headers accepted") == true, refused.message)
  }

  @Test
  fun aSignatureLongerThanTheBoundIsRefusedBeforeItIsSplit() {
    val oversized = "t=$MOMENT,v0=" + "a".repeat(Signature.MAX_SIGNATURE_CHARS)

    val refused =
      assertThrows(ClientException::class.java) {
        verified(oversized, delivered(), Instant.ofEpochSecond(MOMENT))
      }

    assertTrue(refused.message?.contains("characters long") == true, refused.message)
  }

  @Test
  fun verifyingAgainstTheCurrentMomentAcceptsASignatureMadeNow() {
    val now = Instant.now().epochSecond

    Webhooks.verify("t=$now,v0=${signed(now)}", PAYLOAD, delivered(), SECRET, TOLERANCE)
  }

  @Test
  fun theToleranceIsAppliedInBothDirections() {
    // The window is the width a delivery is accepted within, so its own edge is inside it, and a
    // window that only looked backwards would be one a sender widens by dating its own delivery in
    // the future.
    for (drift in listOf(-TOLERANCE.toSeconds(), TOLERANCE.toSeconds())) {
      val moment = MOMENT + drift
      verified("t=$moment,v0=${signed(moment)}", delivered(), Instant.ofEpochSecond(MOMENT))
    }
    for (drift in listOf(-TOLERANCE.toSeconds() - 1, TOLERANCE.toSeconds() + 1)) {
      val moment = MOMENT + drift
      val refused =
        assertThrows(ClientException::class.java) {
          verified("t=$moment,v0=${signed(moment)}", delivered(), Instant.ofEpochSecond(MOMENT))
        }

      assertTrue(refused.message?.contains("outside the") == true, refused.message)
    }
  }

  @Test
  fun theCodeTheCorpusCarriesIsTheOneThisClientComputes() {
    // The corpus computed its codes with a general-purpose tool, outside every implementation. This
    // holds what this client signs against one of them, so the two agreeing says the algorithm is
    // the shared one rather than whatever this client happened to do.
    assertEquals(BODY_CODE, signed(MOMENT))
  }

  companion object {
    private const val SECRET = "a-subscription-secret"
    private const val PAYLOAD = "{\"event\":\"user.created\"}"
    private const val MOMENT = 1_800_000_000L
    private const val BODY_CODE =
      "d17d66b66fca89390c5b967c45e8928fc732db07a0aabe8167b1e98213081ffe"
    private const val HEADERS_CODE =
      "19a6fb8f6581715b241a93af02a58611c3b0ac7b747a8d2a5b120ee418d0c347"
    private const val HEADER_SIGNATURE =
      "t=$MOMENT,h=x-event-id x-delivery-id,v1=$HEADERS_CODE"
    private val TOLERANCE: Duration = Duration.ofSeconds(300)

    private fun verified(signature: String, headers: List<Pair<String, String>>, now: Instant) {
      Webhooks.verifyAt(signature, PAYLOAD, headers, SECRET, TOLERANCE, now)
    }

    private fun delivered(): List<Pair<String, String>> = listOf(
      "x-event-id" to "evt-1",
      "x-delivery-id" to "dlv-1",
      "content-type" to "application/json"
    )

    /**
     * The body-scheme code for that moment, computed here rather than by the client.
     *
     * A suite that signed with the module it is testing and verified with the same module would pass
     * whatever the two agreed on. This builds the message the shared contract describes and computes
     * the code over it directly, so what the client verifies is an answer it had no say in.
     */
    private fun signed(moment: Long): String {
      val mac = Mac.getInstance("HmacSHA256")
      mac.init(SecretKeySpec(SECRET.toByteArray(StandardCharsets.UTF_8), "HmacSHA256"))
      val code = mac.doFinal("$moment.$PAYLOAD".toByteArray(StandardCharsets.UTF_8))
      return code.joinToString("") { String.format(Locale.ROOT, "%02x", it.toInt() and 0xFF) }
    }
  }
}
