package com.hook0.kotlin

import java.lang.reflect.InvocationTargetException
import java.nio.file.Files
import java.nio.file.Path
import kotlin.io.path.name

/**
 * Everything the generator wrote, found by looking at what it wrote.
 *
 * Nothing lists the types anywhere in this suite: a schema the document adds, a problem the API
 * grows or an entity it declares joins the cases that walk this the moment the generated files carry
 * it. The files are what is walked rather than the classpath, since reading a package's members at
 * run time would take a dependency this artefact does not have.
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

  /** Whether that type is one of the values the API declares rather than a group or an exception. */
  fun isAValue(declared: Class<*>): Boolean = companionOf(declared) != null

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
  fun written(value: Any): Any? = value.javaClass.getMethod(WRITE_MEMBER).invoke(value)

  /** The companion a generated value declares its reader on, or nothing when it declares none. */
  private fun companionOf(declared: Class<*>): Any? {
    if (declared.declaredMethods.none { it.name == WRITE_MEMBER && it.parameterCount == 0 }) {
      return null
    }
    return try {
      declared.getDeclaredField("Companion").also { it.isAccessible = true }.get(null)
    } catch (missing: NoSuchFieldException) {
      null
    }
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
