package com.hook0.client;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.io.IOException;
import java.io.UncheckedIOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import javax.xml.XMLConstants;
import javax.xml.parsers.DocumentBuilder;
import javax.xml.parsers.DocumentBuilderFactory;
import javax.xml.parsers.ParserConfigurationException;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.Timeout;
import org.w3c.dom.Element;
import org.w3c.dom.Node;
import org.w3c.dom.NodeList;
import org.xml.sax.SAXException;

/**
 * What this artefact promises whoever adds it to an application.
 *
 * <p>The promise is that adding it drags nothing else onto the classpath: it reaches the network, verifies signatures
 * and reads what the API answers with nothing but the standard library. These cases are what keep that sentence true
 * rather than aspirational — a dependency that is not test-scoped fails here rather than in somebody's dependency
 * report months later.
 *
 * <p>The build file is read as the document it is rather than searched as text, so a plugin's own dependencies and the
 * ranges an enforcer rule is written with are not mistaken for what this artefact ships.
 */
@Timeout(60)
final class PackagingTest {

  private static Element project() {
    Path pom = Path.of("pom.xml").toAbsolutePath();
    try {
      DocumentBuilderFactory factory = DocumentBuilderFactory.newInstance();
      // The build file is committed beside this suite, and a reader that resolved what a document
      // points at would still be one more thing a document could ask for.
      factory.setAttribute(XMLConstants.ACCESS_EXTERNAL_DTD, "");
      factory.setAttribute(XMLConstants.ACCESS_EXTERNAL_SCHEMA, "");
      factory.setFeature("http://apache.org/xml/features/disallow-doctype-decl", true);
      DocumentBuilder reader = factory.newDocumentBuilder();
      return reader.parse(pom.toFile()).getDocumentElement();
    } catch (ParserConfigurationException | SAXException unreadable) {
      throw new IllegalStateException("the build file does not read as one: " + pom, unreadable);
    } catch (IOException missing) {
      throw new UncheckedIOException("the build file is not where this suite looks for it: " + pom, missing);
    }
  }

  /** The children of that element carrying that name, one level down and no further. */
  private static List<Element> children(Element parent, String name) {
    List<Element> found = new ArrayList<>();
    NodeList nodes = parent.getChildNodes();
    for (int index = 0; index < nodes.getLength(); index++) {
      Node node = nodes.item(index);
      if (node instanceof Element element && element.getTagName().equals(name)) {
        found.add(element);
      }
    }
    return found;
  }

  private static String textOf(Element parent, String name) {
    List<Element> found = children(parent, name);
    return found.isEmpty() ? "" : found.get(0).getTextContent().strip();
  }

  /** Every element of that name anywhere in the build file, however deep. */
  private static List<Element> everywhere(Element root, String name) {
    List<Element> found = new ArrayList<>();
    NodeList nodes = root.getElementsByTagName(name);
    for (int index = 0; index < nodes.getLength(); index++) {
      found.add((Element) nodes.item(index));
    }
    return found;
  }

  @Test
  void theArtefactDeclaresNoRuntimeDependency() {
    Element project = project();
    List<String> carried = new ArrayList<>();

    for (Element block : children(project, "dependencies")) {
      for (Element declared : children(block, "dependency")) {
        if (!"test".equals(textOf(declared, "scope"))) {
          carried.add(textOf(declared, "artifactId"));
        }
      }
    }

    assertTrue(
        carried.isEmpty(),
        "this artefact would drag " + carried + " onto the classpath of every application that adds it");
  }

  @Test
  void everyVersionTheBuildResolvesIsPinned() {
    // What bounds the build's reach. A range or a snapshot lets a pipeline resolve whatever was
    // published since the last run, which is a build nobody chose.
    Element project = project();
    List<String> unpinned = new ArrayList<>();

    for (Element declared : everywhere(project, "dependency")) {
      String version = textOf(declared, "version");
      if (!version.isEmpty() && unpinned(version)) {
        unpinned.add(textOf(declared, "artifactId") + " " + version);
      }
    }
    for (Element declared : everywhere(project, "plugin")) {
      String version = textOf(declared, "version");
      if (version.isEmpty() || unpinned(version)) {
        unpinned.add(textOf(declared, "artifactId") + " " + version);
      }
    }

    assertTrue(unpinned.isEmpty(), "the build resolves what it was not told to resolve: " + unpinned);
    assertTrue(text().contains("<requirePluginVersions/>"), "nothing refuses an unpinned plugin");
    assertTrue(text().contains("<banDynamicVersions>"), "nothing refuses a dynamic version");
  }

  private static boolean unpinned(String version) {
    String resolved = version.startsWith("${") ? property(version) : version;
    return resolved.isEmpty()
        || resolved.contains("SNAPSHOT")
        || resolved.contains("LATEST")
        || resolved.contains("RELEASE")
        || resolved.startsWith("[")
        || resolved.startsWith("(");
  }

  private static String property(String reference) {
    String named = reference.substring(2, reference.length() - 1);
    for (Element properties : children(project(), "properties")) {
      String value = textOf(properties, named);
      if (!value.isEmpty()) {
        return value;
      }
    }
    return "";
  }

  @Test
  void theArtefactIsPublishedUnderTheCoordinatesItsHomeControls() {
    Element project = project();

    assertTrue(
        "com.hook0".equals(textOf(project, "groupId")),
        "the group is not one whose DNS this project controls, which is what a registry verifies");
    assertTrue("hook0-client".equals(textOf(project, "artifactId")));
    assertFalse(children(project, "licenses").isEmpty(), "the artefact names no licence");
  }

  @Test
  void theSuiteSitsOutsideTheTreeTheGeneratorRewrites() {
    // `src/main/java/com/hook0/client/generated` is rewritten wholesale on every regeneration, and
    // the generator removes whatever it did not write. A test file that drifted anywhere under `src`
    // would be deleted without a word, and the coverage it carries with it.
    Path suite = Path.of("test", "java").toAbsolutePath();
    Path sources = Path.of("src").toAbsolutePath();

    assertTrue(Files.isDirectory(suite), "the suite is not where the build looks for it");
    assertFalse(suite.startsWith(sources), "the suite sits under the tree the generator rewrites");
    assertTrue(
        text().contains("<testSourceDirectory>${project.basedir}/test/java</testSourceDirectory>"),
        "the build does not point at the suite outside the generated tree");
  }

  @Test
  void theGeneratedTreeCarriesTheBannerOnEveryFile() {
    List<String> unmarked = new ArrayList<>();
    for (Class<?> declared : Generated.types()) {
      Path file =
          Path.of("src", "main", "java", "com", "hook0", "client", "generated", declared.getSimpleName() + ".java")
              .toAbsolutePath();
      try {
        String source = Files.readString(file, StandardCharsets.UTF_8);
        if (!source.startsWith("// Generated by hook0-sdkgen")
            || !source.contains("UPDATE_SDK=java cargo test -p hook0-sdkgen sdk_targets")) {
          unmarked.add(declared.getSimpleName());
        }
      } catch (IOException unreadable) {
        throw new UncheckedIOException("a generated file cannot be read: " + file, unreadable);
      }
    }

    assertFalse(Generated.types().isEmpty(), "no generated file was found, so this guard checked nothing");
    assertTrue(unmarked.isEmpty(), "generated files carry no banner naming what rewrites them: " + unmarked);
  }

  @Test
  void theArtefactIsBuiltForTheReleaseItSaysItIs() {
    assertTrue(
        Runtime.version().feature() >= 21,
        "this suite runs on a runtime older than the release the artefact is built for");
    assertTrue(text().contains("<maven.compiler.release>21</maven.compiler.release>"));
  }

  private static String text() {
    try {
      return Files.readString(Path.of("pom.xml").toAbsolutePath(), StandardCharsets.UTF_8);
    } catch (IOException unreadable) {
      throw new UncheckedIOException("the build file is not where this suite looks for it", unreadable);
    }
  }
}
