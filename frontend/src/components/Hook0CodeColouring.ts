import type { Hook0CodeLanguage } from '@/components/Hook0Code';

/**
 * The languages the code block renders as plain text on purpose, because no ecosystem publishes a
 * CodeMirror grammar for them.
 *
 * This is the one place that decision is declared. `Hook0Code.vue` types its grammar map from it —
 * every language not named here owes a grammar, and a language named here owes `null` — so a block
 * silently dropping to plain text is a type error rather than something noticed on screen. The name
 * of a language reaching the block is not what decides its colour; membership in this list is.
 *
 * Zig is the only one: writing and maintaining a grammar for the least used of the languages this
 * block shows, whose syntax is still moving, is not worth it, and plain text renders it correctly.
 */
export const PLAIN_TEXT_LANGUAGES = ['zig'] as const;

export type PlainTextLanguage = (typeof PLAIN_TEXT_LANGUAGES)[number];

/** A language the block is expected to colour: everything the code block knows that is not plain. */
export type ColouredLanguage = Exclude<Hook0CodeLanguage, PlainTextLanguage>;

/** Whether the block renders this language as plain text rather than colouring it. */
export function isPlainTextLanguage(language: Hook0CodeLanguage): language is PlainTextLanguage {
  return (PLAIN_TEXT_LANGUAGES as readonly string[]).includes(language);
}
