import { sendEventSchema, type SendEventForm } from './sendEvent.schema';

/**
 * What the form holds when nothing is wrong with it.
 *
 * Everything below moves one thing away from this, so a message that appears is about the thing
 * that moved rather than about a form that was never valid to begin with.
 */
function theForm(overrides: Partial<SendEventForm> = {}): SendEventForm {
  return {
    eventType: 'user.account.created',
    labels: [{ key: 'user_id', value: '1' }],
    occurredAt: '2026-08-27T09:15',
    payload: '{"test": true}',
    ...overrides,
  };
}

/** Everything the schema refuses about a form, in its own words. */
function refusals(form: SendEventForm): string[] {
  const checked = sendEventSchema.safeParse(form);
  return checked.success ? [] : checked.error.issues.map((issue) => issue.message);
}

function labelsOf(count: number): SendEventForm['labels'] {
  return [...Array(count).keys()].map((index) => ({
    key: `k${index}`,
    value: `${index}`,
  }));
}

describe('sendEventSchema', () => {
  it('takes an event the API would take', () => {
    expect(refusals(theForm())).toEqual([]);
  });

  // The bounds the API applies, each of which used to be reached only by the server: the form left
  // Send enabled, twelve panels printed the request, and the call came back a 400.
  it('takes the last label the API would take', () => {
    expect(refusals(theForm({ labels: labelsOf(10) }))).toEqual([]);
  });

  it('refuses the eleventh label, which the API refuses outright', () => {
    expect(refusals(theForm({ labels: labelsOf(11) }))).toContain('At most 10 labels are allowed');
  });

  it('takes a key and a value of the longest length the API takes', () => {
    const longest = 'x'.repeat(50);
    expect(refusals(theForm({ labels: [{ key: longest, value: longest }] }))).toEqual([]);
  });

  it('refuses a key longer than the API accepts', () => {
    expect(refusals(theForm({ labels: [{ key: 'x'.repeat(51), value: '1' }] }))).toContain(
      'Label key must be at most 50 bytes'
    );
  });

  it('refuses a value longer than the API accepts', () => {
    expect(refusals(theForm({ labels: [{ key: 'k', value: 'x'.repeat(51) }] }))).toContain(
      'Label value must be at most 50 bytes'
    );
  });

  // The server bounds a label with `k.len()` and `v.len()`, which are UTF-8 byte counts. A label of
  // fifty non-ASCII characters is fifty UTF-16 code units — the unit a JavaScript string's length
  // reports — and a hundred-odd bytes, so counting characters here would pass what the server then
  // refuses with a 400 after it had been printed in every snippet on the screen.
  it('refuses a fifty-character CJK value the server would refuse for its byte length', () => {
    // Fifty code units — what a length bound reads — but a hundred and fifty bytes, what the server
    // reads. A character bound would let this through; a byte bound refuses it, as the server does.
    const fiftyCjk = '事'.repeat(50);
    expect(fiftyCjk.length).toBe(50);
    expect(refusals(theForm({ labels: [{ key: 'k', value: fiftyCjk }] }))).toContain(
      'Label value must be at most 50 bytes'
    );
  });

  it('refuses a fifty-character accented key the server would refuse for its byte length', () => {
    const fiftyAccented = 'é'.repeat(50); // 50 characters, 100 bytes
    expect(fiftyAccented.length).toBe(50);
    expect(refusals(theForm({ labels: [{ key: fiftyAccented, value: '1' }] }))).toContain(
      'Label key must be at most 50 bytes'
    );
  });

  it('takes a non-ASCII value that fits the byte budget', () => {
    // Sixteen three-byte characters is forty-eight bytes: inside the bound, and non-ASCII, so it is
    // what keeps the two refusals above from being satisfied by a rule that refuses every accent.
    const withinBudget = '事'.repeat(16); // 16 characters, 48 bytes
    expect(refusals(theForm({ labels: [{ key: 'k', value: withinBudget }] }))).toEqual([]);
  });

  it('refuses a payload longer than the API accepts', () => {
    expect(refusals(theForm({ payload: `"${'x'.repeat(699_050)}"` }))).toContain(
      'Payload must be at most 699050 characters'
    );
  });

  // Zig admits no raw control character inside a string literal, so a value pasted out of a
  // terminal rendered an example of that language which would not compile.
  it.each([
    ['a null', '\u0000'],
    ['a vertical tab', '\u000b'],
    ['a form feed', '\u000c'],
    ['an escape', '\u001b'],
    ['a delete', '\u007f'],
  ])('refuses %s in a label value', (_named, character) => {
    expect(refusals(theForm({ labels: [{ key: 'k', value: `pro${character}d` }] }))).toContain(
      'Label value must not contain a control character'
    );
  });

  it('refuses a control character in a label key', () => {
    expect(refusals(theForm({ labels: [{ key: 'env\u001b', value: '1' }] }))).toContain(
      'Label key must not contain a control character'
    );
  });

  it('takes the accented and punctuated values a control character sits beside', () => {
    // What keeps the refusals above from being satisfied by a rule that refuses everything unusual.
    expect(refusals(theForm({ labels: [{ key: 'clé', value: "l'évènement — n°1" }] }))).toEqual([]);
  });

  // The properties that were already the schema's, kept under the bounds added beside them.
  it('refuses a row that is nothing but spaces', () => {
    expect(refusals(theForm({ labels: [{ key: '  ', value: '1' }] }))).toContain(
      'Label key must not be blank'
    );
    expect(refusals(theForm({ labels: [{ key: 'k', value: '  ' }] }))).toContain(
      'Label value must not be blank'
    );
  });

  it('refuses an event with no label at all, which the API refuses too', () => {
    expect(refusals(theForm({ labels: [] }))).toContain('At least one label is required');
  });

  it('refuses a payload that is not JSON', () => {
    expect(refusals(theForm({ payload: '{' }))).toContain('Payload must be valid JSON');
  });

  it('refuses an event with no type and no instant', () => {
    expect(refusals(theForm({ eventType: '', occurredAt: '' }))).toEqual([
      'Event type is required',
      'Occurred at is required',
    ]);
  });
});
