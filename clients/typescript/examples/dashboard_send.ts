/**
 * What the dashboard shows under "Send an event", for TypeScript.
 *
 * This file exists so that the snippet is type-checked against the real client. A renamed method, a
 * dropped argument or an argument of the wrong type turns `clients.typescript.check` red on the day
 * it happens, which is the whole reason the snippet lives here rather than in the dashboard: one
 * written by hand over there is backed by nothing and drifts in silence.
 *
 * Two pairs of markers say how it is read. `hook0:snippet` delimits what a reader is shown, so that
 * anything this file needs only in order to type-check stays out of it. `hook0:label` delimits the
 * one rendering of a label, which the dashboard repeats once per label the form carries and joins
 * with the separator its manifest declares — the region carries no trailing separator of its own,
 * and sits inside its container, so no label at all leaves a valid empty one.
 *
 * The `__HOOK0_*__` words are string literals, which is what lets a file full of them type-check.
 * They never resolve to anything: this example is checked, never run.
 *
 * The label key sits in a computed position — `['…']` — rather than in a quoted one, because a label
 * key is whatever the user typed. Written as a quoted key, prettier drops the quotes (it is
 * `quoteProps: as-needed` by default, and `__HOOK0_LABEL_KEY__` happens to read as an identifier),
 * and the rendered snippet would then break on the first key that is not one.
 */

// hook0:snippet:begin
import { Event, Hook0Client } from 'hook0-client';

const hook0 = new Hook0Client('__HOOK0_API_URL__', '__HOOK0_APPLICATION_ID__', '__HOOK0_TOKEN__');

// `Event` takes its arguments positionally, and `labels` is the fourth of them — required, unlike
// the three that follow, because a subscription routes on it.
const eventId = await hook0.sendEvent(
  new Event('__HOOK0_EVENT_TYPE__', '__HOOK0_PAYLOAD__', 'application/json', {
    // hook0:label:begin
    ['__HOOK0_LABEL_KEY__']: '__HOOK0_LABEL_VALUE__', // hook0:label:end
  })
);

console.log(`ingested as ${eventId}`);
// hook0:snippet:end
