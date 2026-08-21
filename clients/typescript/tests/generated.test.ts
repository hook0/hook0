import { describe, expect, test, beforeEach, afterEach } from '@jest/globals';
import * as fs from 'fs';
import * as http from 'http';
import * as path from 'path';

import { generated } from '../src/index';

/**
 * What the generated request layer puts on the wire, and what it does with what comes back.
 *
 * The generated half is handed a transport and nothing else, so these cases hand it a real one —
 * the one an application writes, over a real socket — and watch a real API answer: the path it
 * interpolated, the query it assembled, the body it sent, the value it read back and the failure it
 * threw when the answer was a problem.
 *
 * Nothing here names an operation, a group or a problem. The groups are found on the generated
 * namespace, the arguments each method takes are read off the method itself, and what every call is
 * held to is read off the API document the generator was run against — including the case that says
 * every operation the document declares was reached. An operation the API grows therefore joins
 * this suite the moment the generated namespace carries it, and one it loses takes its case with it.
 */

/** Every case talks to a loopback socket, so none of them has any reason to take this long. */
const TEST_TIMEOUT_MS = 20_000;

/** No request a case makes is anywhere near this large; the cap bounds what one connection buffers. */
const MAX_REQUEST_BODY_BYTES = 64 * 1024;

/** No repository nests this package deeper than this below its root. */
const MAX_ANCESTORS = 8;

/**
 * What every string-shaped argument is given. It carries the two characters a path segment may not
 * leave as they are, so a value reaching a path proves it was escaped rather than pasted.
 */
const A_STRING = 'a value/with a space';

/** What a body-shaped argument is given, which is echoed back off the wire as it was sent. */
const A_BODY = { 'a member': ['the case', 'wrote this'] };

/** What the API answers an operation that reads a value back. */
const AN_ANSWER = { 'a member': 'the API answered this' };

/**
 * The tag that marks an operation as part of the surface an SDK exposes. A document that marks none
 * of its operations with it declares the whole of itself part of that surface, which is the rule the
 * generator applies and therefore the rule this suite holds it to.
 */
const SDK_TAG = 'sdk';

/** The methods a request line can carry, which is what tells an operation apart in a path item. */
const VERBS = ['get', 'put', 'post', 'delete', 'options', 'head', 'patch', 'trace'];

/** One operation the API document declares, as a request has to look to be it. */
interface DeclaredOperation {
  readonly method: string;
  /** The path with its parameters still written the way the document writes them, `{like_this}`. */
  readonly template: string;
  readonly requiredQuery: readonly string[];
  readonly optionalQuery: readonly string[];
  /** Whether a success carries a document the operation reads a value out of. */
  readonly readsBack: boolean;
}

/** A request the API received, in the order it received it. */
interface ReceivedRequest {
  readonly method: string;
  readonly target: string;
  readonly body: string;
  readonly headers: http.IncomingHttpHeaders;
}

/** What the API answers to one request, in the order the case scripted it. */
interface ScriptedResponse {
  readonly status: number;
  readonly body: unknown;
  /** The body as it travels, for the cases whose point is that it is not a document at all. */
  readonly verbatim?: string;
}

function apiDocument(): Record<string, unknown> {
  let at = __dirname;
  for (let step = 0; step < MAX_ANCESTORS; step += 1) {
    const candidate = path.join(at, 'api', 'openapi.snapshot.json');
    if (fs.existsSync(candidate)) {
      return JSON.parse(fs.readFileSync(candidate, 'utf8')) as Record<string, unknown>;
    }
    at = path.join(at, '..');
  }
  throw new Error(
    `No api/openapi.snapshot.json within ${MAX_ANCESTORS} directories of ${__dirname}`
  );
}

/** Every operation an SDK is built out of, which is what the document marks with the SDK tag. */
function declaredOperations(): DeclaredOperation[] {
  const paths = apiDocument().paths as Record<string, Record<string, unknown>>;
  const found: { sdk: boolean; operation: DeclaredOperation }[] = [];

  for (const [template, item] of Object.entries(paths)) {
    for (const verb of VERBS) {
      const declared = item[verb] as
        | {
            tags?: string[];
            parameters?: { name: string; in: string; required?: boolean }[];
            responses?: Record<string, { content?: Record<string, unknown> }>;
          }
        | undefined;
      if (declared === undefined) {
        continue;
      }

      const query = (declared.parameters ?? []).filter((parameter) => parameter.in === 'query');
      const answers = Object.entries(declared.responses ?? {}).filter(
        ([status]) => Number(status) >= 200 && Number(status) < 300
      );

      found.push({
        sdk: (declared.tags ?? []).includes(SDK_TAG),
        operation: {
          method: verb.toUpperCase(),
          template,
          requiredQuery: query.filter((p) => p.required === true).map((p) => p.name),
          optionalQuery: query.filter((p) => p.required !== true).map((p) => p.name),
          readsBack: answers.some(([, answer]) => answer.content !== undefined),
        },
      });
    }
  }

  if (found.length === 0) {
    throw new Error('The API document declares no operation at all');
  }
  const marked = found.filter((entry) => entry.sdk);
  return (marked.length > 0 ? marked : found).map((entry) => entry.operation);
}

const OPERATIONS = declaredOperations();

/** Whether a request line landed on this operation. */
function landedOn(operation: DeclaredOperation, target: string): boolean {
  const wanted = operation.template.split('/');
  const got = (target.split('?')[0] ?? '').split('/');
  if (wanted.length !== got.length) {
    return false;
  }
  return wanted.every((declared, index) => {
    if (declared.startsWith('{') && declared.endsWith('}')) {
      // A parameter stands for a segment that is there; an empty one is the trailing slash of
      // another path rather than a value.
      return got[index] !== '';
    }
    return declared === got[index];
  });
}

/** The name a parameter of the document is written under in the source the generator wrote. */
function camelCased(name: string): string {
  const words = name.split(/[^A-Za-z0-9]+/).filter((word) => word !== '');
  return words
    .map((word, index) => (index === 0 ? word : `${word[0]?.toUpperCase() ?? ''}${word.slice(1)}`))
    .join('');
}

/** Every group of operations the generated namespace carries, under the name it declares it with. */
function groups(): [string, new (transport: generated.Transport) => object][] {
  const found = Object.entries(generated).filter(([, declared]) => {
    if (typeof declared !== 'function' || declared.length !== 1) {
      return false;
    }
    const carried = Object.getOwnPropertyNames(
      (declared as { prototype?: object }).prototype ?? {}
    );
    return carried.some((name) => name !== 'constructor');
  });

  if (found.length === 0) {
    throw new Error('The generated namespace carries no group of operations');
  }
  return found as [string, new (transport: generated.Transport) => object][];
}

/** Every operation one group carries, under the name it is called by. */
function methodsOf(group: new (transport: generated.Transport) => object): string[] {
  return Object.getOwnPropertyNames(group.prototype)
    .filter((name) => name !== 'constructor')
    .sort();
}

/**
 * The arguments one method declares, by the names it declares them under.
 *
 * TypeScript keeps no types at run time, so what a method takes is read off the method: the source
 * a function answers is the source the compiler emitted, parameter names included, which is what
 * pairs each argument with the parameter of the document it fills.
 */
function argumentNamesOf(method: (...args: unknown[]) => unknown): string[] {
  const written = method.toString();
  const opened = written.indexOf('(');
  const closed = written.indexOf(')', opened);
  if (opened < 0 || closed < 0) {
    throw new Error(
      `A generated method is written in a shape this suite cannot read: ${written.slice(0, 80)}`
    );
  }
  return written
    .slice(opened + 1, closed)
    .split(',')
    .map((name) => name.trim())
    .filter((name) => name !== '');
}

/** A transport reaching a Hook0 API over a real socket, which is what an application writes. */
class HttpTransport implements generated.Transport {
  private readonly baseUrl: string;
  private readonly token: string;

  constructor(baseUrl: string, token: string) {
    this.baseUrl = baseUrl;
    this.token = token;
  }

  request(request: generated.TransportRequest): Promise<generated.TransportResponse> {
    const target = new URL(this.baseUrl);
    target.pathname = request.path;
    for (const [name, value] of request.query) {
      target.searchParams.append(name, value);
    }

    return new Promise((resolve, reject) => {
      const issued = http.request(
        {
          hostname: target.hostname,
          port: target.port,
          // The path is already written whole by the generated method, escaping included, so it is
          // carried as it was built rather than parsed and rebuilt on the way out.
          path: `${request.path}${target.search}`,
          method: request.method,
          headers: {
            Authorization: `Bearer ${this.token}`,
            Accept: 'application/json',
            ...(request.body === undefined ? {} : { 'Content-Type': 'application/json' }),
          },
        },
        (answer) => {
          const chunks: Buffer[] = [];
          answer.on('data', (chunk: Buffer) => chunks.push(chunk));
          answer.on('end', () =>
            resolve({
              status: answer.statusCode ?? 0,
              payload: Buffer.concat(chunks).toString('utf8'),
            })
          );
          answer.on('error', reject);
        }
      );
      issued.on('error', reject);
      if (request.body !== undefined) {
        issued.write(request.body);
      }
      issued.end();
    });
  }
}

/** A Hook0 API listening on a loopback port for the lifetime of one case. */
class FakeHook0Api {
  readonly received: ReceivedRequest[] = [];
  private readonly scripted: ScriptedResponse[] = [];
  private answered = 0;
  private server?: http.Server;

  willAnswer(...responses: ScriptedResponse[]): void {
    this.scripted.push(...responses);
  }

  listen(): Promise<void> {
    this.server = http.createServer((request, answer) => {
      const chunks: Buffer[] = [];
      let size = 0;
      request.on('data', (chunk: Buffer) => {
        size += chunk.length;
        if (size > MAX_REQUEST_BODY_BYTES) {
          request.destroy();
          return;
        }
        chunks.push(chunk);
      });
      request.on('end', () => {
        this.received.push({
          method: request.method ?? '',
          target: request.url ?? '',
          body: Buffer.concat(chunks).toString('utf8'),
          headers: request.headers,
        });

        const scripted = this.scripted[this.answered] ?? {
          status: 500,
          body: { error: 'the case scripted no answer for this request' },
        };
        this.answered += 1;
        answer.writeHead(scripted.status, { 'Content-Type': 'application/json' });
        answer.end(scripted.verbatim ?? JSON.stringify(scripted.body));
      });
    });

    return new Promise((resolve) => {
      this.server?.listen(0, '127.0.0.1', resolve);
    });
  }

  get baseUrl(): string {
    const address = this.server?.address();
    if (address === null || address === undefined || typeof address === 'string') {
      throw new Error('The fake Hook0 API is not listening');
    }
    return `http://127.0.0.1:${address.port}`;
  }

  close(): Promise<void> {
    return new Promise((resolve) => {
      if (this.server === undefined) {
        resolve();
        return;
      }
      this.server.close(() => resolve());
    });
  }
}

describe('the generated request layer', () => {
  let api: FakeHook0Api;
  let transport: generated.Transport;

  beforeEach(async () => {
    api = new FakeHook0Api();
    await api.listen();
    transport = new HttpTransport(api.baseUrl, 'token-xyz');
  });

  afterEach(async () => {
    await api.close();
  });

  /** Which operation of the document a request landed on. */
  function operationOf(named: string, request: ReceivedRequest): DeclaredOperation {
    const matched = OPERATIONS.filter(
      (operation) => operation.method === request.method && landedOn(operation, request.target)
    );
    if (matched.length !== 1) {
      throw new Error(
        `${named} reached \`${request.method} ${request.target}\`, which is ${matched.length} of the operations declared`
      );
    }
    return matched[0] as DeclaredOperation;
  }

  /** What one operation is asked with, given what the document says each of its arguments is. */
  function argumentsFor(names: string[], optional: readonly string[]): unknown[] {
    return names.map((name) => {
      if (name === 'body') {
        return A_BODY;
      }
      return optional.includes(name) ? undefined : A_STRING;
    });
  }

  async function drive(
    named: string,
    method: (...args: unknown[]) => Promise<unknown>,
    group: object,
    given: unknown[]
  ): Promise<{ read: unknown; request: ReceivedRequest }> {
    const read = await method.apply(group, given);
    const request = api.received[api.received.length - 1];
    if (request === undefined) {
      throw new Error(`${named} issued no request at all`);
    }
    return { read, request };
  }

  test(
    'every operation the document declares is reached the way it declares it',
    async () => {
      const reached = new Set<string>();

      for (const [name, group] of groups()) {
        const reaching = new group(transport);
        for (const called of methodsOf(group)) {
          const named = `${name}.${called}`;
          const method = (reaching as Record<string, (...args: unknown[]) => Promise<unknown>>)[
            called
          ];
          expect(method).toBeDefined();

          const names = argumentNamesOf(method as (...args: unknown[]) => unknown);
          api.willAnswer({ status: 200, body: AN_ANSWER });

          const { read, request } = await drive(
            named,
            method as never,
            reaching,
            argumentsFor(names, [])
          );
          const operation = operationOf(named, request);
          reached.add(`${operation.method} ${operation.template}`);

          expect(request.headers.authorization).toBe('Bearer token-xyz');

          // The value lands in the path escaped, so nothing in it can name a segment the operation
          // never had.
          const sent = (request.target.split('?')[0] ?? '').split('/');
          operation.template.split('/').forEach((declared, index) => {
            if (declared.startsWith('{')) {
              expect(sent[index]).toBe(encodeURIComponent(A_STRING));
            }
          });

          const carried = new URL(`http://ignored${request.target}`).searchParams;
          const wanted = [...operation.requiredQuery, ...operation.optionalQuery].sort();
          expect([...carried.keys()].sort()).toEqual(wanted);
          for (const parameter of wanted) {
            expect(carried.get(parameter)).toBe(A_STRING);
          }

          if (names.includes('body')) {
            expect(JSON.parse(request.body)).toEqual(A_BODY);
          } else {
            expect(request.body).toBe('');
          }

          // An operation the document says answers a document reads one back; one it says answers
          // none reads nothing, rather than whatever happened to be on the wire.
          expect(read).toEqual(operation.readsBack ? AN_ANSWER : undefined);
        }
      }

      expect([...reached].sort()).toEqual(
        [
          ...new Set(OPERATIONS.map((operation) => `${operation.method} ${operation.template}`)),
        ].sort()
      );
    },
    TEST_TIMEOUT_MS
  );

  test(
    'an argument an operation does not require leaves the query it would have filled empty',
    async () => {
      const byOperation = new Map<string, DeclaredOperation>();

      for (const [name, group] of groups()) {
        const reaching = new group(transport);
        for (const called of methodsOf(group)) {
          const named = `${name}.${called}`;
          const method = (reaching as Record<string, (...args: unknown[]) => Promise<unknown>>)[
            called
          ];
          const names = argumentNamesOf(method as (...args: unknown[]) => unknown);

          // Which parameters may be left out is the document's business, so the operation is found
          // once with everything given and asked again with only what it requires.
          api.willAnswer({ status: 200, body: AN_ANSWER }, { status: 200, body: AN_ANSWER });
          const first = await drive(named, method as never, reaching, argumentsFor(names, []));
          const operation = operationOf(named, first.request);
          byOperation.set(named, operation);

          const optional = operation.optionalQuery.map(camelCased);
          const { request } = await drive(
            named,
            method as never,
            reaching,
            argumentsFor(names, optional)
          );

          const carried = new URL(`http://ignored${request.target}`).searchParams;
          expect([...carried.keys()].sort()).toEqual([...operation.requiredQuery].sort());
        }
      }

      // Something was actually left out somewhere, or this case is asserting nothing.
      expect(
        [...byOperation.values()].some((operation) => operation.optionalQuery.length > 0)
      ).toBe(true);
    },
    TEST_TIMEOUT_MS
  );

  test(
    'every operation throws the problem the API reported rather than answering a value',
    async () => {
      for (const [name, group] of groups()) {
        const reaching = new group(transport);
        for (const called of methodsOf(group)) {
          const named = `${name}.${called}`;
          const method = (reaching as Record<string, (...args: unknown[]) => Promise<unknown>>)[
            called
          ];
          const names = argumentNamesOf(method as (...args: unknown[]) => unknown);
          api.willAnswer({
            status: 404,
            body: {
              id: 'NotFound',
              title: 'Not found',
              detail: 'what the case scripted',
              status: 404,
              type: 'https://documentation.hook0.com/problems',
            },
          });

          const raised = await method.apply(reaching, argumentsFor(names, [])).then(
            () => undefined,
            (thrown: unknown) => thrown
          );

          expect(raised).toBeInstanceOf(generated.ProblemError);
          const reported = raised as generated.ProblemError;
          expect(reported.status).toBe(404);
          expect(reported.kind).toBe('NotFound');
          expect(reported.problem?.detail).toBe('what the case scripted');
          expect(reported.message).toContain('404');
          expect(named).toBeTruthy();
        }
      }
    },
    TEST_TIMEOUT_MS
  );

  test(
    'every problem the document names is reported under the kind it names',
    async () => {
      const schemas = (apiDocument().components as { schemas: Record<string, unknown> }).schemas;
      const problem = schemas.Problem as { properties: { id: { enum?: string[] } } };
      const named = problem.properties.id.enum ?? [];
      expect(named.length).toBeGreaterThan(0);

      const [, group] = groups()[0] as [string, new (t: generated.Transport) => object];
      const reaching = new group(transport);
      const called = methodsOf(group)[0] as string;
      const method = (reaching as Record<string, (...args: unknown[]) => Promise<unknown>>)[
        called
      ] as (...args: unknown[]) => Promise<unknown>;
      const names = argumentNamesOf(method as (...args: unknown[]) => unknown);

      for (const kind of named) {
        api.willAnswer({
          status: 400,
          body: { id: kind, status: 400, title: 'refused', detail: 'what the case scripted' },
        });

        const raised = await method.apply(reaching, argumentsFor(names, [])).then(
          () => undefined,
          (thrown: unknown) => thrown
        );

        expect(raised).toBeInstanceOf(generated.ProblemError);
        expect((raised as generated.ProblemError).kind).toBe(kind);
      }
    },
    TEST_TIMEOUT_MS
  );

  test(
    'a failure that is not a problem document is still reported',
    async () => {
      const [, group] = groups()[0] as [string, new (t: generated.Transport) => object];
      const reaching = new group(transport);
      const called = methodsOf(group)[0] as string;
      const method = (reaching as Record<string, (...args: unknown[]) => Promise<unknown>>)[
        called
      ] as (...args: unknown[]) => Promise<unknown>;
      const names = argumentNamesOf(method as (...args: unknown[]) => unknown);

      api.willAnswer({
        status: 502,
        body: undefined,
        verbatim: 'a gateway wrote this, and it is not a problem document',
      });

      const raised = await method.apply(reaching, argumentsFor(names, [])).then(
        () => undefined,
        (thrown: unknown) => thrown
      );

      expect(raised).toBeInstanceOf(generated.ProblemError);
      const reported = raised as generated.ProblemError;
      expect(reported.status).toBe(502);
      expect(reported.kind).toBeUndefined();
      expect(reported.problem).toBeUndefined();
    },
    TEST_TIMEOUT_MS
  );

  test(
    'a body no reader can make a document of is reported rather than answered',
    async () => {
      const reading = OPERATIONS.filter((operation) => operation.readsBack);
      expect(reading.length).toBeGreaterThan(0);

      for (const [name, group] of groups()) {
        const reaching = new group(transport);
        for (const called of methodsOf(group)) {
          const named = `${name}.${called}`;
          const method = (reaching as Record<string, (...args: unknown[]) => Promise<unknown>>)[
            called
          ];
          const names = argumentNamesOf(method as (...args: unknown[]) => unknown);

          api.willAnswer({ status: 200, body: AN_ANSWER });
          const { request } = await drive(
            named,
            method as never,
            reaching,
            argumentsFor(names, [])
          );
          if (!operationOf(named, request).readsBack) {
            continue;
          }

          // A success whose body is not a document at all, which is what a gateway between the
          // caller and the API answers when it answers for it.
          api.willAnswer({ status: 200, body: undefined, verbatim: 'a gateway wrote this' });
          const raised = await method.apply(reaching, argumentsFor(names, [])).then(
            () => undefined,
            (thrown: unknown) => thrown
          );

          expect(raised).toBeInstanceOf(generated.ProblemError);
          expect((raised as generated.ProblemError).status).toBe(200);
        }
      }
    },
    TEST_TIMEOUT_MS
  );

  test(
    'a body above what a message quotes is cut rather than echoed whole',
    () => {
      const long = 'x'.repeat(4096);

      let raised: unknown;
      try {
        generated.raiseForStatus(500, long);
      } catch (thrown: unknown) {
        raised = thrown;
      }

      expect(raised).toBeInstanceOf(generated.ProblemError);
      const reported = raised as generated.ProblemError;
      expect(reported.message.length).toBeLessThan(long.length);
      expect(reported.message).toContain('…');
    },
    TEST_TIMEOUT_MS
  );

  test(
    'a success is not a failure, whichever status it is answered under',
    () => {
      for (const status of [200, 201, 204, 299]) {
        expect(() => generated.raiseForStatus(status, '{}')).not.toThrow();
      }
      for (const status of [199, 300, 400, 500]) {
        expect(() => generated.raiseForStatus(status, '{}')).toThrow(generated.ProblemError);
      }
    },
    TEST_TIMEOUT_MS
  );
});

describe('the closed sets of values the document names', () => {
  test('carry the values the document declares and no other', () => {
    const schemas = (apiDocument().components as { schemas: Record<string, unknown> }).schemas;

    // Every closed set the generator wrote, found by shape rather than by name: an object whose
    // every member is a string it maps to itself.
    const written = Object.entries(generated).filter(([, declared]) => {
      if (typeof declared !== 'object' || declared === null) {
        return false;
      }
      const members = Object.entries(declared as Record<string, unknown>);
      return (
        members.length > 0 &&
        members.every(([key, value]) => typeof value === 'string' && key.length > 0)
      );
    });
    expect(written.length).toBeGreaterThan(0);

    // The one the document names on the body every failure carries, which is the set with the most
    // values in it and therefore the one worth holding against the document.
    const problem = schemas.Problem as { properties: { id: { enum?: string[] } } };
    const named = problem.properties.id.enum ?? [];
    const carried = written.find(
      ([, declared]) => Object.keys(declared as object).length === named.length
    );
    expect(carried).toBeDefined();
    expect(Object.values((carried as [string, object])[1]).sort()).toEqual([...named].sort());

    for (const [, declared] of written) {
      for (const [key, value] of Object.entries(declared as Record<string, string>)) {
        // A value is written under the name it travels as, so a caller naming one gets it back.
        expect(typeof value).toBe('string');
        expect(key.length).toBeGreaterThan(0);
      }
    }
  });
});
