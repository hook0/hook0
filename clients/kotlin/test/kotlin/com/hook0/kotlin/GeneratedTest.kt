package com.hook0.kotlin

import com.hook0.kotlin.generated.ApplicationInfo
import com.hook0.kotlin.generated.ApplicationsApi
import com.hook0.kotlin.generated.ApplicationsSuspendingApi
import com.hook0.kotlin.generated.NotFoundException
import com.hook0.kotlin.generated.ProblemException
import com.hook0.kotlin.generated.ProblemId
import com.hook0.kotlin.generated.Problems
import java.lang.reflect.InvocationTargetException
import java.lang.reflect.Method
import java.lang.reflect.Modifier
import java.lang.reflect.ParameterizedType
import java.lang.reflect.Type
import java.net.URLDecoder
import java.nio.charset.StandardCharsets
import java.nio.file.Files
import java.time.LocalDate
import java.time.OffsetDateTime
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
import org.junit.jupiter.params.ParameterizedTest
import org.junit.jupiter.params.provider.ValueSource

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
  fun everyMemberTheDocumentDoesNotRequireIsAbsentRatherThanWrittenBackAsNothing() {
    // Every value, and every member of it, one member at a time: the document arrives without it and
    // what happens next has to be one of two things. Either the member is required and the read
    // stops, or it is not and the value reads — in which case writing it back leaves the member out
    // rather than answering it as nothing, which is a difference the API can see.
    var required = 0
    var optional = 0

    for (declared in Generated.types()) {
      if (!Generated.isAValue(declared)) {
        continue
      }
      val whole = Generated.written(Generated.filled(declared))
      for (name in whole.keys) {
        val read = Generated.readOrNothing(declared, whole - name)
        if (read == null) {
          required++
          continue
        }

        val back = Generated.written(read)
        assertFalse(
          back.containsKey(name),
          "${declared.simpleName}.$name was answered none of and written back as nothing"
        )
        assertEquals(read, Generated.readOrNothing(declared, back), "${declared.simpleName} does not read back")
        optional++
      }
    }

    assertTrue(required > 0, "no member the document requires was found, so this guard checked nothing")
    assertTrue(optional > 0, "no member the document leaves out was found, so this guard checked nothing")
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
  fun everyProblemTheCatalogueNamesIsRaisedAsItsOwnFailure() {
    // Declaring one exception per problem is half of it; the other half is the dispatch, which is
    // where a problem wired to the exception beside it in the catalogue would sit unnoticed until a
    // caller caught the wrong type in production. The catalogue is walked rather than listed, so a
    // problem the API grows is put through this the moment the generator writes it.
    for (named in ProblemId.entries) {
      val answered = linkedMapOf<String, Any?>(
        "id" to named.wireValue,
        "status" to 400L,
        "title" to "refused",
        "detail" to "what this case scripted",
        "type" to "https://hook0.com/documentation/errors/${named.wireValue}"
      )

      val raised =
        assertThrows(ProblemException::class.java) { Problems.raiseForStatus(400, Json.write(answered)) }

      assertEquals(
        "${named.wireValue}Exception",
        raised.javaClass.simpleName,
        "`${named.wireValue}` is not raised as the failure named after it"
      )
      assertEquals(400, raised.status)
      assertEquals(named, raised.problem?.id)
      assertTrue(raised.message?.contains("what this case scripted") == true, raised.message)
    }
  }

  @Test
  fun aValueTravelsInARequestLineTheWayTheApiReadsOne() {
    // What a generated operation hands the transport for a path segment or a query value. The
    // spellings are the API's, not the platform's: a moment is RFC 3339 and a day is ISO 8601,
    // whichever way the runtime would have printed either on its own.
    assertEquals("", Wire.written(null))
    assertEquals("what the caller passed", Wire.written("what the caller passed"))
    assertEquals("true", Wire.written(true))
    assertEquals("7", Wire.written(7))
    assertEquals("2026-01-02", Wire.written(LocalDate.parse("2026-01-02")))
    assertEquals("2026-01-02T03:04:05Z", Wire.written(OffsetDateTime.parse("2026-01-02T03:04:05Z")))
    assertEquals(
      "2026-01-02T03:04:05Z",
      Wire.queryValue(OffsetDateTime.parse("2026-01-02T03:04:05Z"))
    )

    // Every character a segment carries that could name another one travels encoded.
    assertEquals("an%20application%2F..%2F..", Wire.pathSegment("an application/../.."))
    assertEquals("what-the.caller_passed~", Wire.pathSegment("what-the.caller_passed~"))
  }

  @Test
  fun aDocumentThatDoesNotSayWhatItWasDeclaredToSayStopsTheRead() {
    // One reader per shape the document can declare, each handed something it was not declared to
    // carry. What matters is that the read stops: a reader that widened, coerced or shrugged would
    // hand a caller a value the API never sent, and nothing downstream would know.
    assertThrows(DecodeException::class.java) { Wire.asText(42L) }
    assertThrows(DecodeException::class.java) { Wire.asUuid("not a uuid") }
    assertThrows(DecodeException::class.java) { Wire.asLong("7") }
    assertThrows(DecodeException::class.java) { Wire.asBoolean("true") }
    assertThrows(DecodeException::class.java) { Wire.asMoment("2026-01-02") }
    assertThrows(DecodeException::class.java) { Wire.asDay("2026-01-02T03:04:05Z") }
    assertThrows(DecodeException::class.java) { Wire.asList(Wire::asText)("not an array") }
    assertThrows(DecodeException::class.java) { Wire.asMap(Wire::asText)("not an object") }
    assertThrows(DecodeException::class.java) { Wire.asFields("not an object", "a value") }
    assertThrows(DecodeException::class.java) { Wire.asJson(null) }

    // A number is not a boolean and a boolean is not a number, here or on the wire.
    assertThrows(DecodeException::class.java) { Wire.asLong(true) }
    assertThrows(DecodeException::class.java) { Wire.asDouble(true) }

    // And the same readers, handed what they were declared to carry.
    assertEquals("what arrived", Wire.asText("what arrived"))
    assertEquals(7L, Wire.asLong(7))
    assertEquals(true, Wire.asBoolean(true))
    assertEquals(LocalDate.parse("2026-01-02"), Wire.asDay("2026-01-02"))
    assertEquals(OffsetDateTime.parse("2026-01-02T03:04:05Z"), Wire.asMoment("2026-01-02T03:04:05Z"))
    assertEquals(listOf("one"), Wire.asList(Wire::asText)(listOf("one")))
    assertEquals(mapOf("a key" to "one"), Wire.asMap(Wire::asText)(mapOf("a key" to "one")))
  }

  @Test
  fun anIdentifierWrittenShortIsRefusedRatherThanWidened() {
    // `UUID.fromString` reads a group written with fewer digits than it has and pads it, so two
    // different texts would become one identifier and a caller comparing what it sent against what
    // came back would see them agree. What the API wrote is held against what was read.
    assertThrows(DecodeException::class.java) { Wire.asUuid("3f2504e0-4f89-41d3-9a0c-305e82c3310") }
    assertThrows(DecodeException::class.java) { Wire.asUuid("3f2504e0-4f89-41d3-9a0c-0305e82c33100") }
    assertEquals(
      UUID.fromString("3f2504e0-4f89-41d3-9a0c-0305e82c3310"),
      Wire.asUuid("3f2504e0-4f89-41d3-9a0c-0305e82c3310")
    )
  }

  @Test
  fun aWholeNumberIsRefusedWhenItIsWiderThanWhatItWasDeclaredAs() {
    // The document declares a 32-bit member; the wire carries whatever the API wrote. A number that
    // does not fit is refused rather than truncated into a different number.
    assertEquals(7, Wire.asInteger(7L))
    assertEquals(Int.MAX_VALUE, Wire.asInteger(Int.MAX_VALUE.toLong()))
    assertEquals(Int.MIN_VALUE, Wire.asInteger(Int.MIN_VALUE.toLong()))
    assertThrows(DecodeException::class.java) { Wire.asInteger(Int.MAX_VALUE.toLong() + 1L) }
    assertThrows(DecodeException::class.java) { Wire.asInteger(Int.MIN_VALUE.toLong() - 1L) }
  }

  @Test
  fun aNumberIsReadWhicheverWayTheDocumentWroteIt() {
    // `asDouble` is published on this artefact and the snapshot happens to declare no member that
    // uses it, so nothing else in this suite reaches it. A caller can, which is the whole reason it
    // is asserted: what it promises is that a number reads as a number whether or not the document
    // bothered to write a fractional part.
    assertEquals(1.5, Wire.asDouble(1.5))
    assertEquals(7.0, Wire.asDouble(7L))
    assertEquals(7.0, Wire.asDouble(7))
    assertEquals(-1.5, Wire.asDouble(-1.5))
    assertEquals(0.0, Wire.asDouble(0))

    assertThrows(DecodeException::class.java) { Wire.asDouble("1.5") }
    assertThrows(DecodeException::class.java) { Wire.asDouble(null) }
  }

  @Test
  fun everyValueOfACollectionIsWrittenBackTheWayOneOfThemIs() {
    // `writeMap` is published beside `writeList` and, like `asDouble`, the snapshot declares no
    // member that reaches it. Both promise the same thing: the writer of one value, applied to
    // every value.
    assertEquals(
      listOf("2026-01-02"),
      Wire.writeList(listOf(LocalDate.parse("2026-01-02")), Wire::writeDay)
    )
    assertEquals(
      mapOf("a key the document leaves open" to "2026-01-02"),
      Wire.writeMap(mapOf("a key the document leaves open" to LocalDate.parse("2026-01-02")), Wire::writeDay)
    )

    assertEquals(emptyList<Any?>(), Wire.writeList(emptyList<LocalDate>(), Wire::writeDay))
    assertEquals(emptyMap<String, Any?>(), Wire.writeMap(emptyMap<String, LocalDate>(), Wire::writeDay))
  }

  @Test
  fun anAnswerIsQuotedBackAtAFixedBudgetRatherThanWhole() {
    // What goes into a failure message is written by a server this client does not control, so it is
    // cut rather than echoed into whatever the caller logs.
    val oversized = "x".repeat(Wire.MAX_PREVIEW_CHARS + 1)

    assertEquals(Wire.MAX_PREVIEW_CHARS + 1, Wire.preview(oversized).length)
    assertTrue(Wire.preview(oversized).endsWith("\u2026"), Wire.preview(oversized))
    assertEquals("what the API answered", Wire.preview("what the API answered"))
  }

  @Test
  fun anAnswerThatIsNotADocumentAtAllStopsTheRead() {
    val refused =
      assertThrows(DecodeException::class.java) { Wire.decodePayload("<html>a proxy answered this</html>") }

    assertTrue(refused.message?.contains("a proxy answered this") == true, refused.message)
    assertEquals(mapOf("read" to "back"), Wire.decodePayload("{\"read\": \"back\"}"))
  }

  @Test
  fun everyClosedListCarriesEveryStringTheApiAnswersAndNothingElse() {
    // The constants are the wire values themselves, so a name moved out of the way of the language
    // never reaches the wire. Every list the generator wrote is walked and every value of it read
    // back from its own text, which is what says the whole list travels rather than the one value a
    // case happened to name.
    var exercised = 0
    for (declared in Generated.types()) {
      if (!declared.isEnum) {
        continue
      }
      for (named in declared.enumConstants) {
        val carried = wireValue(named)

        assertEquals(named, Generated.read(declared, carried), "${declared.simpleName} does not read back `$carried`")
        exercised++
      }

      assertThrows(
        DecodeException::class.java,
        { Generated.read(declared, "a value the API never declared") },
        "${declared.simpleName} carried a value the API declares none of"
      )
    }

    assertTrue(exercised > 0, "no closed list was found, so this guard checked nothing")
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
    // Every value the generator wrote, discovered from what it wrote and built out of what it
    // declares: a schema the document adds joins this case the moment the generated files carry it,
    // and one whose reader and writer disagree about a member fails here rather than at the first
    // answer that carries it.
    var exercised = 0
    for (declared in Generated.types()) {
      if (!Generated.isAValue(declared)) {
        continue
      }
      val filled = Generated.filled(declared)

      assertEquals(
        filled,
        Generated.readOrNothing(declared, Generated.written(filled)),
        "${declared.simpleName} does not read back what it wrote"
      )
      exercised++
    }

    assertTrue(exercised > 0, "no value the generator wrote could be built, so this guard checked nothing")
  }

  @Test
  fun everyMemberTheDocumentDoesNotRequireCanBeLeftOutByTheCallerToo() {
    // The other side of an optional member. A caller writing `Problem(detail, id, status, title,
    // type)` reaches the value through the bridge the compiler writes for a declaration carrying
    // defaults, and what is asserted is that leaving a member out and answering none of it come to
    // the same document — a default that was anything but nothing would show up here.
    //
    // Which positions may be left out is worked out from the value itself: a member is one the
    // document does not require exactly when the read still succeeds without it.
    var exercised = 0
    for (declared in Generated.types()) {
      if (!Generated.isAValue(declared) || Generated.defaulted(declared) == null) {
        continue
      }
      val whole = Generated.written(Generated.filled(declared))
      val names = whole.keys.toList()
      val leftOut =
        names.indices.filter { Generated.readOrNothing(declared, whole - names[it]) != null }.toSet()
      if (leftOut.isEmpty()) {
        continue
      }

      val written = Generated.written(Generated.filledLeavingOut(declared, leftOut))

      assertEquals(
        names.filterIndexed { at, _ -> at !in leftOut },
        written.keys.toList(),
        "${declared.simpleName} wrote back something other than the members the API requires"
      )
      exercised++
    }

    assertTrue(exercised > 0, "no value carrying a member a caller may leave out was found")
  }

  @ParameterizedTest
  @ValueSource(booleans = [false, true])
  fun everyOperationTheApiDeclaresIsIssuedByTheMethodWrittenForIt(suspending: Boolean) {
    // What an SDK is for: the API declares an operation, and the generated half issues exactly that
    // request for it and reads back exactly what the answer carried. Both sides are discovered — the
    // operations out of the API's own description, the methods out of what the generator wrote — so
    // an operation the document adds and the generator misses fails here rather than at whichever
    // caller reached for it first.
    val declared = Generated.declaredOperations()
    val issued = TreeSet<String>()

    for (blocking in Generated.groups(false)) {
      val group = if (suspending) checkNotNull(Generated.counterpart(blocking)) else blocking
      for (operation in Generated.operations(blocking)) {
        val named = "${group.simpleName}.${operation.name}"
        val arguments = argumentsOf(operation)
        val answer = answering(operation.genericReturnType)
        val called = if (suspending) twin(group, operation) else operation

        FakeApi().use { api ->
          transport(api).use { transport ->
            api.willAnswer(
              FakeApi.Scripted.of(200, scripted(answer)),
              FakeApi.Scripted.of(200, scripted(answer)),
              FakeApi.Scripted.of(200, scripted(answer))
            )
            val driven = group.getDeclaredConstructor(Transport::class.java).newInstance(transport)
            val read = calling(called, driven, arguments, suspending)

            if (answer != null) {
              assertEquals(answer, read, "$named answered something other than what the API carried")
            }

            val sent = api.received()[0]
            val matched = matching(declared, sent.verb, sent.target.substringBefore('?'))
            assertNotNull(matched, "$named issued `${sent.target}`, which the API declares no operation at")
            issued.add(matched!!.named())

            for (argument in arguments) {
              if (argument is String) {
                assertTrue(sent.target.contains(argument), "$named dropped `$argument` from ${sent.target}")
              } else {
                assertEquals(
                  Json.parse(Json.write(Generated.written(argument!!))),
                  sent.json(),
                  "$named sent something other than the value it was handed"
                )
              }
            }
            assertEquals(matched.query, assembled(sent), "$named assembled a query the API declares nothing of")

            if (matched.query != matched.required) {
              // The same operation again, with everything the API does not ask for left out: a
              // parameter a caller sends none of has to be absent from the query rather than sent as
              // an empty one, which is a difference the API can see.
              val optional = theOptional(arguments, sent, matched)
              calling(
                called,
                driven,
                arguments.mapIndexed { at, value -> if (at in optional) null else value }.toTypedArray(),
                suspending
              )

              assertEquals(
                matched.required,
                assembled(api.received()[1]),
                "$named sent a parameter the caller left out"
              )

              // And once more the way a caller who *omits* them reaches it, through the bridge the
              // compiler writes for a function carrying defaults. A default that was anything but
              // nothing would assemble a different query here than passing nothing did.
              val bridge = checkNotNull(Generated.defaulted(group, called)) {
                "$named takes an argument the API does not ask for and carries no default for it"
              }
              Generated.leavingOut(bridge, driven, arguments, optional, suspending)

              assertEquals(
                assembled(api.received()[1]),
                assembled(api.received()[2]),
                "$named assembled a different query for an argument left out than for one passed as nothing"
              )
            }
          }
        }
      }
    }

    assertEquals(declared.keys, issued, "the API and the generated half do not name the same operations")
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

    /** The text one value of a closed list travels as. */
    private fun wireValue(named: Any): String = named.javaClass.getMethod("getWireValue").invoke(named) as String

    /** What one operation is called with: one distinct value per name it takes, and one value per body. */
    private fun argumentsOf(operation: Method): Array<Any?> = operation.parameterTypes
      .mapIndexed { at, shape ->
        if (shape == String::class.java) "what-this-case-passed-$at" else Generated.filled(shape)
      }
      .toTypedArray()

    /** A value of that shape for the API to answer, or nothing where the operation answers none. */
    private fun answering(answered: Type): Any? = when {
      answered == Void.TYPE || answered == Unit::class.java -> null
      answered is ParameterizedType -> listOf(Generated.value(answered.actualTypeArguments[0]))
      else -> Generated.filled(answered as Class<*>)
    }

    /** That value as the document the API writes it as. */
    private fun scripted(answered: Any?): Any? = when (answered) {
      null -> null
      is List<*> -> answered.map { if (Generated.isAValue(it!!.javaClass)) Generated.written(it) else it }
      else -> Generated.written(answered)
    }

    /** The same operation on the flavour that suspends, which takes one more argument than it declares. */
    private fun twin(group: Class<*>, operation: Method): Method = Generated.operations(group).single {
      it.name == operation.name && it.parameterTypes.dropLast(1) == operation.parameterTypes.toList()
    }

    /** What one operation answered, whichever flavour it was called through. */
    private fun calling(operation: Method, group: Any, arguments: Array<Any?>, suspending: Boolean): Any? =
      if (suspending) {
        Generated.awaiting(operation, group, arguments)
      } else {
        try {
          operation.invoke(group, *arguments)
        } catch (raised: InvocationTargetException) {
          throw raised.cause ?: raised
        }
      }

    /** The one operation the API declares under that verb and that path, or nothing when it declares none. */
    private fun matching(declared: Map<String, Generated.Operation>, verb: String, path: String): Generated.Operation? {
      var found: Generated.Operation? = null
      for (operation in declared.values) {
        if (operation.verb != verb || !fills(operation.path, path)) {
          continue
        }
        assertNull(found, "`$verb $path` reads as both `$found` and `${operation.named()}`")
        found = operation
      }
      return found
    }

    /** Whether that path is that template with every placeholder of it filled in. */
    private fun fills(template: String, path: String): Boolean {
      val declared = template.split("/")
      val issued = path.split("/")
      if (declared.size != issued.size) {
        return false
      }
      return declared.zip(issued).all { (segment, carried) ->
        if (segment.startsWith("{") && segment.endsWith("}")) carried.isNotEmpty() else segment == carried
      }
    }

    /** The parameters one request carried in its query string, by name. */
    private fun assembled(sent: FakeApi.Received): TreeSet<String> {
      val named = TreeSet<String>()
      val query = sent.target.substringAfter('?', "")
      if (query.isEmpty()) {
        return named
      }
      for (parameter in query.split("&")) {
        named.add(URLDecoder.decode(parameter.substringBefore('='), StandardCharsets.UTF_8))
      }
      return named
    }

    /** Which arguments landed under a parameter the API does not ask for. */
    private fun theOptional(arguments: Array<Any?>, sent: FakeApi.Received, matched: Generated.Operation): Set<Int> =
      arguments.indices.filter { at ->
        val carried = arguments[at] as? String
        carried != null && matched.query.any { it !in matched.required && sent.target.contains("$it=$carried") }
      }.toSet()

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
