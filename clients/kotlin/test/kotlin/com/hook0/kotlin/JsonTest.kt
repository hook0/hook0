package com.hook0.kotlin

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertThrows
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.Timeout

/**
 * The reader this artefact carries instead of a dependency.
 *
 * Every ceiling is a refusal rather than a truncation, which is the whole reason a reader of our own
 * is defensible at all: one that quietly read half a document would be worse than the dependency it
 * was written to avoid.
 */
@Timeout(60)
class JsonTest {

  @Test
  fun aDocumentReadsBackAsWhatItSays() {
    val read = Json.parse("""{"a":1,"b":[true,null,"x"],"c":{"d":1.5}}""")

    assertEquals(
      linkedMapOf<String, Any?>(
        "a" to 1L,
        "b" to listOf(true, null, "x"),
        "c" to linkedMapOf<String, Any?>("d" to 1.5)
      ),
      read
    )
  }

  @Test
  fun aWholeNumberReadsAsOneAndAFractionalOneDoesNot() {
    assertEquals(1L, Json.parse("1"))
    assertEquals(1.5, Json.parse("1.5"))
    assertEquals(1.0e3, Json.parse("1e3"))
  }

  @Test
  fun theOrderMembersArrivedInIsTheOrderTheyAreWrittenBackIn() {
    val document = """{"z":1,"a":2,"m":3}"""

    assertEquals(document, Json.write(Json.parse(document)))
  }

  @Test
  fun aDocumentThatCarriesSomethingPastItsEndIsRefused() {
    val refused = assertThrows(JsonException::class.java) { Json.parse("{} {}") }

    assertTrue(refused.message?.contains("past its end") == true, refused.message)
  }

  @Test
  fun everyEscapeJsonDeclaresIsReadAndWrittenBack() {
    val read = Json.parse("\"\\\" \\\\ \\/ \\b \\f \\n \\r \\t \\u0041\"")

    assertEquals("\" \\ / \b \u000C \n \r \t A", read)
    assertEquals("\"\\\" \\\\ / \\b \\f \\n \\r \\t A\"", Json.write(read))
  }

  @Test
  fun aStringCarryingARawControlCharacterIsRefused() {
    val refused = assertThrows(JsonException::class.java) { Json.parse("\"a\u0001b\"") }

    assertTrue(refused.message?.contains("control character") == true, refused.message)
  }

  @Test
  fun aDocumentNestedDeeperThanTheBoundIsRefusedRatherThanTrimmed() {
    val deep = "[".repeat(Json.MAX_DEPTH + 2) + "]".repeat(Json.MAX_DEPTH + 2)

    val refused = assertThrows(JsonException::class.java) { Json.parse(deep) }

    assertTrue(refused.message?.contains("nests deeper") == true, refused.message)
  }

  @Test
  fun aValueJsonHasNoWayToCarryIsRefusedRatherThanPrinted() {
    val refused = assertThrows(JsonException::class.java) { Json.write(Any()) }

    assertTrue(refused.message?.contains("is not something JSON carries") == true, refused.message)
  }

  @Test
  fun aNumberJsonHasNoWayToCarryIsRefused() {
    for (number in listOf(Double.NaN, Double.POSITIVE_INFINITY, Double.NEGATIVE_INFINITY)) {
      assertThrows(JsonException::class.java) { Json.write(number) }
    }
  }

  @Test
  fun anObjectKeyedBySomethingThatIsNotAStringIsRefused() {
    val refused = assertThrows(JsonException::class.java) { Json.write(mapOf(1 to "a")) }

    assertTrue(refused.message?.contains("keyed by") == true, refused.message)
  }

  @Test
  fun aWholeNumberWrittenAsADoubleTravelsWithoutAFraction() {
    assertEquals("12", Json.write(12.0))
    assertEquals("1.5", Json.write(1.5))
  }

  @Test
  fun everyCeilingThisWriterNamesRefusesOnTheWayOutAsWellAsOnTheWayIn() {
    // Each bound is written down twice, once for reading and once for writing, and until now neither
    // half of the collection ceilings nor the writing half of any of them was held to it. A ceiling
    // enforced in one direction only is a client that will compose a document it would refuse to
    // read back.
    val members = HashMap<String, Any?>()
    val items = ArrayList<Any?>(Json.MAX_ITEMS + 1)
    val crowded = StringBuilder("{")
    val listed = StringBuilder("[")
    for (item in 0..Json.MAX_ITEMS) {
      members["m$item"] = item.toLong()
      items.add(item.toLong())
      crowded.append(if (item > 0) "," else "").append("\"m").append(item).append("\":1")
      listed.append(if (item > 0) "," else "").append('1')
    }
    var nested: Any? = 1L
    for (depth in 0..Json.MAX_DEPTH) {
      nested = listOf(nested)
    }
    val whole = (0 until 10).map { "x".repeat(900_000) }

    assertTrue(refusing { Json.write(members) }.contains("members, above"))
    assertTrue(refusing { Json.write(items) }.contains("items, above"))
    assertTrue(refusing { Json.write(nested) }.contains("nests deeper"))
    assertTrue(refusing { Json.write("x".repeat(Json.MAX_STRING_CHARS + 8)) }.contains("characters long"))
    assertTrue(refusing { Json.write(whole) }.contains("characters long, above"))

    assertTrue(refusing { Json.parse("$crowded}") }.contains("members read"))
    assertTrue(refusing { Json.parse("$listed]") }.contains("items read"))
    val longString = "[\"" + "x".repeat(Json.MAX_STRING_CHARS + 8) + "\"]"
    val longDocument = "\"" + "x".repeat(Json.MAX_DOCUMENT_CHARS) + "\""

    assertTrue(refusing { Json.parse(longString) }.contains("characters read"))
    assertTrue(refusing { Json.parse(longDocument) }.contains("characters long"))
  }

  @Test
  fun aStringThatStopsInsideAnEscapeIsRefusedRatherThanReadUpToThere() {
    // Three ways an escape can be cut short or wrong, and a literal spelled almost right. Each ends
    // the document somewhere the reader cannot know what was meant, and reading what came before as
    // the whole of it would hand a caller a value the sender never wrote.
    for (malformed in listOf("\"a\\", "\"a\\u00\"", "\"a\\uzzzz\"", "tru", "nul", "fals")) {
      assertThrows(JsonException::class.java, { Json.parse(malformed) }, "`$malformed` was read as a document")
    }
  }

  @Test
  fun aCollectionThatRunsOnWithoutASeparatorIsRefusedRatherThanReadUpToThere() {
    for (malformed in listOf("{\"a\":1 \"b\":2}", "[1 2]")) {
      assertThrows(JsonException::class.java, { Json.parse(malformed) }, "`$malformed` was read as a document")
    }
  }

  @Test
  fun everyItemOfAnArrayIsWrittenAndSeparatedFromTheOneBefore() {
    // A separator only appears between the second item and the first, so an array of one never shows
    // whether there is one at all — and everything written until now was an array of one or none.
    assertEquals("[]", Json.write(emptyList<Any?>()))
    assertEquals("[1]", Json.write(listOf(1L)))
    assertEquals("[1,2,3]", Json.write(listOf(1L, 2L, 3L)))
    assertEquals(listOf(1L, 2L, 3L), Json.parse(Json.write(listOf(1L, 2L, 3L))))
  }

  @Test
  fun aNumberWithNothingAfterThePointIsWrittenWithoutOneWhicheverWidthItArrivedAs() {
    // A single-width float is the same number to the API as a double-width one, and travels the same
    // way: as the shorter of `7` and `7.0`, which is what JSON says they both are.
    assertEquals("7", Json.write(7.0f))
    assertEquals("1.5", Json.write(1.5f))
    assertEquals("-7", Json.write(-7.0))

    // Past where a double tells whole numbers apart one from the next, it travels as the
    // floating-point value it is rather than as a whole number it only looks like.
    assertEquals(1e17, Json.parse(Json.write(1e17)))
  }

  @Test
  fun aStringCarryingAControlCharacterIsWrittenAsTheCodePointItIs() {
    // The reader refuses a raw control character in a document; the writer has to make sure it never
    // produces one, which is the other side of the same rule.
    assertEquals("\"a\\u0001b\"", Json.write("a\u0001b"))
    assertEquals("a\u0001b", Json.parse("\"a\\u0001b\""))
  }

  @Test
  fun aDocumentSpacedOutTheWayAPersonWritesOneIsRead() {
    val read = Json.parse(" \t\r\n{ \"a\" : [ 1 , 2 ] , \"b\" : { } } \t\r\n")

    assertEquals(mapOf("a" to listOf(1L, 2L), "b" to emptyMap<String, Any?>()), read)
  }

  /** What refusing that said, for a case holding several ceilings to their own words. */
  private fun refusing(written: () -> Unit): String =
    assertThrows(JsonException::class.java) { written() }.message ?: ""
}
