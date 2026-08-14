package com.hook0.kotlin

import java.nio.charset.StandardCharsets
import java.nio.file.Files
import java.nio.file.Path

/**
 * The shared contract every SDK is held to, read from where it is committed.
 *
 * It sits at `clients/conformance`, is hand-authored, and is read by the suite of every target.
 * Nothing in this suite writes down a verdict, a bound or a signature of its own: they are read out
 * of the committed documents and this client is driven against them over a real socket. A case added
 * to the corpus is therefore exercised here without a line of Kotlin being touched, and a verdict
 * changed there fails here until this client agrees with it again.
 */
object Corpus {

  /**
   * Largest document of the corpus read back.
   *
   * The corpus is committed, so one above this is one that grew out of shape rather than one
   * somebody meant.
   */
  private const val MAX_CORPUS_BYTES = 512L * 1024

  /** Where the shared contract sits, from the directory this suite runs out of. */
  private fun directory(): Path = Path.of("").toAbsolutePath().parent.resolve("conformance")

  /** One document of the shared contract, as the text it was committed as. */
  fun text(name: String): String {
    val path = directory().resolve(name)
    val size = Files.size(path)
    check(size <= MAX_CORPUS_BYTES) {
      "$path is $size bytes long, above the $MAX_CORPUS_BYTES read"
    }
    return Files.readString(path, StandardCharsets.UTF_8)
  }

  /** One document of the shared contract, bounded before it is read. */
  @Suppress("UNCHECKED_CAST")
  fun document(name: String): Map<String, Any?> = Json.parse(text(name)) as Map<String, Any?>

  /** One list of entries out of a document of the shared contract. */
  @Suppress("UNCHECKED_CAST")
  fun entries(document: Map<String, Any?>, key: String): List<Map<String, Any?>> =
    document[key] as List<Map<String, Any?>>

  /** One list of plain values out of a document of the shared contract. */
  @Suppress("UNCHECKED_CAST")
  fun values(document: Map<String, Any?>, key: String): List<Any?> = document[key] as List<Any?>

  /**
   * The counter-examples worth keeping, committed beside the properties they broke.
   *
   * One JSON value per line, so that a header carrying a comma, a newline or nothing at all is read
   * back exactly as it was written down.
   */
  fun regressions(name: String): List<Any?> {
    val path = Path.of("test", "resources", "regressions", "$name.jsonl").toAbsolutePath()
    return Files.readAllLines(path, StandardCharsets.UTF_8)
      .filter { it.isNotBlank() }
      .map(Json::parse)
  }
}
