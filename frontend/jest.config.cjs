/** @type {import('ts-jest').JestConfigWithTsJest} */
module.exports = {
  preset: 'ts-jest',
  coverageDirectory: 'coverage',
  coverageReporters: ['clover', 'lcov', 'text', 'cobertura', 'text-summary'],
  // A floor read off a measurement, not a target nobody meets. These were all 100 and
  // enforced by nothing, because no command passed `--coverage`, and a threshold nothing
  // evaluates is a number rather than a guard.
  //
  // The two modules the send-event screen renders through are held at 100 by name. They
  // are pure and were written to be reachable from tests, which is the whole reason that
  // logic does not live in the component.
  //
  // There is no `collectCoverageFrom`, so jest measures only the modules the suite actually
  // imports — a module nothing imports cannot lower the floor, and the figures below are read
  // off those imported modules rather than the whole `src` tree. Naming a file moves it out of
  // that global pool, so these four are what is left once the two above are taken out, rounded
  // down to whole percents. They were measured under this arrangement, the one they are judged
  // by, so that adding the per-file entries does not itself turn them red. The rounding is what
  // keeps a difference between two Node versions from turning red while a real regression still
  // does.
  coverageThreshold: {
    global: {
      branches: 98,
      functions: 93,
      lines: 97,
      statements: 97,
    },
    './src/pages/organizations/applications/events/sendEventSnippets.ts': {
      branches: 100,
      functions: 100,
      lines: 100,
      statements: 100,
    },
    './src/pages/organizations/applications/events/sendEvent.schema.ts': {
      branches: 100,
      functions: 100,
      lines: 100,
      statements: 100,
    },
  },
  testEnvironment: 'node',
  // Mirrors the `@/*` alias from tsconfig.json. Without it the only testable
  // modules are those importing nothing but relative paths, which left the
  // validation schemas — the code deciding what the user is allowed to submit —
  // outside the suite.
  moduleNameMapper: {
    '^@/(.*)$': '<rootDir>/src/$1',
  },
};
