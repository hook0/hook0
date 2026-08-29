#!/usr/bin/env node

/**
 * Builds the llms.txt index of the documentation.
 *
 * The file follows the llmstxt.org layout: an H1, a blockquote summary, then
 * H2 sections listing `[title](url): description`. It is what an assistant
 * reads when someone asks it to set up Hook0, so it has to name every page we
 * publish — a page missing here is a page the assistant will not know exists.
 *
 * Nothing is listed by hand. The page set comes from the docs Docusaurus has
 * just built, and the section labels and reading order come from the sidebar
 * that already drives the site navigation. Pages the sidebar does not mention
 * are still listed, at the end, so adding a page can never silently drop it.
 */

// A description longer than this is cut: the index is meant to be scanned, and
// a few pages carry a full paragraph of front matter.
const MAX_DESCRIPTION_LENGTH = 200;

// Guard against an index that grows without anyone noticing. Well above the
// ~100 pages published today; crossing it means the shape of the docs changed.
const MAX_ENTRIES = 500;

const UNSORTED_SECTION_LABEL = 'Other pages';

// Guard against a full-text dump that grows without anyone noticing. The whole
// prose documentation inlined is well under this today; crossing it means a page
// pulled in something it should not (a generated file, a build artefact).
const MAX_FULL_TXT_BYTES = 8_000_000;

/**
 * A file whose name starts with an underscore is a fragment other pages pull
 * in, not a page anyone should be sent to. Docusaurus still emits a route for
 * it, and the sitemap already keeps one of them out by name; this keeps all of
 * them out of the index without anyone having to list them.
 */
function isPartial(docId) {
  return docId.split('/').pop().startsWith('_');
}

/**
 * Walks the sidebar and returns, in reading order, the sections it defines:
 * `[{ label, docIds }]`. Nested categories are flattened into their parent —
 * llms.txt lists pages under one heading level, so `Reference > SDKs`
 * contributes its pages to `Reference`.
 *
 * A top-level page that belongs to no category (the docs home) opens the file
 * in a section labelled with the site title.
 */
function readSidebarSections(sidebarItems) {
  const sections = [];
  let looseSection = null;

  function collectDocIds(items, into) {
    for (const item of items) {
      if (typeof item === 'string') {
        into.push(item);
      } else if (item.type === 'doc') {
        into.push(item.id);
      } else if (item.type === 'category') {
        if (item.link && item.link.type === 'doc') into.push(item.link.id);
        collectDocIds(item.items || [], into);
      }
      // `link` items point outside the docs plugin (e.g. /api) and carry no
      // description of their own — the pages they lead to are listed elsewhere.
    }
  }

  for (const item of sidebarItems) {
    if (typeof item === 'string' || item.type === 'doc') {
      if (!looseSection) {
        looseSection = { label: null, docIds: [] };
        sections.unshift(looseSection);
      }
      looseSection.docIds.push(typeof item === 'string' ? item : item.id);
    } else if (item.type === 'category') {
      const docIds = [];
      if (item.link && item.link.type === 'doc') docIds.push(item.link.id);
      collectDocIds(item.items || [], docIds);
      sections.push({ label: item.label, docIds });
    }
  }

  return sections;
}

function truncate(text, max) {
  const collapsed = String(text || '').replace(/\s+/g, ' ').trim();
  if (collapsed.length <= max) return collapsed;
  return `${collapsed.slice(0, max - 1).trimEnd()}…`;
}

function renderEntry(doc, siteUrl) {
  const url = new URL(doc.permalink, siteUrl).toString();
  const description = truncate(doc.description, MAX_DESCRIPTION_LENGTH);
  return description
    ? `- [${doc.title}](${url}): ${description}`
    : `- [${doc.title}](${url})`;
}

/**
 * @param {object} options
 * @param {string} options.siteUrl      Site URL, e.g. https://documentation.hook0.com/
 * @param {string} options.title        H1 of the index
 * @param {string} options.summary      Blockquote under the H1
 * @param {Array}  options.sidebarItems The sidebar that drives the site navigation
 * @param {Array}  options.docs         Built docs: `{ id, title, description, permalink }`
 * @returns {string} the contents of llms.txt
 */
function buildLlmsTxt({ siteUrl, title, summary, sidebarItems, docs }) {
  const pages = docs.filter((doc) => !isPartial(doc.id));

  if (pages.length > MAX_ENTRIES) {
    throw new Error(
      `llms.txt would list ${pages.length} pages, over the ${MAX_ENTRIES} cap. ` +
        'Raise the cap deliberately or split the index.',
    );
  }

  const byId = new Map(pages.map((doc) => [doc.id, doc]));
  const listed = new Set();
  const lines = [`# ${title}`, '', `> ${truncate(summary, 600)}`];

  for (const section of readSidebarSections(sidebarItems)) {
    const entries = [];
    for (const id of section.docIds) {
      const doc = byId.get(id);
      // A sidebar can name a page that is not built (excluded, or renamed).
      // Skipping it keeps the index honest rather than emitting a dead link.
      if (!doc || listed.has(id)) continue;
      listed.add(id);
      entries.push(renderEntry(doc, siteUrl));
    }
    if (!entries.length) continue;
    lines.push('', `## ${section.label || title}`, '', ...entries);
  }

  // Everything the sidebar does not mention. Sorted by URL so the file only
  // changes when the docs do.
  const rest = pages
    .filter((doc) => !listed.has(doc.id))
    .sort((a, b) => a.permalink.localeCompare(b.permalink));
  if (rest.length) {
    lines.push('', `## ${UNSORTED_SECTION_LABEL}`, '');
    lines.push(...rest.map((doc) => renderEntry(doc, siteUrl)));
  }

  return `${lines.join('\n')}\n`;
}

/**
 * The published pages, in the order llms.txt lists them: the sidebar's reading
 * order first, then whatever the sidebar never names, sorted by URL so the file
 * only moves when the docs do. Fragments (`_`-prefixed) are left out, exactly as
 * the index leaves them out. This is the ordering the index and the full-text
 * file share, so a page can never appear in one and not the other.
 */
function orderedPages({ sidebarItems, docs }) {
  const pages = docs.filter((doc) => !isPartial(doc.id));

  if (pages.length > MAX_ENTRIES) {
    throw new Error(
      `llms-full.txt would inline ${pages.length} pages, over the ${MAX_ENTRIES} cap. ` +
        'Raise the cap deliberately or split the file.',
    );
  }

  const byId = new Map(pages.map((doc) => [doc.id, doc]));
  const listed = new Set();
  const ordered = [];

  for (const section of readSidebarSections(sidebarItems)) {
    for (const id of section.docIds) {
      const doc = byId.get(id);
      if (!doc || listed.has(id)) continue;
      listed.add(id);
      ordered.push(doc);
    }
  }

  const rest = pages
    .filter((doc) => !listed.has(doc.id))
    .sort((a, b) => a.permalink.localeCompare(b.permalink));
  ordered.push(...rest);

  return ordered;
}

/**
 * Removes a leading YAML front-matter block (`---` … `---`) so the inlined page
 * carries its prose, not the metadata Docusaurus already turned into a title and
 * a description. Anything before the block, or a page without one, is left as is.
 */
function stripFrontMatter(content) {
  const text = String(content || '');
  const match = text.match(/^---\r?\n[\s\S]*?\r?\n---\r?\n?/);
  return match ? text.slice(match[0].length) : text;
}

/**
 * The full-text companion to llms.txt: the same pages, in the same order, each
 * one inlined whole so an assistant can read the whole documentation without
 * fetching a hundred pages. Every page opens with a `<!-- Source: url -->`
 * marker so a model (or a human) can trace a passage back to its page.
 *
 * @param {object} options
 * @param {string} options.siteUrl      Site URL, e.g. https://documentation.hook0.com/
 * @param {string} options.title        H1 of the file
 * @param {string} options.summary      Blockquote under the H1
 * @param {Array}  options.sidebarItems The sidebar that drives the site navigation
 * @param {Array}  options.docs         Built docs: `{ id, title, permalink, content }`
 * @returns {string} the contents of llms-full.txt
 */
function buildLlmsFullTxt({ siteUrl, title, summary, sidebarItems, docs }) {
  const lines = [
    `# ${title}`,
    '',
    `> ${truncate(summary, 600)}`,
    '',
    'This is the full-text companion to /llms.txt: every page it lists is inlined ' +
      'below in full, in the same order, so an assistant can read the whole ' +
      'documentation in one file.',
  ];

  for (const doc of orderedPages({ sidebarItems, docs })) {
    const url = new URL(doc.permalink, siteUrl).toString();
    const body = stripFrontMatter(doc.content).trim();
    if (!body) {
      throw new Error(
        `llms-full.txt: page ${doc.id} has no content to inline; ` +
          'refusing to publish it as an empty page.',
      );
    }
    lines.push('', `<!-- Source: ${url} -->`, '', body);
  }

  const output = `${lines.join('\n')}\n`;
  const bytes = Buffer.byteLength(output, 'utf8');
  if (bytes > MAX_FULL_TXT_BYTES) {
    throw new Error(
      `llms-full.txt would be ${bytes} bytes, over the ${MAX_FULL_TXT_BYTES} cap. ` +
        'The docs grew a lot, or a page inlined something it should not.',
    );
  }

  return output;
}

module.exports = {
  buildLlmsTxt,
  buildLlmsFullTxt,
  orderedPages,
  stripFrontMatter,
  readSidebarSections,
  MAX_DESCRIPTION_LENGTH,
  MAX_ENTRIES,
  MAX_FULL_TXT_BYTES,
  UNSORTED_SECTION_LABEL,
};
