/**
 * The fetch script decides whether the site has an API reference to publish at
 * all, and every byte it lets through ends up both in the published document
 * and in the cache the next build falls back on. These tests hold that gate
 * shut: a body must survive transport unchanged whatever the chunking, and a
 * document that documents nothing must never reach the site.
 */

const test = require('node:test');
const assert = require('node:assert/strict');
const http = require('node:http');
const { Readable } = require('node:stream');

const {
  readBody,
  parseSpec,
  unusableReason,
  enhanceSpec,
  renameSecurityScheme,
} = require('./fetch-openapi.js');

/** A stream that emits exactly the given buffers, one `data` event each. */
const streamOf = (chunks) => Readable.from(chunks);

/** The given text as buffers cut at `index`, so a character can straddle both. */
const cutAt = (text, index) => {
  const bytes = Buffer.from(text, 'utf8');
  return [bytes.subarray(0, index), bytes.subarray(index)];
};

/** Every byte of the given text as its own chunk: every boundary is exercised. */
const byteByByte = (text) => {
  const bytes = Buffer.from(text, 'utf8');
  return Array.from({ length: bytes.length }, (_, i) => bytes.subarray(i, i + 1));
};

const A_MEGABYTE = 1024 * 1024;

// The real specification carries an em dash in `info.description`.
const DOCUMENT = '{"info":{"description":"Core REST API of Hook0 — Webhooks 🪝"}}';

test('a character split across two chunks arrives whole', () => {
  const emDash = Buffer.from(DOCUMENT, 'utf8').indexOf(Buffer.from('—', 'utf8'));
  // The em dash spans three bytes: cut after its first, then after its second.
  return Promise.all(
    [1, 2].map((offset) =>
      readBody(streamOf(cutAt(DOCUMENT, emDash + offset)), A_MEGABYTE).then((read) => {
        assert.deepEqual(read, { available: true, body: DOCUMENT });
      })
    )
  );
});

test('a four-byte character split across two chunks arrives whole', () => {
  const hook = Buffer.from(DOCUMENT, 'utf8').indexOf(Buffer.from('🪝', 'utf8'));
  return Promise.all(
    [1, 2, 3].map((offset) =>
      readBody(streamOf(cutAt(DOCUMENT, hook + offset)), A_MEGABYTE).then((read) => {
        assert.deepEqual(read, { available: true, body: DOCUMENT });
      })
    )
  );
});

test('a document delivered one byte at a time is still parseable', () =>
  readBody(streamOf(byteByByte(DOCUMENT)), A_MEGABYTE).then((read) => {
    assert.equal(read.available, true);
    assert.equal(read.body.includes('�'), false);
    return parseSpec(read.body, 'the stream').then((parsed) => {
      assert.equal(parsed.available, true);
      assert.equal(parsed.spec.info.description, 'Core REST API of Hook0 — Webhooks 🪝');
    });
  }));

test('the ceiling is spent in bytes, not in characters', () => {
  // Four em dashes: four characters, twelve bytes. A ceiling counting decoded
  // characters would wave this through.
  const body = '—'.repeat(4);
  return readBody(streamOf([Buffer.from(body, 'utf8')]), 8).then((read) => {
    assert.equal(read.available, false);
    assert.match(read.reason, /sent more than 8 bytes/);
  });
});

test('a body of exactly the ceiling is accepted', () => {
  const body = '—'.repeat(4); // twelve bytes
  return readBody(streamOf(byteByByte(body)), 12).then((read) => {
    assert.deepEqual(read, { available: true, body });
  });
});

test('a stream that fails mid-response is not a source', () => {
  const failing = new Readable({
    read() {
      this.push(Buffer.from('{"pat'));
      this.destroy(new Error('socket hang up'));
    },
  });
  return readBody(failing, A_MEGABYTE).then((read) => {
    assert.equal(read.available, false);
    assert.match(read.reason, /socket hang up/);
  });
});

test('a specification served over HTTP keeps its non-ASCII text', { timeout: 10000 }, () => {
  const server = http.createServer((request, response) => {
    response.writeHead(200, { 'Content-Type': 'application/json' });
    const emDash = Buffer.from(DOCUMENT, 'utf8').indexOf(Buffer.from('—', 'utf8'));
    const [head, tail] = cutAt(DOCUMENT, emDash + 1);
    response.write(head);
    // Flush the first half on its own so the em dash lands across two packets.
    setTimeout(() => response.end(tail), 10);
  });

  return new Promise((listening) => server.listen(0, '127.0.0.1', listening))
    .then(() => {
      process.env.HOOK0_API_URL = `http://127.0.0.1:${server.address().port}`;
      delete require.cache[require.resolve('./fetch-openapi.js')];
      return require('./fetch-openapi.js').fetchFromApi();
    })
    .then((source) => {
      assert.equal(source.available, true);
      assert.equal(source.spec.info.description, 'Core REST API of Hook0 — Webhooks 🪝');
    })
    .then(
      () => {
        delete process.env.HOOK0_API_URL;
        server.close();
      },
      (error) => {
        delete process.env.HOOK0_API_URL;
        server.close();
        throw error;
      }
    );
});

test('text that is not JSON names where it came from', () =>
  parseSpec('<html>404</html>', 'https://app.hook0.com/api/v1/swagger.json').then((parsed) => {
    assert.equal(parsed.available, false);
    assert.match(parsed.reason, /^https:\/\/app\.hook0\.com\/api\/v1\/swagger\.json is not valid JSON/);
  }));

test('a document that is not a JSON object cannot be published', () => {
  assert.equal(unusableReason('a string'), 'it is not a JSON object');
  assert.equal(unusableReason(null), 'it is not a JSON object');
  assert.equal(unusableReason([{ paths: { '/a': {} } }]), 'it is not a JSON object');
  assert.equal(unusableReason(42), 'it is not a JSON object');
});

test('a document without a paths object cannot be published', () => {
  assert.equal(unusableReason({ openapi: '3.0.0' }), 'it has no `paths` object');
  assert.equal(unusableReason({ paths: null }), 'it has no `paths` object');
  assert.equal(unusableReason({ paths: [] }), 'it has no `paths` object');
  assert.equal(unusableReason({ paths: 'none' }), 'it has no `paths` object');
});

test('a document with no endpoint at all cannot be published', () => {
  assert.equal(unusableReason({ paths: {} }), 'it documents no path at all');
});

test('a document with one endpoint can be published', () => {
  assert.equal(unusableReason({ paths: { '/api/v1/events': { get: {} } } }), '');
});

test('a specification without security schemes is still enhanced', () => {
  const enhanced = enhanceSpec({
    info: { title: 'Hook0', description: '' },
    paths: { '/api/v1/events': { get: { tags: ['events', 'mcp'] } } },
  });

  assert.deepEqual(enhanced.servers, [
    { url: 'https://app.hook0.com', description: 'Production API' },
  ]);
  assert.match(enhanced.info.description, /^Hook0 is a robust webhook infrastructure/);
  assert.equal(enhanced.info.contact.email, 'support@hook0.com');
  assert.deepEqual(enhanced.paths['/api/v1/events'].get.tags, ['events']);
});

test('a production server already declared is not declared twice', () => {
  const enhanced = enhanceSpec({
    servers: [{ url: 'https://app.hook0.com', description: 'Hook0 Cloud' }],
    info: { title: 'Hook0', description: 'Already written' },
    paths: { '/api/v1/events': { get: {} } },
  });

  assert.deepEqual(enhanced.servers, [
    { url: 'https://app.hook0.com', description: 'Hook0 Cloud' },
  ]);
  assert.equal(enhanced.info.description, 'Already written');
});

test('biscuit schemes are renamed wherever they are required', () => {
  const enhanced = enhanceSpec({
    info: { title: 'Hook0', description: 'Hook0' },
    components: {
      securitySchemes: {
        biscuit: { type: 'http', scheme: 'bearer' },
        biscuit_refresh: { type: 'http', scheme: 'bearer' },
      },
    },
    security: [{ biscuit: [] }],
    paths: { '/api/v1/events': { get: { security: [{ biscuit_refresh: ['read'] }] } } },
  });

  assert.deepEqual(Object.keys(enhanced.components.securitySchemes), [
    'apiToken',
    'refreshToken',
  ]);
  assert.equal(enhanced.components.securitySchemes.apiToken.scheme, 'bearer');
  assert.deepEqual(enhanced.security, [{ apiToken: [] }]);
  assert.deepEqual(enhanced.paths['/api/v1/events'].get.security, [{ refreshToken: ['read'] }]);
});

test('a security field that is not a list is left as it was found', () => {
  const enhanced = enhanceSpec({
    info: { title: 'Hook0', description: 'Hook0' },
    components: { securitySchemes: { biscuit: { type: 'http' } } },
    security: 'everything',
    paths: { '/api/v1/events': { get: { security: 'everything' } } },
  });

  assert.equal(enhanced.security, 'everything');
  assert.equal(enhanced.paths['/api/v1/events'].get.security, 'everything');
  assert.deepEqual(Object.keys(enhanced.components.securitySchemes), ['apiToken']);
});

test('a scheme the API does not serve is left alone', () => {
  const spec = {
    components: { securitySchemes: { apiToken: { type: 'http' } } },
    security: [{ apiToken: [] }],
    paths: { '/api/v1/events': { get: {} } },
  };
  renameSecurityScheme(spec, 'biscuit', 'apiToken', 'API Token authentication');

  assert.deepEqual(spec.components.securitySchemes, { apiToken: { type: 'http' } });
  assert.deepEqual(spec.security, [{ apiToken: [] }]);
});

test('the mcp tag is stripped from the tag list too', () => {
  const enhanced = enhanceSpec({
    info: { title: 'Hook0', description: 'Hook0' },
    tags: [{ name: 'events' }, { name: 'mcp' }],
    paths: { '/api/v1/events': { get: { tags: ['mcp'] } } },
  });

  assert.deepEqual(enhanced.tags, [{ name: 'events' }]);
  assert.deepEqual(enhanced.paths['/api/v1/events'].get.tags, []);
});
