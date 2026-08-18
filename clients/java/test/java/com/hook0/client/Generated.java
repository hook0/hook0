package com.hook0.client;

import java.io.IOException;
import java.io.UncheckedIOException;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.lang.reflect.ParameterizedType;
import java.lang.reflect.RecordComponent;
import java.lang.reflect.Type;
import java.nio.file.Files;
import java.nio.file.Path;
import java.time.LocalDate;
import java.time.OffsetDateTime;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.TreeMap;
import java.util.TreeSet;
import java.util.UUID;
import java.util.stream.Stream;

/**
 * Everything the generator wrote, found by looking at what it wrote — and the document it wrote it from.
 *
 * <p>Nothing lists the types anywhere in this suite: a schema the document adds, a problem the API grows or an entity
 * it declares joins the cases that walk this the moment the generated files carry it. The files are what is walked
 * rather than the classpath, since reading a package's members at run time would take a dependency this artefact does
 * not have.
 *
 * <p>The API's own description is read too, so that what the generated half issues can be held against what the API
 * declares rather than against what the generator happened to write. An operation the document adds is therefore one
 * this suite starts asking for without a line of it being touched.
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

  /** The tag the API marks the operations an SDK is written for with. */
  private static final String PUBLIC_TAG = "public";

  /** The API's own description, from the directory this suite runs out of. */
  private static final Path SNAPSHOT =
      Path.of("").toAbsolutePath().getParent().getParent().resolve("api").resolve("openapi.snapshot.json");

  /** The description is committed, so one above this grew out of shape rather than being one somebody meant. */
  private static final long MAX_SNAPSHOT_BYTES = 4 * 1024 * 1024;

  /** No API declares anywhere near this many operations; the cap bounds what one case walks. */
  private static final int MAX_OPERATIONS = 512;

  /** How far a value is built into itself before this suite reads the shape as one it cannot build. */
  private static final int MAX_NESTING = 16;

  /** The API's description, read once: it is committed, so it does not change under a run. */
  private static Map<String, Operation> declared;

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

  /** The operations one group declares, in a settled order so that answers can be queued against them. */
  static List<Method> operations(Class<?> group) {
    List<Method> declared = new ArrayList<>();
    for (Method method : group.getDeclaredMethods()) {
      if (Modifier.isPublic(method.getModifiers()) && !method.isSynthetic()) {
        declared.add(method);
      }
    }
    declared.sort(Comparator.comparing(Method::getName).thenComparing(method -> method.getParameterCount()));
    return List.copyOf(declared);
  }

  /** Whether that type is one of the values the API declares rather than a group or a failure. */
  static boolean isAValue(Class<?> declared) {
    if (!declared.isRecord()) {
      return false;
    }
    try {
      declared.getMethod("fromJson", Object.class);
      declared.getMethod("toJson");
      return true;
    } catch (NoSuchMethodException missing) {
      return false;
    }
  }

  /**
   * One of that value, with every member the document names carrying something.
   *
   * <p>Built out of what the record declares rather than out of a document written here, so that a member the API adds
   * is filled in the moment the generated file carries it — and so that a member this suite cannot build is a failure
   * saying so rather than a case that quietly stops covering it.
   */
  static Object filled(Class<?> declared) {
    return filled(declared, MAX_NESTING);
  }

  /** One value of that shape, for a member or an answer declared to carry one. */
  static Object value(Type type) {
    return member(type.getTypeName(), type, MAX_NESTING);
  }

  private static Object filled(Class<?> declared, int depth) {
    if (depth <= 0) {
      throw new IllegalStateException(declared.getSimpleName() + " nests deeper than the " + MAX_NESTING + " built");
    }
    RecordComponent[] components = declared.getRecordComponents();
    Class<?>[] shape = new Class<?>[components.length];
    Object[] members = new Object[components.length];
    for (int at = 0; at < components.length; at++) {
      shape[at] = components[at].getType();
      members[at] = member(declared.getSimpleName(), components[at].getGenericType(), depth);
    }
    try {
      return declared.getDeclaredConstructor(shape).newInstance(members);
    } catch (ReflectiveOperationException unusable) {
      throw new IllegalStateException(declared.getSimpleName() + " cannot be built out of what it declares", unusable);
    }
  }

  private static Object member(String owner, Type type, int depth) {
    if (type instanceof ParameterizedType parameterized) {
      Type[] arguments = parameterized.getActualTypeArguments();
      if (parameterized.getRawType() == List.class) {
        return List.of(member(owner, arguments[0], depth - 1));
      }
      if (parameterized.getRawType() == Map.class) {
        return Map.of("a key the document leaves open", member(owner, arguments[1], depth - 1));
      }
      throw new IllegalStateException(owner + " declares a member of a shape this suite cannot build: " + type);
    }
    Class<?> declared = (Class<?>) type;
    if (declared == String.class) {
      return "what the document names";
    }
    if (declared == UUID.class) {
      return UUID.fromString("3f2504e0-4f89-41d3-9a0c-0305e82c3310");
    }
    if (declared == Integer.class) {
      return Integer.valueOf(7);
    }
    if (declared == Long.class) {
      return Long.valueOf(7);
    }
    if (declared == Double.class) {
      return Double.valueOf(1.5);
    }
    if (declared == Boolean.class) {
      return Boolean.TRUE;
    }
    if (declared == OffsetDateTime.class) {
      return OffsetDateTime.parse("2026-01-02T03:04:05Z");
    }
    if (declared == LocalDate.class) {
      return LocalDate.parse("2026-01-02");
    }
    if (declared == Object.class) {
      // A member the document leaves undescribed, which therefore travels as whatever arrived.
      return Map.of("a member the document does not describe", "what it carried");
    }
    if (declared.isEnum()) {
      return declared.getEnumConstants()[0];
    }
    if (declared.isRecord()) {
      return filled(declared, depth - 1);
    }
    throw new IllegalStateException(
        owner + " declares a member of a type this suite cannot build: " + declared.getSimpleName());
  }

  /**
   * One operation the API declares for an SDK: where it lands, and what it names in its query string.
   *
   * @param verb the method it is issued under
   * @param path where it lands, with a placeholder per member of the path it names
   * @param query every parameter it takes in its query string
   * @param required the ones of those a caller cannot leave out
   */
  record Operation(String verb, String path, TreeSet<String> query, TreeSet<String> required) {

    /** How this operation is named where one is held against a request that arrived. */
    String named() {
      return verb + " " + path;
    }
  }

  /**
   * Every operation the API declares for an SDK, by the verb and the path it declares it under.
   *
   * <p>Only the ones the document marks public: those are what the generator writes a method for, and holding the
   * generated half against the rest would be holding it against operations it was never asked to carry.
   */
  static Map<String, Operation> declaredOperations() {
    if (declared == null) {
      Map<String, Operation> found = new TreeMap<>();
      for (Map.Entry<String, Object> path : paths().entrySet()) {
        for (Map.Entry<String, Object> operation : asObject(path.getValue()).entrySet()) {
          if (!tags(operation.getValue()).contains(PUBLIC_TAG)) {
            continue;
          }
          Operation read =
              new Operation(
                  operation.getKey().toUpperCase(Locale.ROOT),
                  path.getKey(),
                  query(operation.getValue(), false),
                  query(operation.getValue(), true));
          found.put(read.named(), read);
          if (found.size() > MAX_OPERATIONS) {
            throw new IllegalStateException("the API declares more than the " + MAX_OPERATIONS + " operations walked");
          }
        }
      }
      declared = Map.copyOf(found);
    }
    return declared;
  }

  /** What one operation names in its query string, either all of it or only what a caller cannot leave out. */
  private static TreeSet<String> query(Object operation, boolean onlyRequired) {
    TreeSet<String> named = new TreeSet<>();
    Object parameters = asObject(operation).get("parameters");
    if (!(parameters instanceof List<?> declaredParameters)) {
      return named;
    }
    for (Object parameter : declaredParameters) {
      Map<String, Object> read = asObject(parameter);
      if (!"query".equals(read.get("in")) || (onlyRequired && !Boolean.TRUE.equals(read.get("required")))) {
        continue;
      }
      named.add((String) read.get("name"));
    }
    return named;
  }

  private static Map<String, Object> paths() {
    try {
      long size = Files.size(SNAPSHOT);
      if (size > MAX_SNAPSHOT_BYTES) {
        throw new IllegalStateException(SNAPSHOT + " is " + size + " bytes long, above the " + MAX_SNAPSHOT_BYTES);
      }
      return asObject(asObject(Json.parse(Files.readString(SNAPSHOT))).get("paths"));
    } catch (IOException unreadable) {
      throw new UncheckedIOException("the API description is not where this suite looks for it: " + SNAPSHOT,
          unreadable);
    }
  }

  @SuppressWarnings("unchecked")
  private static Map<String, Object> asObject(Object value) {
    return (Map<String, Object>) value;
  }

  private static List<?> tags(Object operation) {
    Object named = asObject(operation).get("tags");
    return named instanceof List<?> carried ? carried : List.of();
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
  @SuppressWarnings("unchecked")
  static Map<String, Object> written(Object value) {
    try {
      return (Map<String, Object>) value.getClass().getMethod("toJson").invoke(value);
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
