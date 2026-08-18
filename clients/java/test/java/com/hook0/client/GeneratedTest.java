package com.hook0.client;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertInstanceOf;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.hook0.client.generated.ApplicationInfo;
import com.hook0.client.generated.ApplicationsApi;
import com.hook0.client.generated.ApplicationsAsyncApi;
import com.hook0.client.generated.NotFoundException;
import com.hook0.client.generated.ProblemException;
import com.hook0.client.generated.ProblemId;
import com.hook0.client.generated.Problems;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.lang.reflect.ParameterizedType;
import java.lang.reflect.RecordComponent;
import java.lang.reflect.Type;
import java.net.URLDecoder;
import java.nio.charset.StandardCharsets;
import java.time.LocalDate;
import java.time.OffsetDateTime;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.TreeSet;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.Timeout;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.ValueSource;

/**
 * What the generated request layer puts on the wire, and what it does with what comes back.
 *
 * <p>The generated half is handed a transport and nothing else, so these cases hand it the real one and watch a real
 * API answer: the path it filled in, the query it assembled, the credential it carried, the type it read back, and the
 * exception it raised when the answer was a problem.
 *
 * <p>Both flavours are exercised, and the last cases hold them against each other by reflection: an operation that
 * exists on one surface and not on the other, or that answers a different shape, fails here rather than at whichever
 * caller reached for it first.
 */
@Timeout(120)
final class GeneratedTest {

  private static final String APPLICATION_ID = "3f2504e0-4f89-41d3-9a0c-0305e82c3301";
  private static final String ORGANIZATION_ID = "3f2504e0-4f89-41d3-9a0c-0305e82c3302";

  private static HttpTransport transport(FakeApi api) {
    return new HttpTransport(api.baseUrl(), "token-xyz", Options.defaults().withRetryPolicy(RetryPolicy.disabled()));
  }

  private static Map<String, Object> anApplication() {
    Map<String, Object> application = new LinkedHashMap<>();
    application.put("application_id", APPLICATION_ID);
    application.put("name", "an application");
    application.put("organization_id", ORGANIZATION_ID);
    application.put("consumption", Map.of("events_per_day", Long.valueOf(12)));
    application.put(
        "onboarding_steps", Map.of("event", "Done", "event_type", "ToDo", "subscription", "ToDo"));
    application.put(
        "quotas",
        Map.of("days_of_events_retention_limit", Long.valueOf(7), "events_per_day_limit", Long.valueOf(100)));
    return application;
  }

  private static Map<String, Object> aProblem() {
    Map<String, Object> problem = new LinkedHashMap<>();
    problem.put("id", "NotFound");
    problem.put("title", "Not found");
    problem.put("detail", "This application does not exist.");
    problem.put("status", Long.valueOf(404));
    problem.put("type", "https://documentation.hook0.com/problems");
    return problem;
  }

  @Test
  void aGeneratedMethodReadsBackTheTypeTheApiDeclares() {
    try (FakeApi api = new FakeApi(); HttpTransport transport = transport(api)) {
      api.willAnswer(FakeApi.Scripted.of(200, anApplication()), FakeApi.Scripted.of(200, anApplication()));

      ApplicationInfo blocking = new ApplicationsApi(transport).get(APPLICATION_ID);
      ApplicationInfo waiting =
          new ApplicationsAsyncApi(transport).get(APPLICATION_ID).join();

      assertEquals(UUID.fromString(APPLICATION_ID), blocking.applicationId());
      assertEquals(Integer.valueOf(100), blocking.quotas().eventsPerDayLimit());
      assertEquals("Done", blocking.onboardingSteps().event().wireValue());
      assertEquals(blocking, waiting, "the two surfaces read the same answer as two different values");
    }
  }

  @Test
  void aGeneratedMethodFillsThePathAndCarriesTheCredential() {
    try (FakeApi api = new FakeApi(); HttpTransport transport = transport(api)) {
      api.willAnswer(FakeApi.Scripted.of(200, anApplication()));
      new ApplicationsApi(transport).get(APPLICATION_ID);

      FakeApi.Received sent = api.received().get(0);

      assertEquals("GET", sent.verb());
      // The identifier lands in the path rather than staying as the placeholder that named it.
      assertEquals("/api/v1/applications/" + APPLICATION_ID, sent.target());
      assertEquals("Bearer token-xyz", sent.headers().get("authorization"));
    }
  }

  @Test
  void aGeneratedMethodAssemblesTheQueryTheOperationDeclares() {
    try (FakeApi api = new FakeApi(); HttpTransport transport = transport(api)) {
      api.willAnswer(FakeApi.Scripted.of(200, List.of()), FakeApi.Scripted.of(200, List.of()));

      assertTrue(new ApplicationsApi(transport).list(ORGANIZATION_ID).isEmpty());
      assertTrue(new ApplicationsAsyncApi(transport).list(ORGANIZATION_ID).join().isEmpty());

      assertEquals("/api/v1/applications/?organization_id=" + ORGANIZATION_ID, api.received().get(0).target());
      assertEquals(
          api.received().get(0).target(),
          api.received().get(1).target(),
          "the two surfaces assembled different requests for the same operation");
    }
  }

  @Test
  void aGeneratedMethodReadsAListOfTheTypeTheApiDeclares() {
    try (FakeApi api = new FakeApi(); HttpTransport transport = transport(api)) {
      api.willAnswer(
          FakeApi.Scripted.of(
              200,
              List.of(
                  Map.of(
                      "application_id", APPLICATION_ID,
                      "name", "an application",
                      "organization_id", ORGANIZATION_ID))));

      var listed = new ApplicationsApi(transport).list(ORGANIZATION_ID);

      assertEquals(1, listed.size());
      assertEquals("an application", listed.get(0).name());
    }
  }

  @Test
  void aProblemTheApiReportsIsRaisedAsTheExceptionItNames() {
    try (FakeApi api = new FakeApi(); HttpTransport transport = transport(api)) {
      api.willAnswer(FakeApi.Scripted.of(404, aProblem()), FakeApi.Scripted.of(404, aProblem()));

      NotFoundException blocking =
          assertThrows(
              NotFoundException.class,
              () -> new ApplicationsApi(transport).get(APPLICATION_ID));

      assertEquals(404, blocking.status());
      assertEquals("This application does not exist.", blocking.problem().detail());
      assertInstanceOf(ProblemException.class, blocking);

      // The same problem, through the other surface, has to be the same exception rather than a
      // future that failed with something else.
      NotFoundException waiting =
          assertThrows(
              NotFoundException.class,
              () ->
                  Surface.joined(
                      () -> new ApplicationsAsyncApi(transport).get(APPLICATION_ID).join()));

      assertEquals(blocking.status(), waiting.status());
    }
  }

  @Test
  void aFailureThatIsNotAProblemDocumentIsStillReported() {
    try (FakeApi api = new FakeApi(); HttpTransport transport = transport(api)) {
      api.willAnswer(FakeApi.Scripted.of(502, "a gateway wrote this, and it is not a problem document"));

      ProblemException reported =
          assertThrows(
              ProblemException.class,
              () -> new ApplicationsApi(transport).get(APPLICATION_ID));

      assertEquals(502, reported.status());
      assertNull(reported.problem());
    }
  }

  @Test
  void everyMemberTheDocumentDoesNotRequireIsAbsentRatherThanWrittenBackAsNothing() {
    // Every value, and every member of it, one member at a time: the document arrives without it and
    // what happens next has to be one of two things. Either the member is required and the read stops,
    // or it is not and the value reads — in which case writing it back leaves the member out rather
    // than answering it as nothing, which is a difference the API can see.
    int required = 0;
    int optional = 0;

    for (Class<?> declared : Generated.types()) {
      if (!Generated.isAValue(declared)) {
        continue;
      }
      Map<String, Object> whole = Generated.written(Generated.filled(declared));
      for (String name : whole.keySet()) {
        Map<String, Object> without = new LinkedHashMap<>(whole);
        without.remove(name);

        Object read = Generated.readOrNothing(declared, without);
        if (read == null) {
          required++;
          continue;
        }

        Map<String, Object> back = Generated.written(read);
        assertFalse(
            back.containsKey(name),
            declared.getSimpleName() + "." + name + " was answered none of and written back as nothing");
        assertEquals(read, Generated.readOrNothing(declared, back), declared.getSimpleName() + " does not read back");
        optional++;
      }
    }

    assertTrue(required > 0, "no member the document requires was found, so this guard checked nothing");
    assertTrue(optional > 0, "no member the document leaves out was found, so this guard checked nothing");
  }

  @Test
  void aDocumentThatDoesNotSayWhatItDeclaresStopsTheRead() {
    Map<String, Object> unreadable = anApplication();
    unreadable.put("name", Long.valueOf(42));

    DecodeException refused = assertThrows(DecodeException.class, () -> ApplicationInfo.fromJson(unreadable));

    assertTrue(refused.getMessage().contains("name"), refused.getMessage());
  }

  @Test
  void aValueOutsideAClosedListIsRefusedRatherThanCarried() {
    Map<String, Object> unreadable = anApplication();
    unreadable.put("onboarding_steps", Map.of("event", "Maybe", "event_type", "ToDo", "subscription", "ToDo"));

    assertThrows(DecodeException.class, () -> ApplicationInfo.fromJson(unreadable));
  }

  @Test
  void aValueTravelsInARequestLineTheWayTheApiReadsOne() {
    // What a generated operation hands the transport for a path segment or a query value. The
    // spellings are the API's, not the platform's: a moment is RFC 3339 and a day is ISO 8601,
    // whichever way the runtime would have printed either on its own.
    assertEquals("", Wire.written(null));
    assertEquals("what the caller passed", Wire.written("what the caller passed"));
    assertEquals("true", Wire.written(Boolean.TRUE));
    assertEquals("7", Wire.written(Integer.valueOf(7)));
    assertEquals("2026-01-02", Wire.written(LocalDate.parse("2026-01-02")));
    assertEquals("2026-01-02T03:04:05Z", Wire.written(OffsetDateTime.parse("2026-01-02T03:04:05Z")));
    assertEquals("2026-01-02T03:04:05Z", Wire.queryValue(OffsetDateTime.parse("2026-01-02T03:04:05Z")));

    // Every character a segment carries that could name another one travels encoded.
    assertEquals("an%20application%2F..%2F..", Wire.pathSegment("an application/../.."));
    assertEquals("what-the.caller_passed~", Wire.pathSegment("what-the.caller_passed~"));
  }

  @Test
  void everyClosedListCarriesEveryStringTheApiAnswersAndNothingElse() {
    // The constants are the wire values themselves, so a name moved out of the way of the language
    // never reaches the wire. Every list the generator wrote is walked and every value of it read
    // back from its own text, which is what says the whole list travels rather than the one value a
    // case happened to name.
    int exercised = 0;
    for (Class<?> declared : Generated.types()) {
      if (!declared.isEnum()) {
        continue;
      }
      for (Object named : declared.getEnumConstants()) {
        String carried = wireValue(named);

        assertEquals(
            named, read(declared, carried), declared.getSimpleName() + " does not read back `" + carried + "`");
        exercised++;
      }

      assertThrows(
          DecodeException.class,
          () -> read(declared, "a value the API never declared"),
          declared.getSimpleName() + " carried a value the API declares none of");
    }

    assertTrue(exercised > 0, "no closed list was found, so this guard checked nothing");
  }

  /** The text one value of a closed list travels as. */
  private static String wireValue(Object named) {
    try {
      return (String) named.getClass().getMethod("wireValue").invoke(named);
    } catch (ReflectiveOperationException unusable) {
      throw new IllegalStateException(named + " does not say what it travels as", unusable);
    }
  }

  /** The value of a closed list that text names. */
  private static Object read(Class<?> declared, String carried) {
    try {
      return declared.getMethod("fromJson", Object.class).invoke(null, carried);
    } catch (IllegalAccessException | NoSuchMethodException unusable) {
      throw new IllegalStateException(declared.getSimpleName() + " reads nothing back", unusable);
    } catch (InvocationTargetException raised) {
      if (raised.getCause() instanceof RuntimeException reported) {
        throw reported;
      }
      throw new IllegalStateException(declared.getSimpleName() + " refused with something no caller could catch",
          raised);
    }
  }

  @Test
  void everyProblemTheCatalogueNamesIsRaisedAsItsOwnFailure() {
    // Declaring one exception per problem is half of it; the other half is the dispatch, which is
    // where a problem wired to the exception beside it in the catalogue would sit unnoticed until a
    // caller caught the wrong type in production. The catalogue is walked rather than listed, so a
    // problem the API grows is put through this the moment the generator writes it.
    for (ProblemId named : ProblemId.values()) {
      Map<String, Object> answered = new LinkedHashMap<>();
      answered.put("id", named.wireValue());
      answered.put("status", Long.valueOf(400));
      answered.put("title", "refused");
      answered.put("detail", "what this case scripted");
      answered.put("type", "https://hook0.com/documentation/errors/" + named.wireValue());

      ProblemException raised =
          assertThrows(ProblemException.class, () -> Problems.raiseForStatus(400, Json.write(answered)));

      assertEquals(
          named.wireValue() + "Exception",
          raised.getClass().getSimpleName(),
          "`" + named.wireValue() + "` is not raised as the failure named after it");
      assertEquals(400, raised.status());
      assertEquals(named, raised.problem().id());
      assertTrue(raised.getMessage().contains("what this case scripted"), raised.getMessage());
    }
  }

  @Test
  void everyProblemTheCatalogueNamesIsAnExceptionOfItsOwn() {
    // The catalogue is closed and the hierarchy is sealed, so the two have to name the same set: a
    // problem the API grows and the generator missed would show up here as one the base permits no
    // exception for.
    TreeSet<String> permitted = new TreeSet<>();
    for (Class<?> declared : ProblemException.class.getPermittedSubclasses()) {
      permitted.add(declared.getSimpleName());
    }

    assertEquals(ProblemId.values().length, permitted.size(), "the catalogue and the sealed hierarchy differ in size");
    assertTrue(ProblemException.class.isSealed(), "the problem hierarchy is not closed");

    for (ProblemId named : ProblemId.values()) {
      assertNotNull(named.wireValue());
      assertEquals(named, ProblemId.fromJson(named.wireValue()));
    }
  }

  @Test
  void theTwoSurfacesDeclareTheSameOperations() {
    // What two surfaces cost: an operation added to one and forgotten on the other. Both are
    // discovered from what the generator wrote rather than listed here, so a group added to the API
    // is held to this the moment it is emitted.
    TreeSet<String> checked = new TreeSet<>();
    for (Class<?> blocking : Generated.groups(false)) {
      Class<?> waiting = Generated.counterpart(blocking);

      assertNotNull(waiting, blocking.getSimpleName() + " has no counterpart answering a future");
      assertEquals(
          operationsOf(blocking),
          operationsOf(waiting),
          blocking.getSimpleName() + " and " + waiting.getSimpleName() + " do not declare the same operations");

      for (Method method : waiting.getDeclaredMethods()) {
        if (Modifier.isPublic(method.getModifiers()) && !method.isSynthetic()) {
          assertEquals(
              CompletableFuture.class,
              method.getReturnType(),
              waiting.getSimpleName() + "." + method.getName() + " does not answer a future");
        }
      }
      checked.add(blocking.getSimpleName());
    }

    assertFalse(checked.isEmpty(), "no operation group was found, so this guard checked nothing");
  }

  @Test
  void nothingTheGeneratorDeclaredIsSpelledAfterWhatEveryObjectAnswersTo() {
    // A record component declares a field and the accessor that reads it, so one spelled after a
    // no-argument method of `Object` would replace it — which the language refuses outright. What is
    // asserted is that the generator moved such a name out of the way rather than that javac caught
    // it, since a name it renamed is one no caller ever sees the collision of.
    TreeSet<String> shadowed = new TreeSet<>(ReservedWordsTest.objectMethodsTakingNoArgument());
    TreeSet<String> offenders = new TreeSet<>();

    for (Class<?> declared : Generated.types()) {
      if (!declared.isRecord()) {
        continue;
      }
      for (RecordComponent component : declared.getRecordComponents()) {
        if (shadowed.contains(component.getName())) {
          offenders.add(declared.getSimpleName() + "." + component.getName());
        }
      }
    }

    assertTrue(offenders.isEmpty(), "the generator declared members every object already answers to: " + offenders);
  }

  @Test
  void everyGeneratedValueReadsBackWhatItWrote() {
    // Every value the generator wrote, discovered from what it wrote and built out of what it
    // declares: a schema the document adds joins this case the moment the generated files carry it,
    // and one whose reader and writer disagree about a member fails here rather than at the first
    // answer that carries it.
    int exercised = 0;
    for (Class<?> declared : Generated.types()) {
      if (!Generated.isAValue(declared)) {
        continue;
      }
      Object filled = Generated.filled(declared);

      assertEquals(
          filled,
          Generated.readOrNothing(declared, Generated.written(filled)),
          declared.getSimpleName() + " does not read back what it wrote");
      exercised++;
    }

    assertTrue(exercised > 0, "no value the generator wrote could be built, so this guard checked nothing");
  }

  @ParameterizedTest
  @ValueSource(booleans = {false, true})
  void everyOperationTheApiDeclaresIsIssuedByTheMethodWrittenForIt(boolean answeringAFuture) {
    // What an SDK is for: the API declares an operation, and the generated half issues exactly that
    // request for it and reads back exactly what the answer carried. Both sides are discovered — the
    // operations out of the API's own description, the methods out of what the generator wrote — so
    // an operation the document adds and the generator misses fails here rather than at whichever
    // caller reached for it first.
    Map<String, Generated.Operation> declared = Generated.declaredOperations();
    TreeSet<String> issued = new TreeSet<>();

    for (Class<?> group : Generated.groups(answeringAFuture)) {
      for (Method operation : Generated.operations(group)) {
        String named = group.getSimpleName() + "." + operation.getName();
        Object[] arguments = argumentsOf(operation);
        Object answer = answering(answers(operation, answeringAFuture));

        try (FakeApi api = new FakeApi(); HttpTransport transport = transport(api)) {
          api.willAnswer(FakeApi.Scripted.of(200, scripted(answer)), FakeApi.Scripted.of(200, scripted(answer)));
          Object driven = driving(group, transport);

          assertEquals(
              answer,
              answeredBy(operation, driven, arguments, answeringAFuture),
              named + " answered something other than what the API carried");

          FakeApi.Received sent = api.received().get(0);
          Generated.Operation matched = matching(declared, sent.verb(), sent.target().split("\\?", 2)[0]);
          assertNotNull(matched, named + " issued `" + sent.target() + "`, which the API declares no operation at");
          issued.add(matched.named());

          for (Object argument : arguments) {
            if (argument instanceof String carried) {
              assertTrue(sent.target().contains(carried), named + " dropped `" + carried + "` from " + sent.target());
            } else {
              assertEquals(
                  Json.parse(Json.write(Generated.written(argument))),
                  sent.json(),
                  named + " sent something other than the value it was handed");
            }
          }
          assertEquals(matched.query(), assembled(sent), named + " assembled a query the API declares nothing of");

          if (matched.query().equals(matched.required())) {
            continue;
          }

          // The same operation again, with everything the API does not ask for left out: a parameter
          // a caller sends none of has to be absent from the query rather than sent as an empty one,
          // which is a difference the API can see.
          answeredBy(operation, driven, withoutTheOptional(arguments, sent, matched), answeringAFuture);

          assertEquals(
              matched.required(),
              assembled(api.received().get(1)),
              named + " sent a parameter the caller left out");
        }
      }
    }

    assertEquals(declared.keySet(), issued, "the API and the generated half do not name the same operations");
  }

  /** The parameters one request carried in its query string, by name. */
  private static TreeSet<String> assembled(FakeApi.Received sent) {
    TreeSet<String> named = new TreeSet<>();
    String[] target = sent.target().split("\\?", 2);
    if (target.length < 2 || target[1].isEmpty()) {
      return named;
    }
    for (String parameter : target[1].split("&")) {
      named.add(URLDecoder.decode(parameter.split("=", 2)[0], StandardCharsets.UTF_8));
    }
    return named;
  }

  /** The same arguments, less the ones that landed under a parameter the API does not ask for. */
  private static Object[] withoutTheOptional(
      Object[] arguments, FakeApi.Received sent, Generated.Operation matched) {
    Object[] left = arguments.clone();
    for (int at = 0; at < left.length; at++) {
      if (!(left[at] instanceof String carried)) {
        continue;
      }
      for (String parameter : matched.query()) {
        if (!matched.required().contains(parameter) && sent.target().contains(parameter + "=" + carried)) {
          left[at] = null;
        }
      }
    }
    return left;
  }

  /** The group, built on the transport this case hands it. */
  private static Object driving(Class<?> group, Transport transport) {
    try {
      return group.getDeclaredConstructor(Transport.class).newInstance(transport);
    } catch (ReflectiveOperationException unusable) {
      throw new IllegalStateException(group.getSimpleName() + " is not built on a transport", unusable);
    }
  }

  /** What one operation is called with: one distinct value per name it takes, and one value per body. */
  private static Object[] argumentsOf(Method operation) {
    Class<?>[] shape = operation.getParameterTypes();
    Object[] arguments = new Object[shape.length];
    for (int at = 0; at < shape.length; at++) {
      arguments[at] = shape[at] == String.class ? "what-this-case-passed-" + at : Generated.filled(shape[at]);
    }
    return arguments;
  }

  /** What one operation of that flavour is declared to answer. */
  private static Type answers(Method operation, boolean answeringAFuture) {
    Type answered = operation.getGenericReturnType();
    return answeringAFuture ? ((ParameterizedType) answered).getActualTypeArguments()[0] : answered;
  }

  /** A value of that shape for the API to answer, or {@code null} where the operation answers nothing. */
  private static Object answering(Type answered) {
    if (answered == void.class || answered == Void.class) {
      return null;
    }
    if (answered instanceof ParameterizedType listed) {
      return List.of(Generated.value(listed.getActualTypeArguments()[0]));
    }
    return Generated.filled((Class<?>) answered);
  }

  /** That value as the document the API writes it as. */
  private static Object scripted(Object answered) {
    if (answered == null) {
      return null;
    }
    if (answered instanceof List<?> items) {
      List<Object> written = new ArrayList<>(items.size());
      for (Object item : items) {
        written.add(Generated.isAValue(item.getClass()) ? Generated.written(item) : item);
      }
      return written;
    }
    return Generated.written(answered);
  }

  /** What one operation answered, whichever flavour it was called through. */
  private static Object answeredBy(Method operation, Object group, Object[] arguments, boolean answeringAFuture) {
    Object outcome;
    try {
      outcome = operation.invoke(group, arguments);
    } catch (IllegalAccessException unusable) {
      throw new IllegalStateException(operation.getName() + " cannot be called", unusable);
    } catch (InvocationTargetException raised) {
      if (raised.getCause() instanceof RuntimeException reported) {
        throw reported;
      }
      throw new IllegalStateException(operation.getName() + " failed with something no caller could catch", raised);
    }
    return answeringAFuture ? Surface.joined(() -> ((CompletableFuture<?>) outcome).join()) : outcome;
  }

  /** The one operation the API declares under that verb and that path, or {@code null} when it declares none. */
  private static Generated.Operation matching(
      Map<String, Generated.Operation> declared, String verb, String path) {
    Generated.Operation found = null;
    for (Generated.Operation operation : declared.values()) {
      if (!operation.verb().equals(verb) || !fills(operation.path(), path)) {
        continue;
      }
      assertNull(found, "`" + verb + " " + path + "` reads as both `" + found + "` and `" + operation.named() + "`");
      found = operation;
    }
    return found;
  }

  /** Whether that path is that template with every placeholder of it filled in. */
  private static boolean fills(String template, String path) {
    String[] declared = template.split("/", -1);
    String[] issued = path.split("/", -1);
    if (declared.length != issued.length) {
      return false;
    }
    for (int at = 0; at < declared.length; at++) {
      boolean placeholder = declared[at].startsWith("{") && declared[at].endsWith("}");
      if (placeholder ? issued[at].isEmpty() : !declared[at].equals(issued[at])) {
        return false;
      }
    }
    return true;
  }

  private static TreeSet<String> operationsOf(Class<?> group) {
    TreeSet<String> named = new TreeSet<>();
    for (Method method : group.getDeclaredMethods()) {
      if (Modifier.isPublic(method.getModifiers()) && !method.isSynthetic()) {
        List<String> parameters = new ArrayList<>();
        for (Class<?> parameter : method.getParameterTypes()) {
          parameters.add(parameter.getSimpleName());
        }
        named.add(method.getName() + "(" + String.join(", ", parameters) + ")");
      }
    }
    return named;
  }
}
