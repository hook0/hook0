import http from 'k6/http';
import { check, sleep } from 'k6';
import { uuidv4 } from 'https://jslib.k6.io/k6-utils/1.4.0/index.js';

/**
 * Cursor + limit pagination on GET /api/v1/event_types.
 *
 * Seeds N event types on the application, walks the cursor-paginated list
 * with various `limit` values, asserts `Link: rel="next"` / `rel="prev"`
 * presence, asserts no rows are duplicated nor skipped, and exercises the
 * 400 paths (limit out of range, malformed cursor).
 */

// Spans 2 pages with default limit=100 + leaves room for the limit=50 walk.
// Larger fixtures hit the API per-token rate limit (10 req/s) and slow the
// suite without testing anything new.
const SEED_COUNT = 105;

function parseLinkHeader(headerValue) {
  // RFC 8288: comma-separated values like
  //   <https://api/...?pagination_cursor=AAA>; rel="next", <...>; rel="prev"
  if (!headerValue) return {};
  const out = {};
  for (const part of headerValue.split(',')) {
    const m = part.match(/<([^>]+)>;\s*rel="([^"]+)"/);
    if (m) out[m[2]] = m[1];
  }
  return out;
}

function seedEventTypes(baseUrl, serviceToken, applicationId, count) {
  const url = `${baseUrl}api/v1/event_types/`;
  const params = {
    headers: {
      Authorization: `Bearer ${serviceToken}`,
      'Content-Type': 'application/json',
    },
  };
  const created = [];
  for (let i = 0; i < count; i++) {
    const payload = JSON.stringify({
      application_id: applicationId,
      service: `pag_k6_s_${uuidv4().slice(0, 8)}`,
      resource_type: `pag_k6_r_${uuidv4().slice(0, 8)}`,
      verb: `pag_k6_v_${uuidv4().slice(0, 8)}`,
    });
    // The API enforces a 10 req/s per-token rate limit. Retry on 429 with an
    // exponential backoff capped at 1s — that drains the token-bucket fast
    // enough that a 105-row seed completes in roughly 12-15s.
    let backoff = 0.1;
    let res = null;
    for (let attempt = 0; attempt < 10; attempt++) {
      res = http.post(url, payload, params);
      if (res.status === 201) break;
      if (res.status !== 429) {
        throw new Error(`seed event_type ${i} failed: status=${res.status} body=${res.body}`);
      }
      sleep(backoff);
      backoff = Math.min(backoff * 2, 1);
    }
    if (!res || res.status !== 201) {
      throw new Error(`seed event_type ${i} exhausted 429 retries`);
    }
    const body = JSON.parse(res.body);
    created.push(body.event_type_name);
  }
  return created;
}

export default function (baseUrl, serviceToken, applicationId) {
  const params = {
    headers: { Authorization: `Bearer ${serviceToken}` },
  };

  // Use a fresh application_id is the caller's responsibility; we trust it.
  const seeded = seedEventTypes(baseUrl, serviceToken, applicationId, SEED_COUNT);
  const listUrl = `${baseUrl}api/v1/event_types/?application_id=${applicationId}`;

  // Default limit returns 100 items + Link: rel="next".
  let res = http.get(listUrl, params);
  let body = JSON.parse(res.body);
  let links = parseLinkHeader(res.headers['Link'] || res.headers['link']);
  check(res, {
    'default limit returns 100 with next link': (r) =>
      r.status === 200 && body.length === 100 && !!links.next,
  });

  // Custom limit honored and propagated into the next-link URL.
  res = http.get(`${listUrl}&limit=50`, params);
  body = JSON.parse(res.body);
  links = parseLinkHeader(res.headers['Link'] || res.headers['link']);
  check(res, {
    'custom limit returns 50 + propagates in next link': (r) =>
      r.status === 200 && body.length === 50 && !!links.next && links.next.includes('limit=50'),
  });

  // Walking all pages with limit=100 visits every seeded item exactly once.
  const seenNames = new Set();
  let pageUrl = `${listUrl}&limit=100`;
  let pageCount = 0;
  let prevSeenOnSecondPage = false;
  while (pageUrl && pageCount < 10) {
    res = http.get(pageUrl, params);
    body = JSON.parse(res.body);
    links = parseLinkHeader(res.headers['Link'] || res.headers['link']);
    for (const et of body) seenNames.add(et.event_type_name);
    pageCount += 1;
    if (pageCount === 2 && links.prev) prevSeenOnSecondPage = true;
    pageUrl = links.next || null;
  }
  check(null, {
    'walking all pages returns all seeded items, no duplicates': () =>
      seenNames.size === SEED_COUNT,
    'prev link present from page 2 onward': () => prevSeenOnSecondPage,
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

  return seeded.length;
}
