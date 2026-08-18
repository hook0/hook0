package com.hook0.client;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.time.Duration;
import java.time.Instant;
import java.util.List;
import java.util.Map;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.Timeout;

/**
 * What a signature header has to be before it is one, beyond the vectors the shared corpus carries.
 *
 * <p>The corpus owns the codes: it holds every accepted and refused delivery, computed outside every implementation,
 * and {@code ConformanceTest} drives them. What is here is the reading — the shapes a header can arrive in that never
 * reach a code at all — and the two ways a caller can hand the headers over.
 */
@Timeout(60)
final class SignatureTest {

  private static final String SECRET = "a-subscription-secret";
  private static final String PAYLOAD = "{\"event\":\"user.created\"}";
  private static final long MOMENT = 1_800_000_000L;
  private static final String BODY_CODE = "d17d66b66fca89390c5b967c45e8928fc732db07a0aabe8167b1e98213081ffe";
  private static final Duration TOLERANCE = Duration.ofSeconds(300);

  private static void verified(String signature, List<Map.Entry<String, String>> headers, Instant now) {
    Webhooks.verifyAt(signature, PAYLOAD, headers, SECRET, TOLERANCE, now);
  }

  private static List<Map.Entry<String, String>> delivered() {
    return List.of(
        Map.entry("x-event-id", "evt-1"),
        Map.entry("x-delivery-id", "dlv-1"),
        Map.entry("content-type", "application/json"));
  }

  @Test
  void aSignatureCarryingNoMomentIsRefused() {
    ClientException refused =
        assertThrows(
            ClientException.class,
            () -> verified("v0=" + BODY_CODE + ",x=1", delivered(), Instant.ofEpochSecond(MOMENT)));

    assertTrue(refused.getMessage().contains("no `t` part"), refused.getMessage());
  }

  @Test
  void aSignatureCarryingNoCodeIsRefused() {
    ClientException refused =
        assertThrows(
            ClientException.class,
            () -> verified("t=" + MOMENT + ",h=x-event-id", delivered(), Instant.ofEpochSecond(MOMENT)));

    assertTrue(refused.getMessage().contains("nor a `v1` code"), refused.getMessage());
  }

  @Test
  void aMomentNoClockCouldHoldIsRefused() {
    ClientException refused =
        assertThrows(
            ClientException.class,
            () -> verified("t=99999999999999,v0=" + BODY_CODE, delivered(), Instant.ofEpochSecond(MOMENT)));

    assertTrue(refused.getMessage().contains("from the epoch"), refused.getMessage());
  }

  @Test
  void aMomentThatIsNotANumberOfSecondsIsRefusedRatherThanRead() {
    for (String written : List.of("1_0", "1.0", "0x10", " ", "+1", "9".repeat(40))) {
      assertThrows(
          ClientException.class,
          () -> verified("t=" + written + ",v0=" + BODY_CODE, delivered(), Instant.ofEpochSecond(MOMENT)),
          "`" + written + "` was read as a number of seconds");
    }
  }

  @Test
  void thePartsOfASignatureAreReadAroundTheSpacesTheyArrivedWith() {
    verified(" t = " + MOMENT + " , v0 = " + BODY_CODE + " ", delivered(), Instant.ofEpochSecond(MOMENT));
  }

  @Test
  void onlyTheFirstAssignatorOfAPartSeparatesItsNameFromItsValue() {
    // A value carrying a further one is a value, not a third part: splitting on all of them would
    // silently drop everything past the second and verify against a truncated code.
    ClientException refused =
        assertThrows(
            ClientException.class,
            () -> verified("t=" + MOMENT + ",v0=" + BODY_CODE + "=extra", delivered(), Instant.ofEpochSecond(MOMENT)));

    assertTrue(refused.getMessage().contains("not hexadecimal"), refused.getMessage());
  }

  @Test
  void headersGivenAsAMappingAreReadLikeHeadersGivenAsPairs() {
    String signature =
        "t=" + MOMENT + ",h=x-event-id x-delivery-id,"
            + "v1=19a6fb8f6581715b241a93af02a58611c3b0ac7b747a8d2a5b120ee418d0c347";

    Webhooks.verifyAt(signature, PAYLOAD, delivered(), SECRET, TOLERANCE, Instant.ofEpochSecond(MOMENT));
    Webhooks.verify(
        signature,
        PAYLOAD,
        Map.of("X-Event-Id", "evt-1", "X-Delivery-Id", "dlv-1", "Content-Type", "application/json"),
        SECRET,
        Duration.ofDays(365 * 100));
  }

  @Test
  void theDeliveredHeadersAreFoundWhateverCaseTheyArrivedIn() {
    String signature =
        "t=" + MOMENT + ",h=x-event-id x-delivery-id,"
            + "v1=19a6fb8f6581715b241a93af02a58611c3b0ac7b747a8d2a5b120ee418d0c347";

    verified(
        signature,
        List.of(Map.entry("X-EVENT-ID", "evt-1"), Map.entry("X-Delivery-Id", "dlv-1")),
        Instant.ofEpochSecond(MOMENT));
  }

  @Test
  void aBodyThatChangedAfterItWasSignedIsRefused() {
    ClientException refused =
        assertThrows(
            ClientException.class,
            () ->
                Webhooks.verifyAt(
                    "t=" + MOMENT + ",v0=" + BODY_CODE,
                    PAYLOAD + " ",
                    delivered(),
                    SECRET,
                    TOLERANCE,
                    Instant.ofEpochSecond(MOMENT)));

    assertTrue(refused.getMessage().contains("does not match"), refused.getMessage());
  }

  @Test
  void aSignatureMadeUnderAnotherSecretIsRefused() {
    ClientException refused =
        assertThrows(
            ClientException.class,
            () ->
                Webhooks.verifyAt(
                    "t=" + MOMENT + ",v0=" + BODY_CODE,
                    PAYLOAD,
                    delivered(),
                    "another-secret",
                    TOLERANCE,
                    Instant.ofEpochSecond(MOMENT)));

    assertTrue(refused.getMessage().contains("does not match"), refused.getMessage());
  }

  @Test
  void aSignatureCoveringMoreHeadersThanTheBoundIsRefused() {
    String covered = String.join(" ", java.util.Collections.nCopies(Signature.MAX_COVERED_HEADERS + 1, "x-pad"));

    ClientException refused =
        assertThrows(
            ClientException.class,
            () ->
                verified(
                    "t=" + MOMENT + ",h=" + covered + ",v0=" + BODY_CODE,
                    delivered(),
                    Instant.ofEpochSecond(MOMENT)));

    assertTrue(refused.getMessage().contains("headers accepted"), refused.getMessage());
  }

  @Test
  void aSignatureLongerThanTheBoundIsRefusedBeforeItIsSplit() {
    String oversized = "t=" + MOMENT + ",v0=" + "a".repeat(Signature.MAX_SIGNATURE_CHARS);

    ClientException refused =
        assertThrows(ClientException.class, () -> verified(oversized, delivered(), Instant.ofEpochSecond(MOMENT)));

    assertTrue(refused.getMessage().contains("characters long"), refused.getMessage());
  }

  @Test
  void verifyingAgainstTheCurrentMomentAcceptsASignatureMadeNow() {
    long now = Instant.now().getEpochSecond();
    String code = signed(now);

    Webhooks.verify("t=" + now + ",v0=" + code, PAYLOAD, delivered(), SECRET, TOLERANCE);
  }

  @Test
  void theToleranceIsAppliedInBothDirections() {
    // The window is the width a delivery is accepted within, so its own edge is inside it, and a
    // window that only looked backwards would be one a sender widens by dating its own delivery in
    // the future.
    for (long drift : List.of(-TOLERANCE.toSeconds(), TOLERANCE.toSeconds())) {
      long moment = MOMENT + drift;
      verified("t=" + moment + ",v0=" + signed(moment), delivered(), Instant.ofEpochSecond(MOMENT));
    }
    for (long drift : List.of(-TOLERANCE.toSeconds() - 1, TOLERANCE.toSeconds() + 1)) {
      long moment = MOMENT + drift;
      ClientException refused =
          assertThrows(
              ClientException.class,
              () -> verified("t=" + moment + ",v0=" + signed(moment), delivered(), Instant.ofEpochSecond(MOMENT)));

      assertTrue(refused.getMessage().contains("outside the"), refused.getMessage());
    }
  }

  @Test
  void theCodeTheCorpusCarriesIsTheOneThisClientComputes() {
    // The corpus computed its codes with a general-purpose tool, outside every implementation. This
    // holds what this client signs against one of them, so the two agreeing says the algorithm is
    // the shared one rather than whatever this client happened to do.
    assertEquals(BODY_CODE, signed(MOMENT));
  }

  @Test
  void aHeaderThatIsNothingAtAllIsRefusedRatherThanRead() {
    ClientException refused = assertThrows(ClientException.class, () -> Signature.parse(null));

    assertTrue(refused.getMessage().contains("no signature to read"), refused.getMessage());
  }

  @Test
  void aSecretNobodySetVerifiesNothing() {
    // An empty key is one the runtime's HMAC refuses outright, so a caller that passed no secret
    // hears that the delivery is refused rather than an error about key lengths — and, crucially,
    // never hears that it verified.
    String signature = "t=" + MOMENT + ",v0=" + signed(MOMENT);
    Instant now = Instant.ofEpochSecond(MOMENT);

    for (String secret : new String[] {null, ""}) {
      ClientException refused =
          assertThrows(
              ClientException.class,
              () -> Webhooks.verifyAt(signature, PAYLOAD, delivered(), secret, TOLERANCE, now));

      assertTrue(refused.getMessage().contains("no subscription secret"), refused.getMessage());
    }
  }

  @Test
  void aDeliveryCarryingNothingWhereItsPartsGoIsRefusedRatherThanVerified() {
    // Every way a caller can hand over nothing: no header list, no header map, a header carrying no
    // name or no value, and no body at all. None of them verifies, and each says so as the refusal
    // this client raises rather than as whatever the runtime would have raised first.
    String signature = "t=" + MOMENT + ",v0=" + signed(MOMENT);
    String covering = "t=" + MOMENT + ",h=x-event-id,v1=" + BODY_CODE;
    Instant now = Instant.ofEpochSecond(MOMENT);

    // A signature that covers a header, held against a delivery that carried none: the header it
    // covers was not delivered, whichever way the caller passes them over.
    ClientException uncovered =
        assertThrows(
            ClientException.class,
            () ->
                Webhooks.verifyAt(
                    covering, PAYLOAD, (List<Map.Entry<String, String>>) null, SECRET, TOLERANCE, now));
    assertTrue(uncovered.getMessage().contains("was not delivered"), uncovered.getMessage());
    assertThrows(
        ClientException.class,
        () -> Webhooks.verify(covering, PAYLOAD, (Map<String, String>) null, SECRET, TOLERANCE));

    ClientException nameless =
        assertThrows(
            ClientException.class,
            () -> {
              List<Map.Entry<String, String>> carried = new java.util.ArrayList<>();
              carried.add(null);
              verified(signature, carried, now);
            });
    assertTrue(nameless.getMessage().contains("no name or no value"), nameless.getMessage());

    // A body-scheme signature covers the body, so no body at all is not the body it was signed over.
    ClientException bodiless =
        assertThrows(
            ClientException.class,
            () -> Webhooks.verifyAt(signature, null, delivered(), SECRET, TOLERANCE, now));
    assertTrue(bodiless.getMessage().contains("does not match"), bodiless.getMessage());
  }

  /**
   * The body-scheme code for that moment, computed here rather than by the client.
   *
   * <p>A suite that signed with the module it is testing and verified with the same module would pass whatever the two
   * agreed on. This builds the message the shared contract describes and computes the code over it directly, so what
   * the client verifies is an answer it had no say in.
   */
  private static String signed(long moment) {
    try {
      javax.crypto.Mac mac = javax.crypto.Mac.getInstance("HmacSHA256");
      mac.init(
          new javax.crypto.spec.SecretKeySpec(
              SECRET.getBytes(java.nio.charset.StandardCharsets.UTF_8), "HmacSHA256"));
      byte[] code = mac.doFinal((moment + "." + PAYLOAD).getBytes(java.nio.charset.StandardCharsets.UTF_8));
      StringBuilder written = new StringBuilder(code.length * 2);
      for (byte one : code) {
        written.append(String.format(java.util.Locale.ROOT, "%02x", one & 0xFF));
      }
      return written.toString();
    } catch (java.security.GeneralSecurityException unusable) {
      throw new IllegalStateException("this runtime cannot compute HmacSHA256", unusable);
    }
  }
}
