import { check } from 'k6';

/**
 * Public pagination documentation matches the shipped contract.
 *
 * Static, no-network test: reads `documentation/openapi/info.mdx` and
 * `documentation/resources/changelog.md` at module-init time (k6 `open()`)
 * and asserts the strings the docs MUST and MUST NOT contain.
 *
 * Paths are relative to this file:
 *   tests-api-integrations/src/docs/pagination_doc_test.js
 *   -> ../../../documentation/openapi/info.mdx
 *   -> ../../../documentation/resources/changelog.md
 */

const INFO_MDX = open('../../../documentation/openapi/info.mdx');
const CHANGELOG_MD = open('../../../documentation/resources/changelog.md');

/**
 * Extract the "## Pagination" section of info.mdx so the assertions are
 * scoped: a stray mention of the word "offset" elsewhere in the file
 * (e.g. inside a future section about timestamps) must not give a false
 * positive. The section ends at the next top-level heading.
 */
function extractPaginationSection(mdx) {
  const start = mdx.indexOf('## Pagination');
  if (start < 0) {
    throw new Error('Pagination section not found in info.mdx');
  }
  const after = mdx.indexOf('\n## ', start + 1);
  return after < 0 ? mdx.slice(start) : mdx.slice(start, after);
}

export default function () {
  const paginationSection = extractPaginationSection(INFO_MDX);

  // The pagination section must not advertise `offset` or `sort:` query params
  // (orphaned promises that the API never implemented). We match `offset` as a
  // whole word to avoid a false match on, e.g., "offset" appearing inside a
  // base64-looking example token.
  const offsetRegex = /\boffset\b/i;
  const sortColonRegex = /\bsort:/i;

  // The cursor + Link header contract must be documented.
  const hasCursor = paginationSection.includes('pagination_cursor');
  const hasLink = paginationSection.includes('Link');

  // Changelog references the originating issue.
  const refsIssue45 = /(?:issue\s*#?45\b|#45\b|issues\/45\b)/i.test(CHANGELOG_MD);

  check(null, {
    'pagination doc does not advertise offset': () => !offsetRegex.test(paginationSection),
    'pagination doc does not advertise sort:': () => !sortColonRegex.test(paginationSection),
    'pagination doc documents pagination_cursor': () => hasCursor,
    'pagination doc documents Link header': () => hasLink,
    'changelog references issue #45': () => refsIssue45,
  });

  if (offsetRegex.test(paginationSection)) {
    throw new Error('info.mdx Pagination section still mentions `offset`');
  }
  if (sortColonRegex.test(paginationSection)) {
    throw new Error('info.mdx Pagination section still mentions `sort:`');
  }
  if (!hasCursor) {
    throw new Error('info.mdx Pagination section is missing `pagination_cursor`');
  }
  if (!hasLink) {
    throw new Error('info.mdx Pagination section is missing `Link` header documentation');
  }
  if (!refsIssue45) {
    throw new Error('changelog.md is missing a reference to issue #45');
  }
}
