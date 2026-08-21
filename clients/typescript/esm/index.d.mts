/**
 * What TypeScript reads when a consumer resolves `hook0-client` as an ES module.
 *
 * `.d.mts` beside `.mjs` is what makes the ES module half describable at all: a `.d.ts` is a
 * CommonJS declaration file, and pointing the `import` condition at one tells TypeScript the
 * package is CommonJS while Node loads it as an ES module. The two then disagree about what
 * `default` is, and the consumer is told their code type-checks right up until it runs.
 *
 * The surface itself is re-exported rather than restated, so the ES module half cannot describe a
 * type the CommonJS half does not have. `api-surface.md` stays the one place the contract is
 * written down.
 */
export * from '../dist/index.js';

/**
 * The whole package as one object, matching what `esm/index.mjs` exports by default and what Node
 * hands an ES module importing the compiled CommonJS directly.
 */
declare const hook0Client: typeof import('../dist/index.js');
export default hook0Client;
