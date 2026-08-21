package com.hook0.client;

import java.io.IOException;
import java.io.UncheckedIOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;

/**
 * The shared contract every SDK is held to, read from where it is committed.
 *
 * <p>It sits at {@code clients/conformance}, is hand-authored, and is read by the suite of every target. Nothing in
 * this suite writes down a verdict, a bound or a signature of its own: they are read out of the committed documents and
 * this client is driven against them over a real socket. A case added to the corpus is therefore exercised here
 * without a line of Java being touched, and a verdict changed there fails here until this client agrees with it again.
 */
final class Corpus {

  /**
   * Largest document of the corpus read back.
   *
   * <p>The corpus is committed, so one above this is one that grew out of shape rather than one somebody meant.
   */
  private static final long MAX_CORPUS_BYTES = 512 * 1024;

  private Corpus() {}

  /** Where the shared contract sits, from the directory this suite runs out of. */
  private static Path directory() {
    return Path.of("").toAbsolutePath().getParent().resolve("conformance");
  }

  /** One document of the shared contract, as the text it was committed as. */
  static String text(String name) {
    Path path = directory().resolve(name);
    try {
      long size = Files.size(path);
      if (size > MAX_CORPUS_BYTES) {
        throw new IllegalStateException(path + " is " + size + " bytes long, above the " + MAX_CORPUS_BYTES + " read");
      }
      return Files.readString(path, StandardCharsets.UTF_8);
    } catch (IOException unreadable) {
      throw new UncheckedIOException("the shared contract is not where this suite looks for it: " + path, unreadable);
    }
  }

  /** One document of the shared contract, bounded before it is read. */
  @SuppressWarnings("unchecked")
  static Map<String, Object> document(String name) {
    return (Map<String, Object>) Json.parse(text(name));
  }

  /** One list of entries out of a document of the shared contract. */
  @SuppressWarnings("unchecked")
  static List<Map<String, Object>> entries(Map<String, Object> document, String key) {
    return (List<Map<String, Object>>) document.get(key);
  }

  /** One list of plain values out of a document of the shared contract. */
  @SuppressWarnings("unchecked")
  static List<Object> values(Map<String, Object> document, String key) {
    return (List<Object>) document.get(key);
  }

  /**
   * The counter-examples worth keeping, committed beside the properties they broke.
   *
   * <p>One JSON value per line, so that a header carrying a comma, a newline or nothing at all is read back exactly as
   * it was written down.
   */
  static List<Object> regressions(String name) {
    Path path = Path.of("test", "resources", "regressions", name + ".jsonl").toAbsolutePath();
    List<Object> read = new ArrayList<>();
    try {
      for (String line : Files.readAllLines(path, StandardCharsets.UTF_8)) {
        if (!line.isBlank()) {
          read.add(Json.parse(line));
        }
      }
    } catch (IOException unreadable) {
      throw new UncheckedIOException("the committed counter-examples are not where this suite looks: " + path,
          unreadable);
    }
    return read;
  }
}
