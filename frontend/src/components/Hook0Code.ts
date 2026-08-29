/**
 * What a code block can be asked to colour.
 *
 * Beyond `json` and `bash`, the names are the SDK target names, so a language reaches the block
 * under the one name the rest of the product knows it by rather than under a second, private
 * spelling. It lives beside the component rather than inside it so that a caller holding a target
 * name can be type-checked against it.
 */
export const HOOK0_CODE_LANGUAGES = [
  'json',
  'bash',
  'javascript',
  'typescript',
  'python',
  'go',
  'ruby',
  'lua',
  'java',
  'csharp',
  'kotlin',
  'php',
  'rust',
  'zig',
] as const;

export type Hook0CodeLanguage = (typeof HOOK0_CODE_LANGUAGES)[number];
