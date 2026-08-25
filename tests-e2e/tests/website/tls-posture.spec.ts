import { test, expect } from "@playwright/test";
import tls from "node:tls";

/**
 * What the public hostnames actually negotiate.
 *
 * Scanners report weak cipher suites against Hook0 Cloud regularly, and the
 * answer has to come from a measurement rather than from a belief. Two reports
 * arrived saying the hostnames accept CBC-SHA1 suites. They are right, and the
 * shape of the exposure is what this file pins down.
 *
 * A probe of this kind fails in one particular way: it offers suites the client
 * cannot actually put on the wire, the handshake dies locally, and the file
 * reports a clean posture it never observed. An earlier version of this test
 * did exactly that — it offered only RSA-authenticated suites while the edge
 * serves an ECDSA certificate, so there was nothing in common and the local
 * failure read as a refusal. Every outcome below is therefore required to come
 * from the server: either a completed handshake, or an alert the server sent.
 */

/** Long enough for a slow path out, short enough to fail rather than hang. */
const HANDSHAKE_TIMEOUT_MS = 10_000;

/**
 * Suites built on CBC with a SHA-1 MAC, under both authentication algorithms:
 * which one is offered decides whether the edge can answer at all. `@SECLEVEL=0`
 * is what lets the client offer them — at the default security level OpenSSL
 * drops them before the handshake and the refusal would be ours.
 */
const WEAK_SUITES =
  "ECDHE-ECDSA-AES128-SHA:ECDHE-ECDSA-AES256-SHA:ECDHE-RSA-AES128-SHA:ECDHE-RSA-AES256-SHA:AES128-SHA:AES256-SHA@SECLEVEL=0";

/**
 * Error codes OpenSSL raises for an alert the peer sent. Anything else — a
 * local "no ciphers available", a name that does not resolve, a timeout — means
 * the probe never reached the point of asking.
 */
const SERVER_ALERT = /ALERT/;

interface HandshakeResult {
  negotiated: boolean;
  detail: string;
}

function handshake(host: string, options: tls.ConnectionOptions): Promise<HandshakeResult> {
  return new Promise((resolve) => {
    // A cipher string the local OpenSSL cannot use at all throws right here,
    // before anything reaches the network. Left uncaught it escapes the promise
    // and reads as a broken test, hiding what it really is: the probe never got
    // to ask the server anything.
    let socket: tls.TLSSocket;
    try {
      socket = tls.connect(
        { host, port: 443, servername: host, timeout: HANDSHAKE_TIMEOUT_MS, ...options },
        () => {
          const cipher = socket.getCipher();
          const protocol = socket.getProtocol();
          socket.destroy();
          resolve({ negotiated: true, detail: `${protocol} ${cipher ? cipher.name : "unknown"}` });
        }
      );
    } catch (error) {
      const code = (error as NodeJS.ErrnoException).code;
      resolve({ negotiated: false, detail: `LOCAL ${code ?? String(error)}` });
      return;
    }
    socket.on("error", (error: NodeJS.ErrnoException) => {
      socket.destroy();
      resolve({ negotiated: false, detail: error.code ?? error.message });
    });
    socket.on("timeout", () => {
      socket.destroy();
      resolve({ negotiated: false, detail: "TIMEOUT" });
    });
  });
}

/**
 * The hostnames the site itself points at, read off the served homepage rather
 * than listed here: a property that moves to a new subdomain is covered the day
 * the site links to it.
 */
function publicHostnames(baseUrl: string, html: string): string[] {
  const hosts = new Set<string>([new URL(baseUrl).hostname]);
  for (const match of html.matchAll(/https:\/\/([a-z0-9.-]*hook0\.com)/gi)) {
    hosts.add(match[1].toLowerCase());
  }
  return [...hosts].sort();
}

async function discoverHostnames(baseURL: string, request: { get: (url: string) => Promise<{ ok: () => boolean; text: () => Promise<string> }> }) {
  const homepage = await request.get(baseURL);
  expect(homepage.ok(), `could not read ${baseURL} to discover hostnames`).toBe(true);
  const hosts = publicHostnames(baseURL, await homepage.text());
  expect(
    hosts.length,
    "no hostname was discovered, so this test would pass without checking anything"
  ).toBeGreaterThan(1);
  return hosts;
}

test.describe("TLS posture", () => {
  /**
   * The floor holds today and is the one worth guarding: a minimum version is a
   * setting anybody can turn back down.
   */
  test("no public hostname speaks a protocol below TLS 1.2, and the probe reaches every one of them", async ({
    request,
    baseURL,
  }) => {
    test.slow();
    expect(baseURL, "the website suite needs a base URL to read hostnames from").toBeTruthy();
    const hosts = await discoverHostnames(baseURL!, request);

    for (const host of hosts) {
      const control = await handshake(host, { minVersion: "TLSv1.2" });
      expect(
        control.negotiated,
        `${host} refused an ordinary handshake (${control.detail}) — the probe is broken, not the host`
      ).toBe(true);

      const legacy = await handshake(host, {
        ciphers: WEAK_SUITES,
        minVersion: "TLSv1",
        maxVersion: "TLSv1.1",
      });
      expect(legacy.negotiated, `${host} negotiated ${legacy.detail}`).toBe(false);
      expect(
        legacy.detail,
        `${host} did not answer the sub-1.2 offer (${legacy.detail}); the client never got to ask, so its refusal proves nothing`
      ).toMatch(SERVER_ALERT);

      // Whatever the weak suites do below, the answer has to be the server's.
      const weak = await handshake(host, {
        ciphers: WEAK_SUITES,
        minVersion: "TLSv1.2",
        maxVersion: "TLSv1.2",
      });
      if (!weak.negotiated) {
        expect(
          weak.detail,
          `${host} did not answer the CBC-SHA1 offer (${weak.detail}); the client never put those suites on the wire`
        ).toMatch(SERVER_ALERT);
      }
    }
  });

  /**
   * The state we would prefer, and it does not hold. The hostnames behind the
   * CDN negotiate ECDHE-ECDSA-AES128-SHA, and they keep doing so on purpose:
   * customer integrations still reach the API over TLS 1.2, and dropping the
   * protocol version to be rid of the suites would stop them at the handshake.
   * Restricting the suites while keeping TLS 1.2 needs a paid option on the
   * zone, which has not been bought. The origin the apex is served from refuses
   * them already, which is why one hostname passes here and the rest do not.
   *
   * So the marker below records a decision rather than a debt. It exists so
   * that the day the suites do go away, this test passes, and a test marked
   * this way passing is itself a red run, which forces the choice to be made
   * again in the open. Deleting it while the suites are still offered is the
   * one thing it must not be used for.
   */
  test("no public hostname negotiates a CBC-SHA1 suite", async ({ request, baseURL }) => {
    test.fail();
    test.slow();
    expect(baseURL, "the website suite needs a base URL to read hostnames from").toBeTruthy();
    const hosts = await discoverHostnames(baseURL!, request);

    for (const host of hosts) {
      const weak = await handshake(host, {
        ciphers: WEAK_SUITES,
        minVersion: "TLSv1.2",
        maxVersion: "TLSv1.2",
      });
      expect(weak.negotiated, `${host} negotiated a CBC-SHA1 suite (${weak.detail})`).toBe(false);
    }
  });
});
