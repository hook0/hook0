package com.hook0.client;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.Timeout;

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
}
