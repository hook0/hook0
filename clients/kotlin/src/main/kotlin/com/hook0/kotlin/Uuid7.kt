package com.hook0.kotlin

import java.security.SecureRandom
import java.util.UUID

/**
 * The shape of identifier Hook0 mints when it is the one choosing.
 *
 * Its leading 48 bits are the current time in milliseconds, so identifiers minted in different
 * milliseconds come out ordered, which is what keeps the index they end up in from being written all
 * over. The 74 bits after the version and variant markers are random, so two identifiers minted
 * inside one millisecond are *not* ordered with respect to each other — only their millisecond
 * prefixes are.
 *
 * Written here because the JDK has none: `UUID.randomUUID()` is a version 4, whose every bit is
 * random.
 */
object Uuid7 {

  /** How many bytes of an identifier the moment it was minted takes. */
  private const val MOMENT_BYTES = 6

  /** How many bytes an identifier is, and how many of them each half of it holds. */
  private const val IDENTIFIER_BYTES = 16
  private const val HALF = 8

  /** Where the version and the variant markers sit, and what they are. */
  private const val VERSION_BYTE = 6
  private const val VARIANT_BYTE = 8

  private val RANDOM = SecureRandom()

  /**
   * Mints one out of the current moment.
   *
   * @return an identifier of version 7
   */
  fun generate(): UUID = generate(System.currentTimeMillis())

  /**
   * Mints one out of the moment given, which is what lets a suite say when it was minted.
   *
   * @param milliseconds the moment to carry, in milliseconds since the Unix epoch
   * @return an identifier of version 7 carrying that moment
   */
  fun generate(milliseconds: Long): UUID {
    val drawn = ByteArray(IDENTIFIER_BYTES)
    RANDOM.nextBytes(drawn)

    for (index in 0 until MOMENT_BYTES) {
      drawn[index] = ((milliseconds ushr (8 * (MOMENT_BYTES - 1 - index))) and 0xFF).toByte()
    }
    // Version 7 in the high nibble of the seventh byte, and the two-bit variant marker RFC 9562
    // asks for in the top of the ninth.
    drawn[VERSION_BYTE] = ((drawn[VERSION_BYTE].toInt() and 0x0F) or 0x70).toByte()
    drawn[VARIANT_BYTE] = ((drawn[VARIANT_BYTE].toInt() and 0x3F) or 0x80).toByte()

    var high = 0L
    var low = 0L
    for (index in 0 until HALF) {
      high = (high shl 8) or (drawn[index].toLong() and 0xFF)
    }
    for (index in HALF until IDENTIFIER_BYTES) {
      low = (low shl 8) or (drawn[index].toLong() and 0xFF)
    }
    return UUID(high, low)
  }
}
