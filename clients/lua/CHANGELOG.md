# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Let every SDK state the retry policy behind its requests (clients)
- Let every SDK say what it is on the wire (clients)
- Add the Kotlin, Lua and Zig SDKs (clients)

### CI/CD

- Hold the suite to counts, because the ratio does not survive the interpreter (lua)
- Make the lua client's coverage a floor rather than a snapshot (lua)

### Changed

- Dissolve two dashes into the sentences around them (lua,deps)

### Fixed

- Make what a client states about its policy what it actually does (clients)
- Type an event's labels the same way whether you read or write them (api)

### Testing

- Find what the generator wrote by looking at what it wrote, in php, ruby, lua and zig (clients)

[Unreleased]: https://gitlab.com/hook0/hook0/-/compare/sdk-v1.1.0...HEAD
