# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Let every SDK state the retry policy behind its requests (clients)
- Let every SDK say what it is on the wire (clients)
- Add the Kotlin, Lua and Zig SDKs (clients)

### Breaking Changes

- Stop a release from being smaller than its own commits (ci)

### CI/CD

- Refuse a coverage run that measured nothing (java,kotlin)
- Hold the kotlin client to what it covers today (kotlin)

### Documentation

- Record what the clock allowance was measured against (clients)

### Fixed

- Read a policy's durations through one accessor, not three (clients)
- Make what a client states about its policy what it actually does (clients)
- Type an event's labels the same way whether you read or write them (api)

### Testing

- Prove the json readers apply the bounds their own docs promise (clients)
- Prove the bounds the jvm and dotnet clients say they apply (clients)
- Drive the whole generated surface of the java, kotlin and csharp clients (clients)
- Stop comparing two clocks as though they were one (clients)

[Unreleased]: https://gitlab.com/hook0/hook0/-/compare/sdk-v1.1.0...HEAD
