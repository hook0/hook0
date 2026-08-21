import { afterAll, beforeAll, describe, expect, test } from '@jest/globals';
import { execFileSync } from 'child_process';
import * as fs from 'fs';
import * as os from 'os';
import * as path from 'path';

/**
 * Drives the built package the way npm hands it to a consumer, through the `exports` map.
 *
 * `npm run check-full` type-checks and tests the sources; none of that says anything about whether
 * `require('hook0-client')` or `import 'hook0-client'` resolve to a file that exists. A package can
 * be perfectly written and still unusable — a condition pointing at a path the build does not emit,
 * a path the `files` list does not publish, an ES module half whose export list drifted from the
 * CommonJS one. Every one of those is invisible to a compiler and obvious to the first consumer who
 * installs it.
 *
 * So the package is installed here, near enough: a directory outside the tree with the package
 * linked under `node_modules`, which is all Node's resolver needs to read `exports` and pick a
 * condition exactly as it would after `npm install`. The probes below are separate processes
 * running real `require` and real `import`.
 */

const PACKAGE_ROOT = path.resolve(__dirname, '..');
const MANIFEST = JSON.parse(
  fs.readFileSync(path.join(PACKAGE_ROOT, 'package.json'), 'utf8')
) as Manifest;

/**
 * The same file read as the fields it carries rather than as the shape declared below, so that the
 * dependency guard looks at what appeared in the manifest instead of at what a type says may.
 */
const MANIFEST_FIELDS = JSON.parse(
  fs.readFileSync(path.join(PACKAGE_ROOT, 'package.json'), 'utf8')
) as Record<string, Record<string, string>>;

/**
 * The manifest fields npm installs into the tree of whoever depends on this package.
 * `devDependencies` is not one of them: it is never installed by a consumer.
 */
const INSTALLED_INTO_CONSUMERS = ['dependencies', 'peerDependencies', 'optionalDependencies'];

/** No probe here reaches the network or the disk beyond a handful of files; a slow one is a hung one. */
const PROBE_TIMEOUT_MS = 60_000;

/** Room for the probe to spawn on a loaded machine, without ever letting a hung one hold the suite. */
const TEST_TIMEOUT_MS = 120_000;

/** What this test reads out of `package.json`. */
interface Manifest {
  name: string;
  main: string;
  module: string;
  types: string;
  files: string[];
  exports: Record<string, ExportCondition | string>;
}

type ExportCondition = { [condition: string]: ExportCondition | string };

let consumer: string;

/**
 * A directory outside the package with the package linked under `node_modules`, so that a bare
 * `hook0-client` specifier resolves through the `exports` map rather than through a relative path.
 */
beforeAll(() => {
  consumer = fs.mkdtempSync(path.join(os.tmpdir(), 'hook0-client-packaging-'));
  fs.mkdirSync(path.join(consumer, 'node_modules'), { recursive: true });
  fs.symlinkSync(PACKAGE_ROOT, path.join(consumer, 'node_modules', MANIFEST.name), 'dir');
  fs.writeFileSync(
    path.join(consumer, 'package.json'),
    JSON.stringify({ name: 'packaging-probe', version: '0.0.0', private: true, type: 'module' })
  );
});

afterAll(() => {
  if (consumer !== undefined) {
    fs.rmSync(consumer, { recursive: true, force: true });
  }
});

/** Runs one probe in its own process and returns what it printed. */
function probe(fileName: string, source: string): string {
  fs.writeFileSync(path.join(consumer, fileName), source);
  return execFileSync(process.execPath, [fileName], {
    cwd: consumer,
    timeout: PROBE_TIMEOUT_MS,
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
  }).trim();
}

/** Every path an `exports` map, `main`, `module` or `types` points at, flattened. */
function declaredPaths(): string[] {
  const found: string[] = [MANIFEST.main, MANIFEST.module, MANIFEST.types];

  const walk = (node: ExportCondition | string): void => {
    if (typeof node === 'string') {
      found.push(node);
      return;
    }
    Object.values(node).forEach(walk);
  };
  Object.values(MANIFEST.exports).forEach(walk);

  return [...new Set(found.map((candidate) => candidate.replace(/^\.\//, '')))].sort();
}

describe('published package', () => {
  test('installs nothing into a consumer', () => {
    // Every client in this repository reaches the network, signs and decodes with what its runtime
    // already ships, and that sentence is worth exactly what the guard behind it is worth. A
    // dependency here is installed by everyone who installs `hook0-client`, whether they wanted it
    // or not — including a bundler polyfill standing in for a builtin, which is how the `url`
    // package got here in the first place.
    const declared = Object.entries(MANIFEST_FIELDS)
      .filter(([field]) => INSTALLED_INTO_CONSUMERS.includes(field))
      .flatMap(([field, named]) => Object.keys(named).map((name) => `${field}.${name}`));

    expect({ declared }).toEqual({ declared: [] });
  });

  test('points every condition at a file the build produces', () => {
    const missing = declaredPaths().filter(
      (declared) => !fs.existsSync(path.join(PACKAGE_ROOT, declared))
    );

    expect({ missing }).toEqual({ missing: [] });
  });

  test('publishes every file a condition points at', () => {
    // `files` entries are anchored at the package root, with or without a leading slash; npm always
    // ships `package.json` whatever the list says.
    const published = new Set(MANIFEST.files.map((entry) => entry.replace(/^\//, '')));

    const unpublished = declaredPaths().filter(
      (declared) => declared !== 'package.json' && !published.has(declared.split('/')[0])
    );

    expect({ unpublished }).toEqual({ unpublished: [] });
  });

  test(
    'hands both module formats the same names, backed by the same objects',
    () => {
      const printed = probe(
        'both.mjs',
        `import { createRequire } from 'module';
         import * as esm from 'hook0-client';
         const require = createRequire(import.meta.url);
         const cjs = require('hook0-client');

         const names = Object.keys(cjs).sort();
         console.log(JSON.stringify({
           esm: Object.keys(esm).filter((name) => name !== 'default').sort(),
           cjs: names,
           differing: names.filter((name) => esm[name] !== cjs[name]),
           defaultIsWholeModule: esm.default === cjs,
           esmFile: import.meta.resolve('hook0-client'),
           cjsFile: require.resolve('hook0-client'),
         }));`
      );

      const resolved = JSON.parse(printed) as {
        esm: string[];
        cjs: string[];
        differing: string[];
        defaultIsWholeModule: boolean;
        esmFile: string;
        cjsFile: string;
      };

      // The names the ES module half declares are the names the CommonJS half has: neither list may
      // drift ahead of the other.
      expect(resolved.esm).toEqual(resolved.cjs);
      expect(resolved.esm.length).toBeGreaterThan(0);

      // And they are the same objects, not equal-looking copies of them. This is what keeps
      // `instanceof Hook0ClientError` answering true across a dependency tree that imported the
      // package both ways.
      expect({ differing: resolved.differing }).toEqual({ differing: [] });

      // `import hook0Client from 'hook0-client'` keeps meaning what it meant before the ES module
      // half existed: the whole module.
      expect(resolved.defaultIsWholeModule).toBe(true);

      // Each condition led somewhere different, or the map is not doing anything.
      expect(resolved.esmFile).toContain(MANIFEST.module);
      expect(resolved.cjsFile).toContain(path.normalize(MANIFEST.main));
      expect(resolved.esmFile).not.toContain(MANIFEST.main);
    },
    TEST_TIMEOUT_MS
  );

  test(
    'keeps the compiled layout out of the contract',
    () => {
      // With an `exports` map in place, the compiled tree stops being reachable — a consumer who
      // reached into `hook0-client/dist/lib` is told so at resolution rather than being quietly
      // handed a second copy of the module. That refusal is the map proving it is in force.
      const printed = probe(
        'deep.mjs',
        `import { createRequire } from 'module';
         const require = createRequire(import.meta.url);
         try {
           require('hook0-client/dist/lib');
           console.log(JSON.stringify({ code: 'RESOLVED' }));
         } catch (refusal) {
           console.log(JSON.stringify({ code: refusal.code }));
         }`
      );

      expect(JSON.parse(printed)).toEqual({ code: 'ERR_PACKAGE_PATH_NOT_EXPORTED' });
    },
    TEST_TIMEOUT_MS
  );
});
