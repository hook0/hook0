import { isUnfulfilled } from './unfulfilled';

const problem = (id: string) => ({ id, detail: 'd' });

describe('isUnfulfilled', () => {
  // The two handleError synthesises, and the two the API pronounces when it
  // failed before deciding anything about the address.
  it.each(['TimeoutExceeded', 'unknown', 'InternalServerError', 'ServiceUnavailable'])(
    'treats %s as unfulfilled',
    (id) => {
      expect(isUnfulfilled(problem(id))).toBe(true);
    }
  );

  // Anything the API decided about the request itself is an answer, however
  // unwelcome. Reading one of these as "no answer" would put a verdict about
  // the address on screen.
  it.each(['RateLimited', 'Validation', 'AuthFailedLogin', 'Forbidden', 'NotFound'])(
    'treats %s as an answer',
    (id) => {
      expect(isUnfulfilled(problem(id))).toBe(false);
    }
  );

  // The same body reached through a rejection that never went via
  // unwrapResponse: read from response.data rather than from the top level.
  it('reads a problem still wrapped in an Axios error', () => {
    expect(isUnfulfilled({ response: { data: problem('ServiceUnavailable') } })).toBe(true);
    expect(isUnfulfilled({ response: { data: problem('RateLimited') } })).toBe(false);
  });

  it.each([
    ['null', null],
    ['undefined', undefined],
    ['a string', 'unknown'],
    ['a number', 500],
    ['an object with no id', { detail: 'd' }],
    ['an object whose id is not a string', { id: 42, detail: 'd' }],
    ['a problem with no detail', { id: 'unknown' }],
  ])('does not mistake %s for an unfulfilled request', (_label, value) => {
    expect(isUnfulfilled(value)).toBe(false);
  });
});
