# Security policy

## Reporting a vulnerability

Email **security@hook0.com**. You can encrypt with our
[PGP key](https://keybase.io/fgribreau/pgp_keys.asc). Please do not open a
public issue or merge request for a suspected vulnerability.

Send enough to reproduce it. Give the affected endpoint, the steps, and a
working proof of concept. We acknowledge reports as soon as we can, and we work
with the first person to report a given issue.

## What is in scope

The Hook0 API is in scope, on `app.hook0.com/api` and in a self-hosted
deployment, since both run the same code. The marketing site, the documentation
site, the `play.hook0.com` playground, and the configuration of a self-hosted
deployment are out of scope. The full list of what we do and do not treat as a
vulnerability, including findings we already know about, is in the
[vulnerability disclosure policy](https://documentation.hook0.com/resources/vulnerability-disclosure-policy).

## After the fix

We publish a [security advisory](https://documentation.hook0.com/resources/security-advisories)
for vulnerabilities that affected users, and we credit the reporter unless they
prefer otherwise. Hook0 runs the same code hosted or self-hosted, so a finding
in that code affects both. Where one is warranted, we request a CVE through
GitHub Security Advisories and publish it with the fix. You do not need to
contact a CVE Numbering Authority yourself.

Everyone whose report we act on is listed on our
[security acknowledgments](https://documentation.hook0.com/resources/security-acknowledgments)
page, whatever the severity.
