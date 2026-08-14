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
import com.hook0.client.generated.Problem;
import com.hook0.client.generated.ProblemException;
import com.hook0.client.generated.ProblemId;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.lang.reflect.RecordComponent;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.TreeSet;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.Timeout;

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
  void aMemberTheDocumentDoesNotRequireIsAbsentRatherThanWrittenBackAsNothing() {
    Problem problem = Problem.fromJson(aProblem());

    assertFalse(problem.toJson().containsKey("validation"), "a member the API answered none of was written back");
    assertEquals(aProblem().keySet(), problem.toJson().keySet());
    // What was written back reads as the same value, which is what says nothing was lost on the way
    // out — the numbers travel as the widths the document declares rather than as the ones a reader
    // happened to build them at.
    assertEquals(problem, Problem.fromJson(problem.toJson()));
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
    // Every record the generator wrote, discovered from what it wrote: a schema the document adds
    // joins this case the moment the generated files carry it.
    int exercised = 0;
    for (Class<?> declared : Generated.types()) {
      if (!declared.isRecord()) {
        continue;
      }
      Object read = Generated.readOrNothing(declared, anApplication());
      if (read == null) {
        continue;
      }
      Object written = Generated.written(read);

      assertEquals(read, Generated.readOrNothing(declared, written), declared.getSimpleName() + " does not read back");
      exercised++;
    }

    assertTrue(exercised > 0, "no generated value could be read out of the document this case carries");
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
