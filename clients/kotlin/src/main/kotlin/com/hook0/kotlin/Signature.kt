package com.hook0.kotlin

import java.io.ByteArrayOutputStream
import java.nio.charset.StandardCharsets
import java.security.GeneralSecurityException
import java.security.MessageDigest
import java.util.Locale
import javax.crypto.Mac
import javax.crypto.spec.SecretKeySpec

/**
 * A signature header, read into the pieces a verification needs.
 *
 * A signature names the moment it was signed and one or two message authentication codes over the
 * body. The `v1` scheme also covers a list of request headers, so a receiver can tell apart two
 * deliveries that carry the same body but not the same context; `v0` covers the body alone and is
 * what an older sender still produces. When both are offered, `v1` is the one verified: accepting
 * the weaker of two schemes on the strength of the sender offering it is how a downgrade works.
 *
 * Two things are refused before any code is computed. A header the signature says it covers but the
 * request did not carry is refused outright, because signing over an absent value would let a sender
 * drop a header and keep the signature valid. And a signature whose codes are not whole hexadecimal
 * is refused rather than decoded as far as it goes: a decoder that stops at the first bad character
 * compares a prefix, and a prefix of the right code is not the right code.
 *
 * A plain class rather than a data one: two of the four members are byte arrays, and an equality
 * derived from them would compare where they sit rather than what they hold.
 *
 * @property timestamp the moment the delivery was signed, in whole seconds since the Unix epoch
 * @property coveredHeaders the headers the stronger scheme covers, lowercased and in order
 * @property bodyCode the `v0` code, decoded, or nothing when the sender offered none
 * @property headersCode the `v1` code, decoded, or nothing when the sender offered none
 */
class Signature(
  val timestamp: Long,
  val coveredHeaders: List<String>,
  val bodyCode: ByteArray?,
  val headersCode: ByteArray?
) {

  /**
   * Whether the code this signature carries is the one the secret produces.
   *
   * The stronger scheme wins when both are offered, and the comparison is made in constant time: one
   * that gave up at the first differing byte would say, by how long it took, how much of a guess was
   * right.
   *
   * @param payload the raw body of the webhook request
   * @param coveredValues the values of the covered headers, in the order the signature names them
   * @param subscriptionSecret the signing secret of the subscription the delivery was made for
   * @return whether the delivery is the one that was signed
   */
  fun matches(payload: String, coveredValues: List<String>, subscriptionSecret: String): Boolean {
    val message = ByteArrayOutputStream()
    append(message, timestamp.toString())
    append(message, MESSAGE_SEPARATOR)

    val offered = headersCode
    if (offered != null) {
      append(message, coveredHeaders.joinToString(HEADER_NAME_SEPARATOR))
      append(message, MESSAGE_SEPARATOR)
      append(message, coveredValues.joinToString(MESSAGE_SEPARATOR))
      append(message, MESSAGE_SEPARATOR)
      append(message, payload)
      return MessageDigest.isEqual(code(subscriptionSecret, message.toByteArray()), offered)
    }

    // A signature carrying neither code is refused while it is being read, so what is left here is
    // the body-only scheme.
    append(message, payload)
    return MessageDigest.isEqual(code(subscriptionSecret, message.toByteArray()), bodyCode)
  }

  companion object {
    /**
     * Longest signature header read.
     *
     * The header is written by whoever reached the endpoint, so its size is bounded before any of it
     * is split, decoded or compared.
     */
    const val MAX_SIGNATURE_CHARS = 8 * 1024

    /** Most `key=value` parts one signature header is split into. */
    const val MAX_SIGNATURE_PARTS = 32

    /** Most header names one signature covers. */
    const val MAX_COVERED_HEADERS = 64

    /** Furthest from the epoch, in either direction, a signature's moment may sit. */
    const val MAX_TIMESTAMP = 1_000_000_000_000L

    /** Longest a moment may be written as, which bounds it before it is read as a number at all. */
    private const val MAX_TIMESTAMP_CHARS = 20

    /** How few parts a header carries before it is not a signature at all: a moment and a code. */
    private const val LEAST_PARTS = 2

    /** What separates one part of the signature header from the next. */
    private const val PART_SEPARATOR = ","

    /**
     * What separates the name of a part from its value.
     *
     * Only the first one counts: a value may hold further ones, and splitting on all of them would
     * silently drop everything past the second.
     */
    private const val PART_ASSIGNATOR = '='

    /** What separates two header names inside the `h` part, and what they are joined back with. */
    private const val HEADER_NAME_SEPARATOR = " "

    /** What separates the pieces of the message a code is computed over. */
    private const val MESSAGE_SEPARATOR = "."

    /** Part naming the moment the delivery was signed, in whole seconds since the Unix epoch. */
    private const val TIMESTAMP_PART = "t"

    /** Part carrying the code covering the body alone. */
    private const val BODY_SCHEME_PART = "v0"

    /** Part carrying the code covering the covered headers and the body. */
    private const val HEADERS_SCHEME_PART = "v1"

    /** Part listing the headers the `v1` code covers, in the order it covers them. */
    private const val COVERED_HEADERS_PART = "h"

    /** The base a code is written in, and how many of its digits one byte takes. */
    private const val HEXADECIMAL = 16
    private const val DIGITS_PER_BYTE = 2

    /** What a whole number of seconds reads as. */
    private val WHOLE_SECONDS = Regex("\\A-?[0-9]+\\z")

    /** What a code reads as: whole pairs of hexadecimal digits, and nothing else. */
    private val WHOLE_HEXADECIMAL = Regex("\\A(?:[0-9A-Fa-f][0-9A-Fa-f])+\\z")

    /** What a header name is written with, as RFC 9110 spells a token. */
    private val HEADER_NAME = Regex("\\A[A-Za-z0-9!#\$%&'*+\\-.^_`|~]+\\z")

    /** What the codes are computed with. */
    private const val DIGEST = "HmacSHA256"

    /**
     * Reads a signature header, refusing anything it cannot read whole.
     *
     * @param signature the value of the `X-Hook0-Signature` header
     * @return the pieces it names
     * @throws ClientException for every way a header can fail to be one
     */
    fun parse(signature: String): Signature {
      if (signature.length > MAX_SIGNATURE_CHARS) {
        throw ClientException.refusedDelivery(
          "the signature is ${signature.length} characters long, above the " +
            "$MAX_SIGNATURE_CHARS accepted"
        )
      }

      val read = partsOf(signature)
      if (read.size < LEAST_PARTS) {
        throw ClientException.refusedDelivery("the signature carries neither a timestamp nor a code")
      }

      val bodyCode = codeOf(read, BODY_SCHEME_PART)
      val headersCode = codeOf(read, HEADERS_SCHEME_PART)
      if (bodyCode == null && headersCode == null) {
        throw ClientException.refusedDelivery(
          "the signature carries neither a `$BODY_SCHEME_PART` nor a `$HEADERS_SCHEME_PART` code"
        )
      }

      return Signature(timestampOf(read), coveredHeadersOf(read), bodyCode, headersCode)
    }

    /** The `key=value` parts of a header, split on the first assignator of each and trimmed. */
    private fun partsOf(signature: String): Map<String, String> {
      val parts = signature.split(PART_SEPARATOR)
      if (parts.size > MAX_SIGNATURE_PARTS) {
        throw ClientException.refusedDelivery(
          "the signature carries more than the $MAX_SIGNATURE_PARTS parts accepted"
        )
      }

      val read = LinkedHashMap<String, String>()
      for (part in parts) {
        val at = part.indexOf(PART_ASSIGNATOR)
        if (at < 0) {
          continue
        }
        read[part.substring(0, at).trim()] = part.substring(at + 1).trim()
      }
      return read
    }

    /** The moment the signature names, which it is not a signature without. */
    private fun timestampOf(read: Map<String, String>): Long {
      val written =
        read[TIMESTAMP_PART]
          ?: throw ClientException.refusedDelivery(
            "the signature carries no `$TIMESTAMP_PART` part"
          )
      if (written.length > MAX_TIMESTAMP_CHARS || !WHOLE_SECONDS.matches(written)) {
        throw ClientException.refusedDelivery("`$written` is not a number of seconds")
      }

      val seconds = written.toLong()
      if (Math.abs(seconds) > MAX_TIMESTAMP) {
        throw ClientException.refusedDelivery(
          "the signature's moment is further than $MAX_TIMESTAMP seconds from the epoch"
        )
      }
      return seconds
    }

    /** One of the codes a signature offers, decoded whole or not at all. */
    private fun codeOf(read: Map<String, String>, part: String): ByteArray? {
      val written = read[part] ?: return null
      if (!WHOLE_HEXADECIMAL.matches(written)) {
        throw ClientException.refusedDelivery("the `$part` code is not hexadecimal")
      }

      val decoded = ByteArray(written.length / DIGITS_PER_BYTE)
      for (index in decoded.indices) {
        val at = index * DIGITS_PER_BYTE
        decoded[index] = written.substring(at, at + DIGITS_PER_BYTE).toInt(HEXADECIMAL).toByte()
      }
      return decoded
    }

    /** The headers the stronger scheme covers, in the order it covers them. */
    private fun coveredHeadersOf(read: Map<String, String>): List<String> {
      val written = read[COVERED_HEADERS_PART]
      if (written == null || written.isEmpty()) {
        return emptyList()
      }

      val names = written.split(HEADER_NAME_SEPARATOR)
      if (names.size > MAX_COVERED_HEADERS) {
        throw ClientException.refusedDelivery(
          "the signature covers more than the $MAX_COVERED_HEADERS headers accepted"
        )
      }

      return names.map { name ->
        if (!HEADER_NAME.matches(name)) {
          throw ClientException.refusedDelivery("`$name` is not a header name")
        }
        name.lowercase(Locale.ROOT)
      }
    }

    private fun append(message: ByteArrayOutputStream, text: String) {
      message.writeBytes(text.toByteArray(StandardCharsets.UTF_8))
    }

    private fun code(subscriptionSecret: String, message: ByteArray): ByteArray {
      val key = subscriptionSecret.toByteArray(StandardCharsets.UTF_8)
      if (key.isEmpty()) {
        // `SecretKeySpec` refuses an empty key outright, and a secret nobody set is a refusal rather
        // than a delivery this client has any way to accept.
        throw ClientException.refusedDelivery(
          "the signature does not match: no subscription secret was given"
        )
      }
      return try {
        val mac = Mac.getInstance(DIGEST)
        mac.init(SecretKeySpec(key, DIGEST))
        mac.doFinal(message)
      } catch (unusable: GeneralSecurityException) {
        throw ClientException.refusedDelivery(
          "this runtime cannot compute $DIGEST: ${unusable.message}"
        )
      }
    }
  }
}
