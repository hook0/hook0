/** @type {import('ts-jest').JestConfigWithTsJest} */
module.exports = {
  preset: 'ts-jest',
  coverageDirectory: 'coverage',
  coverageReporters: ['clover', 'lcov', 'text', 'cobertura', 'text-summary'],
  coverageThreshold: {
    global: {
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
