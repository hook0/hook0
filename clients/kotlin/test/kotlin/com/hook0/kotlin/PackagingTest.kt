package com.hook0.kotlin

import java.nio.charset.StandardCharsets
import java.nio.file.Files
import java.nio.file.Path
import javax.xml.XMLConstants
import javax.xml.parsers.DocumentBuilderFactory
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertFalse
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.Timeout
import org.w3c.dom.Element
import org.w3c.dom.Node

/**
 * What this artefact promises whoever adds it to an application.
 *
 * The promise is that adding it drags nothing onto the classpath beyond the language's own runtime:
 * it reaches the network, verifies signatures, reads what the API answers and suspends with the
 * standard library and nothing else. These cases are what keep that sentence true rather than
 * aspirational — a dependency that is neither test-scoped nor `kotlin-stdlib` fails here rather than
 * in somebody's dependency report months later.
 *
 * The build file is read as the document it is rather than searched as text, so a plugin's own
 * dependencies and the ranges an enforcer rule is written with are not mistaken for what this
 * artefact ships.
 */
@Timeout(60)
class PackagingTest {

  @Test
  fun theArtefactDeclaresNoRuntimeDependencyBeyondTheLanguagesOwn() {
    val carried = mutableListOf<String>()

    for (block in children(project(), "dependencies")) {
      for (declared in children(block, "dependency")) {
        if (textOf(declared, "scope") == "test") {
          continue
        }
        val named = "${textOf(declared, "groupId")}:${textOf(declared, "artifactId")}"
        if (named != LANGUAGE_RUNTIME) {
          carried.add(named)
        }
      }
    }

    assertTrue(
      carried.isEmpty(),
      "this artefact would drag $carried onto the classpath of every application that adds it"
    )
  }

  @Test
  fun theLanguagesOwnRuntimeIsTheVersionTheCompilerIs() {
    // `kotlin-stdlib` is not a library this artefact chose; it is what every class file it emits
    // links against. Naming a version other than the compiler's would be a build whose output and
    // whose runtime disagree, which is the one way this dependency can be wrong.
    val declared =
      children(project(), "dependencies")
        .flatMap { children(it, "dependency") }
        .single { "${textOf(it, "groupId")}:${textOf(it, "artifactId")}" == LANGUAGE_RUNTIME }

    assertEquals("\${kotlin.version}", textOf(declared, "version"))
    assertTrue(
      text().contains("<artifactId>kotlin-maven-plugin</artifactId>"),
      "nothing says which compiler writes the class files this runtime is for"
    )
  }

  @Test
  fun everyVersionTheBuildResolvesIsPinned() {
    // What bounds the build's reach. A range or a snapshot lets a pipeline resolve whatever was
    // published since the last run, which is a build nobody chose.
    val unpinned = mutableListOf<String>()

    for (declared in everywhere(project(), "dependency")) {
      val version = textOf(declared, "version")
      if (version.isNotEmpty() && unpinned(version)) {
        unpinned.add("${textOf(declared, "artifactId")} $version")
      }
    }
    for (declared in everywhere(project(), "plugin")) {
      val version = textOf(declared, "version")
      if (version.isEmpty() || unpinned(version)) {
        unpinned.add("${textOf(declared, "artifactId")} $version")
      }
    }

    assertTrue(unpinned.isEmpty(), "the build resolves what it was not told to resolve: $unpinned")
    assertTrue(text().contains("<requirePluginVersions/>"), "nothing refuses an unpinned plugin")
    assertTrue(text().contains("<banDynamicVersions>"), "nothing refuses a dynamic version")
  }

  @Test
  fun theArtefactIsPublishedUnderTheCoordinatesItsHomeControls() {
    assertEquals(
      "com.hook0",
      textOf(project(), "groupId"),
      "the group is not one whose DNS this project controls, which is what a registry verifies"
    )
    assertEquals("hook0-client-kotlin", textOf(project(), "artifactId"))
    assertFalse(children(project(), "licenses").isEmpty(), "the artefact names no licence")
  }

  @Test
  fun theSuiteSitsOutsideTheTreeTheGeneratorRewrites() {
    // `src/main/kotlin/com/hook0/kotlin/generated` is rewritten wholesale on every regeneration, and
    // the generator removes whatever it did not write. A directory named `test` anywhere under `src`
    // is also what the layout guard beside the generator refuses, so the suite sits outside `src`
    // entirely rather than at the place a build tool would look for it by default.
    val suite = Path.of("test", "kotlin").toAbsolutePath()
    val sources = Path.of("src").toAbsolutePath()

    assertTrue(Files.isDirectory(suite), "the suite is not where the build looks for it")
    assertFalse(suite.startsWith(sources), "the suite sits under the tree the generator rewrites")
    assertTrue(
      text().contains(
        "<testSourceDirectory>\${project.basedir}/test/kotlin</testSourceDirectory>"
      ),
      "the build does not point at the suite outside the generated tree"
    )
  }

  @Test
  fun theGeneratedTreeCarriesTheBannerOnEveryFile() {
    val unmarked = mutableListOf<String>()
    for (declared in Generated.types()) {
      val file = Generated.ROOT.resolve("${declared.simpleName}.kt").toAbsolutePath()
      val source = Files.readString(file, StandardCharsets.UTF_8)
      if (!source.startsWith("// Generated by hook0-sdkgen") ||
        !source.contains("UPDATE_SDK=kotlin cargo test -p hook0-sdkgen sdk_targets")
      ) {
        unmarked.add(declared.simpleName)
      }
    }

    assertFalse(
      Generated.types().isEmpty(),
      "no generated file was found, so this guard checked nothing"
    )
    assertTrue(
      unmarked.isEmpty(),
      "generated files carry no banner naming what rewrites them: $unmarked"
    )
  }

  @Test
  fun theArtefactIsBuiltForTheReleaseItSaysItIs() {
    assertTrue(
      Runtime.version().feature() >= 21,
      "this suite runs on a runtime older than the release the artefact is built for"
    )
    assertTrue(text().contains("<kotlin.compiler.jvmTarget>21</kotlin.compiler.jvmTarget>"))
    assertTrue(text().contains("<arg>-Werror</arg>"), "the build reads a warning as something else")
  }

  companion object {
    /** What every Kotlin class file links against, which is the language rather than a library. */
    private const val LANGUAGE_RUNTIME = "org.jetbrains.kotlin:kotlin-stdlib"

    private fun project(): Element {
      val pom = Path.of("pom.xml").toAbsolutePath()
      val factory = DocumentBuilderFactory.newInstance()
      // The build file is committed beside this suite, and a reader that resolved what a document
      // points at would still be one more thing a document could ask for.
      factory.setAttribute(XMLConstants.ACCESS_EXTERNAL_DTD, "")
      factory.setAttribute(XMLConstants.ACCESS_EXTERNAL_SCHEMA, "")
      factory.setFeature("http://apache.org/xml/features/disallow-doctype-decl", true)
      return factory.newDocumentBuilder().parse(pom.toFile()).documentElement
    }

    /** The children of that element carrying that name, one level down and no further. */
    private fun children(parent: Element, name: String): List<Element> {
      val found = mutableListOf<Element>()
      val nodes = parent.childNodes
      for (index in 0 until nodes.length) {
        val node: Node = nodes.item(index)
        if (node is Element && node.tagName == name) {
          found.add(node)
        }
      }
      return found
    }

    private fun textOf(parent: Element, name: String): String =
      children(parent, name).firstOrNull()?.textContent?.trim() ?: ""

    /** Every element of that name anywhere in the build file, however deep. */
    private fun everywhere(root: Element, name: String): List<Element> {
      val nodes = root.getElementsByTagName(name)
      return (0 until nodes.length).map { nodes.item(it) as Element }
    }

    private fun unpinned(version: String): Boolean {
      val resolved = if (version.startsWith("\${")) property(version) else version
      return resolved.isEmpty() ||
        resolved.contains("SNAPSHOT") ||
        resolved.contains("LATEST") ||
        resolved.contains("RELEASE") ||
        resolved.startsWith("[") ||
        resolved.startsWith("(")
    }

    private fun property(reference: String): String {
      val named = reference.substring(2, reference.length - 1)
      for (properties in children(project(), "properties")) {
        val value = textOf(properties, named)
        if (value.isNotEmpty()) {
          return value
        }
      }
      return ""
    }

    private fun text(): String = Files.readString(Path.of("pom.xml").toAbsolutePath(), StandardCharsets.UTF_8)
  }
}
