package com.hook0.kotlin

import java.lang.reflect.Constructor
import java.lang.reflect.InvocationTargetException
import java.lang.reflect.Method
import java.lang.reflect.Modifier
import java.lang.reflect.ParameterizedType
import java.lang.reflect.Type
import java.nio.file.Files
import java.nio.file.Path
import java.time.Duration
import java.time.LocalDate
import java.time.OffsetDateTime
import java.util.Locale
import java.util.TreeSet
import java.util.UUID
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicReference
import kotlin.coroutines.Continuation
import kotlin.coroutines.CoroutineContext
import kotlin.coroutines.EmptyCoroutineContext
import kotlin.coroutines.intrinsics.COROUTINE_SUSPENDED
import kotlin.io.path.name

/**
 * Everything the generator wrote, found by looking at what it wrote — and the document it wrote it
 * from.
 *
 * Nothing lists the types anywhere in this suite: a schema the document adds, a problem the API
 * grows or an entity it declares joins the cases that walk this the moment the generated files carry
 * it. The files are what is walked rather than the classpath, since reading a package's members at
 * run time would take a dependency this artefact does not have.
 *
 * The API's own description is read too, so that what the generated half issues can be held against
 * what the API declares rather than against what the generator happened to write. An operation the
 * document adds is therefore one this suite starts asking for without a line of it being touched.
 */
object Generated {

  /** The package the generator writes into. */
  private const val PACKAGE = "com.hook0.kotlin.generated"

  /** Where it writes, from the directory this suite runs out of. */
  val ROOT: Path = Path.of("src", "main", "kotlin", "com", "hook0", "kotlin", "generated")

  /** No generated tree is anywhere near this large; the cap turns a pathological one into a failure. */
  private const val MAX_TYPES = 4096

  /** Suffix an operation group carries, and the one the flavour that suspends carries. */
  private const val GROUP_SUFFIX = "Api"
  private const val SUSPENDING_GROUP_SUFFIX = "SuspendingApi"

  /** Member every value declares to be read out of what the API answered, and to write itself back. */
  const val READ_MEMBER = "fromJson"
  const val WRITE_MEMBER = "toJson"

  /** The tag the API marks the operations an SDK is written for with. */
  private const val PUBLIC_TAG = "public"

  /** What the compiler names the bridge and the marker it writes for a declaration carrying defaults. */
  private const val DEFAULT_SUFFIX = "\$default"
  private const val DEFAULT_MARKER = "DefaultConstructorMarker"

  /** The API's own description, from the directory this suite runs out of. */
  private val SNAPSHOT: Path =
    Path.of("").toAbsolutePath().parent.parent.resolve("api").resolve("openapi.snapshot.json")

  /** The description is committed, so one above this grew out of shape rather than being one somebody meant. */
  private const val MAX_SNAPSHOT_BYTES = 4L * 1024 * 1024

  /** No API declares anywhere near this many operations; the cap bounds what one case walks. */
  private const val MAX_OPERATIONS = 512

  /** How far a value is built into itself before this suite reads the shape as one it cannot build. */
  private const val MAX_NESTING = 16

  /** Longest a suspending call is given before the suite reads it as stuck rather than slow. */
  private val PATIENCE: Duration = Duration.ofSeconds(60)

  /** The API's description, read once: it is committed, so it does not change under a run. */
  private var declared: Map<String, Operation>? = null

  /** Every type the generator declared, in the order the filesystem holds them, sorted. */
  fun types(): List<Class<*>> {
    val files =
      Files.list(ROOT.toAbsolutePath()).use { written ->
        written.sorted().toList()
      }
    check(files.size <= MAX_TYPES) { "$ROOT holds more than the $MAX_TYPES types walked" }

    return files
      .filter { it.name.endsWith(".kt") }
      .map { loaded(it.name.removeSuffix(".kt")) }
  }

  /** The operation groups of one flavour. */
  fun groups(suspending: Boolean): List<Class<*>> = types().filter { declared ->
    val name = declared.simpleName
    name.endsWith(GROUP_SUFFIX) && name.endsWith(SUSPENDING_GROUP_SUFFIX) == suspending
  }

  /** The group of the other flavour covering the same entity, or nothing when there is none. */
  fun counterpart(blocking: Class<*>): Class<*>? {
    val wanted = blocking.simpleName.removeSuffix(GROUP_SUFFIX) + SUSPENDING_GROUP_SUFFIX
    return groups(true).firstOrNull { it.simpleName == wanted }
  }

  /** Whether that type is one of the values the API declares rather than a group, a list or a failure. */
  fun isAValue(declared: Class<*>): Boolean =
    declared.declaredMethods.any { it.name == WRITE_MEMBER && it.parameterCount == 0 } &&
      companionOf(declared) != null

  /** The operations one group declares, in a settled order so that answers can be queued against them. */
  fun operations(group: Class<*>): List<Method> = group.declaredMethods
    .filter { Modifier.isPublic(it.modifiers) && !it.isSynthetic }
    .sortedWith(compareBy({ it.name }, { it.parameterCount }))

  /**
   * One of that value, with every member the document names carrying something.
   *
   * Built out of what the class declares rather than out of a document written here, so that a
   * member the API adds is filled in the moment the generated file carries it — and so that a member
   * this suite cannot build is a failure saying so rather than a case that quietly stops covering
   * it.
   */
  fun filled(declared: Class<*>): Any = filled(declared, MAX_NESTING)

  /** One value of that shape, for a member or an answer declared to carry one. */
  fun value(type: Type): Any = member(type.typeName, type, MAX_NESTING)

  private fun filled(declared: Class<*>, depth: Int): Any {
    check(depth > 0) { "${declared.simpleName} nests deeper than the $MAX_NESTING built" }
    val building = primary(declared)
    val members = building.genericParameterTypes.map { member(declared.simpleName, it, depth) }
    return building.newInstance(*members.toTypedArray())
  }

  /**
   * The constructor a value declares, which is not always the only one it carries: a member with a
   * default value has the compiler write a second one taking a mask beside it, and that one takes no
   * value this suite could hand it.
   */
  private fun primary(declared: Class<*>): Constructor<*> = declared.declaredConstructors
    .filter { candidate -> candidate.parameterTypes.none { it.simpleName == "DefaultConstructorMarker" } }
    .maxByOrNull { it.parameterCount }
    ?: throw IllegalStateException("${declared.simpleName} declares no constructor this suite can call")

  private fun member(owner: String, type: Type, depth: Int): Any {
    if (type is ParameterizedType) {
      val arguments = type.actualTypeArguments
      return when (type.rawType) {
        List::class.java -> listOf(member(owner, arguments[0], depth - 1))
        Map::class.java -> mapOf("a key the document leaves open" to member(owner, arguments[1], depth - 1))
        else -> throw IllegalStateException("$owner declares a member of a shape this suite cannot build: $type")
      }
    }
    val declared = type as Class<*>
    // A member declared as `Any` is one the document leaves undescribed, which therefore travels as
    // whatever arrived.
    return when {
      declared == String::class.java -> "what the document names"
      declared == UUID::class.java -> UUID.fromString("3f2504e0-4f89-41d3-9a0c-0305e82c3310")
      declared == Int::class.java || declared == Int::class.javaObjectType -> 7
      declared == Long::class.java || declared == Long::class.javaObjectType -> 7L
      declared == Double::class.java || declared == Double::class.javaObjectType -> 1.5
      declared == Boolean::class.java || declared == Boolean::class.javaObjectType -> true
      declared == OffsetDateTime::class.java -> OffsetDateTime.parse("2026-01-02T03:04:05Z")
      declared == LocalDate::class.java -> LocalDate.parse("2026-01-02")
      declared == Any::class.java -> mapOf("a member the document does not describe" to "what it carried")
      declared.isEnum -> declared.enumConstants[0]
      isAValue(declared) -> filled(declared, depth - 1)
      else -> throw IllegalStateException("$owner declares a member this suite cannot build: $declared")
    }
  }

  /**
   * One operation the API declares for an SDK: where it lands, and what it names in its query
   * string.
   *
   * @property verb the method it is issued under
   * @property path where it lands, with a placeholder per member of the path it names
   * @property query every parameter it takes in its query string
   * @property required the ones of those a caller cannot leave out
   */
  data class Operation(val verb: String, val path: String, val query: TreeSet<String>, val required: TreeSet<String>) {

    /** How this operation is named where one is held against a request that arrived. */
    fun named(): String = "$verb $path"
  }

  /**
   * Every operation the API declares for an SDK, by the verb and the path it declares it under.
   *
   * Only the ones the document marks public: those are what the generator writes a method for, and
   * holding the generated half against the rest would be holding it against operations it was never
   * asked to carry.
   */
  fun declaredOperations(): Map<String, Operation> {
    declared?.let { return it }

    val found = sortedMapOf<String, Operation>()
    for ((path, operations) in paths()) {
      for ((verb, operation) in asObject(operations)) {
        if (PUBLIC_TAG !in tags(operation)) {
          continue
        }
        val read =
          Operation(
            verb.uppercase(Locale.ROOT),
            path,
            query(operation, false),
            query(operation, true)
          )
        found[read.named()] = read
        check(found.size <= MAX_OPERATIONS) { "the API declares more than the $MAX_OPERATIONS operations walked" }
      }
    }
    return found.toMap().also { declared = it }
  }

  /** What one operation names in its query string, either all of it or only what a caller cannot leave out. */
  private fun query(operation: Any?, onlyRequired: Boolean): TreeSet<String> {
    val named = TreeSet<String>()
    val parameters = asObject(operation)["parameters"]
    if (parameters !is List<*>) {
      return named
    }
    for (parameter in parameters) {
      val read = asObject(parameter)
      if (read["in"] != "query" || (onlyRequired && read["required"] != true)) {
        continue
      }
      named.add(read["name"] as String)
    }
    return named
  }

  private fun paths(): Map<String, Any?> {
    val size = Files.size(SNAPSHOT)
    check(size <= MAX_SNAPSHOT_BYTES) { "$SNAPSHOT is $size bytes long, above the $MAX_SNAPSHOT_BYTES read" }
    return asObject(asObject(Json.parse(Files.readString(SNAPSHOT)))["paths"])
  }

  @Suppress("UNCHECKED_CAST")
  private fun asObject(value: Any?): Map<String, Any?> = value as Map<String, Any?>

  private fun tags(operation: Any?): List<*> = asObject(operation)["tags"] as? List<*> ?: emptyList<Any?>()

  /**
   * What one suspending call answered, driven to its end on the calling thread.
   *
   * A suspending function compiles to one taking a continuation, so this is how a case calls one it
   * found rather than one it named. Written out of the standard library on purpose, and for the same
   * reason the SDK itself names no coroutine library.
   */
  fun awaiting(operation: Method, group: Any, arguments: Array<Any?>): Any? =
    driven { resuming -> operation.invoke(group, *arguments, resuming) }

  /**
   * The bridge the compiler writes for an operation carrying default arguments, or nothing when it
   * writes none.
   */
  fun defaulted(group: Class<*>, operation: Method): Method? =
    group.declaredMethods.firstOrNull { it.name == "${operation.name}$DEFAULT_SUFFIX" }

  /**
   * That operation as a Kotlin caller who leaves arguments out calls it.
   *
   * The compiler writes one static bridge per function carrying defaults, taking the receiver, every
   * argument, a mask naming the ones the caller left out, and a marker. A caller writing
   * `list(applicationId)` reaches the group through it, so this is the path that says what a default
   * actually defaults to — an assertion no call passing every argument can make.
   */
  fun leavingOut(bridge: Method, group: Any, arguments: Array<Any?>, leftOut: Set<Int>, suspending: Boolean): Any? {
    val passed = mutableListOf<Any?>(group)
    arguments.forEachIndexed { at, value -> passed.add(if (at in leftOut) null else value) }

    if (!suspending) {
      return invoked(bridge, passed + listOf(masking(leftOut), null))
    }
    return driven { resuming -> invoked(bridge, passed + listOf(resuming, masking(leftOut), null)) }
  }

  /**
   * One of that value with the members at those positions left out, as a Kotlin caller leaving them
   * out builds it.
   */
  fun filledLeavingOut(declared: Class<*>, leftOut: Set<Int>): Any {
    val bridge =
      defaulted(declared)
        ?: throw IllegalStateException("${declared.simpleName} carries no member a caller may leave out")
    val members = primary(declared).genericParameterTypes.mapIndexed { at, type ->
      if (at in leftOut) null else member(declared.simpleName, type, MAX_NESTING)
    }
    bridge.isAccessible = true
    return bridge.newInstance(*(members + listOf(masking(leftOut), null)).toTypedArray())
  }

  /** The constructor the compiler writes for a value carrying default members, or nothing when it writes none. */
  fun defaulted(declared: Class<*>): Constructor<*>? = declared.declaredConstructors
    .firstOrNull { candidate -> candidate.parameterTypes.any { it.simpleName == DEFAULT_MARKER } }

  /** The mask the compiler reads to know which arguments the caller left out: one bit per position. */
  private fun masking(leftOut: Set<Int>): Int = leftOut.fold(0) { mask, at -> mask or (1 shl at) }

  private fun invoked(bridge: Method, arguments: List<Any?>): Any? = try {
    bridge.invoke(null, *arguments.toTypedArray())
  } catch (raised: InvocationTargetException) {
    throw raised.cause ?: raised
  }

  /** One suspending call, started and waited out on the calling thread. */
  private fun driven(call: (Continuation<Any?>) -> Any?): Any? {
    val outcome = AtomicReference<Result<Any?>>()
    val done = CountDownLatch(1)
    val resuming = object : Continuation<Any?> {
      override val context: CoroutineContext = EmptyCoroutineContext

      override fun resumeWith(result: Result<Any?>) {
        outcome.set(result)
        done.countDown()
      }
    }

    val answered =
      try {
        call(resuming)
      } catch (raised: InvocationTargetException) {
        throw raised.cause ?: raised
      }
    if (answered !== COROUTINE_SUSPENDED) {
      return answered
    }

    check(done.await(PATIENCE.toMillis(), TimeUnit.MILLISECONDS)) {
      "a suspending call did not finish inside the ${PATIENCE.toSeconds()} seconds it is given"
    }
    return outcome.get().getOrThrow()
  }

  /** One value read out of that document, raising what the read raised. */
  fun read(declared: Class<*>, document: Any?): Any? {
    val companion =
      companionOf(declared) ?: throw IllegalStateException("${declared.simpleName} reads nothing back")
    return try {
      companion.javaClass.getMethod(READ_MEMBER, Any::class.java).invoke(companion, document)
    } catch (raised: InvocationTargetException) {
      throw raised.cause ?: raised
    }
  }

  /** One value read out of that document, or nothing when the document does not describe one. */
  fun readOrNothing(declared: Class<*>, document: Any?): Any? {
    val companion = companionOf(declared) ?: return null
    val reading =
      try {
        companion.javaClass.getMethod(READ_MEMBER, Any::class.java)
      } catch (missing: NoSuchMethodException) {
        return null
      }
    return try {
      reading.invoke(companion, document)
    } catch (raised: InvocationTargetException) {
      val carried = raised.cause
      if (carried is DecodeException) {
        return null
      }
      throw IllegalStateException(
        "${declared.simpleName} failed to read a document with something other than a decode failure",
        carried
      )
    }
  }

  /** That value, written back the way the API reads it. */
  @Suppress("UNCHECKED_CAST")
  fun written(value: Any): Map<String, Any?> =
    value.javaClass.getMethod(WRITE_MEMBER).invoke(value) as Map<String, Any?>

  /** The companion a generated type declares its reader on, or nothing when it declares none. */
  private fun companionOf(declared: Class<*>): Any? = try {
    declared.getDeclaredField("Companion").also { it.isAccessible = true }.get(null)
  } catch (missing: NoSuchFieldException) {
    null
  }

  private fun loaded(simple: String): Class<*> = try {
    Class.forName("$PACKAGE.$simple")
  } catch (missing: ClassNotFoundException) {
    throw IllegalStateException(
      "the generator wrote $simple.kt and nothing compiled from it",
      missing
    )
  }
}
