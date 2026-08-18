// The rest of the file, for every TypeScript example of the SDK reference.
//
// A snippet on a page is written for a reader: it leaves out the imports it already showed higher
// up the page, it assumes a client or an event is already built, and it names an application id, a
// token or a secret without saying where it came from. Each region below is the file that snippet
// would live in, with a hole where it goes. The page points at one by name on the fence, so what a
// snippet is standing on is one word away from the snippet itself.
//
// `moduleDetection: force` (set in `tsconfig.json`) is what lets these regions sit next to each
// other without naming a distinct package the way the Go harness does with `package {{name}}`: every
// assembled file is its own module regardless of whether it happens to carry an `import` of its own,
// so two regions both declaring `const hook0` never collide.
//
// A region never imports a name a snippet dropped into it might import again on its own: two
// `import`s of the same binding from the same module is a file `tsc` refuses, so a region used by a
// snippet that already shows, say, `Hook0ClientError` leaves that one for the snippet to bring.
//
// This file never compiles as it stands, and `tsc` never sees it whole: each region becomes the one
// file of its own example, which is why a region that needs a name in scope declares it above the
// hole rather than assuming the file around it.

// HARNESS program
// What a fully self-contained snippet needs from this file: nothing. It carries its own imports and
// declares everything it uses, the way a reader would paste it into a fresh file.
EXAMPLE

// END HARNESS

// HARNESS event
// For the snippet that shows what the seven arguments of `Event` are, without building a client to
// send one with.
import { Event } from 'hook0-client';

// The identifier a send may be given, when the caller has one of its own to give.
const eventId = '018f7c1e-0000-7000-8000-000000000000';

const built =
  EXAMPLE

// END HARNESS

// HARNESS usingClient
// For a snippet that calls a method on a client it never builds.
import { Event, Hook0Client } from 'hook0-client';

const hook0 = new Hook0Client(
  'https://app.hook0.com/api/v1',
  '0d0ea1e0-8b1f-4f4d-9c2a-1b5c9a3d7e21',
  'a-service-token'
);

EXAMPLE

// END HARNESS

// HARNESS usingClientAndEvent
// For a snippet that reads a client, an event and `Hook0ClientError` without importing any of
// them, and logs through whatever the reader's own logger is — `console` stands in for it, since any
// object with an `error` method would do exactly as well here.
import { Event, Hook0Client, Hook0ClientError } from 'hook0-client';

const hook0 = new Hook0Client(
  'https://app.hook0.com/api/v1',
  '0d0ea1e0-8b1f-4f4d-9c2a-1b5c9a3d7e21',
  'a-service-token'
);
const event = new Event(
  'user.account.created',
  JSON.stringify({ user_id: 'user_123' }),
  'application/json',
  {}
);
const logger = console;

EXAMPLE

// END HARNESS

// HARNESS usingEvent
// For a snippet that reads a client and an event it never builds, but imports `Hook0ClientError`
// itself.
import { Event, Hook0Client } from 'hook0-client';

const hook0 = new Hook0Client(
  'https://app.hook0.com/api/v1',
  '0d0ea1e0-8b1f-4f4d-9c2a-1b5c9a3d7e21',
  'a-service-token'
);
const event = new Event(
  'user.account.created',
  JSON.stringify({ user_id: 'user_123' }),
  'application/json',
  {}
);

EXAMPLE

// END HARNESS

// HARNESS eventType
// For the snippet that reads an event type back apart, which brings everything it names itself.
EXAMPLE

// END HARNESS

// HARNESS verify
// For the shortest form of a verification: the four values a delivery arrives with, and the
// tolerance the caller picks.
const signature = 't=1636936200,v0=00';
const rawBody = Buffer.from('{}');
const headers = new Headers();
const subscriptionSecret = 'a-subscription-secret';

EXAMPLE

// END HARNESS

// HARNESS restApiGroup
// For a snippet that reaches a generated API group through a transport it never builds. The
// transport answers every request the same way, which is enough for a page showing how the group is
// constructed rather than what it returns.
const applicationId = '0d0ea1e0-8b1f-4f4d-9c2a-1b5c9a3d7e21';
const transport: import('hook0-client').generated.Transport = {
  async request() {
    return { status: 200, payload: '{}' };
  },
};

EXAMPLE

// END HARNESS

// HARNESS restApiTransport
// For the snippet that writes that transport, which reaches the API with a token it never declares.
const token = 'a-service-token';

EXAMPLE

// END HARNESS

// HARNESS options
// For a snippet that builds its own client with its own bounds, from an API URL, an application id
// and a token none of which it declares.
const apiUrl = 'https://app.hook0.com/api/v1';
const applicationId = '0d0ea1e0-8b1f-4f4d-9c2a-1b5c9a3d7e21';
const token = 'a-service-token';

EXAMPLE

// END HARNESS

// HARNESS webhookHandlerFull
// For the one webhook handler that builds its own Express app: everything it reaches is its own,
// except the function it hands the parsed payload to once a signature has already been verified.
function processWebhook(payload: unknown): void {
  console.log('processing', payload);
}

EXAMPLE

// END HARNESS

// HARNESS webhookHandlerUsingApp
// For the overview's webhook handler, which is shown without the Express app it is registered on —
// the SDK page beside it is where a reader meets that part.
import express from 'express';

const app = express();

EXAMPLE

// END HARNESS
