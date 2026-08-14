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
}
