import { createEventTypeSchema, type EventTypeFormValues } from './eventType.schema';

const schema = createEventTypeSchema();

/**
 * The three segments the form holds when nothing is wrong with them. Each case
 * below moves one segment away from this, so a refusal is about the character
 * that moved rather than about a form that was never valid to begin with.
 */
function theForm(overrides: Partial<EventTypeFormValues> = {}): EventTypeFormValues {
  return {
    service: 'order',
    resource_type: 'payment',
    verb: 'completed',
    ...overrides,
  };
}

/** The fields the schema refuses, by name. */
function refusedFields(values: EventTypeFormValues): string[] {
  const result = schema.safeParse(values);
  return result.success ? [] : result.error.issues.map((issue) => issue.path.join('.'));
}

describe('createEventTypeSchema', () => {
  it('accepts conventional segments the API accepts', () => {
    expect(refusedFields(theForm())).toEqual([]);
  });

  it('accepts underscores and digits, as Hook0 own event types use', () => {
    expect(
      refusedFields(
        theForm({ service: 'api', resource_type: 'application_secret', verb: 'created' })
      )
    ).toEqual([]);
  });

  // Every character that would let a stored name build a different URL path than
  // the event type it labels. The frontend interpolates the name into the
  // delete/get URL, so each of these has to be refused before it can be stored.
  it.each([
    ['a slash', '../application'],
    ['a dot', 'a.b'],
    ['a fragment', 'a#b'],
    ['a query', 'a?b'],
    ['a percent', 'a%2f'],
    ['a space', 'a b'],
  ])('refuses %s in the service segment', (_named, segment) => {
    expect(refusedFields(theForm({ service: segment }))).toContain('service');
  });

  it('refuses the exact three-field payload from the disclosure', () => {
    expect(
      refusedFields({
        service: '../application/as/',
        resource_type: './00000000-0000-0000-0000-000000000000#',
        verb: '#',
      })
    ).toEqual(['service', 'resource_type', 'verb']);
  });

  it('refuses a segment past the 50-character bound', () => {
    expect(refusedFields(theForm({ verb: 'a'.repeat(51) }))).toContain('verb');
  });

  it('refuses an empty segment', () => {
    expect(refusedFields(theForm({ resource_type: '' }))).toContain('resource_type');
  });
});
