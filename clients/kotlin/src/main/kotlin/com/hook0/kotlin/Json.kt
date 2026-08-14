package com.hook0.kotlin

import java.util.Locale

/**
 * JSON, read and written with nothing but the language's own standard library.
 *
 * The JVM ships no JSON parser and Kotlin adds none, and this artefact takes no dependency to get
 * one. That is a deliberate trade rather than an oversight: every other Hook0 SDK installs with
 * nothing beside it, and a Kotlin artefact that dragged `kotlinx.serialization` onto the classpath
 * would put an application that only wanted to send an event in the position of reconciling its own
 * pinned version — and its own compiler plugin — with this one. A webhook SDK is not worth a
 * dependency conflict, and what the API speaks is a small, closed subset of JSON.
 *
 * Everything is bounded, in both directions. A document longer than [MAX_DOCUMENT_CHARS], nested
 * deeper than [MAX_DEPTH], carrying a string longer than [MAX_STRING_CHARS] or a collection larger
 * than [MAX_ITEMS] is *refused* rather than trimmed: half a document read as a whole one is worse
 * than no document at all, so the ceiling is a failure and never a truncation.
 *
 * What a document reads as: an object is a `LinkedHashMap` in the order it arrived, an array is a
 * `List`, a string is a `String`, `true` and `false` are `Boolean`, `null` is nothing, and a number
 * is a `Long` when it is whole and fits and a `Double` otherwise.
 */
class Json private constructor(private val text: String) {

  private var at: Int = 0

  private fun value(depth: Int): Any? {
    if (depth > MAX_DEPTH) {
      throw JsonException("the document nests deeper than the $MAX_DEPTH read")
    }
    if (at >= text.length) {
      throw JsonException("the document stops where a value was expected")
    }

    return when (text[at]) {
      '{' -> obj(depth)
      '[' -> array(depth)
      '"' -> string()
      't' -> literal("true", true)
      'f' -> literal("false", false)
      'n' -> literal("null", null)
      else -> number()
    }
  }

  private fun obj(depth: Int): Map<String, Any?> {
    at++
    val members = LinkedHashMap<String, Any?>()
    skipBlanks()
    if (peek() == '}') {
      at++
      return members
    }

    while (true) {
      skipBlanks()
      if (peek() != '"') {
        throw JsonException("an object names a member with something that is not a string at character $at")
      }
      val name = string()
      skipBlanks()
      if (peek() != ':') {
        throw JsonException("an object member carries no value at character $at")
      }
      at++
      skipBlanks()
      members[name] = value(depth + 1)
      if (members.size > MAX_ITEMS) {
        throw JsonException("an object carries more than the $MAX_ITEMS members read")
      }

      skipBlanks()
      val next = peek()
      at++
      if (next == '}') {
        return members
      }
      if (next != ',') {
        throw JsonException("an object is not closed at character ${at - 1}")
      }
    }
  }

  private fun array(depth: Int): List<Any?> {
    at++
    val items = ArrayList<Any?>()
    skipBlanks()
    if (peek() == ']') {
      at++
      return items
    }

    while (true) {
      skipBlanks()
      items.add(value(depth + 1))
      if (items.size > MAX_ITEMS) {
        throw JsonException("an array carries more than the $MAX_ITEMS items read")
      }

      skipBlanks()
      val next = peek()
      at++
      if (next == ']') {
        return items
      }
      if (next != ',') {
        throw JsonException("an array is not closed at character ${at - 1}")
      }
    }
  }

  private fun string(): String {
    at++
    val read = StringBuilder()
    while (true) {
      if (at >= text.length) {
        throw JsonException("a string is not closed before the document ends")
      }
      if (read.length > MAX_STRING_CHARS) {
        throw JsonException("a string is longer than the $MAX_STRING_CHARS characters read")
      }

      val character = text[at]
      at++
      if (character == '"') {
        return read.toString()
      }
      if (character == '\\') {
        read.append(escaped())
        continue
      }
      if (character.code < FIRST_PRINTABLE) {
        throw JsonException("a string carries a control character at ${at - 1}")
      }
      read.append(character)
    }
  }

  private fun escaped(): Char {
    if (at >= text.length) {
      throw JsonException("a string stops on an escape")
    }
    val marker = text[at]
    at++
    return when (marker) {
      '"' -> '"'
      '\\' -> '\\'
      '/' -> '/'
      'b' -> '\b'
      'f' -> FORM_FEED
      'n' -> '\n'
      'r' -> '\r'
      't' -> '\t'
      'u' -> codePoint()
      else -> throw JsonException("`\\$marker` is not an escape JSON declares")
    }
  }

  private fun codePoint(): Char {
    if (at + HEX_DIGITS > text.length) {
      throw JsonException("a string stops inside an escaped code point")
    }
    val digits = text.substring(at, at + HEX_DIGITS)
    for (digit in digits) {
      if (Character.digit(digit, HEXADECIMAL) < 0) {
        throw JsonException("`$digits` is not an escaped code point")
      }
    }
    at += HEX_DIGITS
    return digits.toInt(HEXADECIMAL).toChar()
  }

  private fun literal(written: String, value: Any?): Any? {
    if (!text.startsWith(written, at)) {
      throw JsonException("the document carries something that is not a value at character $at")
    }
    at += written.length
    return value
  }

  private fun number(): Any {
    val start = at
    if (peek() == '-') {
      at++
    }
    var fractional = false
    while (at < text.length) {
      val character = text[at]
      if (character in '0'..'9') {
        at++
        continue
      }
      if (character == '.' || character == 'e' || character == 'E' || character == '+' || character == '-') {
        fractional = true
        at++
        continue
      }
      break
    }

    val written = text.substring(start, at)
    if (written.isEmpty()) {
      throw JsonException("the document carries something that is not a value at character $start")
    }
    val read = if (fractional) written.toDoubleOrNull() else written.toLongOrNull()
    return read ?: throw JsonException("`$written` is not a number")
  }

  private fun peek(): Char {
    if (at >= text.length) {
      throw JsonException("the document stops where more was expected")
    }
    return text[at]
  }

  private fun skipBlanks() {
    while (at < text.length) {
      val character = text[at]
      if (character != ' ' && character != '\t' && character != '\n' && character != '\r') {
        return
      }
      at++
    }
  }

  companion object {
    /** Longest document read or written, in characters. */
    const val MAX_DOCUMENT_CHARS = 8 * 1024 * 1024

    /** Deepest an object or an array may nest, which is what stops a document of nothing but brackets. */
    const val MAX_DEPTH = 64

    /** Longest string a document may carry, in characters. */
    const val MAX_STRING_CHARS = 1024 * 1024

    /** Most members one object or items one array may carry. */
    const val MAX_ITEMS = 100_000

    /** How many digits an escaped code point is written with, and the base it is written in. */
    private const val HEX_DIGITS = 4
    private const val HEXADECIMAL = 16

    /** Lowest character a JSON string may carry without escaping it. */
    private const val FIRST_PRINTABLE = 0x20

    /** The one control character JSON names that Kotlin has no escape of its own for. */
    private const val FORM_FEED = '\u000C'

    /** Largest whole number a document carries without losing a digit to a fraction. */
    private const val LARGEST_EXACT = 1L shl 53

    /**
     * Reads a document, refusing anything it cannot read whole.
     *
     * @param document the text to read
     * @return the value it carries
     * @throws JsonException for every way a document can fail to be one
     */
    fun parse(document: String): Any? {
      if (document.length > MAX_DOCUMENT_CHARS) {
        throw JsonException(
          "the document is ${document.length} characters long, above the $MAX_DOCUMENT_CHARS read"
        )
      }

      val reading = Json(document)
      reading.skipBlanks()
      val value = reading.value(0)
      reading.skipBlanks()
      if (reading.at != document.length) {
        throw JsonException("the document carries something past its end at character ${reading.at}")
      }
      return value
    }

    /**
     * Writes a value the way the API reads one.
     *
     * @param value what to write
     * @return the document carrying it
     * @throws JsonException when the value is one JSON has no way to carry, or crosses a ceiling
     */
    fun write(value: Any?): String {
      val written = StringBuilder()
      writeInto(written, value, 0)
      if (written.length > MAX_DOCUMENT_CHARS) {
        throw JsonException(
          "the document is ${written.length} characters long, above the $MAX_DOCUMENT_CHARS written"
        )
      }
      return written.toString()
    }

    private fun writeInto(out: StringBuilder, value: Any?, depth: Int) {
      if (depth > MAX_DEPTH) {
        throw JsonException("the value nests deeper than the $MAX_DEPTH written")
      }

      when (value) {
        null -> out.append("null")
        is String -> writeText(out, value)
        is Boolean -> out.append(if (value) "true" else "false")
        is Int -> out.append(value)
        is Long -> out.append(value)
        is Double -> writeNumber(out, value)
        is Float -> writeNumber(out, value.toDouble())
        is Map<*, *> -> writeObject(out, value, depth)
        is Collection<*> -> writeArray(out, value, depth)
        else -> throw JsonException("a ${value.javaClass.name} is not something JSON carries")
      }
    }

    private fun writeNumber(out: StringBuilder, number: Double) {
      if (number.isNaN() || number.isInfinite()) {
        throw JsonException("`$number` is not a number JSON carries")
      }
      if (number == Math.floor(number) && Math.abs(number) < LARGEST_EXACT.toDouble()) {
        out.append(number.toLong())
        return
      }
      out.append(number)
    }

    private fun writeObject(out: StringBuilder, members: Map<*, *>, depth: Int) {
      if (members.size > MAX_ITEMS) {
        throw JsonException("an object carries ${members.size} members, above the $MAX_ITEMS written")
      }

      out.append('{')
      var first = true
      for ((name, held) in members) {
        if (name !is String) {
          throw JsonException("an object is keyed by something that is not a string")
        }
        if (!first) {
          out.append(',')
        }
        first = false
        writeText(out, name)
        out.append(':')
        writeInto(out, held, depth + 1)
      }
      out.append('}')
    }

    private fun writeArray(out: StringBuilder, items: Collection<*>, depth: Int) {
      if (items.size > MAX_ITEMS) {
        throw JsonException("an array carries ${items.size} items, above the $MAX_ITEMS written")
      }

      out.append('[')
      var first = true
      for (item in items) {
        if (!first) {
          out.append(',')
        }
        first = false
        writeInto(out, item, depth + 1)
      }
      out.append(']')
    }

    private fun writeText(out: StringBuilder, text: String) {
      if (text.length > MAX_STRING_CHARS) {
        throw JsonException("a string is ${text.length} characters long, above the $MAX_STRING_CHARS written")
      }

      out.append('"')
      for (character in text) {
        when (character) {
          '"' -> out.append("\\\"")

          '\\' -> out.append("\\\\")

          '\n' -> out.append("\\n")

          '\r' -> out.append("\\r")

          '\t' -> out.append("\\t")

          '\b' -> out.append("\\b")

          FORM_FEED -> out.append("\\f")

          else ->
            if (character.code < FIRST_PRINTABLE) {
              out.append(String.format(Locale.ROOT, "\\u%04x", character.code))
            } else {
              out.append(character)
            }
        }
      }
      out.append('"')
    }
  }
}
