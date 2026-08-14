package com.hook0.kotlin

import java.io.ByteArrayOutputStream
import java.io.PrintStream
import java.lang.reflect.Modifier
import java.net.URLClassLoader
import java.nio.charset.StandardCharsets
import java.nio.file.Files
import java.nio.file.Path
import java.util.TreeSet
import kotlin.io.path.deleteRecursively
import org.jetbrains.kotlin.cli.jvm.K2JVMCompiler
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertFalse
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.Timeout

/**
 * What the language actually refuses where the generator puts a name, measured rather than recalled.
 *
 * The vocabulary the generator spells names out of the way of is a table, and a table is a claim.
 * These cases hold that claim against the one thing that can settle it: the compiler itself, run
 * over sources written here.
 *
 * The answer is positional, and it is the opposite way round from Java's. A hard keyword is illegal
 * *everywhere* an identifier goes — as a property, as an argument and as a function alike — and
 * backticks rescue it in every one of those positions. `hashCode` and `toString` are the other half:
 * legal as a property, illegal as a no-argument function, and backticks rescue neither, because
 * `` `toString`() `` is the same declaration as `toString()`. One set is escapable and applies
 * everywhere; the other is not escapable and applies in one position. That is why there are two
 * tables beside the emitter rather than one.
 *
 * Why the emitter renames a keyword instead of backticking it is measured here too, and it is not a
 * matter of taste: a property spelled `` `class` `` compiles to the getter its name dictates, and
 * the last case below reads that getter's name off the class file.
 */
@Timeout(600)
class ReservedWordsTest {

  @Test
  fun everyHardKeywordIsRefusedWhereverAnIdentifierGoes() {
    val accepted = TreeSet<String>()
    for (word in bounded(HARD_KEYWORDS)) {
      if (compiles("data class Probe(val $word: String)") ||
        compiles("class Probe { fun $word(): String = \"\" }") ||
        compiles("class Probe { fun m($word: String): String = $word }")
      ) {
        accepted.add(word)
      }
    }

    assertTrue(accepted.isEmpty(), "the compiler accepts words this suite calls keywords: $accepted")
  }

  @Test
  fun backticksRescueAHardKeywordInEveryPositionAndTheEmitterStillDoesNotUseThem() {
    // Measured rather than assumed, because it is the reason the table escapes with a suffix: the
    // escape the language offers does work, and it is still the wrong one here.
    for (word in bounded(SAMPLED_KEYWORDS)) {
      assertTrue(
        compiles("data class Probe(val `$word`: String)"),
        "`$word` is refused as a backticked property, so backticks rescue less than this says"
      )
      assertTrue(
        compiles("class Probe { fun `$word`(): String = \"\" }"),
        "`$word` is refused as a backticked function"
      )
      assertTrue(
        compiles("class Probe { fun m(`$word`: String): String = `$word` }"),
        "`$word` is refused as a backticked argument"
      )
    }

    // And this is what it would cost. A property named `class` compiles to `getClass()`, which every
    // object already answers to — so a Java caller of the published artefact writes `getClass()` and
    // reads the property. The emitter spells it `class_` instead, which reads the same from either
    // language.
    val declared = declaring("data class Probe(val `class`: String)")
    val getter = declared.declaredMethods.single { it.name == "getClass" }

    assertEquals(
      String::class.java,
      getter.returnType,
      "a backticked `class` no longer compiles to a getter that hides the one every object carries"
    )
  }

  @Test
  fun whatEveryValueAnswersToIsLegalAsAPropertyAndRefusedAsAFunction() {
    // This is the distinction that decides the shape of the tables: one set is refused in every
    // position and escapable, the other only where a function is declared and escapable nowhere.
    val answered = namesEveryValueAnswersTo()

    assertFalse(answered.isEmpty(), "no name every value answers to was found, so this measured nothing")

    for (word in bounded(answered.toList())) {
      assertTrue(
        compiles("data class Probe(val $word: String)"),
        "`$word` is refused as a property, so it belongs to the keyword half rather than this one"
      )
      assertFalse(
        compiles("class Probe { fun `$word`(): String = \"\" }"),
        "`$word` is accepted as a backticked function, so an escape would have done"
      )
    }

    // `equals` takes an argument, so a no-argument one overloads it rather than hiding it, and a
    // declaration of that name compiles. That is why it is deliberately absent from the table.
    assertFalse(answered.contains("equals"))
    assertTrue(compiles("class Probe { fun equals(): String = \"\" }"))
    assertTrue(compiles("data class Probe(val equals: String)"))
  }

  @Test
  fun aSoftKeywordIsLegalWhereverThisTargetPutsAName() {
    // `data`, `value`, `get`, `set`, `where` and the rest are read as names wherever this target
    // puts one, so reserving them would rename a member the API is entitled to declare for a
    // collision the language does not have.
    for (word in bounded(SOFT_KEYWORDS)) {
      assertTrue(
        compiles("data class Probe(val $word: String)"),
        "`$word` is refused as a property, so it belongs in the table after all"
      )
      assertTrue(
        compiles("class Probe { fun $word(): String = \"\" }"),
        "`$word` is refused as a function, so it belongs in the table after all"
      )
    }
  }

  @Test
  fun everyNameTheGeneratorDeclaredCompilesWhereItPutIt() {
    // The whole generated tree is on the classpath of this suite, so it already compiled; what this
    // adds is that the names it declared are the ones a caller reaches, rather than names the
    // compiler happened to tolerate somewhere else.
    val keywords = TreeSet(HARD_KEYWORDS)
    val offenders = TreeSet<String>()

    for (declared in Generated.types()) {
      for (method in declared.declaredMethods) {
        if (Modifier.isPublic(method.modifiers) && keywords.contains(method.name)) {
          offenders.add("${declared.simpleName}.${method.name}")
        }
      }
    }

    assertTrue(
      offenders.isEmpty(),
      "the generator declared names the language keeps for itself: $offenders"
    )
  }

  companion object {
    /** How many candidates one case compiles, which bounds what it costs. */
    private const val MAX_CANDIDATES = 64

    /** The words the language keeps for itself, which no plain identifier may be spelled as. */
    private val HARD_KEYWORDS =
      listOf(
        "as", "break", "class", "continue", "do", "else", "false", "for", "fun", "if", "in",
        "interface", "is", "null", "object", "package", "return", "super", "this", "throw", "true",
        "try", "typealias", "typeof", "val", "var", "when", "while"
      )

    /**
     * The keywords the backtick cases run over.
     *
     * Escaping is a property of the syntax rather than of the word, so a sample settles it; the case
     * that matters — every one of them being refused unescaped — runs over the whole list.
     */
    private val SAMPLED_KEYWORDS = listOf("class", "object", "val", "when", "is", "in")

    /** Words the language reads as names wherever this target puts one. */
    private val SOFT_KEYWORDS =
      listOf(
        "by", "catch", "constructor", "data", "field", "finally", "get", "init", "out", "param",
        "property", "receiver", "sealed", "set", "suspend", "value", "where"
      )

    /** What the compiler is handed to read the language's own runtime from. */
    private val CLASSPATH: String = System.getProperty("java.class.path")

    private val ANSWERED: TreeSet<String> by lazy { measureWhatEveryValueAnswersTo() }

    /**
     * The names every value already answers to and no generated function may carry.
     *
     * Measured rather than written down: each candidate is declared as a no-argument function and
     * the compiler is asked. What comes back today is `hashCode` and `toString`.
     *
     * @return the names a function may not be spelled as
     */
    fun namesEveryValueAnswersTo(): TreeSet<String> = ANSWERED

    private fun measureWhatEveryValueAnswersTo(): TreeSet<String> {
      // Everything `java.lang.Object` declares without an argument is offered up, so a release that
      // moved one of them onto `Any` — or off it — changes what this measures rather than passing.
      val candidates =
        Any::class.java.declaredMethods
          .filter { it.parameterCount == 0 && !Modifier.isStatic(it.modifiers) }
          .map { it.name }
          .toSortedSet()

      val hidden = TreeSet<String>()
      for (word in bounded(candidates.toList())) {
        if (!compiles("class Probe { fun $word(): String = \"\" }")) {
          hidden.add(word)
        }
      }
      return hidden
    }

    private fun bounded(candidates: List<String>): List<String> {
      assertTrue(
        candidates.size <= MAX_CANDIDATES,
        "this case would compile ${candidates.size} candidates, above the $MAX_CANDIDATES it is given"
      )
      return candidates
    }

    /** What the one class a probe declares is called once it has compiled. */
    private const val PROBE = "probe.Probe"

    /** Whether the compiler this build runs accepts that source. */
    @OptIn(kotlin.io.path.ExperimentalPathApi::class)
    private fun compiles(source: String): Boolean {
      val out = compiled(source) ?: return false
      out.parent.deleteRecursively()
      return true
    }

    /** The class that source declared, or a failure saying it did not compile. */
    @OptIn(kotlin.io.path.ExperimentalPathApi::class)
    private fun declaring(source: String): Class<*> {
      val out = compiled(source) ?: throw AssertionError("`$source` does not compile")
      try {
        URLClassLoader(arrayOf(out.toUri().toURL()), ReservedWordsTest::class.java.classLoader)
          .use { loader ->
            return loader.loadClass(PROBE)
          }
      } finally {
        out.parent.deleteRecursively()
      }
    }

    /** Where that source compiled to, or nothing when it did not compile at all. */
    @OptIn(kotlin.io.path.ExperimentalPathApi::class)
    private fun compiled(source: String): Path? {
      val directory = Files.createTempDirectory("hook0-probe")
      val file = directory.resolve("Probe.kt")
      Files.writeString(file, "package probe\n\n$source\n", StandardCharsets.UTF_8)
      val out = directory.resolve("out")

      val reported = ByteArrayOutputStream()
      val exit =
        K2JVMCompiler()
          .exec(
            PrintStream(reported, true, StandardCharsets.UTF_8),
            "-nowarn",
            "-d",
            out.toString(),
            "-classpath",
            CLASSPATH,
            "-jvm-target",
            "21",
            file.toString()
          )

      if (exit.code != 0) {
        directory.deleteRecursively()
        return null
      }
      return out
    }
  }
}
