import {
  NO_COOLDOWN,
  readCooldownStart,
  remainingCooldownSeconds,
  writeCooldownStart,
  type CooldownStart,
  type CooldownStorage,
} from './cooldown';

/** The storage key the module owns, asserted on from the outside. */
const STORAGE_KEY = 'hook0.resend-verification-cooldown';

/**
 * A real, in-memory implementation of the storage contract — the same semantics
 * a browser gives (string in, string out, `null` when absent), just without a
 * browser. The module under test is still driven through its public surface
 * only.
 */
function memoryStorage(): CooldownStorage & { keys(): string[] } {
  const entries = new Map<string, string>();
  return {
    getItem: (key) => {
      const stored = entries.get(key);
      return stored === undefined ? null : stored;
    },
    setItem: (key, value) => {
      entries.set(key, value);
    },
    keys: () => [...entries.keys()],
  };
}

/** Storage that is present but refuses every operation, as blocked storage does. */
function refusingStorage(): CooldownStorage {
  return {
    getItem: () => {
      throw new Error('storage is not available');
    },
    setItem: () => {
      throw new Error('storage is not available');
    },
  };
}

describe('remainingCooldownSeconds', () => {
  const startedAtMs = 1_000_000;
  const start: CooldownStart = { kind: 'started', atMs: startedAtMs };

  it('reads as the full duration at the very start', () => {
    expect(remainingCooldownSeconds(start, 60, startedAtMs)).toBe(60);
  });

  it('counts down as time passes', () => {
    expect(remainingCooldownSeconds(start, 60, startedAtMs + 1_000)).toBe(59);
    expect(remainingCooldownSeconds(start, 60, startedAtMs + 30_500)).toBe(30);
  });

  it('rounds up so the last visible value is 1', () => {
    expect(remainingCooldownSeconds(start, 60, startedAtMs + 59_100)).toBe(1);
  });

  it('is 0 exactly when and after the cooldown elapses', () => {
    expect(remainingCooldownSeconds(start, 60, startedAtMs + 60_000)).toBe(0);
    expect(remainingCooldownSeconds(start, 60, startedAtMs + 120_000)).toBe(0);
  });

  it('is 0 for a cooldown that never started', () => {
    expect(remainingCooldownSeconds(NO_COOLDOWN, 60, startedAtMs)).toBe(0);
  });

  it('never exceeds the duration even with clock skew', () => {
    expect(remainingCooldownSeconds(start, 60, startedAtMs - 5_000)).toBe(60);
  });
});

describe('cooldown persistence', () => {
  const email = 'someone@example.com';

  it('reads back a cooldown written for the same address', () => {
    const storage = memoryStorage();
    writeCooldownStart(storage, email, 1_700_000_000_000);

    expect(readCooldownStart(storage, email)).toEqual({
      kind: 'started',
      atMs: 1_700_000_000_000,
    });
  });

  it('resumes with the time left rather than restarting the countdown', () => {
    // What a page reload 20s into a 60s cooldown has to show.
    const storage = memoryStorage();
    const writtenAt = 1_700_000_000_000;
    writeCooldownStart(storage, email, writtenAt);

    const afterReload = readCooldownStart(storage, email);
    expect(remainingCooldownSeconds(afterReload, 60, writtenAt + 20_000)).toBe(40);
  });

  it('keeps a single entry however many addresses are recorded', () => {
    const storage = memoryStorage();
    writeCooldownStart(storage, 'first@example.com', 1);
    writeCooldownStart(storage, 'second@example.com', 2);
    writeCooldownStart(storage, 'third@example.com', 3);

    expect(storage.keys()).toEqual([STORAGE_KEY]);
  });

  it('never applies a cooldown recorded for another address', () => {
    const storage = memoryStorage();
    writeCooldownStart(storage, 'someone-else@example.com', 1_700_000_000_000);

    expect(readCooldownStart(storage, email)).toEqual(NO_COOLDOWN);
  });

  it('reports no cooldown when nothing was ever recorded', () => {
    expect(readCooldownStart(memoryStorage(), email)).toEqual(NO_COOLDOWN);
  });

  it.each([
    ['unreadable JSON', 'not json at all'],
    ['JSON null', 'null'],
    ['a bare string', '"just a string"'],
    ['no timestamp', JSON.stringify({ email })],
    ['a non-numeric timestamp', JSON.stringify({ email, startedAtMs: 'soon' })],
    ['a non-finite timestamp', `{"email":"${email}","startedAtMs":1e999}`],
  ])('reports no cooldown when the stored record holds %s', (_case, stored) => {
    const storage = memoryStorage();
    storage.setItem(STORAGE_KEY, stored);

    expect(readCooldownStart(storage, email)).toEqual(NO_COOLDOWN);
  });

  it('reports no cooldown when storage refuses to be read', () => {
    expect(readCooldownStart(refusingStorage(), email)).toEqual(NO_COOLDOWN);
  });

  it('gives up quietly when storage refuses to be written', () => {
    expect(() => writeCooldownStart(refusingStorage(), email, 1_700_000_000_000)).not.toThrow();
  });
});
