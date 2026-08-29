import { z } from 'zod';

/**
 * The bounds the API applies to an event, mirrored here so the form refuses what the server would.
 *
 * `validators.rs` refuses more than ten labels, and any label key or value whose UTF-8 length is
 * outside one to fifty bytes — `k.len()` and `v.len()` count bytes, not characters, so the bound is
 * measured here in the same unit; `handlers/events.rs` caps the payload at 512 KiB once base64 has
 * grown it. Left to the server, each of those reached the reader as a snippet printed in twelve
 * panels above a Send button that looked ready, and then as a 400 answering a call they had already
 * pasted.
 */
const LABELS_MAX = 10;
const LABEL_PROPERTY_MAX = 50;
const PAYLOAD_MAX = 699_050;

/** One encoder for the whole module, since a label's byte length is measured on every keystroke. */
const UTF8 = new TextEncoder();

/**
 * Whether a label key or value fits inside the API's bound, measured the way the API measures it.
 *
 * The server bounds a label with `k.len()` and `v.len()`, which are UTF-8 byte counts, while a
 * JavaScript string's `length` is UTF-16 code units — a fifty-character accented or CJK label is
 * fifty code units and a hundred-odd bytes. Counting the string's length here would pass a label the
 * server then refuses with a 400, after it had been printed in every snippet on the screen.
 */
function withinLabelByteBudget(text: string): boolean {
  return UTF8.encode(text).length <= LABEL_PROPERTY_MAX;
}

/** The last code point a terminal treats as a control character, above the C0 range. */
const DELETE_CODE = 0x7f;

/** The first code point that stands for itself rather than for a control. */
const FIRST_PRINTABLE_CODE = 0x20;

/**
 * Whether a value carries a character no label may.
 *
 * Zig admits no raw control character inside a string literal at all — all thirty-three of them —
 * so a value pasted out of a terminal renders an example of that language which will not compile.
 * The payload reaches the same examples but is escaped per language on the way, which is a table of
 * a few rules; a label cannot be, because thirty-three controls on top of the backslash and the
 * quote is more replacements than a literal's table has room for. So the input is where this holds.
 *
 * Refused rather than stripped, because a value quietly rewritten underneath the reader is a value
 * they did not type and would not recognise in the snippet they copy.
 */
function carriesControlCharacter(text: string): boolean {
  for (let index = 0; index < text.length; index += 1) {
    const code = text.charCodeAt(index);
    if (code < FIRST_PRINTABLE_CODE || code === DELETE_CODE) {
      return true;
    }
  }
  return false;
}

export const sendEventSchema = z.object({
  eventType: z.string().min(1, 'Event type is required'),
  labels: z
    .array(
      z.object({
        // Blank rather than empty: a key of nothing but spaces passed `min(1)`, left the submit
        // button enabled, was printed in every snippet on the screen and was then dropped from the
        // request that went out — or refused the send outright when it was the only label.
        key: z
          .string()
          .refine(withinLabelByteBudget, {
            message: `Label key must be at most ${LABEL_PROPERTY_MAX} bytes`,
          })
          .refine((key) => key.trim().length > 0, {
            message: 'Label key must not be blank',
          })
          .refine((key) => !carriesControlCharacter(key), {
            message: 'Label key must not contain a control character',
          }),
        value: z
          .string()
          .refine(withinLabelByteBudget, {
            message: `Label value must be at most ${LABEL_PROPERTY_MAX} bytes`,
          })
          .refine((value) => value.trim().length > 0, {
            message: 'Label value must not be blank',
          })
          .refine((value) => !carriesControlCharacter(value), {
            message: 'Label value must not contain a control character',
          }),
      })
    )
    .min(1, 'At least one label is required')
    .max(LABELS_MAX, `At most ${LABELS_MAX} labels are allowed`),
  occurredAt: z.string().min(1, 'Occurred at is required'),
  payload: z
    .string()
    .min(1, 'Payload is required')
    .max(PAYLOAD_MAX, `Payload must be at most ${PAYLOAD_MAX} characters`)
    .refine(
      (val) => {
        try {
          JSON.parse(val);
          return true;
        } catch {
          return false;
        }
      },
      { message: 'Payload must be valid JSON' }
    ),
});

export type SendEventForm = z.infer<typeof sendEventSchema>;
export type SendEventFormValues = z.infer<typeof sendEventSchema>;
