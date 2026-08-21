package com.hook0.client;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.Timeout;
import org.junit.jupiter.api.function.Executable;

/**
 * The reader and writer this artefact carries instead of a dependency.
 *
 * <p>What is asserted is what a bounded reader has to do: read the subset the API speaks exactly, and <em>refuse</em>
 * at every ceiling rather than trim to it. Half a document read as a whole one is worse than no document at all.
 */
@Timeout(60)
final class JsonTest {

  @Test
  void aDocumentReadsAsTheValuesItCarries() {
    Object read = Json.parse("{\"a\":1,\"b\":[true,null,\"x\"],\"c\":{\"d\":1.5}}");

    Map<String, Object> members = Wire.asFields(read, "a document");

    assertEquals(Long.valueOf(1), members.get("a"));
    assertEquals(List.of(Boolean.TRUE, "x"), List.of(((List<?>) members.get("b")).get(0),
        ((List<?>) members.get("b")).get(2)));
    assertEquals(Double.valueOf(1.5), Wire.asFields(members.get("c"), "c").get("d"));
  }

  @Test
  void aWholeNumberReadsAsOneAndAFractionalOneDoesNot() {
    assertEquals(Long.valueOf(-42), Json.parse("-42"));
    assertEquals(Double.valueOf(1e3), Json.parse("1e3"));
    assertEquals(Double.valueOf(0.5), Json.parse("0.5"));
  }

  @Test
  void everythingAReaderHasToRefuseIsRefused() {
    for (String malformed :
        List.of("", "{", "[1,", "{\"a\"}", "{\"a\":}", "tru", "\"unterminated", "1 2", "{\"a\":1}x", "\\", "'a'")) {
      assertThrows(JsonException.class, () -> Json.parse(malformed), "`" + malformed + "` was read as a document");
    }
  }

  @Test
  void aDocumentNestedDeeperThanTheCeilingIsRefusedRatherThanTrimmed() {
    String tower = "[".repeat(Json.MAX_DEPTH + 4) + "1" + "]".repeat(Json.MAX_DEPTH + 4);

    JsonException refused = assertThrows(JsonException.class, () -> Json.parse(tower));

    assertTrue(refused.getMessage().contains("nests deeper"), refused.getMessage());
  }

  @Test
  void aDocumentLongerThanTheCeilingIsRefusedRatherThanTrimmed() {
    String oversized = "\"" + "x".repeat(Json.MAX_DOCUMENT_CHARS) + "\"";

    JsonException refused = assertThrows(JsonException.class, () -> Json.parse(oversized));

    assertTrue(refused.getMessage().contains("characters long"), refused.getMessage());
  }

  @Test
  void aStringLongerThanTheCeilingIsRefusedRatherThanTrimmed() {
    // Inside the document ceiling and past the string one, so what refuses it is the bound this case
    // is about.
    String oversized = "[\"" + "x".repeat(Json.MAX_STRING_CHARS + 8) + "\"]";

    JsonException refused = assertThrows(JsonException.class, () -> Json.parse(oversized));

    assertTrue(refused.getMessage().contains("characters read"), refused.getMessage());
  }

  @Test
  void whatIsWrittenReadsBackAsWhatWasWritten() {
    Map<String, Object> value = new LinkedHashMap<>();
    value.put("text", "a \"quoted\" \\ line\nand a tab\t");
    value.put("whole", Long.valueOf(7));
    value.put("fractional", Double.valueOf(0.25));
    value.put("flag", Boolean.FALSE);
    value.put("nothing", null);
    value.put("items", List.of("a", "b"));

    assertEquals(value, Json.parse(Json.write(value)));
  }

  @Test
  void aControlCharacterTravelsAsTheCodePointItIs() {
    String carried = "a" + ((char) 1) + "b";

    assertEquals("\"a\\u0001b\"", Json.write(carried));
    assertEquals(carried, Json.parse("\"a\\u0001b\""));
  }

  @Test
  void aValueJsonCannotCarryIsRefusedRatherThanPrinted() {
    JsonException refused = assertThrows(JsonException.class, () -> Json.write(new Object()));

    assertTrue(refused.getMessage().contains("not something JSON carries"), refused.getMessage());
    assertThrows(JsonException.class, () -> Json.write(Double.valueOf(Double.NaN)));
    assertThrows(JsonException.class, () -> Json.write(Map.of(Integer.valueOf(1), "keyed by a number")));
  }

  @Test
  void everyCeilingThisWriterNamesRefusesOnTheWayOutAsWellAsOnTheWayIn() {
    // Each bound is written down twice, once for reading and once for writing, and until now only
    // the reading half was held to it. A ceiling that is only enforced in one direction is a client
    // that will happily compose a document it would refuse to read back.
    Map<String, Object> members = new HashMap<>();
    for (int member = 0; member <= Json.MAX_ITEMS; member++) {
      members.put("m" + member, Long.valueOf(member));
    }
    List<Object> items = new ArrayList<>(Json.MAX_ITEMS + 1);
    for (int item = 0; item <= Json.MAX_ITEMS; item++) {
      items.add(Long.valueOf(item));
    }
    Object nested = Long.valueOf(1);
    for (int depth = 0; depth <= Json.MAX_DEPTH; depth++) {
      nested = List.of(nested);
    }
    Object tower = nested;
    List<Object> whole = new ArrayList<>();
    for (int part = 0; part < 10; part++) {
      whole.add("x".repeat(900_000));
    }

    assertTrue(refusing(() -> Json.write(members)).contains("members, above"));
    assertTrue(refusing(() -> Json.write(items)).contains("items, above"));
    assertTrue(refusing(() -> Json.write(tower)).contains("nests deeper"));
    assertTrue(refusing(() -> Json.write("x".repeat(Json.MAX_STRING_CHARS + 8))).contains("characters long"));
    assertTrue(refusing(() -> Json.write(whole)).contains("characters long, above"));

    // And the reading half of the two collection ceilings, which the cases above hold only the
    // writer to. A document carrying more than this many is refused where it crosses, rather than
    // read up to the ceiling and handed over as though that were all the sender wrote.
    StringBuilder crowded = new StringBuilder("{");
    StringBuilder crowdedList = new StringBuilder("[");
    for (int item = 0; item <= Json.MAX_ITEMS; item++) {
      crowded.append(item > 0 ? "," : "").append("\"m").append(item).append("\":1");
      crowdedList.append(item > 0 ? "," : "").append('1');
    }

    assertTrue(refusing(() -> Json.parse(crowded + "}")).contains("members read"));
    assertTrue(refusing(() -> Json.parse(crowdedList + "]")).contains("items read"));
  }

  @Test
  void everyCharacterThisWriterEscapesIsOneTheReaderTakesBack() {
    // The escape table is the one place where writing and reading have to agree character for
    // character: a writer that escapes what the reader does not unescape produces documents only it
    // can read. Each is asserted as the text that travels, then read back as the character it was.
    Map<String, String> escaped = new LinkedHashMap<>();
    escaped.put("\"", "\\\"");
    escaped.put("\\", "\\\\");
    escaped.put("\n", "\\n");
    escaped.put("\r", "\\r");
    escaped.put("\t", "\\t");
    escaped.put("\b", "\\b");
    escaped.put("\f", "\\f");

    for (Map.Entry<String, String> pair : escaped.entrySet()) {
      assertEquals("\"" + pair.getValue() + "\"", Json.write(pair.getKey()), "wrote `" + pair.getKey() + "`");
      assertEquals(pair.getKey(), Json.parse("\"" + pair.getValue() + "\""), "read `" + pair.getValue() + "`");
    }

    // A solidus is one JSON allows escaped and this writer leaves alone, so the reader has to take
    // it either way — documents written elsewhere carry it.
    assertEquals("/", Json.parse("\"\\/\""));
    assertEquals("\"/\"", Json.write("/"));
  }

  @Test
  void aStringThatStopsInsideAnEscapeIsRefusedRatherThanReadUpToThere() {
    // Four ways an escape can be cut short or wrong. Each of them ends a string somewhere the reader
    // cannot know what it was, and reading what came before it as the whole string would hand a
    // caller a value the sender never wrote.
    for (String malformed : List.of("\"a\\", "\"a\\u00\"", "\"a\\uzzzz\"", "\"a\\q\"")) {
      assertThrows(JsonException.class, () -> Json.parse(malformed), "`" + malformed + "` was read as a string");
    }
  }

  @Test
  void anObjectThatRunsOnWithoutASeparatorIsRefusedRatherThanReadUpToThere() {
    for (String malformed : List.of("{\"a\":1 \"b\":2}", "[1 2]")) {
      assertThrows(JsonException.class, () -> Json.parse(malformed), "`" + malformed + "` was read as a document");
    }
  }

  @Test
  void aNumberWithNothingAfterThePointIsWrittenWithoutOne() {
    // What the API reads is JSON, where `7` and `7.0` are the same number, and what a caller handed
    // in was a floating-point zero-fraction rather than a whole number. It travels as the shorter of
    // the two, from either width of float, and comes back as the whole number it now is.
    assertEquals("7", Json.write(Double.valueOf(7)));
    assertEquals("7", Json.write(Float.valueOf(7)));
    assertEquals("-7", Json.write(Double.valueOf(-7)));
    assertEquals("1.5", Json.write(Float.valueOf(1.5f)));
    assertEquals(Long.valueOf(7), Json.parse(Json.write(Double.valueOf(7))));

    // Past where a double can tell whole numbers apart one from the next, it is written as the
    // floating-point value it is rather than as a whole number it only looks like.
    assertEquals(Double.valueOf(1e17), Json.parse(Json.write(Double.valueOf(1e17))));

    // Neither infinity is a number JSON carries, in either direction of the axis.
    assertThrows(JsonException.class, () -> Json.write(Double.valueOf(Double.POSITIVE_INFINITY)));
    assertThrows(JsonException.class, () -> Json.write(Double.valueOf(Double.NEGATIVE_INFINITY)));
  }

  @Test
  void aDocumentSpacedOutTheWayAPersonWritesOneIsRead() {
    // All four characters JSON counts as space, around every part of a document rather than in the
    // one place a case happened to put them.
    Object read = Json.parse(" \t\r\n{ \"a\" : [ 1 , 2 ] , \"b\" : { } } \t\r\n");

    Map<String, Object> members = Wire.asFields(read, "a document");

    assertEquals(List.of(Long.valueOf(1), Long.valueOf(2)), members.get("a"));
    assertEquals(Map.of(), members.get("b"));
  }

  @Test
  void anExponentIsReadWhicheverWayItIsSpelled() {
    // The characters a number is allowed to carry beyond its digits, each of which a reader that
    // stopped at the first one it did not know would cut a number in half at.
    assertEquals(Double.valueOf(1000), Json.parse("1E+3"));
    assertEquals(Double.valueOf(1000), Json.parse("1e3"));
    assertEquals(Double.valueOf(0.001), Json.parse("1E-3"));
    assertEquals(Double.valueOf(-1.5e-3), Json.parse("-1.5e-3"));
  }

  @Test
  void aDocumentThatIsNothingAtAllIsRefusedRatherThanReadAsEmpty() {
    JsonException refused = assertThrows(JsonException.class, () -> Json.parse(null));

    assertTrue(refused.getMessage().contains("no document"), refused.getMessage());
  }

  /** What refusing that said, for a case holding several ceilings to their own words. */
  private static String refusing(Executable written) {
    return assertThrows(JsonException.class, written).getMessage();
  }
}
