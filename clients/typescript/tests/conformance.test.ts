import { describe, expect, test } from '@jest/globals';
import * as fs from 'fs';
import * as http from 'http';
import * as path from 'path';

import {
  DEFAULT_MAX_PAYLOAD_BYTES,
  DEFAULT_MAX_RESPONSE_BYTES,
  Event,
  Hook0Client,
  Hook0ClientError,
  Hook0ClientOptions,
  MAX_ATTEMPTS_CAP,
  MAX_HEADER_BYTES,
  MAX_HEAD_BYTES,
  MAX_RESPONSE_HEADERS,
  RetryPolicy,
  verifyWebhookSignatureWithCurrentTime,
} from '../src/index';

/**
 * The cases the shared conformance corpus dictates, run against this client.
 *
 * The corpus sits at `clients/conformance`, is hand-authored, and is read by the suite of every SDK.
 * Nothing below writes down a verdict, a bound or a signature of its own: they are read out of the
 * committed documents and this client is driven against them over a real socket. A case added to
 * the corpus is therefore exercised here without this file being touched, and a verdict changed
 * there fails here until this client agrees with it again.
 */

/** Where the shared contract sits, from the directory this suite runs out of. */
const CORPUS = path.resolve(__dirname, '..', '..', 'conformance');

/**
 * Largest document of the corpus read back. The corpus is committed, so one above this is one that
 * grew out of shape rather than one somebody meant.
 */
const MAX_CORPUS_BYTES = 512 * 1024;

/** One document of the shared contract, bounded before it is parsed. */
function corpus(document: string): Record<string, unknown> {
  const at = path.join(CORPUS, document);
  const size = fs.statSync(at).size;
  if (size > MAX_CORPUS_BYTES) {
    throw new Error(`${at} is ${size} bytes long, above the ${MAX_CORPUS_BYTES} read back`);
  }
  return JSON.parse(fs.readFileSync(at, 'utf8')) as Record<string, unknown>;
}

/** The entries the corpus carries at `at`, which no case may find empty. */
function entries(document: unknown, at: string): Record<string, unknown>[] {
  let walked: unknown = document;
  for (const step of at.split('.')) {
    walked = (walked as Record<string, unknown>)[step];
  }
  if (!Array.isArray(walked) || walked.length === 0) {
    throw new Error(`the shared contract carries nothing at \`${at}\``);
  }
  return walked as Record<string, unknown>[];
}

/** What the corpus wrote at `field` of one entry, as text. */
function text(entry: Record<string, unknown>, field: string): string {
  const written = entry[field];
  if (typeof written !== 'string') {
    throw new Error(`\`${field}\` is not the text the contract declares`);
  }
  return written;
}

/** What the corpus wrote at `field` of one entry, as a whole number. */
function number(entry: Record<string, unknown>, field: string): number {
  const written = entry[field];
  if (typeof written !== 'number') {
    throw new Error(`\`${field}\` is not the number the contract declares`);
  }
  return written;
}

/** What the corpus wrote at `field` of one entry, as a verdict. */
function flag(entry: Record<string, unknown>, field: string): boolean {
  const written = entry[field];
  if (typeof written !== 'boolean') {
    throw new Error(`\`${field}\` is not the verdict the contract declares`);
  }
  return written;
}

/**
 * What a value of the request document is made of, once the holes this suite can speak for are
 * filled in.
 *
 * A value is a template: `${name}` is a hole and everything around it is literal. A hole named in
 * `bound` becomes part of the literal text around it; one that is not is a hole no suite can fill
 * without reimplementing the client it is testing, and it separates two chunks. A template whose
 * holes are all bound is therefore one chunk, and the whole value is that chunk.
 */
function templateChunks(template: string, bound: Map<string, string>): string[] {
  const chunks = [''];
  let rest = template;

  for (let opened = rest.indexOf('${'); opened >= 0; opened = rest.indexOf('${')) {
    const closed = rest.indexOf('}', opened);
    if (closed < 0) {
      break;
    }
    chunks[chunks.length - 1] += rest.slice(0, opened);

    const filled = bound.get(rest.slice(opened + 2, closed));
    if (filled === undefined) {
      chunks.push('');
    } else {
      chunks[chunks.length - 1] += filled;
    }
    rest = rest.slice(closed + 1);
  }

  chunks[chunks.length - 1] += rest;
  return chunks;
}

/**
 * Whether what arrived is what those chunks describe: the literal text in order, anchored at both
 * ends, with something non-empty standing in every hole between them.
 */
function matchesChunks(chunks: string[], carried: string): boolean {
  const first = chunks[0];
  if (chunks.length === 1) {
    return carried === first;
  }
  if (!carried.startsWith(first)) {
    return false;
  }

  let rest = carried.slice(first.length);
  for (const chunk of chunks.slice(1, -1)) {
    // A hole stands before this chunk, and nothing is not something, so the search starts past
    // whatever fills it.
    const found = rest.slice(1).indexOf(chunk);
    if (found < 0) {
      return false;
    }
    rest = rest.slice(1 + found + chunk.length);
  }

  const last = chunks[chunks.length - 1];
  return rest.length > last.length && rest.endsWith(last);
}

/** What arrived under one header name, however the runtime chose to hand it over. */
function carriedText(headers: http.IncomingHttpHeaders, name: string): string {
  const arrived = headers[name];
  if (Array.isArray(arrived)) {
    return arrived.join(', ');
  }
  if (typeof arrived === 'string') {
    return arrived;
  }
  return '';
}

const RETRY = corpus('retry.json');
const BOUNDS = corpus('bounds.json').bounds as Record<string, number>;
const REQUEST = corpus('request.json');
const SIGNATURE = corpus('signature.json');

/**
 * The version this package is published under, which is the one it reports on the wire. It is read
 * out of the manifest rather than written down again here, so that the number the client carries
 * and the number the package ships as cannot disagree without this suite saying so.
 */
const VERSION = (
  JSON.parse(fs.readFileSync(path.resolve(__dirname, '..', 'package.json'), 'utf8')) as Record<
    string,
    string
  >
).version;

/** The credential every client below is built with, and the one the corpus expects on the wire. */
const TOKEN = 'token-xyz';

/** The ID the API answers with once it takes the event. */
const INGESTED_ID = '01961234-5678-7abc-8def-0123456789ab';

/** The schedule a case that is not about waiting spends between attempts, in milliseconds. */
const PROMPT_BACKOFF_MS = 5;

/**
 * The budget the delay cases share. A delay the API names above it is expected to be cut down to
 * it, so it also bounds what those cases cost.
 */
const DELAY_BUDGET_MS = 1_100;

/**
 * What a wait may overshoot by before it is read as more than what was asked for: a loopback round
 * trip, a timer and a scheduler all sit inside it.
 */
const DELAY_SLACK_MS = 600;

/**
 * What a wait may come back early by before it is read as a wait that did not happen.
 *
 * A delay is scheduled on one clock and measured on another — the wait goes through `setTimeout`,
 * and what this suite reads is `performance.now()` — so the two disagree by a fraction of a
 * millisecond in either direction. Measured here, a bare `setTimeout` came back before its own
 * deadline on that clock in nine runs out of three hundred, by at most 0.784 ms; read on
 * `Date.now()` instead it came back a whole millisecond early, since that clock is truncated to
 * milliseconds. Neither is a defect, and no client can fix either.
 *
 * Twenty milliseconds is some twenty-five times the worst undershoot measured and twenty times the
 * millisecond `setTimeout` schedules on, so a slower or noisier runner is well inside it. It is
 * also a fiftieth of the shortest delay any case asserts, which is a second, and that headroom is
 * the point: a client waiting nine tenths of a one-second delay comes back about fifty-two
 * milliseconds short once the two loopback round trips inside the measurement are counted, so an
 * allowance of fifty would have let that regression through on the very case it matters for. This
 * one leaves some eighty milliseconds of room before a tenth of a delay could hide in it.
 */
const CLOCK_SLACK_MS = 20;

/** Every case talks to a loopback socket, so none of them has any reason to take this long. */
const TEST_TIMEOUT_MS = 20_000;

/** How a refusal the corpus names reads in this client's own words. */
const REFUSALS: Record<string, string> = {
  code_not_hexadecimal: 'Could not parse signature',
  header_not_delivered: 'was not provided with a value',
  code_mismatch: 'Invalid signature',
  outside_tolerance: 'outside the tolerance window',
};

/** What the API answers to one request, in the order the case scripted it. */
interface ScriptedResponse {
  status: number;
  body: unknown;
  headers: Record<string, string>;
  heldForMs: number;
}

function answer(
  status: number,
  body: unknown,
  headers: Record<string, string> = {},
  heldForMs = 0
): ScriptedResponse {
  return { status, body, headers, heldForMs };
}

/** What the API says when it refuses a request, in the shape every Hook0 failure takes. */
function refusal(
  status: number,
  problem: string,
  headers: Record<string, string> = {}
): ScriptedResponse {
  return answer(
    status,
    {
      id: problem,
      status,
      title: 'refused',
      detail: 'what the corpus scripted',
      type: `https://hook0.com/documentation/errors/${problem}`,
    },
    headers
  );
}

function ingested(): ScriptedResponse {
  return answer(201, {
    application_id: 'app-123',
    event_id: INGESTED_ID,
    received_at: '2026-01-01T00:00:00Z',
  });
}

function anEvent(): Event {
  return new Event('auth.user.create', '{"email": "test@example.com"}', 'application/json', {
    environment: 'production',
  });
}

/** No request here is anywhere near this large; the cap bounds what one connection can buffer. */
const MAX_REQUEST_BODY_BYTES = 64 * 1024;

/**
 * Resolves after `delayMs`, so an answer can be withheld from the client for a while.
 *
 * The wait is one a case is meant to outlive — a client that gave up on an answer moves on while
 * the API is still sitting on it — so the timer is told not to hold the runtime open on its own.
 */
function hold(delayMs: number): Promise<void> {
  if (delayMs === 0) {
    return Promise.resolve();
  }
  return new Promise((resolve) => {
    setTimeout(resolve, delayMs).unref();
  });
}

function readBody(request: http.IncomingMessage): Promise<string> {
  return new Promise((resolve, reject) => {
    const chunks: Buffer[] = [];
    let size = 0;

    request.on('data', (chunk: Buffer) => {
      size += chunk.length;
      if (size > MAX_REQUEST_BODY_BYTES) {
        request.destroy();
        reject(new Error(`Request body is larger than ${MAX_REQUEST_BODY_BYTES} bytes`));
        return;
      }
      chunks.push(chunk);
    });
    request.on('end', () => resolve(Buffer.concat(chunks).toString('utf8')));
    request.on('error', reject);
  });
}

/** A request the API received, as it read it off the wire. */
interface ReceivedRequest {
  headers: http.IncomingHttpHeaders;
  body: string;
}

/** A Hook0 API listening on a loopback port for the lifetime of one case. */
class FakeHook0Api {
  readonly received: ReceivedRequest[] = [];
  private readonly responses: ScriptedResponse[] = [];
  private answered = 0;
  private readonly server: http.Server;

  constructor() {
    this.server = http.createServer((request, response) => {
      readBody(request)
        .then((body) => {
          this.received.push({ headers: request.headers, body });
          const scripted = this.nextResponse();
          return hold(scripted.heldForMs).then(() => {
            response.writeHead(scripted.status, {
              'Content-Type': 'application/json',
              ...scripted.headers,
            });
            response.end(JSON.stringify(scripted.body));
          });
        })
        .catch((error: Error) => {
          response.writeHead(500, { 'Content-Type': 'application/json' });
          response.end(JSON.stringify({ error: error.message }));
        });
    });
  }

  /** Queues the answers the case expects the client to draw, in order. */
  willAnswer(...responses: ScriptedResponse[]): void {
    this.responses.push(...responses);
  }

  private nextResponse(): ScriptedResponse {
    if (this.answered >= this.responses.length) {
      // An unscripted request is one the client should not have issued, and a case detects it
      // through the request count.
      return refusal(500, 'InternalServerError');
    }
    const scripted = this.responses[this.answered];
    this.answered += 1;
    return scripted;
  }

  listen(): Promise<void> {
    return new Promise((resolve) => {
      this.server.listen(0, '127.0.0.1', () => resolve());
    });
  }

  get baseUrl(): string {
    const address = this.server.address();
    if (address === null || typeof address === 'string') {
      throw new Error('The fake Hook0 API is not listening on a TCP port');
    }
    return `http://127.0.0.1:${address.port}`;
  }

  close(): Promise<void> {
    return new Promise((resolve, reject) => {
      // `fetch` keeps its connections alive, and a live connection holds `close` open forever.
      this.server.closeAllConnections();
      this.server.close((error) => {
        if (error === undefined) {
          resolve();
        } else {
          reject(error);
        }
      });
    });
  }
}

/**
 * A retry schedule short enough that a case spends its time on requests rather than on waiting, and
 * whose budget is far above what its delays add up to.
 */
function promptOptions(
  maxAttempts: number,
  requestTimeoutMs = 5_000,
  maxResponseBytes = DEFAULT_MAX_RESPONSE_BYTES
): Hook0ClientOptions {
  return new Hook0ClientOptions(
    new RetryPolicy(maxAttempts, PROMPT_BACKOFF_MS, PROMPT_BACKOFF_MS, 1_000),
    requestTimeoutMs,
    DEFAULT_MAX_PAYLOAD_BYTES,
    maxResponseBytes
  );
}

/**
 * How many attempts a send made, whether it ended up ingesting the event, and what it said.
 *
 * A send that reached a server is counted by what that server received. One that never reached
 * anything — an API URL nothing can be sent to is the corpus's own example — is counted by what the
 * client says it did, which is also the message a caller is left holding: a misconfiguration retried
 * four times reads as a network that would not answer.
 */
async function issuedBy(
  api: FakeHook0Api,
  client: Hook0Client
): Promise<{ issued: number; survived: boolean; said: string }> {
  try {
    await client.sendEvent(anEvent());
    return { issued: api.received.length, survived: true, said: '' };
  } catch (refused) {
    const said = refused instanceof Error ? refused.message : String(refused);
    return {
      issued: Math.max(api.received.length, attemptsOf(said)),
      survived: false,
      said,
    };
  }
}

/**
 * How many attempts a client says it made, read out of what it told its caller. A send that gave up
 * after more than one attempt names the number; one that stopped at its first says nothing, and
 * made one.
 */
function attemptsOf(said: string): number {
  const named = /gave up after (\d+) attempts/.exec(said);
  if (named === null) {
    return 1;
  }
  return Number(named[1]);
}

/**
 * A problem the corpus says is worth repeating and that shares its status with one it says is not.
 * That is the answer the API names a delay beside, and the one a status alone cannot classify.
 */
function pacedProblem(): Record<string, unknown> {
  const problems = entries(RETRY, 'problems');
  const paced = problems.find(
    (rule) =>
      flag(rule, 'retryable') &&
      problems.some(
        (other) => number(other, 'status') === number(rule, 'status') && !flag(other, 'retryable')
      )
  );
  if (paced === undefined) {
    throw new Error('the corpus classifies no problem the API names a delay beside');
  }
  return paced;
}

/**
 * Runs one case against an API listening for the length of that case, and takes it down again
 * however the case ended: a socket left open outlives the case that opened it and holds the suite.
 */
function withApi<T>(run: (api: FakeHook0Api) => Promise<T>): Promise<T> {
  const api = new FakeHook0Api();
  return api
    .listen()
    .then(() => run(api))
    .then(
      (ended) => api.close().then(() => ended),
      (failed: unknown) => api.close().then(() => Promise.reject(failed))
    );
}

/** A client reaching one of those APIs, or whatever else a case wants to point it at. */
function client(api: FakeHook0Api, options: Hook0ClientOptions, baseUrl = ''): Hook0Client {
  return new Hook0Client(baseUrl === '' ? api.baseUrl : baseUrl, 'app-123', TOKEN, false, options);
}

/** How many requests a send made when the API answered that way and then took the event. */
function issuedFor(scripted: ScriptedResponse): Promise<{ issued: number; survived: boolean }> {
  return withApi((api) => {
    api.willAnswer(scripted, ingested());
    return issuedBy(api, client(api, promptOptions(4)));
  });
}

describe('the shared conformance corpus', () => {
  test(
    'every problem the corpus classifies is repeated as it says',
    async () => {
      // The status is not what decides: the corpus carries problems answering the same status with
      // opposite verdicts, and a client reading the status alone fails half of them.
      for (const rule of entries(RETRY, 'problems')) {
        const problem = text(rule, 'problem');
        const status = number(rule, 'status');
        const expected = flag(rule, 'retryable') ? 2 : 1;

        const { issued, survived } = await issuedFor(refusal(status, problem));

        expect({ problem, issued, survived }).toStrictEqual({
          problem,
          issued: expected,
          survived: flag(rule, 'retryable'),
        });
      }
    },
    TEST_TIMEOUT_MS
  );

  test(
    'every status the corpus rules on is repeated as it says',
    async () => {
      // A body naming no problem this client could read is also what an older client meets when the
      // API names a problem it has never heard of.
      for (const rule of entries(RETRY, 'statuses')) {
        const status = number(rule, 'status');
        const expected = flag(rule, 'retryable') ? 2 : 1;

        const { issued } = await issuedFor(refusal(status, 'AProblemThisClientHasNeverHeardOf'));

        expect({ status, issued }).toStrictEqual({ status, issued: expected });
      }
    },
    TEST_TIMEOUT_MS
  );

  /**
   * Makes this client meet one of the causes the corpus names, for real.
   *
   * The API takes the event on its second answer, so a cause the corpus calls retryable is one the
   * send comes back from, and one it does not is a send that stops at its first attempt.
   */
  function provoked(cause: string): Promise<{ survived: boolean; attempts: number; said: string }> {
    return withApi(async (api) => {
      switch (cause) {
        case 'no_answer': {
          api.willAnswer({ ...ingested(), heldForMs: 400 }, ingested());
          const { issued, survived, said } = await issuedBy(
            api,
            client(api, promptOptions(4, 100))
          );
          return { survived, attempts: issued, said };
        }
        case 'answer_above_a_bound': {
          api.willAnswer(answer(200, { padding: 'x'.repeat(1024) }), ingested());
          const { issued, survived, said } = await issuedBy(
            api,
            client(api, promptOptions(4, 5_000, 64))
          );
          return { survived, attempts: issued, said };
        }
        case 'unusable_api_url': {
          // Nothing listens, and nothing is sent: the URL names no scheme a request could travel
          // on.
          api.willAnswer(ingested());
          const { issued, survived, said } = await issuedBy(
            api,
            client(api, promptOptions(4), 'gopher://nowhere.invalid')
          );
          return { survived, attempts: issued, said };
        }
        default:
          throw new Error(
            `the corpus names the transport cause \`${cause}\`, which this suite cannot provoke`
          );
      }
    });
  }

  test(
    'every transport cause the corpus names ends a send as it says',
    async () => {
      // They arrive as one type in this client as in most runtimes, and only one of them could end
      // differently: a client deciding by the type spends four attempts on a mistyped API URL and
      // then hands its caller a message that accuses the network.
      for (const rule of entries(RETRY, 'transport.causes')) {
        const cause = text(rule, 'cause');
        const reason = text(rule, 'reason');

        const { survived, attempts, said } = await provoked(cause);

        if (flag(rule, 'retryable')) {
          expect({ cause, survived, reason }).toStrictEqual({ cause, survived: true, reason });
        } else {
          expect({ cause, survived, reason }).toStrictEqual({ cause, survived: false, reason });
          expect({ cause, attempts, said }).toStrictEqual({ cause, attempts: 1, said });
        }
      }
    },
    TEST_TIMEOUT_MS
  );

  test(
    'a head above the ceilings the corpus names is refused',
    async () => {
      // The head is written by the other end, so a client that bounds the body and not the head has
      // only moved where a broken or hostile server spends its caller's memory. Every ceiling is
      // crossed on its own and well over, and the head that is read is well under: the band around a
      // ceiling is where the runtime of the day answers rather than the client, so nothing here is
      // built in it.
      const lines = BOUNDS.max_response_headers;
      const perLine = BOUNDS.max_header_bytes;
      const whole = BOUNDS.max_head_bytes;

      // A quarter of the lines, an eighth of the whole-head ceiling in bytes: a head this size is
      // one a client reads without a word.
      const wellUnder: Record<string, string> = {};
      for (let line = 0; line < Math.floor(lines / 4); line += 1) {
        wellUnder[`x-filler-${line}`] = 'v'.repeat(Math.floor(whole / lines));
      }

      // Above the count this client holds a head to, and below what the runtime under it will hold:
      // past that the runtime refuses the head before the client sees it, and the case would be
      // reading the runtime rather than the client.
      const tooMany: Record<string, string> = {};
      for (let line = 0; line < lines + 8; line += 1) {
        tooMany[`x-filler-${line}`] = 'filler';
      }
      const tooLong: Record<string, string> = { 'x-filler': 'v'.repeat(perLine * 2) };
      // Lines that are neither too many nor too long on their own — an eighth of the whole-head
      // ceiling each, one short of the count — and eight times too much head together.
      const tooMuch: Record<string, string> = {};
      for (let line = 0; line < lines - 1; line += 1) {
        tooMuch[`x-filler-${line}`] = 'v'.repeat(Math.floor(whole / 8));
      }

      const read = await withApi((api) => {
        api.willAnswer({ ...ingested(), headers: wellUnder });
        return issuedBy(api, client(api, promptOptions(4)));
      });
      expect({
        head: 'well under every ceiling',
        survived: read.survived,
        said: read.said,
      }).toStrictEqual({ head: 'well under every ceiling', survived: true, said: '' });

      for (const [head, headers] of [
        ['more header lines than are read', tooMany],
        ['a header line longer than is read', tooLong],
        ['a whole head above what is read', tooMuch],
      ] as [string, Record<string, string>][]) {
        const { issued, survived, said } = await withApi((api) => {
          api.willAnswer(answer(200, {}, headers), ingested());
          return issuedBy(api, client(api, promptOptions(4)));
        });

        expect({ head, survived, issued, said }).toStrictEqual({
          head,
          survived: false,
          issued: 1,
          said,
        });
      }
    },
    TEST_TIMEOUT_MS
  );

  test(
    'every request carries what the corpus says it does',
    async () => {
      // A representation a client forgets to ask for costs nothing until the API serves a second
      // one, at which point it costs everything, which is exactly the kind of divergence nobody
      // notices by hand.
      const carried = await withApi(async (api) => {
        api.willAnswer(ingested());
        await client(api, promptOptions(4)).sendEvent(anEvent());
        // A send carries a body, so every occasion the corpus declares applies to this one request.
        return api.received[0].headers;
      });
      // The retry policy the client above was built with, which is what it is expected to state on
      // the wire. Read back off that policy rather than written out here, so the case cannot agree
      // with a client that states a schedule nobody configured.
      const policy = promptOptions(4).retryPolicy;
      const bound = new Map([
        ['token', TOKEN],
        ['language', 'typescript'],
        ['version', VERSION],
        ['attempts', String(policy.attempts())],
        ['backoff_ms', String(policy.initialBackoffMs)],
        ['ceiling_ms', String(policy.maxBackoffMs)],
        ['budget_ms', String(policy.maxTotalDelayMs)],
      ]);
      const composedAtMost = number(REQUEST, 'max_composed_bytes');
      for (const header of entries(REQUEST, 'headers')) {
        const name = text(header, 'name').toLowerCase();
        const template = text(header, 'value');
        const reason = text(header, 'reason');
        const arrived = carriedText(carried, name);
        const chunks = templateChunks(template, bound);

        expect({
          name,
          arrived,
          template,
          matched: matchesChunks(chunks, arrived),
          reason,
        }).toStrictEqual({ name, arrived, template, matched: true, reason });

        // A value with a hole this suite cannot fill is one the client composed out of what the
        // platform told it, and what the platform says is as long as it feels like.
        if (chunks.length > 1) {
          const bytes = Buffer.byteLength(arrived, 'utf8');
          expect({ name, bytes, composedAtMost, within: bytes <= composedAtMost }).toStrictEqual({
            name,
            bytes,
            composedAtMost,
            within: true,
          });
        }
      }
    },
    TEST_TIMEOUT_MS
  );

  test(
    'a policy asking for more attempts than the cap states the cap',
    async () => {
      // The one place where what a caller asked for and what the client will do come apart, and so
      // the one place the options header can be read two ways. A client stating the number it was
      // handed puts a reader on watch for a burst that cannot arrive, so the cap is what goes on
      // the wire. The cap is read from the corpus, so every target answers to one number.
      const cap = BOUNDS['max_attempts_cap'];
      const carried = await withApi(async (api) => {
        api.willAnswer(ingested());
        await client(api, promptOptions(cap + 1)).sendEvent(anEvent());
        return api.received[0].headers;
      });

      const stated = carriedText(carried, 'hook0-client-options');
      expect({ stated, statesTheCap: stated.startsWith(`attempts=${cap},`) }).toStrictEqual({
        stated,
        statesTheCap: true,
      });
    },
    TEST_TIMEOUT_MS
  );

  test(
    'the delay the API names is honoured and bounded',
    async () => {
      // The header is written by the other end, so honouring it whole would hand a stranger the
      // length of this client's send. What the corpus asks for is that a delay be waited out when
      // the budget can afford it and cut down to what is left of the budget when it cannot.
      const named = text(RETRY.retry_after as Record<string, unknown>, 'header');
      const paced = pacedProblem();

      for (const delay of entries(RETRY, 'retry_after.cases')) {
        const written = text(delay, 'header');

        const { sent, waited, issued } = await withApi(async (api) => {
          api.willAnswer(
            refusal(number(paced, 'status'), text(paced, 'problem'), { [named]: written }),
            ingested()
          );
          const patient = client(
            api,
            new Hook0ClientOptions(
              new RetryPolicy(4, PROMPT_BACKOFF_MS, PROMPT_BACKOFF_MS, DELAY_BUDGET_MS),
              5_000
            )
          );

          // `performance.now()` rather than `Date.now()`: the wall clock is truncated to whole
          // milliseconds and NTP can step it backwards mid-case, so it can read a wait as shorter
          // than it was for reasons that have nothing to do with the client.
          const started = performance.now();
          const answered = await patient.sendEvent(anEvent());
          return {
            sent: answered,
            waited: performance.now() - started,
            issued: api.received.length,
          };
        });

        expect(sent).toStrictEqual(INGESTED_ID);
        expect(issued).toStrictEqual(2);

        const askedFor = flag(delay, 'honoured')
          ? Math.min(number(delay, 'seconds') * 1000, DELAY_BUDGET_MS)
          : 0;
        expect({ written, sooner: waited < askedFor - CLOCK_SLACK_MS }).toStrictEqual({
          written,
          sooner: false,
        });
        expect({ written, longer: waited > askedFor + DELAY_SLACK_MS }).toStrictEqual({
          written,
          longer: false,
        });
      }
    },
    TEST_TIMEOUT_MS
  );

  test('the bounds are the ones the corpus names', () => {
    // This client's defaults, held against the one place the numbers are written down. What is
    // asserted is read from the corpus rather than listed here, so a bound added there and left
    // unapplied fails instead of passing unnoticed.
    const options = new Hook0ClientOptions();
    const policy = options.retryPolicy;
    const applied: Record<string, number> = {
      max_attempts: policy.maxAttempts,
      max_attempts_cap: MAX_ATTEMPTS_CAP,
      initial_backoff_ms: policy.initialBackoffMs,
      max_backoff_ms: policy.maxBackoffMs,
      max_total_delay_ms: policy.maxTotalDelayMs,
      request_timeout_ms: options.requestTimeoutMs,
      max_payload_bytes: options.maxPayloadBytes,
      max_response_bytes: options.maxResponseBytes,
      max_head_bytes: MAX_HEAD_BYTES,
      max_response_headers: MAX_RESPONSE_HEADERS,
      max_header_bytes: MAX_HEADER_BYTES,
    };

    for (const [bound, named] of Object.entries(BOUNDS)) {
      expect({ bound, carried: applied[bound] }).toStrictEqual({ bound, carried: named });
    }
  });

  test('every refusal the corpus declares reads as one of this client s', () => {
    // A refusal named in the corpus and mapped to nothing here would pass under any wording.
    for (const refused of SIGNATURE.refusals as string[]) {
      expect({ refused, mapped: refused in REFUSALS }).toStrictEqual({ refused, mapped: true });
    }
  });

  test('every delivery of the corpus is verified as it says', () => {
    // A refused delivery has to be refused for the reason the corpus names: a client that computed
    // a code over a header that never arrived and reported a mismatch would otherwise look right.
    for (const vector of entries(SIGNATURE, 'vectors')) {
      const name = text(vector, 'name');
      const delivered = new Headers();
      for (const [header, value] of vector.headers as [string, string][]) {
        delivered.set(header, value);
      }

      const verify = (): boolean | Hook0ClientError =>
        verifyWebhookSignatureWithCurrentTime(
          text(vector, 'signature'),
          Buffer.from(text(vector, 'payload'), 'utf8'),
          delivered,
          text(vector, 'secret'),
          number(vector, 'tolerance_seconds'),
          new Date(number(vector, 'current_time') * 1000)
        );

      if (text(vector, 'verdict') === 'accepted') {
        expect({ name, verified: verify() }).toStrictEqual({ name, verified: true });
        continue;
      }

      expect(verify).toThrow(REFUSALS[text(vector, 'refusal')]);
    }
  });
});
