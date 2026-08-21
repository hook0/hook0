import { JestConfigWithTsJest } from "ts-jest";

/**
 * Coverage is collected on every run rather than behind a flag, because the thresholds below are
 * only enforced when it is. That keeps the gate inside `npm test` — the command the pipeline
 * already runs — instead of a job of its own.
 *
 * The numbers are what the suite reaches today, each truncated to the hundredth rather than
 * rounded, so that a floor can never sit above the run it was read from. Any drop fails. They are
 * not headroom: these tests talk to a loopback socket and draw no randomness, so there is nothing
 * to leave room for, and room would only buy silent decay. Raise them when the suite covers more;
 * lowering one is a decision somebody has to write down here.
 *
 * They are keyed per file rather than globally so that a failure names the file that dropped and
 * by how much, which a single global number cannot do.
 */
const AT_ONE_HUNDRED = { statements: 100, branches: 100, functions: 100, lines: 100 };

const config: JestConfigWithTsJest = {
  preset: "ts-jest",
  testEnvironment: "node",
  transform: {
    "^.+\\.tsx?$": ["ts-jest", { tsconfig: "tsconfig.json", diagnostics: true }],
  },
  testMatch: ["<rootDir>/tests/**/*.test.ts"],
  moduleFileExtensions: ["ts", "js"],
  clearMocks: true,

  collectCoverage: true,
  // What is shipped, and only that: the helpers under `tests/` are not part of the package, and
  // holding them to a number would make the gate about the suite rather than about the client.
  collectCoverageFrom: ["src/**/*.ts"],
  coverageReporters: ["text"],
  coverageThreshold: {
    // Written from the API document and exercised through it, so there is no line of it a caller
    // could reach that the suite does not. Anything less is a generated file nothing drives.
    "src/generated/**/*.ts": AT_ONE_HUNDRED,

    // Re-exports only, so every statement and line of it runs the moment anything imports the
    // package. The one binding short of the mark is a re-export no suite reaches through this
    // module rather than through the file that declares it.
    "src/index.ts": { statements: 100, branches: 100, functions: 93.75, lines: 100 },

    // Short of 100 by design. What is left is: the arm taken when `process` is undefined, which
    // needs a genuine edge runtime and cannot be reached by deleting the global; the two head
    // ceilings this client sets for itself, which the runtime refuses before this code counts them
    // because undici's own ceiling is at least as tight; the walk that gives up after eight links
    // of an error chain; `Signature.parse` throwing rather than answering a falsy value, which
    // makes the guard after it dead; and the `console.debug` calls, which are the one category
    // where a test would assert nothing but that a log line was written.
    "src/lib.ts": { statements: 95.59, branches: 91.27, functions: 98.73, lines: 95.5 },

    // The remainder is the arm a signature carrying neither a `v0` nor a `v1` code would take,
    // which parsing refuses first. Making that state unrepresentable is a change across every
    // client and the generator, not a test.
    "src/signature.ts": { statements: 96.82, branches: 96.66, functions: 100, lines: 96.82 },

    // Every collected file at once, generated half included. Jest computes `global` over the whole
    // of `collectCoverageFrom` rather than over the leftovers the keys above did not claim — which
    // is why this is higher than any per-file number here and not an average of them. It is what a
    // file arriving under `src/` with no threshold of its own still has to clear.
    global: { statements: 97.67, branches: 93.1, functions: 98.89, lines: 97.57 },
  },
};

export default config;
