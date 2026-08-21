package com.hook0.kotlin

import java.nio.charset.StandardCharsets
import java.time.LocalDate
import java.time.OffsetDateTime
import java.time.format.DateTimeFormatter
import java.time.format.DateTimeFormatterBuilder
import java.time.format.DateTimeParseException
import java.time.temporal.ChronoField
import java.util.Locale
import java.util.UUID

/**
 * What the generated half of this artefact reads and writes values through.
 *
 * Everything here is hand-written and never regenerated. It is the one seam between what the API
 * declares — the data classes, the problems and the methods the generator writes under `generated/`
 * — and what it does not: how a JSON document becomes a value, what happens to a document that does
 * not say what it was declared to say, and how a value travels in a path or a query string.
 *
 * A reader is a function from what arrived to what it was declared to be. The scalar ones are
 * members of this object, so a generated file names one as a reference; the ones built around
 * another reader are functions answering a function, since there is one per shape.
 *
 * Absence is a property of the type here rather than of the value: [read] answers what the document
 * requires and refuses nothing where it declared something, and [maybe] is the only way a member
 * comes back as nothing at all.
 */
object Wire {

  /** Longest fragment of a body a message carries. */
  const val MAX_PREVIEW_CHARS = 256

  /** The characters a path segment carries as themselves; everything else travels percent-encoded. */
  private const val UNRESERVED =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~"

  /** How wide a fraction of a second may be written, and the widest a whole field is. */
  private const val NANOSECOND_DIGITS = 9
  private const val TWO_DIGITS = 2

  /**
   * How a moment is written back.
   *
   * Built rather than taken off the shelf: `ISO_OFFSET_DATE_TIME` drops the seconds of a moment that
   * lands on a whole minute, which is a spelling the API has no reason to be handed. Seconds are
   * always written, a fraction is written only when the moment carries one, and the offset is
   * written as the `Z` or the `+hh:mm` RFC 3339 asks for.
   */
  private val MOMENT: DateTimeFormatter =
    DateTimeFormatterBuilder()
      .append(DateTimeFormatter.ISO_LOCAL_DATE)
      .appendLiteral('T')
      .appendValue(ChronoField.HOUR_OF_DAY, TWO_DIGITS)
      .appendLiteral(':')
      .appendValue(ChronoField.MINUTE_OF_HOUR, TWO_DIGITS)
      .appendLiteral(':')
      .appendValue(ChronoField.SECOND_OF_MINUTE, TWO_DIGITS)
      .appendFraction(ChronoField.NANO_OF_SECOND, 0, NANOSECOND_DIGITS, true)
      .appendOffsetId()
      .toFormatter(Locale.ROOT)

  /**
   * The JSON document a response body carries.
   *
   * @param payload the body the API answered
   * @return the value it carries
   * @throws DecodeException when the body is not a document this client reads
   */
  fun decodePayload(payload: String): Any? = try {
    Json.parse(payload)
  } catch (unreadable: JsonException) {
    throw DecodeException("the response is not JSON: ${preview(payload)}", unreadable)
  }

  /**
   * As much of a body as a message may carry.
   *
   * @param payload what the API answered
   * @return a fragment of it, bounded
   */
  fun preview(payload: String): String =
    if (payload.length <= MAX_PREVIEW_CHARS) payload else payload.take(MAX_PREVIEW_CHARS) + "…"

  /**
   * What to say about an answer the API document does not describe.
   *
   * @param status what the API answered under
   * @param payload the body it answered
   * @return the detail of the failure
   */
  fun unreadable(status: Int, payload: String): String =
    "the API answered $status with a body this client cannot read: ${preview(payload)}"

  /**
   * What to say about a problem the API reported.
   *
   * @param status what the API answered under
   * @param problem the problem document, as it was written back
   * @return the detail of the failure
   */
  fun reported(status: Int, problem: Any?): String = "the API answered $status: $problem"

  /**
   * The members of an object the document declares, under the name it declares them with.
   *
   * @param value what arrived
   * @param owner what the document calls the object being read
   * @return its members
   * @throws DecodeException when what arrived is not an object
   */
  @Suppress("UNCHECKED_CAST")
  fun asFields(value: Any?, owner: String): Map<String, Any?> {
    if (value !is Map<*, *>) {
      throw DecodeException("$owner is not a JSON object")
    }
    return value as Map<String, Any?>
  }

  /**
   * A member the document requires, which is therefore missing when it is absent.
   *
   * @param fields the members that arrived
   * @param key the name the member travels under
   * @param reader what turns it into the value it was declared to be
   * @param T what the member carries
   * @return the value it carries
   * @throws DecodeException when the member is absent or is not what it was declared to be
   */
  fun <T : Any> read(fields: Map<String, Any?>, key: String, reader: (Any?) -> T): T {
    if (!fields.containsKey(key)) {
      throw DecodeException("`$key` is required and was not answered")
    }
    return named(key, fields[key], reader)
  }

  /**
   * A member the document does not require, absent as readily as answered as nothing.
   *
   * @param fields the members that arrived
   * @param key the name the member travels under
   * @param reader what turns it into the value it was declared to be
   * @param T what the member carries
   * @return the value it carries, or nothing when the API answered none
   * @throws DecodeException when the member is not what it was declared to be
   */
  fun <T : Any> maybe(fields: Map<String, Any?>, key: String, reader: (Any?) -> T): T? {
    val value = fields[key] ?: return null
    return named(key, value, reader)
  }

  private fun <T : Any> named(key: String, value: Any?, reader: (Any?) -> T): T = try {
    reader(value)
  } catch (unreadable: DecodeException) {
    throw DecodeException("`$key`: ${unreadable.message}", unreadable)
  }

  /**
   * A string, refusing what merely spells like one.
   *
   * @param value what arrived
   * @return the string it is
   */
  fun asText(value: Any?): String {
    if (value !is String) {
      throw DecodeException("expected a string, got ${describe(value)}")
    }
    return value
  }

  /**
   * An identifier, as the document spells one.
   *
   * @param value what arrived
   * @return the identifier it is
   */
  fun asUuid(value: Any?): UUID {
    val text = asText(value)
    val read =
      try {
        UUID.fromString(text)
      } catch (malformed: IllegalArgumentException) {
        throw DecodeException("expected a UUID, got `${preview(text)}`", malformed)
      }
    // `UUID.fromString` accepts a group written with fewer digits than it has, so what it read is
    // held against the text it read it from rather than trusted.
    if (!read.toString().equals(text, ignoreCase = true)) {
      throw DecodeException("expected a UUID, got `${preview(text)}`")
    }
    return read
  }

  /**
   * A whole number that fits in 32 bits.
   *
   * @param value what arrived
   * @return the number it is
   */
  fun asInteger(value: Any?): Int {
    val read = asLong(value)
    if (read < Int.MIN_VALUE || read > Int.MAX_VALUE) {
      throw DecodeException("expected a 32-bit whole number, got `$read`")
    }
    return read.toInt()
  }

  /**
   * A whole number. `true` is not one, here or on the wire.
   *
   * @param value what arrived
   * @return the number it is
   */
  fun asLong(value: Any?): Long = when (value) {
    is Long -> value
    is Int -> value.toLong()
    else -> throw DecodeException("expected a whole number, got ${describe(value)}")
  }

  /**
   * A number, whether the document wrote it with a fractional part or not.
   *
   * @param value what arrived
   * @return the number it is
   */
  fun asDouble(value: Any?): Double = when (value) {
    is Double -> value
    is Long -> value.toDouble()
    is Int -> value.toDouble()
    else -> throw DecodeException("expected a number, got ${describe(value)}")
  }

  /**
   * A boolean, refusing the numbers that stand in for one elsewhere.
   *
   * @param value what arrived
   * @return the boolean it is
   */
  fun asBoolean(value: Any?): Boolean {
    if (value !is Boolean) {
      throw DecodeException("expected a boolean, got ${describe(value)}")
    }
    return value
  }

  /**
   * A moment, as RFC 3339 spells one.
   *
   * @param value what arrived
   * @return the moment it is
   */
  fun asMoment(value: Any?): OffsetDateTime {
    val text = asText(value)
    return try {
      OffsetDateTime.parse(text)
    } catch (malformed: DateTimeParseException) {
      throw DecodeException("expected a date and a time, got `${preview(text)}`", malformed)
    }
  }

  /**
   * A day, as ISO 8601 spells one.
   *
   * @param value what arrived
   * @return the day it is
   */
  fun asDay(value: Any?): LocalDate {
    val text = asText(value)
    return try {
      LocalDate.parse(text)
    } catch (malformed: DateTimeParseException) {
      throw DecodeException("expected a date, got `${preview(text)}`", malformed)
    }
  }

  /**
   * A value the document does not describe, which is therefore kept as it arrived.
   *
   * Nothing is not one of those values. A member the document declares and does not mark optional
   * is one the API said it would answer, so a `null` where a value was promised stops the read here
   * rather than reaching a caller as a member that is somehow both required and absent.
   *
   * @param value what arrived
   * @return the same value
   */
  fun asJson(value: Any?): Any = value ?: throw DecodeException("expected a value, got nothing")

  /**
   * Every item of an array, each one read the same way.
   *
   * @param reader what reads one item
   * @param T what an item carries
   * @return what reads the array
   */
  fun <T : Any> asList(reader: (Any?) -> T): (Any?) -> List<T> = { value ->
    if (value !is List<*>) {
      throw DecodeException("expected an array, got ${describe(value)}")
    }
    value.map(reader)
  }

  /**
   * Every value of an object whose keys the document leaves open.
   *
   * @param reader what reads one value
   * @param T what a value carries
   * @return what reads the object
   */
  fun <T : Any> asMap(reader: (Any?) -> T): (Any?) -> Map<String, T> = { value ->
    if (value !is Map<*, *>) {
      throw DecodeException("expected an object, got ${describe(value)}")
    }
    val read = LinkedHashMap<String, T>()
    for ((name, held) in value) {
      read[asText(name)] = reader(held)
    }
    read
  }

  /**
   * An identifier, written the way the API reads one.
   *
   * @param value the identifier
   * @return the text it travels as
   */
  fun writeUuid(value: UUID): String = value.toString()

  /**
   * A moment, written the way the API reads one.
   *
   * A moment carrying no fraction of a second is written without one, and one that does keeps every
   * digit it has, so that what was read comes back out unchanged either way.
   *
   * @param value the moment
   * @return the text it travels as
   */
  fun writeMoment(value: OffsetDateTime): String = MOMENT.format(value)

  /**
   * A day, written the way the API reads one.
   *
   * @param value the day
   * @return the text it travels as
   */
  fun writeDay(value: LocalDate): String = value.toString()

  /**
   * Every item of a list, written back the same way.
   *
   * @param items what to write
   * @param writer what writes one item
   * @param T what an item carries
   * @return the list the API reads
   */
  fun <T : Any> writeList(items: List<T>, writer: (T) -> Any?): List<Any?> = items.map(writer)

  /**
   * Every value of a map, written back the same way.
   *
   * @param members what to write
   * @param writer what writes one value
   * @param T what a value carries
   * @return the object the API reads
   */
  fun <T : Any> writeMap(members: Map<String, T>, writer: (T) -> Any?): Map<String, Any?> {
    val written = LinkedHashMap<String, Any?>()
    for ((name, held) in members) {
      written[name] = writer(held)
    }
    return written
  }

  /**
   * A value as one segment of a path, with nothing left in it that could name another one.
   *
   * @param value what to write
   * @return the segment it travels as
   */
  fun pathSegment(value: Any?): String {
    val bytes = written(value).toByteArray(StandardCharsets.UTF_8)
    val segment = StringBuilder(bytes.size)
    for (one in bytes) {
      val character = (one.toInt() and 0xFF).toChar()
      if (UNRESERVED.indexOf(character) >= 0) {
        segment.append(character)
      } else {
        segment.append(String.format(Locale.ROOT, "%%%02X", one.toInt() and 0xFF))
      }
    }
    return segment.toString()
  }

  /**
   * A value as it travels in a query string, before anything is escaped.
   *
   * @param value what to write
   * @return the text it travels as
   */
  fun queryValue(value: Any?): String = written(value)

  /**
   * How a value travels in a request line, which is not always how the runtime prints it.
   *
   * @param value what to write
   * @return the text it travels as
   */
  fun written(value: Any?): String = when (value) {
    null -> ""
    is OffsetDateTime -> writeMoment(value)
    is LocalDate -> writeDay(value)
    else -> value.toString()
  }

  private fun describe(value: Any?): String = value?.javaClass?.simpleName ?: "nothing"
}
