package com.hook0.client;

import java.util.ArrayList;
import java.util.Collection;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/**
 * JSON, read and written with nothing but the standard library.
 *
 * <p>Java ships no JSON parser, and this artefact takes no dependency to get one. That is a deliberate trade rather
 * than an oversight: every other Hook0 SDK installs with nothing beside it, and a Java artefact that dragged Jackson or
 * Gson onto the classpath would put an application that only wanted to send an event in the position of reconciling its
 * own pinned version with this one. A webhook SDK is not worth a dependency conflict, and what the API speaks is a
 * small, closed subset of JSON.
 *
 * <p>Everything is bounded, in both directions. A document longer than {@link #MAX_DOCUMENT_CHARS}, nested deeper than
 * {@link #MAX_DEPTH}, carrying a string longer than {@link #MAX_STRING_CHARS} or a collection larger than
 * {@link #MAX_ITEMS} is <em>refused</em> rather than trimmed: half a document read as a whole one is worse than no
 * document at all, so the ceiling is a failure and never a truncation.
 *
 * <p>What a document reads as: an object is a {@link LinkedHashMap} in the order it arrived, an array is a
 * {@link List}, a string is a {@link String}, {@code true} and {@code false} are {@link Boolean}, {@code null} is
 * {@code null}, and a number is a {@link Long} when it is whole and fits and a {@link Double} otherwise.
 */
public final class Json {

  /** Longest document read or written, in characters. */
  public static final int MAX_DOCUMENT_CHARS = 8 * 1024 * 1024;

  /** Deepest an object or an array may nest, which is what stops a document of nothing but brackets. */
  public static final int MAX_DEPTH = 64;

  /** Longest string a document may carry, in characters. */
  public static final int MAX_STRING_CHARS = 1024 * 1024;

  /** Most members one object or items one array may carry. */
  public static final int MAX_ITEMS = 100_000;

  private final String text;
  private int at;

  private Json(String text) {
    this.text = text;
  }

  /**
   * Reads a document, refusing anything it cannot read whole.
   *
   * @param document the text to read
   * @return the value it carries
   * @throws JsonException for every way a document can fail to be one
   */
  public static Object parse(String document) {
    if (document == null) {
      throw new JsonException("there is no document to read");
    }
    if (document.length() > MAX_DOCUMENT_CHARS) {
      throw new JsonException(
          "the document is " + document.length() + " characters long, above the " + MAX_DOCUMENT_CHARS + " read");
    }

    Json reading = new Json(document);
    reading.skipBlanks();
    Object value = reading.value(0);
    reading.skipBlanks();
    if (reading.at != document.length()) {
      throw new JsonException("the document carries something past its end at character " + reading.at);
    }
    return value;
  }

  /**
   * Writes a value the way the API reads one.
   *
   * @param value what to write
   * @return the document carrying it
   * @throws JsonException when the value is one JSON has no way to carry, or crosses a ceiling
   */
  public static String write(Object value) {
    StringBuilder written = new StringBuilder();
    writeInto(written, value, 0);
    if (written.length() > MAX_DOCUMENT_CHARS) {
      throw new JsonException(
          "the document is " + written.length() + " characters long, above the " + MAX_DOCUMENT_CHARS + " written");
    }
    return written.toString();
  }

  private static void writeInto(StringBuilder out, Object value, int depth) {
    if (depth > MAX_DEPTH) {
      throw new JsonException("the value nests deeper than the " + MAX_DEPTH + " written");
    }

    switch (value) {
      case null -> out.append("null");
      case String text -> writeText(out, text);
      case Boolean flag -> out.append(flag.booleanValue() ? "true" : "false");
      case Integer number -> out.append(number.intValue());
      case Long number -> out.append(number.longValue());
      case Double number -> writeNumber(out, number.doubleValue());
      case Float number -> writeNumber(out, number.doubleValue());
      case Map<?, ?> members -> writeObject(out, members, depth);
      case Collection<?> items -> writeArray(out, items, depth);
      default ->
          throw new JsonException("a " + value.getClass().getName() + " is not something JSON carries");
    }
  }

  private static void writeNumber(StringBuilder out, double number) {
    if (Double.isNaN(number) || Double.isInfinite(number)) {
      throw new JsonException("`" + number + "` is not a number JSON carries");
    }
    if (number == Math.floor(number) && Math.abs(number) < (double) (1L << 53)) {
      out.append((long) number);
      return;
    }
    out.append(number);
  }

  private static void writeObject(StringBuilder out, Map<?, ?> members, int depth) {
    if (members.size() > MAX_ITEMS) {
      throw new JsonException("an object carries " + members.size() + " members, above the " + MAX_ITEMS + " written");
    }

    out.append('{');
    boolean first = true;
    for (Map.Entry<?, ?> member : members.entrySet()) {
      if (!(member.getKey() instanceof String name)) {
        throw new JsonException("an object is keyed by something that is not a string");
      }
      if (!first) {
        out.append(',');
      }
      first = false;
      writeText(out, name);
      out.append(':');
      writeInto(out, member.getValue(), depth + 1);
    }
    out.append('}');
  }

  private static void writeArray(StringBuilder out, Collection<?> items, int depth) {
    if (items.size() > MAX_ITEMS) {
      throw new JsonException("an array carries " + items.size() + " items, above the " + MAX_ITEMS + " written");
    }

    out.append('[');
    boolean first = true;
    for (Object item : items) {
      if (!first) {
        out.append(',');
      }
      first = false;
      writeInto(out, item, depth + 1);
    }
    out.append(']');
  }

  private static void writeText(StringBuilder out, String text) {
    if (text.length() > MAX_STRING_CHARS) {
      throw new JsonException(
          "a string is " + text.length() + " characters long, above the " + MAX_STRING_CHARS + " written");
    }

    out.append('"');
    for (int index = 0; index < text.length(); index++) {
      char character = text.charAt(index);
      switch (character) {
        case '"' -> out.append("\\\"");
        case '\\' -> out.append("\\\\");
        case '\n' -> out.append("\\n");
        case '\r' -> out.append("\\r");
        case '\t' -> out.append("\\t");
        case '\b' -> out.append("\\b");
        case '\f' -> out.append("\\f");
        default -> {
          if (character < 0x20) {
            out.append(String.format(java.util.Locale.ROOT, "\\u%04x", (int) character));
          } else {
            out.append(character);
          }
        }
      }
    }
    out.append('"');
  }

  private Object value(int depth) {
    if (depth > MAX_DEPTH) {
      throw new JsonException("the document nests deeper than the " + MAX_DEPTH + " read");
    }
    if (at >= text.length()) {
      throw new JsonException("the document stops where a value was expected");
    }

    return switch (text.charAt(at)) {
      case '{' -> object(depth);
      case '[' -> array(depth);
      case '"' -> string();
      case 't' -> literal("true", Boolean.TRUE);
      case 'f' -> literal("false", Boolean.FALSE);
      case 'n' -> literal("null", null);
      default -> number();
    };
  }

  private Object object(int depth) {
    at++;
    Map<String, Object> members = new LinkedHashMap<>();
    skipBlanks();
    if (peek() == '}') {
      at++;
      return members;
    }

    while (true) {
      skipBlanks();
      if (peek() != '"') {
        throw new JsonException("an object names a member with something that is not a string at character " + at);
      }
      String name = string();
      skipBlanks();
      if (peek() != ':') {
        throw new JsonException("an object member carries no value at character " + at);
      }
      at++;
      skipBlanks();
      members.put(name, value(depth + 1));
      if (members.size() > MAX_ITEMS) {
        throw new JsonException("an object carries more than the " + MAX_ITEMS + " members read");
      }

      skipBlanks();
      char next = peek();
      at++;
      if (next == '}') {
        return members;
      }
      if (next != ',') {
        throw new JsonException("an object is not closed at character " + (at - 1));
      }
    }
  }

  private Object array(int depth) {
    at++;
    List<Object> items = new ArrayList<>();
    skipBlanks();
    if (peek() == ']') {
      at++;
      return items;
    }

    while (true) {
      skipBlanks();
      items.add(value(depth + 1));
      if (items.size() > MAX_ITEMS) {
        throw new JsonException("an array carries more than the " + MAX_ITEMS + " items read");
      }

      skipBlanks();
      char next = peek();
      at++;
      if (next == ']') {
        return items;
      }
      if (next != ',') {
        throw new JsonException("an array is not closed at character " + (at - 1));
      }
    }
  }

  private String string() {
    at++;
    StringBuilder read = new StringBuilder();
    while (true) {
      if (at >= text.length()) {
        throw new JsonException("a string is not closed before the document ends");
      }
      if (read.length() > MAX_STRING_CHARS) {
        throw new JsonException("a string is longer than the " + MAX_STRING_CHARS + " characters read");
      }

      char character = text.charAt(at);
      at++;
      if (character == '"') {
        return read.toString();
      }
      if (character == '\\') {
        read.append(escaped());
        continue;
      }
      if (character < 0x20) {
        throw new JsonException("a string carries a control character at " + (at - 1));
      }
      read.append(character);
    }
  }

  private char escaped() {
    if (at >= text.length()) {
      throw new JsonException("a string stops on an escape");
    }
    char marker = text.charAt(at);
    at++;
    return switch (marker) {
      case '"' -> '"';
      case '\\' -> '\\';
      case '/' -> '/';
      case 'b' -> '\b';
      case 'f' -> '\f';
      case 'n' -> '\n';
      case 'r' -> '\r';
      case 't' -> '\t';
      case 'u' -> codePoint();
      default -> throw new JsonException("`\\" + marker + "` is not an escape JSON declares");
    };
  }

  private char codePoint() {
    if (at + 4 > text.length()) {
      throw new JsonException("a string stops inside an escaped code point");
    }
    String digits = text.substring(at, at + 4);
    for (int index = 0; index < digits.length(); index++) {
      if (Character.digit(digits.charAt(index), 16) < 0) {
        throw new JsonException("`" + digits + "` is not an escaped code point");
      }
    }
    at += 4;
    return (char) Integer.parseInt(digits, 16);
  }

  private Object literal(String written, Object value) {
    if (!text.startsWith(written, at)) {
      throw new JsonException("the document carries something that is not a value at character " + at);
    }
    at += written.length();
    return value;
  }

  private Object number() {
    int start = at;
    if (peek() == '-') {
      at++;
    }
    boolean fractional = false;
    while (at < text.length()) {
      char character = text.charAt(at);
      if (character >= '0' && character <= '9') {
        at++;
        continue;
      }
      if (character == '.' || character == 'e' || character == 'E' || character == '+' || character == '-') {
        fractional = true;
        at++;
        continue;
      }
      break;
    }

    String written = text.substring(start, at);
    if (written.isEmpty()) {
      throw new JsonException("the document carries something that is not a value at character " + start);
    }
    try {
      if (!fractional) {
        return Long.valueOf(written);
      }
      return Double.valueOf(written);
    } catch (NumberFormatException malformed) {
      throw new JsonException("`" + written + "` is not a number");
    }
  }

  private char peek() {
    if (at >= text.length()) {
      throw new JsonException("the document stops where more was expected");
    }
    return text.charAt(at);
  }

  private void skipBlanks() {
    while (at < text.length()) {
      char character = text.charAt(at);
      if (character != ' ' && character != '\t' && character != '\n' && character != '\r') {
        return;
      }
      at++;
    }
  }
}
