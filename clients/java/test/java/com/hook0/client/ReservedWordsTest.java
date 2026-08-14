package com.hook0.client;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.net.URI;
import java.util.ArrayList;
import java.util.List;
import java.util.TreeSet;
import javax.tools.DiagnosticCollector;
import javax.tools.JavaCompiler;
import javax.tools.JavaFileObject;
import javax.tools.SimpleJavaFileObject;
import javax.tools.StandardJavaFileManager;
import javax.tools.ToolProvider;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.Timeout;

/**
 * What the language actually refuses where the generator puts a name, measured rather than recalled.
 *
 * <p>The vocabulary the generator spells names out of the way of is a table, and a table is a claim. These cases hold
 * that claim against the two things that can settle it: the compiler shipped with the runtime, which is what decides
 * whether an identifier is legal, and {@code java.lang.Object} itself, which is what decides which names every value
 * already answers to.
 *
 * <p>The answer is positional, but not the way it is in every language. A keyword is illegal <em>everywhere</em> an
 * identifier goes — as a field, as an argument, as a method and as a type alike — which is the opposite of a language
 * where {@code def next} is a perfectly good method name. What is positional here is the other set: the no-argument
 * methods of {@code Object} are legal as a field and as an argument and illegal as a method, and a {@code record}
 * component is both at once.
 */
@Timeout(300)
final class ReservedWordsTest {

  /** How many candidates one case compiles, which bounds what it costs. */
  private static final int MAX_CANDIDATES = 128;

  /** The words the language keeps for itself, including the two it took back and the three literals. */
  private static final List<String> KEYWORDS =
      List.of(
          "abstract", "assert", "boolean", "break", "byte", "case", "catch", "char", "class", "const", "continue",
          "default", "do", "double", "else", "enum", "extends", "final", "finally", "float", "for", "goto", "if",
          "implements", "import", "instanceof", "int", "interface", "long", "native", "new", "package", "private",
          "protected", "public", "return", "short", "static", "strictfp", "super", "switch", "synchronized", "this",
          "throw", "throws", "transient", "try", "void", "volatile", "while", "_", "true", "false", "null");

  /** Words that are restricted only where a type is named, and legal everywhere else. */
  private static final List<String> CONTEXTUAL = List.of("record", "sealed", "permits", "var", "yield");

  /**
   * The names every value already answers to and no record component may carry.
   *
   * <p>Read off {@code Object} rather than written down, so the day the class grows one this suite says so.
   */
  static TreeSet<String> objectMethodsTakingNoArgument() {
    TreeSet<String> named = new TreeSet<>();
    for (Method method : Object.class.getDeclaredMethods()) {
      if (method.getParameterCount() == 0 && !Modifier.isStatic(method.getModifiers())) {
        named.add(method.getName());
      }
    }
    return named;
  }

  @Test
  void everyKeywordIsRefusedWhereverAnIdentifierGoes() {
    TreeSet<String> accepted = new TreeSet<>();
    for (String word : bounded(KEYWORDS)) {
      if (compiles("public class Probe { private String " + word + "; }")
          || compiles("public class Probe { void " + word + "() {} }")
          || compiles("public class Probe { void m(String " + word + ") {} }")
          || compiles("public class Probe { record R(String " + word + ") {} }")) {
        accepted.add(word);
      }
    }

    assertTrue(accepted.isEmpty(), "the compiler accepts words this suite calls keywords: " + accepted);
  }

  @Test
  void whatEveryObjectAnswersToIsLegalAsAFieldAndRefusedAsARecordComponent() {
    // This is the distinction that decides the shape of the table: one set is refused in every
    // position, the other only where a method is declared — and a record component is a method.
    TreeSet<String> answered = objectMethodsTakingNoArgument();
    assertFalse(answered.isEmpty(), "no no-argument method of Object was found, so this measured nothing");

    for (String word : bounded(new ArrayList<>(answered))) {
      assertTrue(
          compiles("public class Probe { private String " + word + "; }"),
          "`" + word + "` is refused as a field, so it belongs to the keyword half rather than this one");
      assertFalse(
          compiles("public class Probe { record R(String " + word + ") {} }"),
          "`" + word + "` is accepted as a record component, so the generator need not spell it out of the way");
    }

    // `equals` takes an argument, so a no-argument one overloads it rather than replacing it, and a
    // component of that name compiles. That is why it is deliberately absent from the table.
    assertFalse(answered.contains("equals"));
    assertTrue(compiles("public class Probe { record R(String equals) {} }"));
  }

  @Test
  void aRestrictedTypeIdentifierIsLegalWhereverThisTargetPutsAName() {
    // `record`, `sealed`, `permits`, `var` and `yield` are refused only where a type is named, and a
    // type name is rendered upper camel here, so none of them is a name this target can produce.
    for (String word : bounded(CONTEXTUAL)) {
      assertTrue(
          compiles("public class Probe { record R(String " + word + ") {} }"),
          "`" + word + "` is refused as a record component, so it belongs in the table after all");
      assertFalse(
          compiles("public class Probe { static class " + word + " {} }"),
          "`" + word + "` is accepted as a type name, so this case is measuring the wrong thing");
    }
  }

  @Test
  void everyNameTheGeneratorDeclaredCompilesWhereItPutIt() {
    // The whole generated tree is on the classpath of this suite, so it already compiled; what this
    // adds is that the names it declared are the ones a caller reaches, rather than names javac
    // happened to tolerate somewhere else.
    TreeSet<String> keywords = new TreeSet<>(KEYWORDS);
    TreeSet<String> offenders = new TreeSet<>();

    for (Class<?> declared : Generated.types()) {
      for (Method method : declared.getDeclaredMethods()) {
        if (Modifier.isPublic(method.getModifiers()) && keywords.contains(method.getName())) {
          offenders.add(declared.getSimpleName() + "." + method.getName());
        }
      }
    }

    assertTrue(offenders.isEmpty(), "the generator declared names the language keeps for itself: " + offenders);
  }

  private static List<String> bounded(List<String> candidates) {
    assertTrue(
        candidates.size() <= MAX_CANDIDATES,
        "this case would compile " + candidates.size() + " candidates, above the " + MAX_CANDIDATES + " it is given");
    return candidates;
  }

  /** Whether the compiler shipped with this runtime accepts that source. */
  private static boolean compiles(String source) {
    JavaCompiler compiler = ToolProvider.getSystemJavaCompiler();
    assertNotNull(compiler, "this runtime ships no compiler, so nothing here can be measured");

    DiagnosticCollector<JavaFileObject> diagnostics = new DiagnosticCollector<>();
    try (StandardJavaFileManager files = compiler.getStandardFileManager(diagnostics, null, null)) {
      return compiler
          .getTask(
              null,
              files,
              diagnostics,
              List.of("-d", System.getProperty("java.io.tmpdir") + "/hook0-probe"),
              null,
              List.of(new Probe(source)))
          .call()
          .booleanValue();
    } catch (java.io.IOException unusable) {
      throw new IllegalStateException("the compiler could not be run", unusable);
    }
  }

  /** One source held in memory, which is what lets a case ask the compiler a question. */
  private static final class Probe extends SimpleJavaFileObject {

    private final String source;

    Probe(String source) {
      super(URI.create("string:///Probe.java"), Kind.SOURCE);
      this.source = source;
    }

    @Override
    public CharSequence getCharContent(boolean ignoreEncodingErrors) {
      return source;
    }
  }
}
