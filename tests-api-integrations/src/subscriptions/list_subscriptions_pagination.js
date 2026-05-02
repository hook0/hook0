import http from 'k6/http';
import { check } from 'k6';
import { uuidv4 } from 'https://jslib.k6.io/k6-utils/1.4.0/index.js';
import create_event_type from '../event_types/create_event_type.js';
import create_subscription from './create_subscription.js';

/**
 * Cursor + limit pagination on GET /api/v1/subscriptions.
 *
 * Seeds N event types + N subscriptions, walks the cursor-paginated list,
 * asserts no rows duplicated/skipped, exercises 400 paths, and verifies that
 * deleting a subscription mid-pagination doesn't duplicate or skip rows on
 * subsequent pages.
 */

const SEED_COUNT = 250;

function parseLinkHeader(headerValue) {
  if (!headerValue) return {};
  const out = {};
  for (const part of headerValue.split(',')) {
    const m = part.match(/<([^>]+)>;\s*rel="([^"]+)"/);
    if (m) out[m[2]] = m[1];
  }
  return out;
}

function seedSubscriptions(baseUrl, serviceToken, applicationId, targetUrl, count) {
  // Each subscription needs at least one event type. Reuse a single event type
  // across all to keep seeding fast.
  const sharedEventType = create_event_type(baseUrl, serviceToken, applicationId);
  if (!sharedEventType) {
    throw new Error('failed to seed shared event type for subscription pagination test');
  }
  const seeded = [];
  for (let i = 0; i < count; i++) {
    const sub = create_subscription(
      baseUrl,
      serviceToken,
      applicationId,
      [sharedEventType],
      targetUrl,
      { pag_k6: uuidv4().slice(0, 8) }
    );
    if (!sub) {
      throw new Error(`seed subscription ${i} failed`);
    }
    seeded.push(sub.subscription_id);
  }
  return seeded;
}

export default function (baseUrl, serviceToken, applicationId, targetUrl) {
  const params = {
    headers: { Authorization: `Bearer ${serviceToken}` },
  };

  const seeded = seedSubscriptions(baseUrl, serviceToken, applicationId, targetUrl, SEED_COUNT);
  const listUrl = `${baseUrl}/api/v1/subscriptions/?application_id=${applicationId}`;

  // Default limit returns 100 items + Link: rel="next".
  let res = http.get(listUrl, params);
  let body = JSON.parse(res.body);
  let links = parseLinkHeader(res.headers['Link'] || res.headers['link']);
  check(res, {
    'default limit returns 100 with next link': (r) =>
      r.status === 200 && body.length === 100 && !!links.next,
  });

  // Custom `limit` honored and propagated into the next-link URL.
  res = http.get(`${listUrl}&limit=50`, params);
  body = JSON.parse(res.body);
  links = parseLinkHeader(res.headers['Link'] || res.headers['link']);
  check(res, {
    'custom limit propagated': (r) =>
      r.status === 200 && body.length === 50 && !!links.next && links.next.includes('limit=50'),
  });

  // Walking all pages visits every seeded subscription exactly once.
  const seenIds = new Set();
  let pageUrl = `${listUrl}&limit=100`;
  let pageCount = 0;
  let prevSeenOnSecondPage = false;
  while (pageUrl && pageCount < 10) {
    res = http.get(pageUrl, params);
    body = JSON.parse(res.body);
    links = parseLinkHeader(res.headers['Link'] || res.headers['link']);
    for (const s of body) seenIds.add(s.subscription_id);
    pageCount += 1;
    if (pageCount === 2 && links.prev) prevSeenOnSecondPage = true;
    pageUrl = links.next || null;
  }
  check(null, {
    'walking all pages covers all seeded items': () => seenIds.size === SEED_COUNT,
    'prev link from page 2': () => prevSeenOnSecondPage,
    'last page has no next link': () => !pageUrl,
  });

  // Out-of-range `limit` values return HTTP 400.
  for (const badLimit of ['0', '101', '-1']) {
    res = http.get(`${listUrl}&limit=${badLimit}`, params);
    check(res, {
      [`limit=${badLimit} returns 400`]: (r) => r.status === 400,
    });
  }

  // Malformed cursor returns HTTP 400.
  res = http.get(`${listUrl}&pagination_cursor=not-base64@@`, params);
  check(res, {
    'malformed cursor returns 400': (r) => r.status === 400,
  });

  // Killer feature of cursor over offset: deleting a row mid-pagination
  // must not duplicate or skip rows on the subsequent pages.
  // Re-fetch page 1 with limit=10 to keep the test focused.
  res = http.get(`${listUrl}&limit=10`, params);
  body = JSON.parse(res.body);
  links = parseLinkHeader(res.headers['Link'] || res.headers['link']);
  const page1Ids = body.map((s) => s.subscription_id);
  const nextUrl = links.next;

  // Pick any seeded subscription that's NOT in page1Ids — it lives on a later page.
  const candidatesToDelete = seeded.filter((id) => !page1Ids.includes(id));
  if (candidatesToDelete.length === 0) {
    // Should not happen with seed=250 and limit=10.
    check(null, { 'setup: at least one subscription beyond page 1': () => false });
    return seeded.length;
  }
  const deletedId = candidatesToDelete[0];

  const delRes = http.del(
    `${baseUrl}/api/v1/subscriptions/${deletedId}?application_id=${applicationId}`,
    null,
    params
  );
  check(delRes, {
    'setup: delete subscription mid-pagination 204/200': (r) =>
      r.status === 204 || r.status === 200,
  });

  // Follow the next-link from page 1 and walk to the end. The deleted subscription
  // must NOT appear, and no subscription that was on page 1 must reappear.
  const remainingIds = new Set(page1Ids);
  pageUrl = nextUrl;
  pageCount = 0;
  let dupSeen = false;
  let deletedSeen = false;
  while (pageUrl && pageCount < 30) {
    res = http.get(pageUrl, params);
    body = JSON.parse(res.body);
    links = parseLinkHeader(res.headers['Link'] || res.headers['link']);
    for (const s of body) {
      if (remainingIds.has(s.subscription_id)) dupSeen = true;
      if (s.subscription_id === deletedId) deletedSeen = true;
      remainingIds.add(s.subscription_id);
    }
    pageCount += 1;
    pageUrl = links.next || null;
  }
  check(null, {
    'no duplicate row across delete-mid-pagination': () => !dupSeen,
    'deleted row not seen after delete': () => !deletedSeen,
    'remaining seeded count matches expected (SEED_COUNT - 1)': () =>
      remainingIds.size === SEED_COUNT - 1,
  });

  return seeded.length;
}
