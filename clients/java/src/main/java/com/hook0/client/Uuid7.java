package com.hook0.client;

import java.security.SecureRandom;
import java.util.UUID;

/**
 * The shape of identifier Hook0 mints when it is the one choosing.
 *
 * <p>Its leading 48 bits are the current time in milliseconds, so identifiers minted in different milliseconds come out
 * ordered, which is what keeps the index they end up in from being written all over. The 74 bits after the version and
 * variant markers are random, so two identifiers minted inside one millisecond are <em>not</em> ordered with respect to
 * each other — only their millisecond prefixes are.
 *
 * <p>Written here because the JDK has none: {@link UUID#randomUUID()} is a version 4, whose every bit is random.
 */
public final class Uuid7 {

  /** How many bytes of an identifier the moment it was minted takes. */
  private static final int MOMENT_BYTES = 6;

  private static final SecureRandom RANDOM = new SecureRandom();

  private Uuid7() {}

  /**
   * Mints one out of the current moment.
   *
   * @return an identifier of version 7
   */
  public static UUID generate() {
    return generate(System.currentTimeMillis());
  }

  /**
   * Mints one out of the moment given, which is what lets a suite say when it was minted.
   *
   * @param milliseconds the moment to carry, in milliseconds since the Unix epoch
   * @return an identifier of version 7 carrying that moment
   */
  public static UUID generate(long milliseconds) {
    byte[] drawn = new byte[16];
    RANDOM.nextBytes(drawn);

    for (int index = 0; index < MOMENT_BYTES; index++) {
      drawn[index] = (byte) ((milliseconds >>> (8 * (MOMENT_BYTES - 1 - index))) & 0xFF);
    }
    // Version 7 in the high nibble of the seventh byte, and the two-bit variant marker RFC 9562
    // asks for in the top of the ninth.
    drawn[6] = (byte) ((drawn[6] & 0x0F) | 0x70);
    drawn[8] = (byte) ((drawn[8] & 0x3F) | 0x80);

    long high = 0;
    long low = 0;
    for (int index = 0; index < 8; index++) {
      high = (high << 8) | (drawn[index] & 0xFFL);
    }
    for (int index = 8; index < 16; index++) {
      low = (low << 8) | (drawn[index] & 0xFFL);
    }
    return new UUID(high, low);
  }
}
