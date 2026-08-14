package com.hook0.client;

import java.nio.charset.StandardCharsets;
import java.time.LocalDate;
import java.time.OffsetDateTime;
import java.time.format.DateTimeFormatter;
import java.time.format.DateTimeFormatterBuilder;
import java.time.format.DateTimeParseException;
import java.time.temporal.ChronoField;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.UUID;
import java.util.function.Function;

/**
 * What the generated half of this artefact reads and writes values through.
 *
 * <p>Everything here is hand-written and never regenerated. It is the one seam between what the API declares — the
 * records, the problems and the methods the generator writes under {@code generated/} — and what it does not: how a
 * JSON document becomes a value, what happens to a document that does not say what it was declared to say, and how a
 * value travels in a path or a query string.
 *
 * <p>A reader is a {@link Function} from what arrived to what it was declared to be. The scalar ones are methods, so a
 * generated file names one as a method reference; the ones built around another reader are methods answering a
 * function, since there is one per shape.
 */
public final class Wire {

  /** Longest fragment of a body a message carries. */
  public static final int MAX_PREVIEW_CHARS = 256;

  /** The characters a path segment carries as themselves; everything else travels percent-encoded. */
  private static final String UNRESERVED = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";

  /**
   * How a moment is written back.
   *
   * <p>Built rather than taken off the shelf: {@code ISO_OFFSET_DATE_TIME} drops the seconds of a moment that lands on
   * a whole minute, which is a spelling the API has no reason to be handed. Seconds are always written, a fraction is
   * written only when the moment carries one, and the offset is written as the {@code Z} or the {@code +hh:mm} RFC 3339
   * asks for.
   */
  private static final DateTimeFormatter MOMENT =
      new DateTimeFormatterBuilder()
          .append(DateTimeFormatter.ISO_LOCAL_DATE)
          .appendLiteral('T')
          .appendValue(ChronoField.HOUR_OF_DAY, 2)
          .appendLiteral(':')
          .appendValue(ChronoField.MINUTE_OF_HOUR, 2)
          .appendLiteral(':')
          .appendValue(ChronoField.SECOND_OF_MINUTE, 2)
          .appendFraction(ChronoField.NANO_OF_SECOND, 0, 9, true)
          .appendOffsetId()
          .toFormatter(Locale.ROOT);

  private Wire() {}

  /**
   * The JSON document a response body carries.
   *
   * @param payload the body the API answered
   * @return the value it carries
   * @throws DecodeException when the body is not a document this client reads
   */
  public static Object decodePayload(String payload) {
    try {
      return Json.parse(payload == null ? "" : payload);
    } catch (JsonException unreadable) {
      throw new DecodeException("the response is not JSON: " + preview(payload), unreadable);
    }
  }

  /**
   * As much of a body as a message may carry.
   *
   * @param payload what the API answered
   * @return a fragment of it, bounded
   */
  public static String preview(String payload) {
    if (payload == null) {
      return "";
    }
    if (payload.length() <= MAX_PREVIEW_CHARS) {
      return payload;
    }
    return payload.substring(0, MAX_PREVIEW_CHARS) + "…";
  }

  /**
   * What to say about an answer the API document does not describe.
   *
   * @param status what the API answered under
   * @param payload the body it answered
   * @return the detail of the failure
   */
  public static String unreadable(int status, String payload) {
    return "the API answered " + status + " with a body this client cannot read: " + preview(payload);
  }

  /**
   * What to say about a problem the API reported.
   *
   * @param status what the API answered under
   * @param problem the problem document, as it was written back
   * @return the detail of the failure
   */
  public static String reported(int status, Object problem) {
    return "the API answered " + status + ": " + problem;
  }

  /**
   * The members of an object the document declares, under the name it declares them with.
   *
   * @param value what arrived
   * @param owner what the document calls the object being read
   * @return its members
   * @throws DecodeException when what arrived is not an object
   */
  @SuppressWarnings("unchecked")
  public static Map<String, Object> asFields(Object value, String owner) {
    if (!(value instanceof Map<?, ?> members)) {
      throw new DecodeException(owner + " is not a JSON object");
    }
    return (Map<String, Object>) members;
  }

  /**
   * A member the document requires, which is therefore missing when it is absent.
   *
   * @param fields the members that arrived
   * @param key the name the member travels under
   * @param reader what turns it into the value it was declared to be
   * @param <T> what the member carries
   * @return the value it carries
   * @throws DecodeException when the member is absent or is not what it was declared to be
   */
  public static <T> T read(Map<String, Object> fields, String key, Function<Object, T> reader) {
    if (!fields.containsKey(key)) {
      throw new DecodeException("`" + key + "` is required and was not answered");
    }
    return named(key, fields.get(key), reader);
  }

  /**
   * A member the document does not require, absent as readily as answered as null.
   *
   * @param fields the members that arrived
   * @param key the name the member travels under
   * @param reader what turns it into the value it was declared to be
   * @param <T> what the member carries
   * @return the value it carries, or {@code null} when the API answered none
   * @throws DecodeException when the member is not what it was declared to be
   */
  public static <T> T maybe(Map<String, Object> fields, String key, Function<Object, T> reader) {
    Object value = fields.get(key);
    if (value == null) {
      return null;
    }
    return named(key, value, reader);
  }

  private static <T> T named(String key, Object value, Function<Object, T> reader) {
    try {
      return reader.apply(value);
    } catch (DecodeException unreadable) {
      throw new DecodeException("`" + key + "`: " + unreadable.getMessage(), unreadable);
    }
  }

  /**
   * A string, refusing what merely spells like one.
   *
   * @param value what arrived
   * @return the string it is
   */
  public static String asText(Object value) {
    if (!(value instanceof String text)) {
      throw new DecodeException("expected a string, got " + describe(value));
    }
    return text;
  }

  /**
   * An identifier, as the document spells one.
   *
   * @param value what arrived
   * @return the identifier it is
   */
  public static UUID asUuid(Object value) {
    String text = asText(value);
    try {
      UUID read = UUID.fromString(text);
      // `UUID.fromString` accepts a group written with fewer digits than it has, so what it read is
      // held against the text it read it from rather than trusted.
      if (!read.toString().equalsIgnoreCase(text)) {
        throw new DecodeException("expected a UUID, got `" + preview(text) + "`");
      }
      return read;
    } catch (IllegalArgumentException malformed) {
      throw new DecodeException("expected a UUID, got `" + preview(text) + "`", malformed);
    }
  }

  /**
   * A whole number that fits in 32 bits.
   *
   * @param value what arrived
   * @return the number it is
   */
  public static Integer asInteger(Object value) {
    long read = asLong(value).longValue();
    if (read < Integer.MIN_VALUE || read > Integer.MAX_VALUE) {
      throw new DecodeException("expected a 32-bit whole number, got `" + read + "`");
    }
    return Integer.valueOf((int) read);
  }

  /**
   * A whole number. {@code true} is not one, here or on the wire.
   *
   * @param value what arrived
   * @return the number it is
   */
  public static Long asLong(Object value) {
    if (value instanceof Long number) {
      return number;
    }
    if (value instanceof Integer number) {
      return Long.valueOf(number.longValue());
    }
    throw new DecodeException("expected a whole number, got " + describe(value));
  }

  /**
   * A number, whether the document wrote it with a fractional part or not.
   *
   * @param value what arrived
   * @return the number it is
   */
  public static Double asDouble(Object value) {
    if (value instanceof Double number) {
      return number;
    }
    if (value instanceof Long number) {
      return Double.valueOf(number.doubleValue());
    }
    if (value instanceof Integer number) {
      return Double.valueOf(number.doubleValue());
    }
    throw new DecodeException("expected a number, got " + describe(value));
  }

  /**
   * A boolean, refusing the numbers that stand in for one elsewhere.
   *
   * @param value what arrived
   * @return the boolean it is
   */
  public static Boolean asBoolean(Object value) {
    if (!(value instanceof Boolean flag)) {
      throw new DecodeException("expected a boolean, got " + describe(value));
    }
    return flag;
  }

  /**
   * A moment, as RFC 3339 spells one.
   *
   * @param value what arrived
   * @return the moment it is
   */
  public static OffsetDateTime asMoment(Object value) {
    String text = asText(value);
    try {
      return OffsetDateTime.parse(text);
    } catch (DateTimeParseException malformed) {
      throw new DecodeException("expected a date and a time, got `" + preview(text) + "`", malformed);
    }
  }

  /**
   * A day, as ISO 8601 spells one.
   *
   * @param value what arrived
   * @return the day it is
   */
  public static LocalDate asDay(Object value) {
    String text = asText(value);
    try {
      return LocalDate.parse(text);
    } catch (DateTimeParseException malformed) {
      throw new DecodeException("expected a date, got `" + preview(text) + "`", malformed);
    }
  }

  /**
   * A value the document does not describe, which is therefore kept as it arrived.
   *
   * @param value what arrived
   * @return the same value
   */
  public static Object asJson(Object value) {
    return value;
  }

  /**
   * Every item of an array, each one read the same way.
   *
   * @param reader what reads one item
   * @param <T> what an item carries
   * @return what reads the array
   */
  public static <T> Function<Object, List<T>> asList(Function<Object, T> reader) {
    return value -> {
      if (!(value instanceof List<?> items)) {
        throw new DecodeException("expected an array, got " + describe(value));
      }
      List<T> read = new ArrayList<>(items.size());
      for (Object item : items) {
        read.add(reader.apply(item));
      }
      return List.copyOf(read);
    };
  }

  /**
   * Every value of an object whose keys the document leaves open.
   *
   * @param reader what reads one value
   * @param <T> what a value carries
   * @return what reads the object
   */
  public static <T> Function<Object, Map<String, T>> asMap(Function<Object, T> reader) {
    return value -> {
      if (!(value instanceof Map<?, ?> members)) {
        throw new DecodeException("expected an object, got " + describe(value));
      }
      Map<String, T> read = new LinkedHashMap<>();
      for (Map.Entry<?, ?> member : members.entrySet()) {
        read.put(asText(member.getKey()), reader.apply(member.getValue()));
      }
      return Map.copyOf(read);
    };
  }

  /**
   * An identifier, written the way the API reads one.
   *
   * @param value the identifier
   * @return the text it travels as
   */
  public static String writeUuid(UUID value) {
    return value == null ? null : value.toString();
  }

  /**
   * A moment, written the way the API reads one.
   *
   * <p>A moment carrying no fraction of a second is written without one, and one that does keeps every digit it has, so
   * that what was read comes back out unchanged either way.
   *
   * @param value the moment
   * @return the text it travels as
   */
  public static String writeMoment(OffsetDateTime value) {
    return value == null ? null : MOMENT.format(value);
  }

  /**
   * A day, written the way the API reads one.
   *
   * @param value the day
   * @return the text it travels as
   */
  public static String writeDay(LocalDate value) {
    return value == null ? null : value.toString();
  }

  /**
   * Every item of a list, written back the same way.
   *
   * @param items what to write
   * @param writer what writes one item
   * @param <T> what an item carries
   * @return the list the API reads
   */
  public static <T> List<Object> writeList(List<T> items, Function<T, Object> writer) {
    if (items == null) {
      return null;
    }
    List<Object> written = new ArrayList<>(items.size());
    for (T item : items) {
      written.add(writer.apply(item));
    }
    return written;
  }

  /**
   * Every value of a map, written back the same way.
   *
   * @param members what to write
   * @param writer what writes one value
   * @param <T> what a value carries
   * @return the object the API reads
   */
  public static <T> Map<String, Object> writeMap(Map<String, T> members, Function<T, Object> writer) {
    if (members == null) {
      return null;
    }
    Map<String, Object> written = new LinkedHashMap<>();
    for (Map.Entry<String, T> member : members.entrySet()) {
      written.put(member.getKey(), writer.apply(member.getValue()));
    }
    return written;
  }

  /**
   * A value as one segment of a path, with nothing left in it that could name another one.
   *
   * @param value what to write
   * @return the segment it travels as
   */
  public static String pathSegment(Object value) {
    byte[] bytes = written(value).getBytes(StandardCharsets.UTF_8);
    StringBuilder segment = new StringBuilder(bytes.length);
    for (byte one : bytes) {
      char character = (char) (one & 0xFF);
      if (UNRESERVED.indexOf(character) >= 0) {
        segment.append(character);
      } else {
        segment.append(String.format(Locale.ROOT, "%%%02X", one & 0xFF));
      }
    }
    return segment.toString();
  }

  /**
   * A value as it travels in a query string, before anything is escaped.
   *
   * @param value what to write
   * @return the text it travels as
   */
  public static String queryValue(Object value) {
    return written(value);
  }

  /**
   * How a value travels in a request line, which is not always how Java prints it.
   *
   * @param value what to write
   * @return the text it travels as
   */
  public static String written(Object value) {
    return switch (value) {
      case null -> "";
      case OffsetDateTime moment -> writeMoment(moment);
      case LocalDate day -> writeDay(day);
      default -> value.toString();
    };
  }

  private static String describe(Object value) {
    return value == null ? "nothing" : value.getClass().getSimpleName();
  }
}
