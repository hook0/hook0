#!/usr/bin/env node

/**
 * Fetch the Hook0 OpenAPI specification the documentation site publishes.
 *
 * The API reference at /api is a Scalar viewer that loads hook0-api.json from
 * the site root at runtime. A build that cannot produce a usable specification
 * therefore has nothing to publish there, and must stop: a site whose reference
 * answers 404, or lists no endpoint at all, is worse than a build that failed
 * loudly.
 */

const fs = require('fs');
const path = require('path');
const https = require('https');
const http = require('http');

const OPENAPI_URL = process.env.HOOK0_API_URL
  ? `${process.env.HOOK0_API_URL}/api/v1/swagger.json`
  : process.env.NODE_ENV === 'production' || process.env.CI
    ? 'https://app.hook0.com/api/v1/swagger.json'
    : 'http://localhost:8080/api/v1/swagger.json';

const OUTPUT_DIR = path.join(__dirname, '..', 'openapi');
const OUTPUT_FILE = path.join(OUTPUT_DIR, 'hook0-api.json');
const STATIC_DIR = path.join(__dirname, '..', 'static');
const STATIC_FILE = path.join(STATIC_DIR, 'hook0-api.json');

/** Give up on an unresponsive API rather than hanging the build forever. */
const REQUEST_TIMEOUT_MS = 30000;

/** Refuse a response too large to be a specification. */
const MAX_SPEC_BYTES = 16 * 1024 * 1024;

const DEFAULT_DESCRIPTION =
  'Hook0 is a robust webhook infrastructure that handles event delivery, retries, and monitoring for your applications.';

/**
 * Accumulate the body of a byte stream, refusing one larger than `limit` bytes.
 * Resolves to `{ available: true, body }` or `{ available: false, reason }`.
 *
 * The chunks are held as bytes and decoded once, at the end. Decoding each
 * chunk on its own would corrupt every character whose UTF-8 bytes happen to
 * straddle two of them — a `—` cut in half becomes replacement characters, in
 * a document that still parses as JSON. That damage is silent, and it outlives
 * the build that caused it: the same text is written to the published
 * specification and to the cache the next build falls back on. Holding bytes
 * also keeps the ceiling honest, since it is spent in the unit it is named in.
 */
const readBody = (stream, limit) =>
  new Promise((resolve) => {
    const chunks = [];
    let size = 0;
    stream.on('data', (chunk) => {
      size += chunk.length;
      if (size > limit) {
        stream.destroy();
        resolve({ available: false, reason: `sent more than ${limit} bytes` });
        return;
      }
      chunks.push(chunk);
    });
    stream.on('end', () => {
      resolve({ available: true, body: Buffer.concat(chunks).toString('utf8') });
    });
    stream.on('error', (error) => {
      resolve({ available: false, reason: `failed mid-response: ${error.message}` });
    });
  });

/**
 * Read the specification over HTTP.
 * Resolves to `{ available: true, spec }` or `{ available: false, reason }`;
 * it never rejects, so the caller can try the next source.
 */
const fetchFromApi = () =>
  new Promise((resolve) => {
    const protocol = OPENAPI_URL.startsWith('https') ? https : http;
    const request = protocol.get(OPENAPI_URL, (response) => {
      if (response.statusCode !== 200) {
        response.resume();
        resolve({
          available: false,
          reason: `${OPENAPI_URL} answered with status ${response.statusCode}`,
        });
        return;
      }

      readBody(response, MAX_SPEC_BYTES).then((read) => {
        if (!read.available) {
          request.destroy();
          resolve({ available: false, reason: `${OPENAPI_URL} ${read.reason}` });
          return;
        }
        resolve(parseSpec(read.body, OPENAPI_URL));
      });
    });

    request.setTimeout(REQUEST_TIMEOUT_MS, () => {
      request.destroy();
      resolve({
        available: false,
        reason: `${OPENAPI_URL} did not answer within ${REQUEST_TIMEOUT_MS} ms`,
      });
    });

    request.on('error', (error) => {
      resolve({ available: false, reason: `${OPENAPI_URL} is unreachable: ${error.message}` });
    });
  });

/**
 * Read the specification a previous build left behind. Acceptable as a source,
 * but held to the same bar as the network: a cache that is missing, unreadable
 * or unusable is simply not a source.
 */
const readFromCache = () =>
  fs.promises
    .readFile(OUTPUT_FILE, 'utf8')
    .then((body) => parseSpec(body, OUTPUT_FILE))
    .catch((error) => ({ available: false, reason: `${OUTPUT_FILE}: ${error.message}` }));

const parseSpec = (body, origin) =>
  Promise.resolve()
    .then(() => JSON.parse(body))
    .then((spec) => ({ available: true, spec, origin }))
    .catch((error) => ({ available: false, reason: `${origin} is not valid JSON: ${error.message}` }));

/**
 * Why this document cannot be published as an API reference, or an empty string
 * when it can. A specification without a single path documents nothing, whatever
 * it came from.
 */
const unusableReason = (spec) => {
  if (typeof spec !== 'object' || spec === null || Array.isArray(spec)) {
    return 'it is not a JSON object';
  }
  const paths = spec.paths;
  if (typeof paths !== 'object' || paths === null || Array.isArray(paths)) {
    return 'it has no `paths` object';
  }
  if (Object.keys(paths).length === 0) {
    return 'it documents no path at all';
  }
  return '';
};

/** Rewrite biscuit security schemes under the names the reference shows. */
const renameSecurityScheme = (spec, from, to, description) => {
  const schemes = spec.components.securitySchemes;
  if (schemes[from] === undefined) {
    return;
  }
  schemes[to] = { ...schemes[from], description };
  delete schemes[from];

  const rename = (requirements) => {
    if (!Array.isArray(requirements)) {
      return requirements;
    }
    return requirements.map((requirement) =>
      requirement[from] === undefined ? requirement : { [to]: requirement[from] }
    );
  };

  if (Array.isArray(spec.security)) {
    spec.security = rename(spec.security);
  }
  for (const pathItem of Object.values(spec.paths)) {
    for (const operation of Object.values(pathItem)) {
      if (typeof operation === 'object' && operation !== null && operation.security !== undefined) {
        operation.security = rename(operation.security);
      }
    }
  }
};

/** Turn the served document into the one the documentation site publishes. */
const enhanceSpec = (spec) => {
  if (!Array.isArray(spec.servers)) {
    spec.servers = [];
  }
  if (!spec.servers.some((server) => server.url === 'https://app.hook0.com')) {
    spec.servers.push({ url: 'https://app.hook0.com', description: 'Production API' });
  }

  if (typeof spec.info === 'object' && spec.info !== null) {
    if (typeof spec.info.description !== 'string' || spec.info.description.length === 0) {
      spec.info.description = DEFAULT_DESCRIPTION;
    }
    spec.info.contact = {
      name: 'Hook0 Support',
      url: 'https://www.hook0.com',
      email: 'support@hook0.com',
    };
    spec.info.license = {
      name: 'Apache 2.0',
      url: 'https://www.apache.org/licenses/LICENSE-2.0.html',
    };
  }

  if (
    typeof spec.components === 'object' &&
    spec.components !== null &&
    typeof spec.components.securitySchemes === 'object' &&
    spec.components.securitySchemes !== null
  ) {
    renameSecurityScheme(
      spec,
      'biscuit',
      'apiToken',
      'API Token authentication. Use the format: `Bearer YOUR_API_TOKEN`'
    );
    renameSecurityScheme(
      spec,
      'biscuit_user_access',
      'userAccessToken',
      'User access token for authentication'
    );
    renameSecurityScheme(
      spec,
      'biscuit_refresh',
      'refreshToken',
      'Refresh token for obtaining new access tokens'
    );
  }

  // The `mcp` tag drives tool generation, it means nothing to a reader.
  for (const pathItem of Object.values(spec.paths)) {
    for (const operation of Object.values(pathItem)) {
      if (typeof operation === 'object' && operation !== null && Array.isArray(operation.tags)) {
        operation.tags = operation.tags.filter((tag) => tag !== 'mcp');
      }
    }
  }
  if (Array.isArray(spec.tags)) {
    spec.tags = spec.tags.filter((tag) => tag.name !== 'mcp');
  }

  return spec;
};

const writeSpec = (spec) => {
  const serialized = JSON.stringify(spec, null, 2);
  fs.mkdirSync(OUTPUT_DIR, { recursive: true });
  fs.mkdirSync(STATIC_DIR, { recursive: true });
  // Both copies are written from the same document whatever the source: the
  // reference the site serves comes from the static one.
  fs.writeFileSync(OUTPUT_FILE, serialized);
  fs.writeFileSync(STATIC_FILE, serialized);
  console.log(`✅ OpenAPI spec saved to: ${OUTPUT_FILE}`);
  console.log(`✅ OpenAPI spec copied to: ${STATIC_FILE}`);
};

const giveUp = (reasons) => {
  console.error('❌ No usable OpenAPI specification, refusing to build the documentation.');
  console.error('   Publishing the site now would serve an API reference with no endpoint.');
  for (const reason of reasons) {
    console.error(`   · ${reason}`);
  }
  console.error('   Point HOOK0_API_URL at a reachable Hook0 instance, for example:');
  console.error('     HOOK0_API_URL=https://app.hook0.com npm run fetch-openapi');
  console.error('   or start one locally with `docker compose up -d` and retry.');
  process.exit(1);
};

const main = () => {
  console.log(`📥 Fetching OpenAPI spec from: ${OPENAPI_URL}`);

  return fetchFromApi()
    .then((fromApi) => {
      if (fromApi.available) {
        return fromApi;
      }
      console.warn(`⚠️  ${fromApi.reason}`);
      console.warn(`   Falling back to the specification left by a previous build`);
      return readFromCache().then((fromCache) => ({
        ...fromCache,
        reasons: fromCache.available ? [] : [fromApi.reason, fromCache.reason],
      }));
    })
    .then((source) => {
      if (!source.available) {
        giveUp(source.reasons === undefined ? [source.reason] : source.reasons);
        return;
      }

      const reason = unusableReason(source.spec);
      if (reason !== '') {
        giveUp([`${source.origin} cannot be published: ${reason}`]);
        return;
      }

      console.log(`   Specification read from ${source.origin}`);
      writeSpec(enhanceSpec(source.spec));
      console.log('📄 OpenAPI spec ready for documentation generation');
    })
    .catch((error) => {
      giveUp([`the fetch script itself failed: ${error.stack}`]);
    });
};

// Importing the script exposes its parts to tests; only running it builds.
if (require.main === module) {
  main();
}

module.exports = {
  readBody,
  fetchFromApi,
  parseSpec,
  unusableReason,
  enhanceSpec,
  renameSecurityScheme,
};
