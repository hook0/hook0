import { checkEmailHandoverState, readCheckEmailHandover } from './checkEmailHandover';
import { NO_COOLDOWN } from './cooldown';

const email = 'someone@example.com';

describe('check-email hand-off', () => {
  it('carries the address and the send the caller just made', () => {
    const state = checkEmailHandoverState(email, { kind: 'started', atMs: 1_700_000_000_000 });

    expect(readCheckEmailHandover(state)).toEqual({
      kind: 'address',
      email,
      lastVerificationSend: { kind: 'started', atMs: 1_700_000_000_000 },
    });
  });

  it('carries the address alone when the caller sent nothing', () => {
    const state = checkEmailHandoverState(email, NO_COOLDOWN);

    expect(readCheckEmailHandover(state)).toEqual({
      kind: 'address',
      email,
      lastVerificationSend: NO_COOLDOWN,
    });
  });

  it('never puts the address anywhere but the state', () => {
    // The whole reason the hand-off exists: the address must not end up
    // anywhere the router would turn into a URL.
    expect(Object.keys(checkEmailHandoverState(email, NO_COOLDOWN))).toEqual(['email']);
  });

  it('survives the bookkeeping keys the router adds to history state', () => {
    // vue-router merges its own scroll/position keys into the state it stores.
    const state = {
      back: '/register',
      current: '/check-email',
      position: 12,
      scroll: { left: 0, top: 0 },
      ...checkEmailHandoverState(email, { kind: 'started', atMs: 42 }),
    };

    expect(readCheckEmailHandover(state)).toEqual({
      kind: 'address',
      email,
      lastVerificationSend: { kind: 'started', atMs: 42 },
    });
  });

  it.each([
    ['no state at all', null],
    ['a state that is not an object', 'someone@example.com'],
    ['a state carrying no address', { position: 3 }],
    ['a non-string address', { email: 42 }],
    ['an empty address', { email: '' }],
    ['an array', ['someone@example.com']],
  ])('offers no resend target for %s', (_case, state) => {
    expect(readCheckEmailHandover(state)).toEqual({ kind: 'none' });
  });

  it.each([
    ['not a number', 'just now'],
    ['not finite', Number.POSITIVE_INFINITY],
  ])('reads a send stamp that is %s as no send declared', (_case, verificationEmailSentAtMs) => {
    expect(readCheckEmailHandover({ email, verificationEmailSentAtMs })).toEqual({
      kind: 'address',
      email,
      lastVerificationSend: NO_COOLDOWN,
    });
  });
});
