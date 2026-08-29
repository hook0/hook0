/**
 * What the "Send an event" screen shows, for any of the SDKs.
 *
 * The screen knows none of the languages it displays and must never learn one: how a string is
 * quoted, what has to be escaped inside it and how two labels are joined all reach here as data,
 * declared beside the code they describe. What is left over is string replacement, and that is all
 * this module does — which is also why it lives here rather than in the component, where nothing
 * could be run against it.
 */

import type { Hook0Sdk, Hook0SdkStringLiteral } from '@/generated/sdkExamples';

/** One row of the labels editor, as the user typed it. */
export type Hook0SnippetLabel = {
  readonly key: string;
  readonly value: string;
};

/** Everything an example can ask for, in the shape its markers expect. */
export type Hook0SnippetValues = {
  readonly apiUrl: string;
  readonly applicationId: string;
  readonly token: string;
  readonly eventType: string;
  readonly payload: string;
  readonly labels: readonly Hook0SnippetLabel[];
};

/** The three blocks a language is shown as, in the order a reader goes through them. */
export type Hook0RenderedSdk = {
  readonly install: string;
  readonly send: string;
  readonly verify: string;
};

/** What the form is worth outside the one label an example carries. */
const GLOBAL_MARKERS = [
  '__HOOK0_API_URL__',
  '__HOOK0_APPLICATION_ID__',
  '__HOOK0_TOKEN__',
  '__HOOK0_EVENT_TYPE__',
  '__HOOK0_PAYLOAD__',
] as const;

/** What only the repeated label region carries. */
const LABEL_MARKERS = ['__HOOK0_LABEL_KEY__', '__HOOK0_LABEL_VALUE__'] as const;

type GlobalMarker = (typeof GLOBAL_MARKERS)[number];
type LabelMarker = GlobalMarker | (typeof LABEL_MARKERS)[number];

/**
 * Longest first: an alternation hands back the first branch that matches, so a marker spelled as
 * the beginning of a longer one would win over it and leave the rest of the longer one on screen.
 */
function patternOver(markers: readonly string[]): RegExp {
  return new RegExp([...markers].sort((a, b) => b.length - a.length).join('|'), 'g');
}

const GLOBAL_PATTERN = patternOver(GLOBAL_MARKERS);
const LABEL_PATTERN = patternOver([...GLOBAL_MARKERS, ...LABEL_MARKERS]);

/**
 * Replaces every marker of a pattern, in a single pass.
 *
 * A single pass because what the user typed can spell the very words a second pass would look for:
 * a payload containing `__HOOK0_TOKEN__` has to reach the snippet as those characters and not as
 * the token. A replacer function rather than a replacement string for the same reason — `$&` and
 * its family are read out of a replacement string, and a user can type those too.
 */
function substitute<M extends string>(
  text: string,
  pattern: RegExp,
  table: Readonly<Record<M, string>>
): string {
  // The pattern is built from this table's own keys, so every match is one of them.
  return text.replace(pattern, (marker) => table[marker as M]);
}

/**
 * Turns a value the user typed into a literal of the language being shown.
 *
 * Replacements are applied in the order the SDK declares them, which is what lets the backslash
 * come first: escaping it afterwards would escape the backslashes the other rules had introduced.
 * Splitting and joining rather than replacing keeps the replacement text literal, since a user can
 * type the `$&` a replacement string would read.
 */
export function escapeForLiteral(literal: Hook0SdkStringLiteral, raw: string): string {
  return literal.escape.reduce((text, [from, to]) => text.split(from).join(to), raw);
}

/**
 * Every marker of an example sits inside a string literal of that language, which is what lets a
 * file full of them compile — so every value substituted into one is escaped, not only the payload.
 */
function globalTable(sdk: Hook0Sdk, values: Hook0SnippetValues): Record<GlobalMarker, string> {
  const asLiteral = (raw: string) => escapeForLiteral(sdk.stringLiteral, raw);
  return {
    __HOOK0_API_URL__: asLiteral(values.apiUrl),
    __HOOK0_APPLICATION_ID__: asLiteral(values.applicationId),
    __HOOK0_TOKEN__: asLiteral(values.token),
    __HOOK0_EVENT_TYPE__: asLiteral(values.eventType),
    __HOOK0_PAYLOAD__: asLiteral(values.payload),
  };
}

/**
 * The install command, the send example and the verify example, carrying the form's values.
 *
 * The label region is repeated once per label and joined with the separator the SDK declares; no
 * label at all leaves the container it sits inside empty, which every language accepts. The text
 * around the region is substituted apart from the labels rather than after them, so that a label
 * a user spelled as a marker is never read as one.
 *
 * Throws when the example does not carry its label region: the artefact would then be malformed,
 * and a silent fallback would ship a snippet with either a raw marker or a run of bare separators.
 */
export function renderSdkSnippets(sdk: Hook0Sdk, values: Hook0SnippetValues): Hook0RenderedSdk {
  const table = globalTable(sdk, values);
  const region = sdk.send.label;
  const at = sdk.send.body.indexOf(region);
  if (region.length === 0 || at < 0) {
    throw new Error(`the ${sdk.target} send example does not carry its label region`);
  }

  const labels = values.labels
    .map((label) =>
      substitute<LabelMarker>(region, LABEL_PATTERN, {
        ...table,
        __HOOK0_LABEL_KEY__: escapeForLiteral(sdk.stringLiteral, label.key),
        __HOOK0_LABEL_VALUE__: escapeForLiteral(sdk.stringLiteral, label.value),
      })
    )
    .join(sdk.send.labelSeparator);

  return {
    install: substitute<GlobalMarker>(sdk.install, GLOBAL_PATTERN, table),
    send:
      substitute<GlobalMarker>(sdk.send.body.slice(0, at), GLOBAL_PATTERN, table) +
      labels +
      substitute<GlobalMarker>(sdk.send.body.slice(at + region.length), GLOBAL_PATTERN, table),
    verify: substitute<GlobalMarker>(sdk.verify.body, GLOBAL_PATTERN, table),
  };
}

/** What a raw call states that an SDK example leaves to the library: the event's identity and date. */
export type Hook0CurlValues = Hook0SnippetValues & {
  readonly eventId: string;
  readonly occurredAt: string;
};

/**
 * Wraps a value so a shell reads it as the one argument it is.
 *
 * Single quotes hold every character literally, which is what a JSON body needs — every character
 * but the quote itself, which no escape reaches from inside them. So the value is cut at each quote
 * and each one is spliced back on the outside. Without this a payload as ordinary as
 * `{"name": "O'Brien"}` ends the argument early, and the command a reader pasted is answered by
 * their own shell rather than by Hook0.
 */
export function shellSingleQuoted(raw: string): string {
  return `'${raw.split("'").join("'\\''")}'`;
}

/**
 * The call as a shell command, which is the one example no SDK carries.
 *
 * Here rather than in the component, and for the same reason the eleven others are: the quoting is
 * the part that can be wrong, and only what lives in this module can be run against a value chosen
 * to break it.
 */
export function renderCurlSnippet(values: Hook0CurlValues): string {
  const labels = values.labels.reduce<Record<string, string>>((record, label) => {
    record[label.key] = label.value;
    return record;
  }, {});
  const body = JSON.stringify(
    {
      application_id: values.applicationId,
      event_id: values.eventId,
      event_type: values.eventType,
      labels,
      occurred_at: values.occurredAt,
      payload: values.payload,
      payload_content_type: 'application/json',
    },
    null,
    2
  )
    .split('\n')
    .map((line, index) => (index === 0 ? line : `  ${line}`))
    .join('\n');

  return [
    `curl -X POST ${shellSingleQuoted(`${values.apiUrl}/event`)} \\`,
    `  -H 'Content-Type: application/json' \\`,
    `  -H ${shellSingleQuoted(`Authorization: Bearer ${values.token}`)} \\`,
    `  -d ${shellSingleQuoted(body)}`,
  ].join('\n');
}

/**
 * The languages given a tab of their own rather than a line in the picker. A tab bar does not hold
 * twelve entries, and these two carry most of the readers; the rest are one click away rather than
 * off screen.
 *
 * The picker lists what is left in the order the artefact hands it over, which is by published
 * share of developers writing each language, declared in each SDK's manifest beside its source. It
 * is a measurement, so it is read rather than restated: sorting here would be a second opinion on a
 * figure that already has one, and it would drift the day the figure is refreshed.
 */
export const FEATURED_TARGETS = ['typescript', 'python'];

/**
 * The languages the screen carries a mark for.
 *
 * A Vue component cannot be imported by a name computed at runtime, so this association is written
 * out and cannot be discovered. What can be checked is that it says the same thing as the registry:
 * the tests hold this list against `HOOK0_SDKS` in both directions, and the component types its map
 * by it, so a twelfth SDK cannot arrive without a mark and a mark cannot outlive its SDK.
 */
export const ICON_TARGETS = [
  'typescript',
  'python',
  'java',
  'csharp',
  'php',
  'go',
  'rust',
  'kotlin',
  'lua',
  'ruby',
  'zig',
] as const;

export type Hook0IconTarget = (typeof ICON_TARGETS)[number];

/** The panel holding the form rather than a language. */
export const FORM_PANEL = 'easy';

/** The panel holding the one snippet no SDK carries. */
export const CURL_PANEL = 'curl';

/**
 * Fragments that named a language before the screen carried more than two of them, and the target
 * they now stand for. They keep working because they were shared in tickets and in emails.
 */
const LEGACY_FRAGMENTS: ReadonlyArray<readonly [string, string]> = [['js', 'typescript']];

/** Every fragment that opens something, and the panel it opens. */
function fragmentsOf(sdks: readonly Hook0Sdk[]): Array<readonly [string, string]> {
  const targets = sdks.map((sdk) => sdk.target);
  return [
    [CURL_PANEL, CURL_PANEL],
    ...targets.map((target) => [target, target] as const),
    ...LEGACY_FRAGMENTS.filter(([, target]) => targets.includes(target)),
  ];
}

/** The panel a URL fragment opens; anything unknown lands on the form, as an empty fragment does. */
export function panelFromHash(hash: string, sdks: readonly Hook0Sdk[]): string {
  const fragment = hash.replace(/^#/, '');
  for (const [candidate, panel] of fragmentsOf(sdks)) {
    if (candidate === fragment) {
      return panel;
    }
  }
  return FORM_PANEL;
}

/** The fragment that reopens a panel. The form is the default, so it carries none. */
export function hashForPanel(panel: string): string {
  return panel === FORM_PANEL ? '' : `#${panel}`;
}

/**
 * Splits the SDKs into those shown as their own tab and those behind the picker, the tabs following
 * the order asked for and the picker the order of the registry.
 *
 * Throws on a featured target no SDK answers to, rather than dropping a tab the screen promises.
 */
export function partitionSdks(
  sdks: readonly Hook0Sdk[],
  featuredTargets: readonly string[]
): { readonly featured: Hook0Sdk[]; readonly others: Hook0Sdk[] } {
  const featured = featuredTargets.map((target) => {
    const found = sdks.filter((sdk) => sdk.target === target);
    if (found.length === 0) {
      throw new Error(`no SDK targets ${target}, so it cannot be shown as its own tab`);
    }
    return found[0];
  });
  return {
    featured,
    others: sdks.filter((sdk) => !featuredTargets.includes(sdk.target)),
  };
}
