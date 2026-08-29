/**
 * llms.txt is the map an assistant reads before it answers a question about
 * Hook0. A page missing from it is a page the assistant will tell someone does
 * not exist, and nothing on the site would look broken. These tests hold the
 * one property that matters — every built page is listed, exactly once —
 * against the ways a sidebar can drift away from the pages actually built.
 */

const test = require('node:test');
const assert = require('node:assert/strict');

const {
  buildLlmsTxt,
  buildLlmsFullTxt,
  readSidebarSections,
  MAX_DESCRIPTION_LENGTH,
  MAX_ENTRIES,
  MAX_FULL_TXT_BYTES,
  UNSORTED_SECTION_LABEL,
} = require('./generate-llms-txt.js');

const SITE_URL = 'https://documentation.hook0.com/';

const doc = (id, extra = {}) => ({
  id,
  title: id,
  description: `about ${id}`,
  permalink: `/${id}`,
  ...extra,
});

const build = (sidebarItems, docs) =>
  buildLlmsTxt({
    siteUrl: SITE_URL,
    title: 'Hook0 Documentation',
    summary: 'Webhooks as a service.',
    sidebarItems,
    docs,
  });

/** Every `- [title](url)` bullet, in the order the file lists them. */
const bulletUrls = (text) =>
  text
    .split('\n')
    .filter((line) => line.startsWith('- ['))
    .map((line) => line.match(/\]\(([^)]+)\)/)[1]);

const headings = (text) =>
  text
    .split('\n')
    .filter((line) => line.startsWith('## '))
    .map((line) => line.slice(3));

test('lists every built page exactly once, sidebar or no sidebar', () => {
  const docs = [doc('index'), doc('concepts/events'), doc('orphan/page')];
  const sidebar = [
    'index',
    { type: 'category', label: 'Concepts', items: ['concepts/events'] },
  ];

  const urls = bulletUrls(build(sidebar, docs));

  assert.equal(urls.length, docs.length);
  assert.equal(new Set(urls).size, docs.length);
  for (const built of docs) {
    assert.ok(
      urls.includes(`${SITE_URL}${built.id}`),
      `${built.id} is built but absent from llms.txt`,
    );
  }
});

test('a page the sidebar does not mention still reaches the file', () => {
  const output = build(
    [{ type: 'category', label: 'Concepts', items: ['concepts/events'] }],
    [doc('concepts/events'), doc('added/yesterday')],
  );

  assert.ok(headings(output).includes(UNSORTED_SECTION_LABEL));
  assert.ok(output.includes(`${SITE_URL}added/yesterday`));
});

test('a sidebar entry with no built page emits no dead link', () => {
  const output = build(
    [{ type: 'category', label: 'Concepts', items: ['concepts/events', 'concepts/renamed'] }],
    [doc('concepts/events')],
  );

  assert.deepEqual(bulletUrls(output), [`${SITE_URL}concepts/events`]);
  assert.ok(!output.includes('concepts/renamed'));
});

test('a page reachable twice through the sidebar is listed once', () => {
  const sidebar = [
    {
      type: 'category',
      label: 'Reference',
      link: { type: 'doc', id: 'reference/index' },
      items: ['reference/index', 'reference/cli'],
    },
  ];

  const urls = bulletUrls(build(sidebar, [doc('reference/index'), doc('reference/cli')]));

  assert.deepEqual(urls, [`${SITE_URL}reference/index`, `${SITE_URL}reference/cli`]);
});

test('keeps the sidebar reading order rather than sorting', () => {
  const sidebar = [
    { type: 'category', label: 'Zulu', items: ['z/one'] },
    { type: 'category', label: 'Alpha', items: ['a/one'] },
  ];

  assert.deepEqual(headings(build(sidebar, [doc('a/one'), doc('z/one')])), ['Zulu', 'Alpha']);
});

test('a nested category contributes its pages to the section above it', () => {
  const sidebar = [
    {
      type: 'category',
      label: 'Reference',
      items: [
        'reference/cli',
        { type: 'category', label: 'SDKs', link: { type: 'doc', id: 'sdk/index' }, items: ['sdk/rust'] },
      ],
    },
  ];

  const output = build(sidebar, [doc('reference/cli'), doc('sdk/index'), doc('sdk/rust')]);

  assert.deepEqual(headings(output), ['Reference']);
  assert.deepEqual(bulletUrls(output), [
    `${SITE_URL}reference/cli`,
    `${SITE_URL}sdk/index`,
    `${SITE_URL}sdk/rust`,
  ]);
});

test('an external sidebar link contributes no bullet', () => {
  const sidebar = [
    {
      type: 'category',
      label: 'Reference',
      items: ['reference/cli', { type: 'link', label: 'API Reference', href: '/api' }],
    },
  ];

  assert.deepEqual(bulletUrls(build(sidebar, [doc('reference/cli')])), [
    `${SITE_URL}reference/cli`,
  ]);
});

test('descriptions are bounded and collapsed onto one line', () => {
  const output = build(
    ['index'],
    [doc('index', { description: `${'x'.repeat(400)}\nsecond line` })],
  );

  const [bullet] = output.split('\n').filter((line) => line.startsWith('- ['));
  const description = bullet.split('): ')[1];
  assert.equal(description.length, MAX_DESCRIPTION_LENGTH);
  assert.equal(output.split('\n').filter((l) => l.includes('second line')).length, 0);
});

test('a page with no description still gets a bullet', () => {
  const output = build(['index'], [doc('index', { description: undefined })]);

  assert.ok(output.includes(`- [index](${SITE_URL}index)\n`));
  assert.ok(!output.includes('undefined'));
});

test('URLs are absolute, so the file works wherever it is read', () => {
  for (const url of bulletUrls(build(['index'], [doc('index')]))) {
    assert.ok(url.startsWith(SITE_URL), `${url} is not absolute`);
  }
});

test('the same input always produces the same file', () => {
  const sidebar = [{ type: 'category', label: 'Concepts', items: ['b'] }];
  const docs = [doc('b'), doc('c'), doc('a')];

  assert.equal(build(sidebar, docs), build(sidebar, docs.slice().reverse()));
});

test('refuses to write an index past its cap rather than truncating it', () => {
  const docs = Array.from({ length: MAX_ENTRIES + 1 }, (_, i) => doc(`page-${i}`));

  assert.throws(() => build([], docs), /over the 500 cap/);
});

test('an included fragment is not offered as a page to read', () => {
  const docs = [doc('reference/cli'), doc('reference/_cli-generated'), doc('self-hosting/_warning')];

  const urls = bulletUrls(build(['reference/cli'], docs));

  assert.deepEqual(urls, [`${SITE_URL}reference/cli`]);
});

test('a fragment named in the sidebar is still not listed', () => {
  const output = build(
    [{ type: 'category', label: 'Reference', items: ['reference/_cli-generated'] }],
    [doc('reference/_cli-generated')],
  );

  assert.deepEqual(bulletUrls(output), []);
});

test('the docs home opens the file, before any category', () => {
  const sidebar = [
    { type: 'category', label: 'Concepts', items: ['concepts/events'] },
    { type: 'doc', id: 'index', label: 'Home' },
  ];

  const output = build(sidebar, [doc('index'), doc('concepts/events')]);

  assert.deepEqual(bulletUrls(output), [`${SITE_URL}index`, `${SITE_URL}concepts/events`]);
  assert.deepEqual(headings(output), ['Hook0 Documentation', 'Concepts']);
});

test('reads the sidebar sections without needing the built pages', () => {
  const sections = readSidebarSections([
    { type: 'category', label: 'Concepts', link: { type: 'doc', id: 'concepts/index' }, items: ['concepts/events'] },
  ]);

  assert.deepEqual(sections, [
    { label: 'Concepts', docIds: ['concepts/index', 'concepts/events'] },
  ]);
});

/**
 * llms-full.txt inlines the same pages llms.txt lists, whole, so an assistant
 * can read the documentation without fetching each page. The property that
 * matters is the same one — every built page reaches the file, exactly once —
 * plus: the full text is actually there (not the one-line description), and the
 * order never drifts from the index.
 */

const docF = (id, extra = {}) => doc(id, { content: `Body of ${id}.`, ...extra });

const buildFull = (sidebarItems, docs) =>
  buildLlmsFullTxt({
    siteUrl: SITE_URL,
    title: 'Hook0 Documentation',
    summary: 'Webhooks as a service.',
    sidebarItems,
    docs,
  });

/** Every `<!-- Source: url -->` marker, in the order the file emits them. */
const sourceUrls = (text) =>
  text
    .split('\n')
    .filter((line) => line.startsWith('<!-- Source: '))
    .map((line) => line.match(/<!-- Source: (\S+) -->/)[1]);

test('inlines every built page exactly once, sidebar or no sidebar', () => {
  const docs = [docF('index'), docF('concepts/events'), docF('orphan/page')];
  const sidebar = [
    'index',
    { type: 'category', label: 'Concepts', items: ['concepts/events'] },
  ];

  const urls = sourceUrls(buildFull(sidebar, docs));

  assert.equal(urls.length, docs.length);
  assert.equal(new Set(urls).size, docs.length);
  for (const built of docs) {
    assert.ok(
      urls.includes(`${SITE_URL}${built.id}`),
      `${built.id} is built but absent from llms-full.txt`,
    );
  }
});

test('inlines the whole page body, not the one-line description', () => {
  const output = buildFull(
    ['guide'],
    [docF('guide', { description: 'short blurb', content: 'A full paragraph a description would never carry.' })],
  );

  assert.ok(output.includes('A full paragraph a description would never carry.'));
  assert.ok(!output.includes('short blurb'));
});

test('lists pages in the exact order the index lists them', () => {
  const sidebar = [
    { type: 'category', label: 'Zulu', items: ['z/one', 'z/two'] },
    { type: 'category', label: 'Alpha', items: ['a/one'] },
  ];
  const docs = [docF('a/one'), docF('z/two'), docF('z/one'), docF('loose/tail')];

  const indexOrder = bulletUrls(build(sidebar, docs));
  const fullOrder = sourceUrls(buildFull(sidebar, docs));

  assert.deepEqual(fullOrder, indexOrder);
});

test('strips the front matter so the metadata block is not inlined', () => {
  const content = '---\ntitle: Events\nkeywords: [a, b]\n---\n\nEvents are how Hook0 works.\n';
  const output = buildFull(['events'], [docF('events', { content })]);

  assert.ok(output.includes('Events are how Hook0 works.'));
  assert.ok(!output.includes('keywords: [a, b]'));
});

test('a fragment is inlined nowhere, even named in the sidebar', () => {
  const output = buildFull(
    [{ type: 'category', label: 'Reference', items: ['reference/cli', 'reference/_cli-generated'] }],
    [docF('reference/cli'), docF('reference/_cli-generated')],
  );

  assert.deepEqual(sourceUrls(output), [`${SITE_URL}reference/cli`]);
});

test('Source markers are absolute URLs', () => {
  for (const url of sourceUrls(buildFull(['index'], [docF('index')]))) {
    assert.ok(url.startsWith(SITE_URL), `${url} is not absolute`);
  }
});

test('the same input always produces the same file', () => {
  const sidebar = [{ type: 'category', label: 'Concepts', items: ['b'] }];
  const docs = [docF('b'), docF('c'), docF('a')];

  assert.equal(buildFull(sidebar, docs), buildFull(sidebar, docs.slice().reverse()));
});

test('refuses to publish a page with no content rather than inline it empty', () => {
  assert.throws(
    () => buildFull(['blank'], [docF('blank', { content: '   \n  ' })]),
    /page blank has no content/,
  );
});

test('refuses to write a file past its byte cap rather than truncating it', () => {
  const big = 'x'.repeat(MAX_FULL_TXT_BYTES + 1);
  assert.throws(
    () => buildFull(['huge'], [docF('huge', { content: big })]),
    /over the 8000000 cap/,
  );
});

test('inlines a page the sidebar never names, at the end', () => {
  const output = buildFull(
    [{ type: 'category', label: 'Concepts', items: ['concepts/events'] }],
    [docF('concepts/events'), docF('added/yesterday')],
  );

  assert.deepEqual(sourceUrls(output), [
    `${SITE_URL}concepts/events`,
    `${SITE_URL}added/yesterday`,
  ]);
});
