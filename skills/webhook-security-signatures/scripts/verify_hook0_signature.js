#!/usr/bin/env node
// Verify a Hook0 webhook signature (the `X-Hook0-Signature` header).
//
// Header format:
//   t=<unix seconds>,h=<covered header names, space-separated>,v1=<hex HMAC-SHA256>
// Older subscriptions may still receive the body-only scheme:
//   t=<unix seconds>,v0=<hex HMAC-SHA256>
//
// Signed message (HMAC-SHA256 keyed with the subscription secret):
//   v1: "<t>.<names joined by ' '>.<their values, same order, joined by '.'>.<raw body>"
//   v0: "<t>.<raw body>"
// When both codes are present, v1 is the one verified.
//
// Usage:
//   node verify_hook0_signature.js --selftest
//   node verify_hook0_signature.js --conformance path/to/signature.json
//
// In your app (Express example — keep the raw body):
//   app.post("/webhooks", express.raw({ type: "*/*" }), (req, res) => {
//     const { ok } = verify(req.body, req.headers, req.get("X-Hook0-Signature") || "", SECRET);
//     if (!ok) return res.sendStatus(401);
//     ...
//   });
"use strict";
const crypto = require("crypto");
const fs = require("fs");

const DEFAULT_TOLERANCE_SECONDS = 300;

class Refused extends Error {
  constructor(code, detail) {
    super(`${code}: ${detail}`);
    this.code = code;
  }
}

function parse(header) {
  const parts = {};
  for (const part of header.split(",")) {
    const at = part.indexOf("=");
    if (at > 0) parts[part.slice(0, at).trim()] = part.slice(at + 1).trim();
  }
  if (!/^-?\d+$/.test(parts.t || "")) throw new Refused("signature_unreadable", "no readable `t` moment");
  const codes = {};
  for (const scheme of ["v0", "v1"]) {
    if (scheme in parts) {
      // Exactly one SHA-256 code, in hexadecimal: Buffer.from(…, "hex") would silently
      // drop what it cannot read, so the shape is checked before decoding.
      if (!/^[0-9a-fA-F]{64}$/.test(parts[scheme])) {
        throw new Refused("code_not_hexadecimal", `\`${scheme}\` is not a whole hexadecimal SHA-256 code`);
      }
      codes[scheme] = Buffer.from(parts[scheme], "hex");
    }
  }
  if (!codes.v0 && !codes.v1) throw new Refused("signature_unreadable", "neither a `v0` nor a `v1` code");
  const covered = codes.v1 ? (parts.h || "").split(" ").filter(Boolean) : [];
  return { timestamp: Number(parts.t), covered, codes };
}

function header(headers, name) {
  const entries = Array.isArray(headers) ? headers : Object.entries(headers);
  for (const [key, value] of entries) {
    if (key.toLowerCase() === name.toLowerCase()) return Array.isArray(value) ? value.join(", ") : value;
  }
  return undefined;
}

function check(body, headers, signatureHeader, secret, now, tolerance = DEFAULT_TOLERANCE_SECONDS) {
  const { timestamp, covered, codes } = parse(signatureHeader);
  now = now === undefined ? Math.floor(Date.now() / 1000) : now;
  if (Math.abs(now - timestamp) > tolerance) {
    throw new Refused("outside_tolerance", `moment ${timestamp} is more than ${tolerance}s from ${now}`);
  }

  let prefix;
  let expected;
  if (codes.v1) {
    const values = covered.map((name) => {
      const value = header(headers, name);
      if (value === undefined) throw new Refused("header_not_delivered", `covered header \`${name}\` was not delivered`);
      return value;
    });
    prefix = `${timestamp}.${covered.join(" ")}.${values.join(".")}.`;
    expected = codes.v1;
  } else {
    prefix = `${timestamp}.`;
    expected = codes.v0;
  }

  const computed = crypto
    .createHmac("sha256", secret)
    .update(prefix)
    .update(Buffer.isBuffer(body) ? body : Buffer.from(body))
    .digest();
  if (!crypto.timingSafeEqual(computed, expected)) {
    throw new Refused("code_mismatch", "the code does not match this body and these headers");
  }
}

function verify(body, headers, signatureHeader, secret, now, tolerance) {
  try {
    check(body, headers, signatureHeader, secret, now, tolerance);
    return { ok: true, reason: "ok" };
  } catch (error) {
    if (error instanceof Refused) return { ok: false, reason: error.message };
    throw error;
  }
}

function runVectors(vectors) {
  let failures = 0;
  for (const v of vectors) {
    let outcome = "accepted";
    try {
      check(Buffer.from(v.payload), v.headers, v.signature, v.secret, v.current_time, v.tolerance_seconds);
    } catch (error) {
      if (!(error instanceof Refused)) throw error;
      outcome = error.code;
    }
    const wanted = v.verdict === "accepted" ? "accepted" : v.refusal;
    if (outcome !== wanted) failures += 1;
    console.log(`${outcome === wanted ? "OK  " : "FAIL"} ${v.name} (got ${outcome}, want ${wanted})`);
  }
  return failures;
}

// A subset of the vectors every Hook0 SDK is held to (clients/conformance/signature.json
// in the Hook0 repository). Their codes were computed outside any implementation.
const SELFTEST_VECTORS = [
  ["v1 header-scheme signature verifies", "accepted", undefined,
    "t=1800000000,h=x-event-id x-delivery-id,v1=19a6fb8f6581715b241a93af02a58611c3b0ac7b747a8d2a5b120ee418d0c347"],
  ["v0 body-scheme signature verifies", "accepted", undefined,
    "t=1800000000,v0=d17d66b66fca89390c5b967c45e8928fc732db07a0aabe8167b1e98213081ffe"],
  ["covered header order is part of what is signed", "refused", "code_mismatch",
    "t=1800000000,h=x-delivery-id x-event-id,v1=19a6fb8f6581715b241a93af02a58611c3b0ac7b747a8d2a5b120ee418d0c347"],
  ["a covered header that was not delivered", "refused", "header_not_delivered",
    "t=1800000000,h=x-event-id x-missing,v1=19a6fb8f6581715b241a93af02a58611c3b0ac7b747a8d2a5b120ee418d0c347"],
  ["a code replayed under another moment", "refused", "code_mismatch",
    "t=1800000001,v0=d17d66b66fca89390c5b967c45e8928fc732db07a0aabe8167b1e98213081ffe"],
  ["a moment older than the tolerance", "refused", "outside_tolerance",
    "t=1799999000,v0=d17d66b66fca89390c5b967c45e8928fc732db07a0aabe8167b1e98213081ffe"],
].map(([name, verdict, refusal, signature]) => ({
  name, verdict, refusal, signature,
  secret: "a-subscription-secret",
  payload: '{"event":"user.created"}',
  headers: [["x-event-id", "evt-1"], ["x-delivery-id", "dlv-1"], ["content-type", "application/json"]],
  current_time: 1800000000,
  tolerance_seconds: 300,
}));

module.exports = { verify, check, Refused };

if (require.main === module) {
  const args = process.argv.slice(2);
  let vectors;
  if (args.length === 1 && args[0] === "--selftest") {
    vectors = SELFTEST_VECTORS;
  } else if (args.length === 2 && args[0] === "--conformance") {
    vectors = JSON.parse(fs.readFileSync(args[1], "utf8")).vectors;
  } else {
    console.log("usage: verify_hook0_signature.js --selftest | --conformance <signature.json>");
    process.exit(2);
  }
  const failed = runVectors(vectors);
  console.log(`${vectors.length - failed}/${vectors.length} vectors behave as expected`);
  process.exit(failed ? 1 : 0);
}
