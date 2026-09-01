---
name: security-advisory
description: >-
  Standard process for turning a confirmed Hook0 vulnerability into a published
  security advisory and, where warranted, a CVE. Use whenever a security report
  is confirmed and a fix is being prepared or has shipped, or when the team asks
  to "publish an advisory", "request a CVE", "assign a CVE", "credit a
  researcher", or "handle a security disclosure". Covers the CVE decision, the
  GitHub Security Advisory draft, timing, doc updates, and crediting the
  reporter.
---

# Security advisory & CVE process

Hook0 runs the same code on `app.hook0.com` and in a self-hosted deployment. A
vulnerability in that code affects both, so a confirmed finding in the product
is handled as a coordinated disclosure with an advisory, and a CVE when it
warrants one. Reports arrive under the
[vulnerability disclosure policy](../../../documentation/resources/vulnerability-disclosure-policy.md);
this skill is what happens after one is confirmed.

GitHub is a CVE Numbering Authority, and the code is mirrored at
[github.com/hook0/hook0](https://github.com/hook0/hook0), so we request CVEs
there ourselves. A researcher never has to go to a Numbering Authority on their
own, and we keep the identifier on the same timeline as the fix.

## Step 1: Decide whether it warrants a CVE

A CVE describes a flaw in the product that a downstream user can be affected by.
Ask whether the finding is a defect in Hook0's own code that reaches a
self-hoster running an unmodified build.

- **Warrants a CVE:** a flaw in the API code: auth or authorization bypass,
  tenant isolation break, SSRF in webhook delivery, token handling, signature
  verification, injection. It ships to every deployment, so it gets an advisory
  and a CVE.
- **Advisory, CVE optional:** a real but low-impact code defect (account
  enumeration on an endpoint, a missing bound). Publish an advisory; assign a
  CVE only if a downstream user would act on it.
- **No CVE:** anything out of scope in the disclosure policy. Configuration of a
  cloud deployment is not a product flaw. The CBC-SHA1 cipher suites on the CDN
  hostnames are the standing example. They are kept on purpose and documented
  under "known and accepted". A CDN TLS preference does not make Hook0's code
  defective, so no CVE. Credit the reporter on the acknowledgments page all the
  same.

When unsure, ask whether a self-hoster running a dependency scanner would need
to be told to upgrade. If yes, it is a CVE.

## Step 2: Draft the advisory privately

On `github.com/hook0/hook0`: **Security → Advisories → New draft advisory**. The
draft is private until you publish it. Fill in:

- **Title**: what the flaw is, plainly (e.g. "Password reset link remains
  usable after it has reset a password").
- **Description**: what was wrong, the impact, and the fixed version. Write it
  the way the disclosure policy and acknowledgments already read, in terms of
  what the reader can act on, with no ticket numbers and no internal tooling.
- **Affected product / versions**: the ecosystem and the version range. Give
  the first fixed version so the range is closed.
- **Severity**: CVSS v3.1. The GitHub editor computes the vector from the
  metrics; record the vector string in the advisory.
- **CWE**: the weakness class (e.g. CWE-384 session fixation, CWE-918 SSRF,
  CWE-362 race condition, CWE-204 observable response discrepancy).
- **Credit**: add the reporter as the reporting credit, spelled as they gave
  it, unless they asked to stay off.

`references/advisory-template.md` holds the blank fields to fill. Published
advisories are public, on the
[security advisories page](../../../documentation/resources/security-advisories.md),
not in this skill.

## Step 3: Request the CVE

From the draft, use **Request CVE**. GitHub assigns the identifier while the
draft stays private. The CVE and the GHSA number travel together.

## Step 4: Ship the fix first, then publish

The fix reaches production and a tagged release before the advisory is public,
so the write-up never points at the flaw before self-hosters can close it. Keep
the specifics off any public tracker (issue, MR title, commit body) until then.
On the day the fix is public, publish the advisory. GitHub then feeds the
[GitHub Advisory Database](https://github.com/advisories) and
[OSV](https://osv.dev) automatically, so scanners pick it up.

## Step 5: Update the docs

In one merge request:

- Add the entry to
  [`documentation/resources/security-advisories.md`](../../../documentation/resources/security-advisories.md):
  title, CVE, GHSA link, affected and fixed versions, one line of impact.
- Make sure the reporter is on
  [`documentation/resources/security-acknowledgments.md`](../../../documentation/resources/security-acknowledgments.md).
- Note the fix in the changelog if the release does not already.

## Step 6: Tell the reporter

Draft an email back to the reporter with the CVE and a link to the advisory.
Follow the repository email rules. Create a Gmail draft, never send it, and run
the body through `/humanizer pro` first. Thank them, name the identifier, and
link the published advisory.

## When several reports share one fix

The same fix can close more than one report (a reset-token lifecycle fix can
answer both "link reused after use" and "link outlived its expiry"). One
advisory and one CVE cover it, and every reporter behind it is credited.
