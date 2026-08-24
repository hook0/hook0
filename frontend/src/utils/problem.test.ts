import fc from 'fast-check';

import { isFieldRefused, problemCarriedBy, validationErrorsFor } from './problem';

// The body the API actually answers with, copied from `ProblemDetails`
// (api/src/problems.rs) rather than paraphrased.
const problem = {
  id: 'Validation',
  status: 422,
  title: 'Provided input is malformed',
  detail: 'email: Value is not a valid email address',
  validation: {
    email: [{ code: 'length', message: null, params: { max: 100 } }],
  },
};

describe('problemCarriedBy', () => {
  it('reads a Problem already unwrapped by the HTTP layer', () => {
    expect(problemCarriedBy(problem)).toEqual({ carries: true, problem });
  });

  it('reads a raw Axios error carrying the Problem in its response', () => {
    expect(problemCarriedBy({ isAxiosError: true, response: { data: problem } })).toEqual({
      carries: true,
      problem,
    });
  });

  // Both members are read by callers, so a body missing either one is not a
  // problem body: guessing would hand `undefined` to a form field.
  it('refuses a body that is missing a member callers read', () => {
    expect(problemCarriedBy({ id: 'RateLimited' }).carries).toBe(false);
    expect(problemCarriedBy({ detail: 'Slow down.' }).carries).toBe(false);
    expect(problemCarriedBy({ id: 429, detail: 'Slow down.' }).carries).toBe(false);
  });

  it('carries nothing when there is nothing to read', () => {
    expect(problemCarriedBy(null).carries).toBe(false);
    expect(problemCarriedBy(undefined).carries).toBe(false);
    expect(problemCarriedBy('boom').carries).toBe(false);
    expect(problemCarriedBy(new Error('network')).carries).toBe(false);
    expect(problemCarriedBy({ response: null }).carries).toBe(false);
    expect(problemCarriedBy({ response: 'gateway timeout' }).carries).toBe(false);
    expect(problemCarriedBy({ response: { data: '<html>502</html>' } }).carries).toBe(false);
  });

  it('survives anything a rejected promise can carry', () => {
    fc.assert(
      fc.property(fc.anything(), (value) => typeof problemCarriedBy(value).carries === 'boolean'),
      { numRuns: 500, seed: 20260824 }
    );
  });
});

describe('validationErrorsFor', () => {
  it('names the field the API pinned the refusal on', () => {
    expect(validationErrorsFor(problem, 'email')).toEqual({
      named: true,
      entries: problem.validation.email,
    });
  });

  it('leaves a field the refusal says nothing about alone', () => {
    expect(validationErrorsFor(problem, 'password')).toEqual({ named: false });
  });

  // Every problem that is not a `Validation` 422 has no map at all, and a
  // handful of malformed-payload errors carry `validation: null`. Reading a
  // key off either is how a diagnosis becomes a crash.
  it('reads no field out of a body that carries no validation map', () => {
    expect(validationErrorsFor({ id: 'NotFound', detail: 'Nothing here.' }, 'email').named).toBe(
      false
    );
    expect(
      validationErrorsFor(
        { id: 'JsonPayload', detail: 'missing field `email`', validation: null } as never,
        'email'
      ).named
    ).toBe(false);
    expect(
      validationErrorsFor({ id: 'Validation', detail: 'x', validation: 'email' } as never, 'email')
        .named
    ).toBe(false);
    expect(
      validationErrorsFor(
        { id: 'Validation', detail: 'x', validation: { email: 'too long' } } as never,
        'email'
      ).named
    ).toBe(false);
  });
});

describe('isFieldRefused', () => {
  // The whole reason this is safe to show on a page that must not say whether
  // an address is registered: the request struct's validators run before any
  // lookup, so the verdict is about the characters sent and nothing else.
  it('recognises a refusal the API pinned on the field', () => {
    expect(isFieldRefused(problem, 'email')).toBe(true);
    expect(isFieldRefused({ isAxiosError: true, response: { data: problem } }, 'email')).toBe(true);
  });

  it('does not read a refusal about one field as a refusal about another', () => {
    expect(isFieldRefused(problem, 'token')).toBe(false);
  });

  it('claims nothing for a failure that carries no problem body', () => {
    expect(isFieldRefused(new Error('network'), 'email')).toBe(false);
  });

  it('survives anything a rejected promise can carry', () => {
    fc.assert(
      fc.property(
        fc.anything(),
        fc.string(),
        (value, field) => typeof isFieldRefused(value, field) === 'boolean'
      ),
      { numRuns: 500, seed: 20260824 }
    );
  });
});
