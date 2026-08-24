# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Give every SDK client its own mirror, and publish the three that needed one (ci)
- Let every SDK state the retry policy behind its requests (clients)
- Let every SDK say what it is on the wire (clients)
- Add the Kotlin, Lua and Zig SDKs (clients)

### CI/CD

- Hold the php client to a floor and say why zig has none (php,zig)

### Changed

- Read a failure's document out of the body without copying it first (zig)

### Fixed

- Keep what a failure reported out of the arena the call frees (zig)
- Prove the documentation against the toolchains the clients are checked on (ci)
- Type an event's labels the same way whether you read or write them (api)

### Other

- Stop tracking the build cache (zig)
- Ignore what the build writes beside the source (zig)

### Testing

- Find what the generator wrote by looking at what it wrote, in php, ruby, lua and zig (clients)

[Unreleased]: https://gitlab.com/hook0/hook0/-/compare/sdk-v1.1.0...HEAD
