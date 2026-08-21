/**
 * What `import 'hook0-client'` resolves to.
 *
 * The package is compiled to CommonJS, and this file is the ES module half of it: it names every
 * export the package has, so an ES module consumer gets them as declared exports rather than as
 * whatever a lexer manages to guess from the compiled CommonJS. Node reads named exports out of a
 * CommonJS file by scanning it, and the scan is a heuristic — it happens to understand the shape
 * `tsc` emits today, and nothing promises it will understand the shape `tsc` emits tomorrow. A
 * consumer would discover that as `SyntaxError: The requested module does not provide an export
 * named 'Hook0Client'`, at import time, in production. The list below is that promise, written
 * down.
 *
 * It is a wrapper rather than a second compilation of `src` on purpose. Two compilations mean two
 * copies of every class at runtime whenever a dependency tree pulls the package in both ways,
 * Node's dual-package hazard, and this package cannot afford it: `EventType.fromString` answers
 * `EventType | Hook0ClientError`, so a caller has to write `instanceof Hook0ClientError` to tell an
 * answer from a refusal. Against a `Hook0ClientError` built by the other copy that test answers
 * `false`, and a refused event type reads as a parsed one. Every `Hook0ClientError` this package
 * throws is caught the same way and has the same problem.
 * Re-exporting the single compiled module keeps one class per name, so
 * `instanceof` holds no matter which way each half of the dependency tree imported the package.
 *
 * `tests/packaging.test.ts` fails when this list and the compiled module disagree.
 */
import hook0Client from '../dist/index.js';

export const {
  DEFAULT_MAX_PAYLOAD_BYTES,
  DEFAULT_MAX_RESPONSE_BYTES,
  DEFAULT_REQUEST_TIMEOUT_MS,
  MAX_ATTEMPTS_CAP,
  MAX_HEAD_BYTES,
  MAX_HEADER_BYTES,
  MAX_RESPONSE_HEADERS,
  Hook0ClientError,
  Hook0Client,
  Hook0ClientOptions,
  RetryPolicy,
  Event,
  EventType,
  verifyWebhookSignature,
  verifyWebhookSignatureWithCurrentTime,
  Signature,
  generated,
} = hook0Client;

/**
 * The whole package as one object.
 *
 * `import hook0Client from 'hook0-client'` already worked before this file existed, because that is
 * what Node hands an ES module importing a CommonJS one. Keeping it means the ES module half is an
 * addition rather than a swap: no consumer's import breaks on the version that introduces it.
 */
export default hook0Client;
