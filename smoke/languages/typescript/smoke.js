// The TypeScript client against a Hook0 that is really running.
//
// Three things the loopback suite cannot ask: whether an application secret the API minted is
// accepted, whether a second send under an identifier already ingested is reported as the conflict
// it is, and whether a signature the output worker computed verifies. Everything else about this
// client is settled by `clients/typescript/tests`.
//
// Reached as `require('hook0-client')`, which is one of the two ways the package promises to be
// reachable and the only one that goes through its `exports` map.

const fs = require('fs');
const path = require('path');
const { Event, Hook0Client, verifyWebhookSignature } = require('hook0-client');

// The conflict the API answers a duplicated ingestion with.
const ALREADY_INGESTED = 'EventAlreadyIngested';

function setting(name) {
  const value = process.env[name];
  if (typeof value !== 'string' || value.length === 0) {
    throw new Error(`${name} is not set`);
  }
  return value;
}

// The event both sends carry, under the identifier the caller names.
function event(eventType, eventId) {
  return new Event(
    eventType,
    '{"from":"the typescript smoke"}',
    'application/json',
    { language: 'typescript' },
    undefined,
    undefined,
    eventId
  );
}

// Verifies what the output worker really delivered, with this client's own verification.
function verify(delivery) {
  const read = (part) => fs.readFileSync(path.join(delivery, part), 'utf8');
  const headers = new Headers();
  for (const line of read('headers').split('\n')) {
    const at = line.indexOf(': ');
    if (at > 0) {
      headers.append(line.slice(0, at), line.slice(at + 2));
    }
  }
  verifyWebhookSignature(
    read('signature').trim(),
    fs.readFileSync(path.join(delivery, 'body')),
    headers,
    read('secret').trim(),
    Number(read('tolerance').trim())
  );
}

const apiUrl = setting('HOOK0_API_URL');
const applicationId = setting('HOOK0_APPLICATION_ID');
const eventType = setting('HOOK0_EVENT_TYPE');
const delivery = setting('HOOK0_DELIVERY');
const client = new Hook0Client(apiUrl, applicationId, setting('HOOK0_TOKEN'));

client
  .sendEvent(event(eventType, undefined))
  .catch((refused) => {
    throw new Error(
      `the instance refused the first send, with ${apiUrl} as the API URL: ${refused.message}`
    );
  })
  .then((sent) => {
    console.log(`ingested ${sent}`);
    return client.sendEvent(event(eventType, sent)).then(
      () => {
        throw new Error('sending the same event twice was accepted twice');
      },
      (refused) => {
        const said = String(refused && refused.message ? refused.message : refused);
        if (!said.includes(ALREADY_INGESTED)) {
          throw new Error(`the second send failed without naming ${ALREADY_INGESTED}: ${said}`);
        }
        console.log(`the second send reported ${ALREADY_INGESTED}`);
      }
    );
  })
  .then(() => {
    verify(delivery);
    console.log('the signature the instance produced verifies');
  })
  .catch((refused) => {
    console.error(String(refused && refused.stack ? refused.stack : refused));
    process.exitCode = 1;
  });
