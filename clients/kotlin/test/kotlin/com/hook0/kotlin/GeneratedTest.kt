package com.hook0.kotlin

import com.hook0.kotlin.generated.ApplicationInfo
import com.hook0.kotlin.generated.ApplicationsApi
import com.hook0.kotlin.generated.ApplicationsSuspendingApi
import com.hook0.kotlin.generated.NotFoundException
import com.hook0.kotlin.generated.Problem
import com.hook0.kotlin.generated.ProblemException
import com.hook0.kotlin.generated.ProblemId
import java.lang.reflect.Method
import java.lang.reflect.Modifier
import java.nio.charset.StandardCharsets
import java.nio.file.Files
import java.util.TreeSet
import java.util.UUID
import kotlin.coroutines.Continuation
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertFalse
import org.junit.jupiter.api.Assertions.assertInstanceOf
import org.junit.jupiter.api.Assertions.assertNotNull
import org.junit.jupiter.api.Assertions.assertNull
import org.junit.jupiter.api.Assertions.assertThrows
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.Timeout

/**
 * What the generated request layer puts on the wire, and what it does with what comes back.
 *
 * The generated half is handed a transport and nothing else, so these cases hand it the real one and
 * watch a real API answer: the path it filled in, the query it assembled, the credential it carried,
 * the type it read back, and the exception it raised when the answer was a problem.
 *
 * Both flavours are exercised, and the last cases hold them against each other by reflection: an
 * operation that exists on one surface and not on the other, or that answers a different shape,
 * fails here rather than at whichever caller reached for it first.
 */
@Timeout(180)
class GeneratedTest {

  @Test
  fun aGeneratedMethodReadsBackTheTypeTheApiDeclares() {
    FakeApi().use { api ->
      transport(api).use { transport ->
        api.willAnswer(
          FakeApi.Scripted.of(200, anApplication()),
          FakeApi.Scripted.of(200, anApplication())
        )

        val blocking = ApplicationsApi(transport).get(APPLICATION_ID)
        val suspending =
          Surface.awaiting { ApplicationsSuspendingApi(transport).get(APPLICATION_ID) }

        assertEquals(UUID.fromString(APPLICATION_ID), blocking.applicationId)
        assertEquals(100, blocking.quotas.eventsPerDayLimit)
        assertEquals("Done", blocking.onboardingSteps.event.wireValue)
        assertEquals(blocking, suspending, "the two surfaces read the same answer as two values")
      }
    }
  }

  @Test
  fun aGeneratedMethodFillsThePathAndCarriesTheCredential() {
    FakeApi().use { api ->
      transport(api).use { transport ->
        api.willAnswer(FakeApi.Scripted.of(200, anApplication()))
        ApplicationsApi(transport).get(APPLICATION_ID)

        val sent = api.received()[0]

        assertEquals("GET", sent.verb)
        // The identifier lands in the path rather than staying as the placeholder that named it.
        assertEquals("/api/v1/applications/$APPLICATION_ID", sent.target)
        assertEquals("Bearer token-xyz", sent.headers["authorization"])
      }
    }
  }

  @Test
  fun aGeneratedMethodAssemblesTheQueryTheOperationDeclares() {
    FakeApi().use { api ->
      transport(api).use { transport ->
        api.willAnswer(
          FakeApi.Scripted.of(200, emptyList<Any?>()),
          FakeApi.Scripted.of(200, emptyList<Any?>())
        )

        assertTrue(ApplicationsApi(transport).list(ORGANIZATION_ID).isEmpty())
        assertTrue(
          Surface.awaiting { ApplicationsSuspendingApi(transport).list(ORGANIZATION_ID) }.isEmpty()
        )

        assertEquals(
          "/api/v1/applications/?organization_id=$ORGANIZATION_ID",
          api.received()[0].target
        )
        assertEquals(
          api.received()[0].target,
          api.received()[1].target,
          "the two surfaces assembled different requests for the same operation"
        )
      }
    }
  }

  @Test
  fun aGeneratedMethodReadsAListOfTheTypeTheApiDeclares() {
    FakeApi().use { api ->
      transport(api).use { transport ->
        api.willAnswer(
          FakeApi.Scripted.of(
            200,
            listOf(
              mapOf(
                "application_id" to APPLICATION_ID,
                "name" to "an application",
                "organization_id" to ORGANIZATION_ID
              )
            )
          )
        )

        val listed = ApplicationsApi(transport).list(ORGANIZATION_ID)

        assertEquals(1, listed.size)
        assertEquals("an application", listed[0].name)
      }
    }
  }

  @Test
  fun aProblemTheApiReportsIsRaisedAsTheExceptionItNames() {
    FakeApi().use { api ->
      transport(api).use { transport ->
        api.willAnswer(FakeApi.Scripted.of(404, aProblem()), FakeApi.Scripted.of(404, aProblem()))

        val blocking =
          assertThrows(NotFoundException::class.java) { ApplicationsApi(transport).get(APPLICATION_ID) }

        assertEquals(404, blocking.status)
        assertEquals("This application does not exist.", blocking.problem?.detail)
        assertInstanceOf(ProblemException::class.java, blocking)

        // The same problem, through the other surface, has to be the same exception rather than a
        // suspending call that failed with something else.
        val suspending =
          assertThrows(NotFoundException::class.java) {
            Surface.awaiting { ApplicationsSuspendingApi(transport).get(APPLICATION_ID) }
          }

        assertEquals(blocking.status, suspending.status)
      }
    }
  }

  @Test
  fun aFailureThatIsNotAProblemDocumentIsStillReported() {
    FakeApi().use { api ->
      transport(api).use { transport ->
        api.willAnswer(
          FakeApi.Scripted.of(502, "a gateway wrote this, and it is not a problem document")
        )

        val reported =
          assertThrows(ProblemException::class.java) { ApplicationsApi(transport).get(APPLICATION_ID) }

        assertEquals(502, reported.status)
        assertNull(reported.problem)
      }
    }
  }

  @Test
  fun aMemberTheDocumentDoesNotRequireIsAbsentRatherThanWrittenBackAsNothing() {
    val problem = Problem.fromJson(aProblem())

    assertFalse(
      problem.toJson().containsKey("validation"),
      "a member the API answered none of was written back"
    )
    assertEquals(aProblem().keys, problem.toJson().keys)
    // What was written back reads as the same value, which is what says nothing was lost on the way
    // out — the numbers travel as the widths the document declares rather than as the ones a reader
    // happened to build them at.
    assertEquals(problem, Problem.fromJson(problem.toJson()))
  }

  @Test
  fun aDocumentThatDoesNotSayWhatItDeclaresStopsTheRead() {
    val unreadable = anApplication().toMutableMap()
    unreadable["name"] = 42L

    val refused = assertThrows(DecodeException::class.java) { ApplicationInfo.fromJson(unreadable) }

    assertTrue(refused.message?.contains("name") == true, refused.message)
  }

  @Test
  fun aValueOutsideAClosedListIsRefusedRatherThanCarried() {
    val unreadable = anApplication().toMutableMap()
    unreadable["onboarding_steps"] =
      mapOf("event" to "Maybe", "event_type" to "ToDo", "subscription" to "ToDo")

    assertThrows(DecodeException::class.java) { ApplicationInfo.fromJson(unreadable) }
  }

  @Test
  fun everyProblemTheCatalogueNamesIsAnExceptionOfItsOwn() {
    // The catalogue is closed and the hierarchy is sealed, so the two have to name the same set,
    // plus the one member that stands for a failure carrying no problem document at all: a problem
    // the API grows and the generator missed would show up here as one the base permits no exception
    // for.
    val permitted = TreeSet<String>()
    for (declared in ProblemException::class.java.permittedSubclasses) {
      permitted.add(declared.simpleName)
    }

    assertEquals(
      ProblemId.entries.size + 1,
      permitted.size,
      "the catalogue and the sealed hierarchy differ in size"
    )
    assertTrue(ProblemException::class.java.isSealed, "the problem hierarchy is not closed")

    for (named in ProblemId.entries) {
      assertEquals(named, ProblemId.fromJson(named.wireValue))
    }
  }

  @Test
  fun theTwoSurfacesDeclareTheSameOperations() {
    // What two surfaces cost: an operation added to one and forgotten on the other. Both are
    // discovered from what the generator wrote rather than listed here, so a group added to the API
    // is held to this the moment it is emitted.
    val checked = TreeSet<String>()
    for (blocking in Generated.groups(false)) {
      val suspending =
        Generated.counterpart(blocking)
          ?: throw AssertionError("${blocking.simpleName} has no counterpart that suspends")

      assertEquals(
        operationsOf(blocking),
        operationsOf(suspending),
        "${blocking.simpleName} and ${suspending.simpleName} do not declare the same operations"
      )

      for (method in suspending.declaredMethods) {
        if (Modifier.isPublic(method.modifiers) && !method.isSynthetic) {
          assertEquals(
            Continuation::class.java,
            method.parameterTypes.last(),
            "${suspending.simpleName}.${method.name} does not suspend"
          )
        }
      }
      checked.add(blocking.simpleName)
    }

    assertFalse(checked.isEmpty(), "no operation group was found, so this guard checked nothing")
  }

  @Test
  fun nothingTheGeneratorDeclaredHidesWhatEveryValueAlreadyAnswersTo() {
    // A function named `hashCode` or `toString` and taking no argument hides the one `Any` carries,
    // which the compiler refuses outright — so what is asserted is that the generator moved such a
    // name out of the way rather than that the compiler caught it, since a name it renamed is one no
    // caller ever sees the collision of.
    //
    // The emitted *source* is what is read, not the class file: a data class carries a `hashCode`
    // and a `toString` the compiler wrote for it, and those are the ones being protected rather than
    // offences against them. A line saying `override` is likewise a declaration that replaces one on
    // purpose.
    val shadowed = ReservedWordsTest.namesEveryValueAnswersTo()
    val offenders = TreeSet<String>()

    for (declared in Generated.types()) {
      val source =
        Files.readString(
          Generated.ROOT.resolve("${declared.simpleName}.kt").toAbsolutePath(),
          StandardCharsets.UTF_8
        )
      for (line in source.lines()) {
        if (line.contains("override ")) {
          continue
        }
        for (name in shadowed) {
          if (Regex("\\bfun\\s+`?${Regex.escape(name)}`?\\s*\\(").containsMatchIn(line)) {
            offenders.add("${declared.simpleName}.$name")
          }
        }
      }
    }

    assertFalse(shadowed.isEmpty(), "no shadowed name was measured, so this guard checked nothing")
    assertTrue(
      offenders.isEmpty(),
      "the generator declared functions every value already answers to: $offenders"
    )
  }

  @Test
  fun everyGeneratedValueReadsBackWhatItWrote() {
    // Every value the generator wrote, discovered from what it wrote: a schema the document adds
    // joins this case the moment the generated files carry it.
    var exercised = 0
    for (declared in Generated.types()) {
      if (!Generated.isAValue(declared)) {
        continue
      }
      val read = Generated.readOrNothing(declared, anApplication()) ?: continue
      val written = Generated.written(read)

      assertEquals(
        read,
        Generated.readOrNothing(declared, written),
        "${declared.simpleName} does not read back"
      )
      exercised++
    }

    assertTrue(exercised > 0, "no generated value could be read out of the document this case carries")
  }

  companion object {
    private const val APPLICATION_ID = "3f2504e0-4f89-41d3-9a0c-0305e82c3301"
    private const val ORGANIZATION_ID = "3f2504e0-4f89-41d3-9a0c-0305e82c3302"

    private fun transport(api: FakeApi): HttpTransport = HttpTransport(
      api.baseUrl(),
      "token-xyz",
      Options.defaults().copy(retryPolicy = RetryPolicy.disabled())
    )

    private fun anApplication(): Map<String, Any?> = linkedMapOf(
      "application_id" to APPLICATION_ID,
      "name" to "an application",
      "organization_id" to ORGANIZATION_ID,
      "consumption" to mapOf("events_per_day" to 12L),
      "onboarding_steps" to
        mapOf("event" to "Done", "event_type" to "ToDo", "subscription" to "ToDo"),
      "quotas" to
        mapOf("days_of_events_retention_limit" to 7L, "events_per_day_limit" to 100L)
    )

    private fun aProblem(): Map<String, Any?> = linkedMapOf(
      "id" to "NotFound",
      "title" to "Not found",
      "detail" to "This application does not exist.",
      "status" to 404L,
      "type" to "https://documentation.hook0.com/problems"
    )

    private fun operationsOf(group: Class<*>): TreeSet<String> {
      val named = TreeSet<String>()
      for (method in group.declaredMethods) {
        if (!Modifier.isPublic(method.modifiers) || method.isSynthetic) {
          continue
        }
        named.add("${method.name}(${parametersOf(method).joinToString(", ")})")
      }
      return named
    }

    /**
     * What a method takes, less the continuation a suspending one carries.
     *
     * A suspending function compiles to one taking one more argument than it declares, so comparing
     * the two surfaces means comparing what the *source* declares rather than what the bytecode
     * carries.
     */
    private fun parametersOf(method: Method): List<String> = method.parameterTypes
      .filter { it != Continuation::class.java }
      .map { it.simpleName }
  }
}
