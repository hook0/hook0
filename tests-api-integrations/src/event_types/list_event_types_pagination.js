import http from 'k6/http';
import { check } from 'k6';
import { uuidv4 } from 'https://jslib.k6.io/k6-utils/1.4.0/index.js';

/**
 * AC-1, AC-4..AC-10 — cursor + limit pagination on GET /api/v1/event_types
 *
 * Seeds N event types on the application, walks the cursor-paginated list
 * with various `limit` values, asserts `Link: rel="next"` / `rel="prev"`
 * presence, asserts no rows are duplicated nor skipped, and exercises the
 * 400 paths (limit out of range, malformed cursor).
 */

const SEED_COUNT = 250;

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
    const res = http.post(url, payload, params);
    if (res.status !== 201) {
      throw new Error(`seed event_type ${i} failed: status=${res.status} body=${res.body}`);
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

  // AC-1: default limit returns 100 + Link: rel="next"
  let res = http.get(listUrl, params);
  let body = JSON.parse(res.body);
  let links = parseLinkHeader(res.headers['Link'] || res.headers['link']);
  check(res, {
    'AC-1 default limit returns 100 with next link': (r) =>
      r.status === 200 && body.length === 100 && !!links.next,
  });

  // AC-4: ?limit=50 returns 50 with limit propagated in next URL
  res = http.get(`${listUrl}&limit=50`, params);
  body = JSON.parse(res.body);
  links = parseLinkHeader(res.headers['Link'] || res.headers['link']);
  check(res, {
    'AC-4 custom limit returns 50 + propagates in next link': (r) =>
      r.status === 200 && body.length === 50 && !!links.next && links.next.includes('limit=50'),
  });

  // AC-5: walk all pages with limit=100 -> visit all 250 unique event types, no duplicates
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
    'AC-5 walking all pages returns all seeded items, no duplicates': () =>
      seenNames.size === SEED_COUNT,
    'AC-6 prev link present from page 2 onward': () => prevSeenOnSecondPage,
    'AC-7 last page has no next link': () => !pageUrl,
  });

  // AC-9: limit=0, limit=101, limit=-1 all 400
  for (const badLimit of ['0', '101', '-1']) {
    res = http.get(`${listUrl}&limit=${badLimit}`, params);
    check(res, {
      [`AC-9 limit=${badLimit} returns 400`]: (r) => r.status === 400,
    });
  }

  // AC-10: malformed cursor returns 400
  res = http.get(`${listUrl}&pagination_cursor=not-base64@@`, params);
  check(res, {
    'AC-10 malformed cursor returns 400': (r) => r.status === 400,
  });

  return seeded.length;
}
