import { describe, expect, test } from '@jest/globals';
import * as fs from 'fs';
import * as path from 'path';

/**
 * Keeps hand-written tests out of the directories the SDK generator owns.
 *
 * `clients/<target>/src` is generator output: it is rewritten wholesale every time the target is
 * regenerated. `clients/<target>/tests` is hand-written and never regenerated. A test file that
 * drifts back into `src` is therefore deleted without a word at the next regeneration, and the
 * coverage it carried goes with it — which is what this guard refuses.
 *
 * Targets are discovered by looking at what is actually in `clients/`, so a language added later is
 * covered without this file being touched.
 */

/**
 * The words that mark a file or directory as holding tests, across naming conventions:
 * `foo.test.ts`, `foo_test.go`, `test_foo.py`, `FooTests.cs`, `foo_spec.rb`, `__tests__/`. These
 * describe the rule itself; no target and no language is named anywhere in this file.
 */
const TEST_WORDS = ['test', 'tests', 'spec', 'specs'];

/** No target nests its sources near this deep; the bound turns a pathological tree into a failure. */
const MAX_DEPTH = 16;

/** Likewise for breadth: a `src` holding more entries than this is not a source tree. */
const MAX_ENTRIES = 20_000;

const CLIENTS_DIRECTORY = path.resolve(__dirname, '..', '..');
const REPOSITORY_DIRECTORY = path.resolve(CLIENTS_DIRECTORY, '..');

/**
 * Splits a file or directory name into the words it is built from, across the separators the
 * ecosystem uses (`.`, `_`, `-`, and camel-case humps).
 */
function words(name: string): string[] {
  return name
    .split(/[.\-_ ]+/)
    .flatMap((part) => part.split(/(?<=[a-z0-9])(?=[A-Z])/))
    .map((word) => word.toLowerCase())
    .filter((word) => word.length > 0);
}

function marksTests(name: string): boolean {
  return words(name).some((word) => TEST_WORDS.includes(word));
}

function isDirectory(candidate: string): boolean {
  return fs.existsSync(candidate) && fs.statSync(candidate).isDirectory();
}

/**
 * Every file or directory under `root` whose name marks it as holding tests. A directory that is
 * itself reported is not descended into: naming what to move is enough.
 */
function testPathsUnder(root: string): string[] {
  const found: string[] = [];
  const queue: { directory: string; depth: number }[] = [{ directory: root, depth: 0 }];
  let visited = 0;

  for (let cursor = 0; cursor < queue.length; cursor += 1) {
    const current = queue[cursor];
    if (current.depth > MAX_DEPTH) {
      throw new Error(`${current.directory} is nested more than ${MAX_DEPTH} directories deep`);
    }

    for (const entry of fs.readdirSync(current.directory, { withFileTypes: true })) {
      visited += 1;
      if (visited > MAX_ENTRIES) {
        throw new Error(`${root} holds more than ${MAX_ENTRIES} entries`);
      }

      const entryPath = path.join(current.directory, entry.name);
      if (marksTests(entry.name)) {
        found.push(entryPath);
        continue;
      }
      // `isDirectory` on a directory entry is false for a symlink, so a link back up the tree is
      // never descended.
      if (entry.isDirectory()) {
        queue.push({ directory: entryPath, depth: current.depth + 1 });
      }
    }
  }

  return found.sort();
}

describe('SDK target layout', () => {
  test('keeps no test under a target’s src', () => {
    const targets = fs
      .readdirSync(CLIENTS_DIRECTORY, { withFileTypes: true })
      .filter((entry) => entry.isDirectory())
      .map((entry) => path.join(CLIENTS_DIRECTORY, entry.name))
      .filter((target) => isDirectory(path.join(target, 'src')))
      .sort();

    expect(targets.length).toBeGreaterThan(0);

    const offenders = targets.flatMap((target) =>
      testPathsUnder(path.join(target, 'src')).map(
        (offender) =>
          `  ${path.relative(REPOSITORY_DIRECTORY, offender)}  ->  ` +
          `${path.relative(REPOSITORY_DIRECTORY, path.join(target, 'tests'))}/`
      )
    );

    if (offenders.length > 0) {
      throw new Error(
        [
          'These tests live under a target’s `src`, which the SDK generator rewrites wholesale on',
          'every regeneration — they would be deleted without a word, and the coverage they carry',
          'with them:',
          '',
          ...offenders,
          '',
          'Move each of them to the `tests` directory of its target, which is hand-written and',
          'never regenerated, and point that target’s test runner at it.',
        ].join('\n')
      );
    }
  });
});
