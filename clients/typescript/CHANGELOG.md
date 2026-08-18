# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Make a retried send unable to duplicate an event (client)
- Bring the published clients under the generator, and add Ruby (clients)
- Bound the whole head of an answer, and check the contract is read (clients)
- Add the PHP, C# and Java SDKs (clients)
- Declare both module formats instead of relying on a guess (typescript)

### Changed

- Put the hand-written tests where the generator cannot reach (client)
- **Breaking.** Verifying a webhook signature declares `true` instead of
  `boolean | Hook0ClientError` (typescript)

  Nothing changes at runtime: neither function ever returned `false`, and neither ever returned a
  `Hook0ClientError`. Every reason a webhook is refused was already thrown. What changes is your
  code. `result instanceof Hook0ClientError` no longer compiles, with `TS2358`, and a branch on a
  falsy result is now unreachable. Catch the refusal instead of testing the answer:

  ```ts
  try {
    verifyWebhookSignature(signature, rawBody, headers, secret, 300);
  } catch (refused) {
    // every refusal arrives here; answer 400 and do not act on the delivery
  }
  ```

  Both `verifyWebhookSignature` and `verifyWebhookSignatureWithCurrentTime` are affected.

### Fixed

- Record the missing-header error in the locked surface (client)

### Testing

- Lock the published TypeScript surface (client)

## [sdk-v1.1.0] - 2026-04-03

### Added

- Set moduleResolution to node (clients.typescript)
- Sdk can be use in javascript & typescript project (clients.typescript)
- Add badges to readme (clients/typescript)
- Support v1 signature verification (clients/typescript)
- Add sdk release workflow with manual triggers (release)
- Make event ID optional and use UUIDv7 in big tables

### Changed

- A lot of modification and code style (clients.typescript)
- Refactor(clients.rust)
- Various improvements (clients/typescript)

### Documentation

- Improve READMEs (clients)
- Add license (clients/typescript)

### Fixed

- Fix ci (clients.typescript)
- Fix ci (clients.typescript)
- Remove dist folder from linter (clients.typescript)
- Put debug message under a flag (clients/typescript)

### Other

- Prepare for publish (clients/typescript)
- Bump (clients/typescript)
- Mention the license in README.md (clients/typescript)
- Update the license on manifest (clients/typescript)
- Update dependencies

[Unreleased]: https://gitlab.com/hook0/hook0/-/compare/sdk-v1.1.0...HEAD
[sdk-v1.1.0]: https://gitlab.com/hook0/hook0/-/compare/sdk-v1.0.1...sdk-v1.1.0

