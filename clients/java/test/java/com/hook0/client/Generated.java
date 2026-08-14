package com.hook0.client;

import java.io.IOException;
import java.io.UncheckedIOException;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.stream.Stream;

/**
 * Everything the generator wrote, found by looking at what it wrote.
 *
 * <p>Nothing lists the types anywhere in this suite: a schema the document adds, a problem the API grows or an entity
 * it declares joins the cases that walk this the moment the generated files carry it. The files are what is walked
 * rather than the classpath, since reading a package's members at run time would take a dependency this artefact does
 * not have.
 */
final class Generated {

  /** The package the generator writes into. */
  private static final String PACKAGE = "com.hook0.client.generated";

  /** Where it writes, from the directory this suite runs out of. */
  private static final Path ROOT = Path.of("src", "main", "java", "com", "hook0", "client", "generated");

  /** No generated tree is anywhere near this large; the cap turns a pathological one into a failure. */
  private static final int MAX_TYPES = 4096;

  /** Suffix an operation group carries, and the one the flavour answering a future carries. */
  private static final String GROUP_SUFFIX = "Api";
  private static final String ASYNC_GROUP_SUFFIX = "AsyncApi";

  private Generated() {}

  /** Every type the generator declared, in the order the filesystem holds them, sorted. */
  static List<Class<?>> types() {
    List<Class<?>> declared = new ArrayList<>();
    try (Stream<Path> written = Files.list(ROOT.toAbsolutePath())) {
      List<Path> files = written.sorted().toList();
      if (files.size() > MAX_TYPES) {
        throw new IllegalStateException(ROOT + " holds more than the " + MAX_TYPES + " types walked");
      }
      for (Path file : files) {
        String name = file.getFileName().toString();
        if (!name.endsWith(".java")) {
          continue;
        }
        declared.add(loaded(name.substring(0, name.length() - ".java".length())));
      }
    } catch (IOException unreadable) {
      throw new UncheckedIOException("the generated tree is not where this suite looks for it: " + ROOT, unreadable);
    }
    return List.copyOf(declared);
  }

  /** The operation groups of one flavour. */
  static List<Class<?>> groups(boolean answeringAFuture) {
    List<Class<?>> found = new ArrayList<>();
    for (Class<?> declared : types()) {
      String name = declared.getSimpleName();
      boolean waiting = name.endsWith(ASYNC_GROUP_SUFFIX);
      if (name.endsWith(GROUP_SUFFIX) && waiting == answeringAFuture) {
        found.add(declared);
      }
    }
    return List.copyOf(found);
  }

  /** The group of the other flavour covering the same entity, or {@code null} when there is none. */
  static Class<?> counterpart(Class<?> blocking) {
    String stem = blocking.getSimpleName();
    String wanted = stem.substring(0, stem.length() - GROUP_SUFFIX.length()) + ASYNC_GROUP_SUFFIX;
    for (Class<?> declared : groups(true)) {
      if (declared.getSimpleName().equals(wanted)) {
        return declared;
      }
    }
    return null;
  }

  /** One value read out of that document, or {@code null} when the document does not describe one. */
  static Object readOrNothing(Class<?> declared, Object document) {
    try {
      Method reading = declared.getMethod("fromJson", Object.class);
      return reading.invoke(null, document);
    } catch (NoSuchMethodException | IllegalAccessException unusable) {
      return null;
    } catch (InvocationTargetException raised) {
      if (raised.getCause() instanceof DecodeException) {
        return null;
      }
      throw new IllegalStateException(
          declared.getSimpleName() + " failed to read a document with something other than a decode failure",
          raised.getCause());
    }
  }

  /** That value, written back the way the API reads it. */
  static Object written(Object value) {
    try {
      return value.getClass().getMethod("toJson").invoke(value);
    } catch (NoSuchMethodException | IllegalAccessException | InvocationTargetException unusable) {
      throw new IllegalStateException(value.getClass().getSimpleName() + " cannot be written back", unusable);
    }
  }

  private static Class<?> loaded(String simple) {
    try {
      return Class.forName(PACKAGE + "." + simple);
    } catch (ClassNotFoundException missing) {
      throw new IllegalStateException(
          "the generator wrote " + simple + ".java and nothing compiled from it", missing);
    }
  }
}
