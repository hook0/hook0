import { completeVerifiedSession } from './verifiedSessionFlow';
import type { components } from '../../types';

type LoginResponse = components['schemas']['LoginResponse'];

const session: LoginResponse = {
  access_token: 'access-token',
  access_token_expiration: '2026-01-01T00:00:00Z',
  email: 'user@example.com',
  first_name: 'Jane',
  last_name: 'Doe',
  refresh_token: 'refresh-token',
  refresh_token_expiration: '2026-01-02T00:00:00Z',
  user_id: '00000000-0000-0000-0000-000000000000',
};

describe('completeVerifiedSession', () => {
  it('happy path: navigation succeeds, no Home fallback, resolves', () => {
    const setSession = jest.fn((_session: LoginResponse) => Promise.resolve());
    const navigateAfterAuth = jest.fn(() => Promise.resolve('navigated'));
    const goHome = jest.fn(() => Promise.resolve('home'));

    return completeVerifiedSession({ session, setSession, navigateAfterAuth, goHome }).then(
      (result) => {
        expect(setSession).toHaveBeenCalledTimes(1);
        expect(setSession).toHaveBeenCalledWith(session);
        expect(navigateAfterAuth).toHaveBeenCalledTimes(1);
        expect(goHome).not.toHaveBeenCalled();
        expect(result).toBe('navigated');
      }
    );
  });

  it('nav failure after session: falls back to Home, still resolves, no error surfaced', () => {
    const navError = new Error('navigation probe failed');
    const setSession = jest.fn((_session: LoginResponse) => Promise.resolve());
    const navigateAfterAuth = jest.fn(() => Promise.reject(navError));
    const goHome = jest.fn(() => Promise.resolve('home'));
    const consoleError = jest.spyOn(console, 'error').mockImplementation(() => undefined);

    // The promise resolving (instead of rejecting) is what guarantees the
    // caller's `.catch(displayError)` is NOT triggered: the user stays logged in.
    return completeVerifiedSession({ session, setSession, navigateAfterAuth, goHome })
      .then((result) => {
        expect(setSession).toHaveBeenCalledTimes(1);
        expect(navigateAfterAuth).toHaveBeenCalledTimes(1);
        expect(goHome).toHaveBeenCalledTimes(1);
        expect(result).toBe('home');
      })
      .finally(() => consoleError.mockRestore());
  });

  it('setSession failure: error propagates, navigation never runs', () => {
    const sessionError = new Error('token invalid');
    const setSession = jest.fn((_session: LoginResponse) => Promise.reject(sessionError));
    const navigateAfterAuth = jest.fn(() => Promise.resolve('navigated'));
    const goHome = jest.fn(() => Promise.resolve('home'));

    return completeVerifiedSession({ session, setSession, navigateAfterAuth, goHome }).then(
      () => {
        throw new Error('expected the promise to reject');
      },
      (err: unknown) => {
        expect(err).toBe(sessionError);
        expect(navigateAfterAuth).not.toHaveBeenCalled();
        expect(goHome).not.toHaveBeenCalled();
      }
    );
  });
});
