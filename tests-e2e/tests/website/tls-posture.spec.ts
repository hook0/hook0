import { test, expect } from "@playwright/test";
import tls from "node:tls";

/**
 * The public hostnames must not negotiate the CBC-SHA1 suites, and must not
 * speak anything below TLS 1.2.
 *
 * Two reports arrived saying otherwise. Neither reproduced, and this is what
 * says so on demand rather than by hand: the next identical report is answered
 * by running this.
 *
 * Every check carries a control connection. Without one, a host that has gone
 * away, a proxy in the path or a runner with no route out all look exactly like
 * a server refusing a weak cipher, and the whole file passes by failing to
 * connect. The control is what tells those apart.
 */

/** Long enough for a slow path out, short enough to fail rather than hang. */
const HANDSHAKE_TIMEOUT_MS = 10_000;

/**
 * Suites built on CBC with a SHA-1 MAC. `@SECLEVEL=0` is what lets the client
 * offer them at all: at the default security level OpenSSL drops them before
 * the handshake, and the refusal would be ours rather than the server's.
 */
const WEAK_SUITES = "AES128-SHA:AES256-SHA:ECDHE-RSA-AES128-SHA:ECDHE-RSA-AES256-SHA@SECLEVEL=0";

interface HandshakeResult {
  negotiated: boolean;
  detail: string;
}

function handshake(
  host: string,
  options: { ciphers: string; minVersion: "TLSv1" | "TLSv1.2"; maxVersion: "TLSv1.1" | "TLSv1.2" }
): Promise<HandshakeResult> {
  return new Promise((resolve) => {
    const socket = tls.connect(
      {
        host,
        port: 443,
        servername: host,
        ciphers: options.ciphers,
        minVersion: options.minVersion,
        maxVersion: options.maxVersion,
        timeout: HANDSHAKE_TIMEOUT_MS,
      },
      () => {
        const cipher = socket.getCipher();
        const protocol = socket.getProtocol();
        socket.destroy();
        resolve({ negotiated: true, detail: `${protocol} ${cipher ? cipher.name : "unknown"}` });
      }
    );
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
async function publicHostnames(baseUrl: string, html: string): Promise<string[]> {
  const hosts = new Set<string>([new URL(baseUrl).hostname]);
  for (const match of html.matchAll(/https:\/\/([a-z0-9.-]*hook0\.com)/gi)) {
    hosts.add(match[1].toLowerCase());
  }
  return [...hosts].sort();
}

test.describe("TLS posture", () => {
  test("no public hostname negotiates a CBC-SHA1 suite or a protocol below TLS 1.2", async ({
    request,
    baseURL,
  }) => {
    test.slow();
    expect(baseURL, "the website suite needs a base URL to read hostnames from").toBeTruthy();

    const homepage = await request.get(baseURL!);
    expect(homepage.ok(), `could not read ${baseURL} to discover hostnames`).toBe(true);
    const hosts = await publicHostnames(baseURL!, await homepage.text());

    expect(
      hosts.length,
      "no hostname was discovered, so this test would pass without checking anything"
    ).toBeGreaterThan(1);

    for (const host of hosts) {
      // Control first: if a plain handshake does not work, nothing below this
      // line means anything, and the failure is the probe rather than the host.
      const control = await handshake(host, {
        ciphers: "DEFAULT",
        minVersion: "TLSv1.2",
        maxVersion: "TLSv1.2",
      });
      expect(
        control.negotiated,
        `${host} refused an ordinary TLS 1.2 handshake (${control.detail}) — the probe is broken, not the host`
      ).toBe(true);

      const weak = await handshake(host, {
        ciphers: WEAK_SUITES,
        minVersion: "TLSv1.2",
        maxVersion: "TLSv1.2",
      });
      expect(
        weak.negotiated,
        `${host} negotiated a CBC-SHA1 suite (${weak.detail})`
      ).toBe(false);

      const legacy = await handshake(host, {
        ciphers: "DEFAULT@SECLEVEL=0",
        minVersion: "TLSv1",
        maxVersion: "TLSv1.1",
      });
      expect(
        legacy.negotiated,
        `${host} negotiated a protocol below TLS 1.2 (${legacy.detail})`
      ).toBe(false);
    }
  });
});
