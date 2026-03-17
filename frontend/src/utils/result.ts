/**
 * Result type for wrapping synchronous calls that may throw.
 *
 * Provides a discriminated union pattern to avoid try/catch blocks in
 * application code. The `ok` flag discriminates between success and failure,
 * allowing TypeScript to narrow the type automatically.
 */
export type Result<T> = { ok: true; value: T } | { ok: false; error: Error };

/**
 * Wraps a synchronous function call that may throw, returning a Result
 * instead of requiring a try/catch block at the call site.
 *
 * @param fn - A zero-argument function to invoke
 * @returns A Result containing either the return value or the caught Error
 */
export function trySyncCall<T>(fn: () => T): Result<T> {
  // This is the ONE place where try/catch is permitted for synchronous code.
  // All callers use the Result discriminated union instead of try/catch.
  // eslint-disable-next-line no-restricted-syntax
  try {
    return { ok: true, value: fn() };
  } catch (e) {
    return { ok: false, error: e instanceof Error ? e : new Error(String(e)) };
  }
}
