import * as path from 'path';
import * as ts from 'typescript';

/**
 * Extracts the public API surface of the published package: every symbol reachable from
 * `src/index.ts`, with the signature of every public member.
 *
 * `hook0-client` is published on npm, so its exports are a contract. The extracted surface is
 * compared against the committed report by `apiSurface.test.ts` so that no export can be renamed,
 * removed or reshaped without the report (and therefore a reviewer) noticing.
 */

const PACKAGE_ROOT = path.resolve(__dirname, '..');
const SOURCE_DIR = path.join(PACKAGE_ROOT, 'src');
const ENTRY_POINT = path.join(SOURCE_DIR, 'index.ts');

export const REPORT_PATH = path.join(PACKAGE_ROOT, 'api-surface.md');
export const REPORT_NAME = 'api-surface.md';
export const UPDATE_COMMAND = 'npm run api-surface:update';

/** Type names must never be abbreviated with `...`, or a breaking change could hide behind it. */
const TYPE_FORMAT = ts.TypeFormatFlags.NoTruncation;

/** An exported symbol and the signatures of its public members, ready to be rendered. */
interface ApiEntry {
  kind: string;
  name: string;
  heritage: string;
  members: string[];
}

/** Code-unit ordering: identical on every machine, unlike the locale-dependent `localeCompare`. */
function byText(a: string, b: string): number {
  if (a < b) {
    return -1;
  }
  if (a > b) {
    return 1;
  }
  return 0;
}

/**
 * Read the package `tsconfig.json` so the surface is extracted with the same compiler options as
 * the shipped build.
 */
function compilerOptions(): ts.CompilerOptions {
  const configPath = path.join(PACKAGE_ROOT, 'tsconfig.json');
  const configFile = ts.readConfigFile(configPath, ts.sys.readFile);
  if (configFile.error !== undefined) {
    throw new Error(`Could not read ${configPath}: ${configFile.error.messageText}`);
  }

  const parsed = ts.parseJsonConfigFileContent(configFile.config, ts.sys, PACKAGE_ROOT);
  if (parsed.errors.length > 0) {
    throw new Error(`Could not parse ${configPath}: ${parsed.errors[0].messageText}`);
  }

  return parsed.options;
}

/** A symbol without a declaration cannot be described; that would silently shrink the report. */
function declarationOf(symbol: ts.Symbol): ts.Declaration {
  const declarations = symbol.getDeclarations();
  if (declarations === undefined || declarations.length === 0) {
    throw new Error(`Exported symbol \`${symbol.getName()}\` has no declaration`);
  }
  return declarations[0];
}

/** `export { X } from './lib'` yields an alias symbol; the described symbol is the target. */
function targetOf(checker: ts.TypeChecker, symbol: ts.Symbol): ts.Symbol {
  if ((symbol.flags & ts.SymbolFlags.Alias) !== 0) {
    return checker.getAliasedSymbol(symbol);
  }
  return symbol;
}

function isPublic(member: ts.Symbol): boolean {
  const modifiers = ts.getCombinedModifierFlags(declarationOf(member));
  return (modifiers & (ts.ModifierFlags.Private | ts.ModifierFlags.Protected)) === 0;
}

/**
 * Keeps members the package itself declares. Members inherited from the standard library (the
 * `Error` side of `Hook0ClientError`, for instance) belong to the base type, not to this contract,
 * and their rendering follows whichever `@types/node` and TypeScript lib happen to be installed.
 * The inheritance itself stays visible: it is rendered as the `extends` clause of the entry.
 */
function isDeclaredInPackage(member: ts.Symbol): boolean {
  const fileName = declarationOf(member).getSourceFile().fileName;
  const relative = path.relative(SOURCE_DIR, fileName);
  return relative.length > 0 && !relative.startsWith('..') && !path.isAbsolute(relative);
}

/** `extends`/`implements` clauses are part of the contract even when the members come from a base. */
function heritageOf(declaration: ts.Declaration): string {
  if (!ts.isClassDeclaration(declaration) && !ts.isInterfaceDeclaration(declaration)) {
    return '';
  }
  const clauses = declaration.heritageClauses;
  if (clauses === undefined) {
    return '';
  }
  return clauses.map((clause) => ` ${clause.getText()}`).join('');
}

/** Optionality is part of the contract: making a required parameter optional is not the reverse. */
function nameOf(member: ts.Symbol): string {
  if ((member.flags & ts.SymbolFlags.Optional) !== 0) {
    return `${member.getName()}?`;
  }
  return member.getName();
}

/** Renders a property as `name: type` and a method as one line per call signature. */
function describeMember(checker: ts.TypeChecker, member: ts.Symbol): string[] {
  const declaration = declarationOf(member);
  const type = checker.getTypeOfSymbolAtLocation(member, declaration);
  const callSignatures = type.getCallSignatures();

  if (callSignatures.length > 0) {
    return callSignatures.map(
      (signature) =>
        `${nameOf(member)}${checker.signatureToString(signature, declaration, TYPE_FORMAT)}`
    );
  }

  return [`${nameOf(member)}: ${checker.typeToString(type, declaration, TYPE_FORMAT)}`];
}

function describeMembers(checker: ts.TypeChecker, members: ts.Symbol[]): string[] {
  return members
    .filter(isDeclaredInPackage)
    .filter(isPublic)
    .sort((a, b) => byText(a.getName(), b.getName()))
    .flatMap((member) => describeMember(checker, member));
}

/** Constructors, then static members, then instance members — all sorted, so diffs stay minimal. */
function describeClass(checker: ts.TypeChecker, symbol: ts.Symbol): string[] {
  const declaration = declarationOf(symbol);
  const staticType = checker.getTypeOfSymbolAtLocation(symbol, declaration);
  const instanceType = checker.getDeclaredTypeOfSymbol(symbol);

  const constructors = staticType
    .getConstructSignatures()
    .map(
      (signature) => `constructor${checker.signatureToString(signature, declaration, TYPE_FORMAT)}`
    );

  const statics = describeMembers(
    checker,
    staticType.getProperties().filter((member) => member.getName() !== 'prototype')
  ).map((member) => `static ${member}`);

  return [...constructors, ...statics, ...describeMembers(checker, instanceType.getProperties())];
}

function describeFunction(checker: ts.TypeChecker, symbol: ts.Symbol): string[] {
  const declaration = declarationOf(symbol);
  const type = checker.getTypeOfSymbolAtLocation(symbol, declaration);
  return type
    .getCallSignatures()
    .map(
      (signature) =>
        `${symbol.getName()}${checker.signatureToString(signature, declaration, TYPE_FORMAT)}`
    );
}

/**
 * Describes one exported symbol. An export kind this extractor does not know how to describe
 * throws instead of producing an empty entry, so new surface can never slip in undescribed.
 */
function describeExport(checker: ts.TypeChecker, exported: ts.Symbol): ApiEntry {
  const symbol = targetOf(checker, exported);
  const name = exported.getName();
  const declaration = declarationOf(symbol);
  const heritage = heritageOf(declaration);

  if ((symbol.flags & ts.SymbolFlags.Class) !== 0) {
    return { kind: 'class', name, heritage, members: describeClass(checker, symbol) };
  }
  if ((symbol.flags & ts.SymbolFlags.Interface) !== 0) {
    return {
      kind: 'interface',
      name,
      heritage,
      members: describeMembers(checker, checker.getDeclaredTypeOfSymbol(symbol).getProperties()),
    };
  }
  if ((symbol.flags & ts.SymbolFlags.Function) !== 0) {
    return { kind: 'function', name, heritage, members: describeFunction(checker, symbol) };
  }
  if ((symbol.flags & ts.SymbolFlags.Enum) !== 0) {
    const type = checker.getDeclaredTypeOfSymbol(symbol);
    return {
      kind: 'enum',
      name,
      heritage,
      members: [checker.typeToString(type, declaration, TYPE_FORMAT)],
    };
  }
  if ((symbol.flags & ts.SymbolFlags.TypeAlias) !== 0) {
    // `InTypeAlias` expands the aliased shape instead of printing the alias name back, so that
    // reshaping a type without renaming it still shows up in the report.
    const type = checker.getDeclaredTypeOfSymbol(symbol);
    return {
      kind: 'type',
      name,
      heritage,
      members: [
        checker.typeToString(type, declaration, TYPE_FORMAT | ts.TypeFormatFlags.InTypeAlias),
      ],
    };
  }
  if ((symbol.flags & ts.SymbolFlags.Variable) !== 0) {
    const type = checker.getTypeOfSymbolAtLocation(symbol, declaration);
    return {
      kind: 'const',
      name,
      heritage,
      members: [`${name}: ${checker.typeToString(type, declaration, TYPE_FORMAT)}`],
    };
  }

  throw new Error(
    `Exported symbol \`${name}\` has a kind this extractor cannot describe. ` +
      `Teach tests/apiSurface.ts about it so it appears in ${REPORT_NAME}.`
  );
}

/**
 * One entry per exported symbol, and one per member of an exported namespace.
 *
 * A namespace re-export is a whole surface rather than a single symbol: folding it into one line
 * would leave every signature inside it out of the report, which is exactly what the report exists
 * to stop.
 */
function describeExports(checker: ts.TypeChecker, exported: ts.Symbol): ApiEntry[] {
  const symbol = targetOf(checker, exported);
  if ((symbol.flags & ts.SymbolFlags.Module) === 0) {
    return [describeExport(checker, exported)];
  }

  const namespace = exported.getName();
  return checker
    .getExportsOfModule(symbol)
    .map((member) => describeExport(checker, member))
    .map((entry) => ({ ...entry, name: `${namespace}.${entry.name}` }));
}

function render(entries: ApiEntry[]): string {
  const header = [
    '# hook0-client — public API surface',
    '',
    'Everything the published npm package exports from `src/index.ts`, with the signature of every',
    'public member. This file is the contract consumers depend on.',
    '',
    `Generated — do not edit by hand. Regenerate with \`${UPDATE_COMMAND}\`.`,
    '',
    '`tests/apiSurface.test.ts` fails when the code and this file disagree. Renaming, removing',
    'or reshaping anything below breaks consumers and requires a major version bump; adding to it',
    'requires a minor one.',
  ];

  const body = entries.flatMap((entry) => [
    '',
    `## ${entry.kind} ${entry.name}${entry.heritage}`,
    '',
    '```ts',
    ...entry.members,
    '```',
  ]);

  return [...header, ...body, ''].join('\n');
}

/**
 * Builds the public API surface report from the current sources.
 * @returns The report, in the exact form expected on disk.
 */
export function extractApiSurface(): string {
  const program = ts.createProgram([ENTRY_POINT], compilerOptions());
  const checker = program.getTypeChecker();

  const entryPoint = program.getSourceFile(ENTRY_POINT);
  if (entryPoint === undefined) {
    throw new Error(`Could not load the package entry point ${ENTRY_POINT}`);
  }

  const moduleSymbol = checker.getSymbolAtLocation(entryPoint);
  if (moduleSymbol === undefined) {
    throw new Error(`${ENTRY_POINT} exports nothing: the package has no public API surface`);
  }

  const entries = checker
    .getExportsOfModule(moduleSymbol)
    .flatMap((exported) => describeExports(checker, exported))
    .sort((a, b) => byText(a.name, b.name));

  return render(entries);
}
