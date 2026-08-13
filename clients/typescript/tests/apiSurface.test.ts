import { describe, test } from '@jest/globals';
import * as fs from 'fs';

import { extractApiSurface, REPORT_NAME, REPORT_PATH, UPDATE_COMMAND } from './apiSurface';

/** Keeps a runaway diff from burying the instructions at the end of the failure message. */
const MAX_DIFF_LINES_PER_SIDE = 40;

function diffLines(committed: string, actual: string): string[] {
  const committedLines = new Set(committed.split('\n'));
  const actualLines = new Set(actual.split('\n'));

  const removed = [...committedLines].filter((line) => !actualLines.has(line));
  const added = [...actualLines].filter((line) => !committedLines.has(line));

  return [
    ...removed.slice(0, MAX_DIFF_LINES_PER_SIDE).map((line) => `- ${line}`),
    ...added.slice(0, MAX_DIFF_LINES_PER_SIDE).map((line) => `+ ${line}`),
  ];
}

describe('public API surface', () => {
  test(`matches ${REPORT_NAME}`, () => {
    const committed = fs.readFileSync(REPORT_PATH, 'utf8');
    const actual = extractApiSurface();

    if (actual !== committed) {
      throw new Error(
        [
          `The public API surface no longer matches ${REPORT_NAME}:`,
          '',
          ...diffLines(committed, actual),
          '',
          'hook0-client is published on npm, so every exported symbol is a promise to consumers.',
          'A removed or reshaped symbol breaks them and requires a major version bump.',
          '',
          'If this change is intended, run:',
          '',
          `    ${UPDATE_COMMAND}`,
          '',
          `then review and commit the updated ${REPORT_NAME}.`,
        ].join('\n')
      );
    }
  });
});
